use crate::models::{Order, UserPreference};
use crate::recommender::adaptive::{
    AdaptiveScoringConfig, AdaptiveWeights, RecommendationEvidenceProfile, saturated_strength,
};
use crate::recommender::popularity::PopularityCounts;
use serde::Serialize;
use std::collections::HashSet;

/// Interpretable evidence-strength bands for recommendation cards.
///
/// These are heuristic prototype bands, not calibrated satisfaction
/// probabilities.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    Insufficient,
    Low,
    Medium,
    High,
}

impl ConfidenceLevel {
    pub fn from_score(score: f32) -> Self {
        match finite_unit(score) {
            score if score < 0.15 => Self::Insufficient,
            score if score < 0.40 => Self::Low,
            score if score < 0.70 => Self::Medium,
            _ => Self::High,
        }
    }
}

/// Signal contributing most strongly to the recommendation evidence.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    ContentPreference,
    CoOrdering,
    Popularity,
    TimeContext,
    Mixed,
    None,
}

/// Weighted confidence contributions kept separate from ranking scores.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct EvidenceContributions {
    pub content: f32,
    pub co_order: f32,
    pub popularity: f32,
    pub time_context: f32,
}

/// Candidate-specific evidence returned with every production recommendation.
#[derive(Debug, Clone, Serialize)]
pub struct RecommendationEvidence {
    pub total_order_count: usize,
    pub requested_preference_count: usize,
    pub matched_preference_count: usize,
    pub selected_context_order_count: usize,
    pub candidate_pair_count: usize,
    pub candidate_popularity_count: usize,
    pub content_evidence: f32,
    pub collaborative_evidence: f32,
    pub popularity_evidence: f32,
    pub time_context_evidence: f32,
    pub contributions: EvidenceContributions,
    pub overall_confidence: f32,
    pub confidence_level: ConfidenceLevel,
    pub primary_evidence_source: EvidenceSource,
    pub evidence_notes: Vec<String>,
}

/// Inputs needed to calculate evidence for one candidate.
pub struct CandidateEvidenceInput<'a> {
    pub candidate_dish_id: &'a str,
    pub orders: &'a [Order],
    pub popularity_counts: &'a PopularityCounts,
    pub preference: &'a UserPreference,
    pub matched_liked_count: usize,
    pub matched_tag_count: usize,
    pub content_score: f32,
    pub co_order_score: f32,
    pub popularity_score: f32,
    pub time_context_score: f32,
    pub profile: &'a RecommendationEvidenceProfile,
    pub weights: AdaptiveWeights,
    pub config: AdaptiveScoringConfig,
}

