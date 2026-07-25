use crate::models::{Dish, Order, RecommendationResult, UserPreference};
use crate::recommender::adaptive::{
    AdaptiveScoringConfig, AdaptiveWeights, RecommendationEvidenceProfile,
};
use crate::recommender::association_metrics::best_association_metric_from_counts;
use crate::recommender::collaborative_filter::{
    CoOrderMatrix, build_co_order_matrix, calculate_co_order_score_with_max,
    related_selected_dishes, strongest_related_count,
};
use crate::recommender::diversity_reranker::{
    DiversityMetrics, DiversityMode, DiversityRerankerConfig, rerank_recommendations,
};
use crate::recommender::evidence::{
    CandidateEvidenceInput, calculate_candidate_evidence, candidate_context_basket_counts,
};
use crate::recommender::explanation::build_adaptive_explanation;
use crate::recommender::ingredient_filter::{
    calculate_ingredient_score, check_disliked_ingredients, matched_disliked_ingredients,
    matched_liked_ingredients, matched_preferred_tags,
};
use crate::recommender::popularity::{
    PopularityCounts, build_popularity_counts, calculate_popularity_score_with_max,
    maximum_popularity_count,
};
use crate::recommender::time_context::{TimeContext, calculate_time_score};
use std::collections::{HashMap, HashSet};

/// Summary values displayed in the Evaluation / Prototype Testing section.
///
/// This is not a full academic evaluation metric. It provides lightweight demo
/// evidence: how many dishes passed filtering, how many were excluded, and how
/// diverse the top recommendations are by category.
#[derive(Debug, Clone, Default)]
pub struct RecommendationStats {
    pub filtered_dishes: usize,
    pub matched_preference_dishes: usize,
    pub excluded_due_to_disliked: usize,
    pub skipped_selected_dishes: usize,
    pub diversity_count_top_5: usize,
}

/// Complete output from the recommendation engine.
#[derive(Debug, Clone, Default)]
pub struct RecommendationOutput {
    pub recommendations: Vec<RecommendationResult>,
    #[allow(dead_code)]
    pub stats: RecommendationStats,
    pub evidence_profile: RecommendationEvidenceProfile,
    pub adaptive_weights: AdaptiveWeights,
    pub scoring_config: AdaptiveScoringConfig,
    pub diversity_mode: DiversityMode,
    pub diversity_metrics: DiversityMetrics,
}

/// Reusable indexes and configuration for one recommendation request.
///
/// These values are derived once before candidate scoring. In particular, the
/// co-order normalizer, popularity normalizer, context-basket counts, evidence
/// profile, and adaptive weights no longer require repeated historical-order
/// scans for every dish.
struct RecommendationScoringContext {
    co_order_matrix: CoOrderMatrix,
    popularity_counts: PopularityCounts,
    candidate_context_counts: HashMap<String, usize>,
    co_order_max_count: u32,
    popularity_max_count: u32,
    time_context: TimeContext,
    ranking_method: RankingMethod,
    adaptive_config: AdaptiveScoringConfig,
    evidence_profile: RecommendationEvidenceProfile,
    adaptive_weights: AdaptiveWeights,
    selected_set: HashSet<String>,
}

impl RecommendationScoringContext {
    fn build(orders: &[Order], preference: &UserPreference) -> Self {
        let co_order_matrix = build_co_order_matrix(orders);
        let popularity_counts = build_popularity_counts(orders);
        let time_context =
            TimeContext::from_label(preference.time_context.as_deref().unwrap_or("Any"));
        let ranking_method = RankingMethod::from_value(preference.ranking_method.as_deref());
        let adaptive_config = AdaptiveScoringConfig::default();
        let evidence_profile = RecommendationEvidenceProfile::build(
            orders,
            &co_order_matrix,
            &preference.selected_dish_ids,
            preference.has_content_preferences(),
            time_context != TimeContext::Any,
            adaptive_config,
        );
        let adaptive_weights = AdaptiveWeights::for_profile(&evidence_profile);
        let co_order_max_count =
            strongest_related_count(&co_order_matrix, &preference.selected_dish_ids);
        let popularity_max_count = maximum_popularity_count(&popularity_counts);
        let candidate_context_counts =
            candidate_context_basket_counts(orders, &preference.selected_dish_ids);
        let selected_set = preference.selected_dish_ids.iter().cloned().collect();

        Self {
            co_order_matrix,
            popularity_counts,
            candidate_context_counts,
            co_order_max_count,
            popularity_max_count,
            time_context,
            ranking_method,
            adaptive_config,
            evidence_profile,
            adaptive_weights,
            selected_set,
        }
    }
}

/// Runs the production pipeline: base adaptive scoring followed by the
/// deterministic diversity/discovery reranker.
pub fn generate_production_recommendations(
    dishes: &[Dish],
    orders: &[Order],
    preference: &UserPreference,
    diversity_mode: DiversityMode,
) -> RecommendationOutput {
    let mut output = generate_recommendations(dishes, orders, preference);
    let (recommendations, metrics) = rerank_recommendations(
        output.recommendations,
        diversity_mode,
        DiversityRerankerConfig::default(),
    );
    output.recommendations = recommendations;
    output.diversity_mode = diversity_mode;
    output.diversity_metrics = metrics;
    output
}

/// Generates ranked recommendations using content, co-ordering, popularity, and
/// simple time/business-rule scoring.
///
/// Pipeline:
/// 1. Build co-order and popularity evidence from order logs.
/// 2. Exclude dishes containing disliked ingredients.
/// 3. Skip dishes already selected by the user.
/// 4. Calculate all explainable component scores.
/// 5. Combine scores according to ranking method.
/// 6. Sort by final score descending.
pub fn generate_recommendations(
    dishes: &[Dish],
    orders: &[Order],
    preference: &UserPreference,
) -> RecommendationOutput {
    let context = RecommendationScoringContext::build(orders, preference);
    let mut recommendations = Vec::new();
    let mut stats = RecommendationStats::default();

    for dish in dishes {
        if context.selected_set.contains(&dish.dish_id) {
            stats.skipped_selected_dishes += 1;
            continue;
        }

        if check_disliked_ingredients(dish, preference) {
            stats.excluded_due_to_disliked += 1;
            continue;
        }

        let ingredient_score = calculate_ingredient_score(dish, preference);
        let matched_liked_ingredients = matched_liked_ingredients(dish, preference);
        let matched_preferred_tags = matched_preferred_tags(dish, preference);
        let matched_disliked_ingredients = matched_disliked_ingredients(dish, preference);
        let matches_preferences =
            !matched_liked_ingredients.is_empty() || !matched_preferred_tags.is_empty();
        stats.filtered_dishes += 1;
        if matches_preferences {
            stats.matched_preference_dishes += 1;
        }

        let co_order_score = calculate_co_order_score_with_max(
            &context.co_order_matrix,
            &preference.selected_dish_ids,
            &dish.dish_id,
            context.co_order_max_count,
        );
        let popularity_score = calculate_popularity_score_with_max(
            &context.popularity_counts,
            &dish.dish_id,
            context.popularity_max_count,
        );
        let business_rule_score = calculate_time_score(dish, context.time_context);
        let evidence = calculate_candidate_evidence(CandidateEvidenceInput {
            candidate_dish_id: &dish.dish_id,
            candidate_context_basket_count: context
                .candidate_context_counts
                .get(&dish.dish_id)
                .copied()
                .unwrap_or(0),
            popularity_counts: &context.popularity_counts,
            preference,
            matched_liked_count: matched_liked_ingredients.len(),
            matched_tag_count: matched_preferred_tags.len(),
            content_score: ingredient_score,
            co_order_score,
            popularity_score,
            time_context_score: business_rule_score,
            profile: &context.evidence_profile,
            weights: context.adaptive_weights,
            config: context.adaptive_config,
        });
        let mut final_score = combine_scores(
            ingredient_score,
            co_order_score,
            popularity_score,
            business_rule_score,
            context.ranking_method,
            context.adaptive_weights,
        );

        if final_score <= 0.0 {
            if !preference.has_content_preferences() && !preference.has_selected_dishes() {
                // Last-resort fallback: when a brand-new dataset has no order
                // history yet, still show available dishes instead of an empty
                // customer recommendation section.
                final_score = 0.01;
            } else {
                continue;
            }
        }

        let related_selected_dish_ids = related_selected_dishes(
            &context.co_order_matrix,
            &preference.selected_dish_ids,
            &dish.dish_id,
        );
        let association = best_association_metric_from_counts(
            &context.co_order_matrix,
            &context.popularity_counts,
            orders.len(),
            &preference.selected_dish_ids,
            &dish.dish_id,
        );

        recommendations.push(RecommendationResult {
            dish: dish.clone(),
            ingredient_score,
            co_order_score,
            popularity_score,
            business_rule_score,
            final_score,
            base_score: final_score,
            reranked_score: final_score,
            base_rank: 0,
            reranked_rank: 0,
            novelty_score: (1.0 - popularity_score).clamp(0.0, 1.0),
            max_similarity: 0.0,
            category_bonus: 0.0,
            diversity_notes: Vec::new(),
            adaptive_weights: context.adaptive_weights,
            evidence: evidence.clone(),
            association_base_dish_id: association
                .as_ref()
                .map(|metric| metric.base_dish_id.clone()),
            association_pair_count: association
                .as_ref()
                .map(|metric| metric.pair_count)
                .unwrap_or(0),
            association_support: association
                .as_ref()
                .map(|metric| metric.support)
                .unwrap_or(0.0),
            association_confidence: association
                .as_ref()
                .map(|metric| metric.confidence)
                .unwrap_or(0.0),
            association_lift: association
                .as_ref()
                .map(|metric| metric.lift)
                .unwrap_or(0.0),
            matched_liked_ingredients,
            matched_preferred_tags,
            matched_disliked_ingredients,
            related_selected_dish_ids,
            explanation: build_adaptive_explanation(
                dish,
                preference,
                ingredient_score,
                co_order_score,
                popularity_score,
                business_rule_score,
                context.time_context,
                association
                    .as_ref()
                    .map(|metric| metric.lift)
                    .unwrap_or(0.0),
                context.adaptive_weights,
                &evidence,
            ),
        });
    }

    recommendations.sort_by(|a, b| {
        b.final_score
            .total_cmp(&a.final_score)
            .then_with(|| a.dish.name.cmp(&b.dish.name))
    });

    stats.diversity_count_top_5 = recommendations
        .iter()
        .take(5)
        .map(|result| result.dish.category.clone())
        .collect::<HashSet<_>>()
        .len();

    RecommendationOutput {
        recommendations,
        stats,
        evidence_profile: context.evidence_profile,
        adaptive_weights: context.adaptive_weights,
        scoring_config: context.adaptive_config,
        diversity_mode: DiversityMode::Balanced,
        diversity_metrics: DiversityMetrics::default(),
    }
}