pub fn calculate_candidate_evidence(input: CandidateEvidenceInput<'_>) -> RecommendationEvidence {
    let requested_preference_count =
        input.preference.liked_ingredients.len() + input.preference.preferred_tags.len();
    let matched_preference_count =
        (input.matched_liked_count + input.matched_tag_count).min(requested_preference_count);
    let input_strength = match requested_preference_count {
        0 => 0.0,
        1 => 0.75,
        count => (0.75 + 0.125 * (count.saturating_sub(1)) as f32).min(1.0),
    };
    let match_coverage = if requested_preference_count == 0 {
        0.0
    } else {
        matched_preference_count as f32 / requested_preference_count as f32
    };
    let content_evidence =
        finite_unit(input_strength * match_coverage * finite_unit(input.content_score));

    let candidate_pair_count = count_candidate_context_baskets(
        input.orders,
        &input.preference.selected_dish_ids,
        input.candidate_dish_id,
    );
    let candidate_pair_strength =
        saturated_strength(candidate_pair_count, input.config.pair_count_target);
    let collaborative_evidence = finite_unit(
        input.profile.collaborative_confidence
            * candidate_pair_strength
            * finite_unit(input.co_order_score),
    );

    let candidate_popularity_count = input
        .popularity_counts
        .get(&input.candidate_dish_id.trim().to_uppercase())
        .copied()
        .unwrap_or(0) as usize;
    let candidate_popularity_strength = saturated_strength(
        candidate_popularity_count,
        input.config.popularity_count_target,
    );
    let popularity_evidence = finite_unit(
        input.profile.dataset_strength
            * candidate_popularity_strength
            * finite_unit(input.popularity_score),
    );

    let time_context_evidence =
        if input.profile.has_explicit_time_context && input.time_context_score > 0.0 {
            finite_unit(input.time_context_score)
        } else {
            0.0
        };

    let contributions = EvidenceContributions {
        content: input.weights.content * content_evidence,
        co_order: input.weights.co_order * collaborative_evidence,
        popularity: input.weights.popularity * popularity_evidence,
        time_context: input.weights.time_context * time_context_evidence,
    };
    let overall_confidence = finite_unit(
        contributions.content
            + contributions.co_order
            + contributions.popularity
            + contributions.time_context,
    );
    let primary_evidence_source = primary_source(contributions);
    let evidence_notes = build_evidence_notes(
        requested_preference_count,
        matched_preference_count,
        input.profile,
        candidate_pair_count,
        candidate_popularity_count,
        primary_evidence_source,
    );

    RecommendationEvidence {
        total_order_count: input.profile.total_order_count,
        requested_preference_count,
        matched_preference_count,
        selected_context_order_count: input.profile.selected_context_order_count,
        candidate_pair_count,
        candidate_popularity_count,
        content_evidence,
        collaborative_evidence,
        popularity_evidence,
        time_context_evidence,
        contributions,
        overall_confidence,
        confidence_level: ConfidenceLevel::from_score(overall_confidence),
        primary_evidence_source,
        evidence_notes,
    }
}

fn count_candidate_context_baskets(
    orders: &[Order],
    selected_dish_ids: &[String],
    candidate_dish_id: &str,
) -> usize {
    let selected = selected_dish_ids
        .iter()
        .map(|dish_id| dish_id.trim().to_uppercase())
        .filter(|dish_id| !dish_id.is_empty())
        .collect::<HashSet<_>>();
    if selected.is_empty() {
        return 0;
    }
    let candidate = candidate_dish_id.trim().to_uppercase();
    orders
        .iter()
        .filter(|order| {
            let basket = order
                .ordered_dishes
                .iter()
                .map(|dish_id| dish_id.trim().to_uppercase())
                .collect::<HashSet<_>>();
            basket.contains(&candidate) && basket.iter().any(|dish_id| selected.contains(dish_id))
        })
        .count()
}

fn primary_source(contributions: EvidenceContributions) -> EvidenceSource {
    let mut ranked = [
        (EvidenceSource::ContentPreference, contributions.content),
        (EvidenceSource::CoOrdering, contributions.co_order),
        (EvidenceSource::Popularity, contributions.popularity),
        (EvidenceSource::TimeContext, contributions.time_context),
    ];
    ranked.sort_by(|left, right| right.1.total_cmp(&left.1));
    if ranked[0].1 <= 0.0001 {
        EvidenceSource::None
    } else if ranked[1].1 > 0.0001 && (ranked[0].1 - ranked[1].1).abs() <= 0.05 {
        EvidenceSource::Mixed
    } else {
        ranked[0].0
    }
}