/// Combines all component scores.
///
/// Admin tester method overrides:
/// - content-based: content score only
/// - co-ordering: co-order score, with popularity as fallback
/// - hybrid: data-aware production weights calculated once for the request
pub fn combine_scores(
    ingredient_score: f32,
    co_order_score: f32,
    popularity_score: f32,
    business_rule_score: f32,
    ranking_method: RankingMethod,
    adaptive_weights: AdaptiveWeights,
) -> f32 {
    let score = match ranking_method {
        RankingMethod::ContentBased => ingredient_score,
        RankingMethod::CoOrdering => {
            if co_order_score > 0.0 {
                co_order_score
            } else {
                popularity_score * 0.7 + business_rule_score * 0.3
            }
        }
        RankingMethod::Hybrid => adaptive_weights.combine(
            ingredient_score,
            co_order_score,
            popularity_score,
            business_rule_score,
        ),
    };

    score.clamp(0.0, 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankingMethod {
    ContentBased,
    CoOrdering,
    Hybrid,
}

impl RankingMethod {
    fn from_value(value: Option<&str>) -> Self {
        match value.unwrap_or("hybrid").trim().to_lowercase().as_str() {
            "content" | "content-based" | "content_based" => Self::ContentBased,
            "co-ordering" | "co_ordering" | "collaborative" => Self::CoOrdering,
            _ => Self::Hybrid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dish(id: &str, name: &str, category: &str, ingredients: &[&str], tags: &[&str]) -> Dish {
        Dish {
            dish_id: id.to_string(),
            name: name.to_string(),
            ingredients: ingredients.iter().map(|value| value.to_string()).collect(),
            category: category.to_string(),
            tags: tags.iter().map(|value| value.to_string()).collect(),
            image_path: None,
            image_source_url: None,
        }
    }

    #[test]
    fn adaptive_hybrid_uses_request_weights_for_all_component_scores() {
        let weights = AdaptiveWeights {
            content: 0.4,
            co_order: 0.3,
            popularity: 0.2,
            time_context: 0.1,
        };
        let score = combine_scores(1.0, 0.5, 0.25, 1.0, RankingMethod::Hybrid, weights);

        assert!((score - 0.70).abs() < 0.0001);
    }

    #[test]
    fn content_only_and_co_order_only_modes_ignore_adaptive_production_weights() {
        let weights = AdaptiveWeights::default();
        let content = combine_scores(0.8, 1.0, 1.0, 1.0, RankingMethod::ContentBased, weights);
        let co_order = combine_scores(0.8, 0.6, 1.0, 1.0, RankingMethod::CoOrdering, weights);

        assert!((content - 0.8).abs() < 0.0001);
        assert!((co_order - 0.6).abs() < 0.0001);
    }

    #[test]
    fn popularity_fallback_produces_recommendations_without_preferences() {
        let dishes = vec![
            dish("D01", "Nasi Lemak", "main", &["rice"], &["signature"]),
            dish("D02", "Satay", "main", &["chicken"], &["grilled"]),
        ];
        let orders = vec![Order {
            order_id: "O1".to_string(),
            session_user_id: "U1".to_string(),
            ordered_dishes: vec!["D02".to_string(), "D02".to_string()],
            timestamp: "t".to_string(),
        }];

        let output = generate_recommendations(&dishes, &orders, &UserPreference::default());

        assert!(!output.recommendations.is_empty());
        assert_eq!(output.recommendations[0].dish.dish_id, "D02");
        assert!(output.recommendations[0].popularity_score > 0.0);
    }

    #[test]
    fn liked_preferences_are_counted_as_matches_without_hard_filtering() {
        let dishes = vec![
            dish(
                "D01",
                "Chicken Rice",
                "main",
                &["chicken", "rice"],
                &["signature"],
            ),
            dish("D02", "Plain Rice", "main", &["rice"], &["simple"]),
        ];
        let preference = UserPreference {
            liked_ingredients: vec!["chicken".to_string()],
            ..UserPreference::default()
        };

        let output = generate_recommendations(&dishes, &[], &preference);

        assert_eq!(output.stats.filtered_dishes, 2);
        assert_eq!(output.stats.matched_preference_dishes, 1);
        assert_eq!(output.recommendations[0].dish.dish_id, "D01");
    }

    #[test]
    fn empty_history_uses_deterministic_fallback_with_insufficient_evidence() {
        let dishes = vec![dish("D01", "New Dish", "main", &["rice"], &["local"])];
        let output = generate_recommendations(&dishes, &[], &UserPreference::default());
        let result = &output.recommendations[0];

        assert_eq!(
            result.evidence.confidence_level,
            crate::recommender::evidence::ConfidenceLevel::Insufficient
        );
        assert_eq!(result.evidence.overall_confidence, 0.0);
        assert_eq!(result.final_score, 0.01);
    }

    #[test]
    fn new_dish_can_rank_from_content_while_historical_evidence_stays_limited() {
        let dishes = vec![dish(
            "D01",
            "New Chicken",
            "main",
            &["chicken"],
            &["signature"],
        )];
        let preference = UserPreference {
            liked_ingredients: vec!["chicken".to_string()],
            ..UserPreference::default()
        };
        let output = generate_recommendations(&dishes, &[], &preference);
        let result = &output.recommendations[0];

        assert!(result.final_score > result.evidence.overall_confidence);
        assert_eq!(
            result.evidence.primary_evidence_source,
            crate::recommender::evidence::EvidenceSource::ContentPreference
        );
        assert!(
            result
                .evidence
                .evidence_notes
                .iter()
                .any(|note| note.contains("limited historical evidence"))
        );
    }

    #[test]
    fn one_rare_pair_cannot_create_high_evidence_confidence() {
        let dishes = vec![
            dish("D01", "Anchor", "main", &["rice"], &["local"]),
            dish("D02", "Candidate", "side", &["egg"], &["local"]),
        ];
        let orders = vec![Order {
            order_id: "O1".to_string(),
            session_user_id: "U1".to_string(),
            ordered_dishes: vec!["D01".to_string(), "D02".to_string()],
            timestamp: "t".to_string(),
        }];
        let preference = UserPreference {
            selected_dish_ids: vec!["D01".to_string()],
            ..UserPreference::default()
        };
        let output = generate_recommendations(&dishes, &orders, &preference);
        let result = &output.recommendations[0];

        assert_eq!(result.evidence.candidate_pair_count, 1);
        assert_ne!(
            result.evidence.confidence_level,
            crate::recommender::evidence::ConfidenceLevel::High
        );
    }

    #[test]
    fn repeated_context_evidence_increases_adaptive_co_order_weight() {
        let dishes = vec![
            dish("D01", "Anchor", "main", &["rice"], &["local"]),
            dish("D02", "Candidate", "side", &["egg"], &["local"]),
        ];
        let preference = UserPreference {
            liked_ingredients: vec!["egg".to_string()],
            selected_dish_ids: vec!["D01".to_string()],
            ..UserPreference::default()
        };
        let low_orders = vec![Order {
            order_id: "O1".to_string(),
            session_user_id: "U1".to_string(),
            ordered_dishes: vec!["D01".to_string(), "D02".to_string()],
            timestamp: "t".to_string(),
        }];
        let high_orders = (0..50)
            .map(|index| Order {
                order_id: format!("O{index}"),
                session_user_id: format!("U{index}"),
                ordered_dishes: if index < 5 {
                    vec!["D01".to_string(), "D02".to_string()]
                } else {
                    vec!["D01".to_string()]
                },
                timestamp: "t".to_string(),
            })
            .collect::<Vec<_>>();

        let low = generate_recommendations(&dishes, &low_orders, &preference);
        let high = generate_recommendations(&dishes, &high_orders, &preference);
        assert!(high.evidence_profile.dataset_strength > low.evidence_profile.dataset_strength);
        assert!(
            high.evidence_profile.collaborative_confidence
                > low.evidence_profile.collaborative_confidence
        );
        assert!(high.adaptive_weights.co_order > low.adaptive_weights.co_order);
        assert!(high.adaptive_weights.content < low.adaptive_weights.content);
    }

    #[test]
    fn mature_no_input_fallback_is_popularity_led() {
        let dishes = vec![
            dish("D01", "Popular Rice", "main", &["rice"], &["local"]),
            dish("D02", "New Side", "side", &["egg"], &["local"]),
        ];
        let orders = (0..50)
            .map(|index| Order {
                order_id: format!("O{index}"),
                session_user_id: format!("U{index}"),
                ordered_dishes: vec!["D01".to_string()],
                timestamp: "t".to_string(),
            })
            .collect::<Vec<_>>();
        let output = generate_recommendations(&dishes, &orders, &UserPreference::default());
        let result = &output.recommendations[0];

        assert_eq!(result.dish.dish_id, "D01");
        assert_eq!(
            result.evidence.primary_evidence_source,
            crate::recommender::evidence::EvidenceSource::Popularity
        );
        assert!(
            result
                .evidence
                .evidence_notes
                .iter()
                .any(|note| note.contains("popularity-based"))
        );
    }
}