fn build_evidence_notes(
    requested_preferences: usize,
    matched_preferences: usize,
    profile: &RecommendationEvidenceProfile,
    pair_count: usize,
    popularity_count: usize,
    source: EvidenceSource,
) -> Vec<String> {
    let mut notes = Vec::new();
    if requested_preferences > 0 {
        if matched_preferences > 0 {
            notes.push(format!(
                "Matches {matched_preferences} of {requested_preferences} explicit preference(s)."
            ));
        } else {
            notes.push("No explicit positive preference match was found.".to_string());
        }
    } else if !profile.has_selected_context {
        notes.push("No explicit preferences or selected dish context were supplied.".to_string());
    }

    if profile.has_selected_context {
        if pair_count == 0 {
            notes.push("No historical co-order relationship was observed.".to_string());
        } else {
            notes.push(format!(
                "Appeared with the selected dish context in {pair_count} historical order(s)."
            ));
            notes.push(format!(
                "Selected-dish context appears in {} historical order(s).",
                profile.selected_context_order_count
            ));
        }
    }

    if popularity_count > 0 {
        notes.push(format!(
            "Appeared in {popularity_count} historical order(s)."
        ));
    } else {
        notes.push(
            "This dish has limited historical evidence because it is new or rarely ordered."
                .to_string(),
        );
    }

    notes.push(match source {
        EvidenceSource::ContentPreference => {
            "Recommendation is mainly preference-based.".to_string()
        }
        EvidenceSource::CoOrdering => {
            "Recommendation is mainly supported by co-order evidence.".to_string()
        }
        EvidenceSource::Popularity => "Recommendation is mainly popularity-based.".to_string(),
        EvidenceSource::TimeContext => {
            "Recommendation is mainly supported by the selected time context.".to_string()
        }
        EvidenceSource::Mixed => "Recommendation is supported by mixed evidence.".to_string(),
        EvidenceSource::None => {
            "Shown as a deterministic fallback because evidence is limited.".to_string()
        }
    });
    notes
}

fn finite_unit(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> RecommendationEvidenceProfile {
        RecommendationEvidenceProfile {
            total_order_count: 50,
            selected_context_order_count: 10,
            strongest_context_pair_count: 5,
            dataset_strength: 1.0,
            context_strength: 1.0,
            pair_strength: 1.0,
            collaborative_confidence: 1.0,
            has_content_preferences: true,
            has_selected_context: true,
            has_explicit_time_context: false,
        }
    }

    #[test]
    fn confidence_threshold_boundaries_are_stable() {
        assert_eq!(
            ConfidenceLevel::from_score(0.149),
            ConfidenceLevel::Insufficient
        );
        assert_eq!(ConfidenceLevel::from_score(0.150), ConfidenceLevel::Low);
        assert_eq!(ConfidenceLevel::from_score(0.399), ConfidenceLevel::Low);
        assert_eq!(ConfidenceLevel::from_score(0.400), ConfidenceLevel::Medium);
        assert_eq!(ConfidenceLevel::from_score(0.699), ConfidenceLevel::Medium);
        assert_eq!(ConfidenceLevel::from_score(0.700), ConfidenceLevel::High);
    }

    #[test]
    fn one_basket_is_counted_once_with_multiple_selected_dishes() {
        let orders = vec![Order {
            order_id: "O1".to_string(),
            session_user_id: "U1".to_string(),
            ordered_dishes: vec![
                "D01".to_string(),
                "D02".to_string(),
                "D03".to_string(),
                "D03".to_string(),
            ],
            timestamp: "t".to_string(),
        }];
        assert_eq!(
            count_candidate_context_baskets(
                &orders,
                &["D01".to_string(), "D02".to_string()],
                "D03"
            ),
            1
        );
    }

    #[test]
    fn high_pair_count_increases_collaborative_evidence_without_using_lift() {
        let orders = (0..5)
            .map(|index| Order {
                order_id: format!("O{index}"),
                session_user_id: format!("U{index}"),
                ordered_dishes: vec!["D01".to_string(), "D02".to_string()],
                timestamp: "t".to_string(),
            })
            .collect::<Vec<_>>();
        let preference = UserPreference {
            selected_dish_ids: vec!["D01".to_string()],
            ..UserPreference::default()
        };
        let popularity = [("D02".to_string(), 5)].into_iter().collect();
        let evidence = calculate_candidate_evidence(CandidateEvidenceInput {
            candidate_dish_id: "D02",
            orders: &orders,
            popularity_counts: &popularity,
            preference: &preference,
            matched_liked_count: 0,
            matched_tag_count: 0,
            content_score: 0.0,
            co_order_score: 1.0,
            popularity_score: 1.0,
            time_context_score: 0.0,
            profile: &profile(),
            weights: AdaptiveWeights::for_profile(&profile()),
            config: AdaptiveScoringConfig::default(),
        });
        assert_eq!(evidence.candidate_pair_count, 5);
        assert!(evidence.collaborative_evidence > 0.9);
    }
}
