use crate::models::Order;
use crate::recommender::collaborative_filter::CoOrderMatrix;
use serde::Serialize;
use std::collections::HashSet;

/// Central saturation thresholds for the lightweight adaptive model.
///
/// These values are prototype heuristics, not learned constants. Keeping them
/// together makes sensitivity testing straightforward and prevents scoring
/// magic numbers from being scattered through the recommender.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct AdaptiveScoringConfig {
    /// Historical baskets at which global dataset evidence is treated as mature.
    pub total_order_target: usize,
    /// Baskets containing any selected dish needed for mature context evidence.
    pub context_order_target: usize,
    /// Co-orders at which one item pair reaches full pair-evidence strength.
    pub pair_count_target: usize,
    /// Candidate appearances needed for full popularity-evidence strength.
    pub popularity_count_target: usize,
}

impl Default for AdaptiveScoringConfig {
    fn default() -> Self {
        Self {
            total_order_target: 50,
            context_order_target: 10,
            pair_count_target: 5,
            popularity_count_target: 10,
        }
    }
}

/// Request-level evidence shared by every candidate in one recommendation run.
#[derive(Debug, Clone, Default, Serialize)]
pub struct RecommendationEvidenceProfile {
    pub total_order_count: usize,
    pub selected_context_order_count: usize,
    pub strongest_context_pair_count: usize,
    pub dataset_strength: f32,
    pub context_strength: f32,
    pub pair_strength: f32,
    pub collaborative_confidence: f32,
    pub has_content_preferences: bool,
    pub has_selected_context: bool,
    pub has_explicit_time_context: bool,
}

impl RecommendationEvidenceProfile {
    pub fn build(
        orders: &[Order],
        co_order_matrix: &CoOrderMatrix,
        selected_dish_ids: &[String],
        has_content_preferences: bool,
        has_explicit_time_context: bool,
        config: AdaptiveScoringConfig,
    ) -> Self {
        let selected = selected_dish_ids
            .iter()
            .map(|dish_id| dish_id.trim().to_uppercase())
            .filter(|dish_id| !dish_id.is_empty())
            .collect::<HashSet<_>>();
        let has_selected_context = !selected.is_empty();
        let selected_context_order_count = orders
            .iter()
            .filter(|order| {
                order
                    .ordered_dishes
                    .iter()
                    .map(|dish_id| dish_id.trim().to_uppercase())
                    .any(|dish_id| selected.contains(&dish_id))
            })
            .count();

        let strongest_context_pair_count = selected
            .iter()
            .filter_map(|selected_id| co_order_matrix.get(selected_id))
            .flat_map(|related| related.iter())
            .filter(|(candidate_id, _)| !selected.contains(*candidate_id))
            .map(|(_, count)| *count as usize)
            .max()
            .unwrap_or(0);

        let dataset_strength = saturated_strength(orders.len(), config.total_order_target);
        let context_strength =
            saturated_strength(selected_context_order_count, config.context_order_target);
        let pair_strength =
            saturated_strength(strongest_context_pair_count, config.pair_count_target);
        let collaborative_confidence = if !has_selected_context || strongest_context_pair_count == 0
        {
            0.0
        } else {
            (0.20 * dataset_strength + 0.35 * context_strength + 0.45 * pair_strength)
                .clamp(0.0, 1.0)
        };

        Self {
            total_order_count: orders.len(),
            selected_context_order_count,
            strongest_context_pair_count,
            dataset_strength,
            context_strength,
            pair_strength,
            collaborative_confidence,
            has_content_preferences,
            has_selected_context,
            has_explicit_time_context,
        }
    }
}

/// Adaptive production weights used to combine the four recommendation signals.
#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct AdaptiveWeights {
    pub content: f32,
    pub co_order: f32,
    pub popularity: f32,
    pub time_context: f32,
}

impl Default for AdaptiveWeights {
    fn default() -> Self {
        Self {
            content: 0.0,
            co_order: 0.0,
            popularity: 0.85,
            time_context: 0.15,
        }
    }
}

impl AdaptiveWeights {
    pub fn for_profile(profile: &RecommendationEvidenceProfile) -> Self {
        let confidence = finite_unit(profile.collaborative_confidence);
        let weights = match (
            profile.has_content_preferences,
            profile.has_selected_context,
        ) {
            (true, true) => Self {
                content: 0.70 - 0.30 * confidence,
                co_order: 0.05 + 0.35 * confidence,
                popularity: 0.15 - 0.05 * confidence,
                time_context: 0.10,
            },
            (true, false) => Self {
                content: 0.70,
                co_order: 0.0,
                popularity: 0.20,
                time_context: 0.10,
            },
            (false, true) => Self {
                content: 0.0,
                co_order: 0.10 + 0.55 * confidence,
                popularity: 0.80 - 0.55 * confidence,
                time_context: 0.10,
            },
            (false, false) => Self {
                content: 0.0,
                co_order: 0.0,
                popularity: 0.85,
                time_context: 0.15,
            },
        };
        let weights = weights.normalised();
        debug_assert!(weights.validate());
        weights
    }

    pub fn sum(self) -> f32 {
        self.content + self.co_order + self.popularity + self.time_context
    }

    pub fn validate(self) -> bool {
        [
            self.content,
            self.co_order,
            self.popularity,
            self.time_context,
        ]
        .into_iter()
        .all(|weight| weight.is_finite() && (0.0..=1.0).contains(&weight))
            && (self.sum() - 1.0).abs() <= 0.0001
    }

    pub fn normalised(self) -> Self {
        let clean = Self {
            content: finite_unit(self.content),
            co_order: finite_unit(self.co_order),
            popularity: finite_unit(self.popularity),
            time_context: finite_unit(self.time_context),
        };
        let sum = clean.sum();
        if sum <= f32::EPSILON {
            return Self {
                content: 0.0,
                co_order: 0.0,
                popularity: 0.85,
                time_context: 0.15,
            };
        }
        Self {
            content: clean.content / sum,
            co_order: clean.co_order / sum,
            popularity: clean.popularity / sum,
            time_context: clean.time_context / sum,
        }
    }

    /// Returns display percentages that always total exactly 100.
    pub fn as_percentages(self) -> [u8; 4] {
        let values = [
            self.content,
            self.co_order,
            self.popularity,
            self.time_context,
        ];
        let mut rounded = values.map(|value| (value * 100.0).round() as i16);
        let difference = 100 - rounded.iter().sum::<i16>();
        let largest_index = values
            .iter()
            .enumerate()
            .max_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(index, _)| index)
            .unwrap_or(0);
        rounded[largest_index] += difference;
        rounded.map(|value| value.clamp(0, 100) as u8)
    }

    pub fn combine(
        self,
        content_score: f32,
        co_order_score: f32,
        popularity_score: f32,
        time_context_score: f32,
    ) -> f32 {
        (self.content * finite_unit(content_score)
            + self.co_order * finite_unit(co_order_score)
            + self.popularity * finite_unit(popularity_score)
            + self.time_context * finite_unit(time_context_score))
        .clamp(0.0, 1.0)
    }
}

pub fn saturated_strength(count: usize, target: usize) -> f32 {
    if target == 0 {
        return if count == 0 { 0.0 } else { 1.0 };
    }
    (count as f32 / target as f32).clamp(0.0, 1.0)
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
    use crate::models::Order;
    use crate::recommender::collaborative_filter::build_co_order_matrix;

    fn profile(content: bool, context: bool, confidence: f32) -> RecommendationEvidenceProfile {
        RecommendationEvidenceProfile {
            has_content_preferences: content,
            has_selected_context: context,
            collaborative_confidence: confidence,
            ..RecommendationEvidenceProfile::default()
        }
    }

    fn assert_weights(actual: AdaptiveWeights, expected: AdaptiveWeights) {
        assert!((actual.content - expected.content).abs() < 0.0001);
        assert!((actual.co_order - expected.co_order).abs() < 0.0001);
        assert!((actual.popularity - expected.popularity).abs() < 0.0001);
        assert!((actual.time_context - expected.time_context).abs() < 0.0001);
    }

    #[test]
    fn all_adaptive_situations_sum_to_one() {
        for content in [false, true] {
            for context in [false, true] {
                for confidence in [0.0, 0.25, 0.5, 0.75, 1.0] {
                    assert!(
                        AdaptiveWeights::for_profile(&profile(content, context, confidence))
                            .validate()
                    );
                }
            }
        }
    }

    #[test]
    fn required_boundary_weight_sets_are_exact() {
        assert_weights(
            AdaptiveWeights::for_profile(&profile(true, false, 0.5)),
            AdaptiveWeights {
                content: 0.70,
                co_order: 0.0,
                popularity: 0.20,
                time_context: 0.10,
            },
        );
        assert_weights(
            AdaptiveWeights::for_profile(&profile(false, false, 0.5)),
            AdaptiveWeights {
                content: 0.0,
                co_order: 0.0,
                popularity: 0.85,
                time_context: 0.15,
            },
        );
        assert_weights(
            AdaptiveWeights::for_profile(&profile(true, true, 0.0)),
            AdaptiveWeights {
                content: 0.70,
                co_order: 0.05,
                popularity: 0.15,
                time_context: 0.10,
            },
        );
        assert_weights(
            AdaptiveWeights::for_profile(&profile(true, true, 1.0)),
            AdaptiveWeights {
                content: 0.40,
                co_order: 0.40,
                popularity: 0.10,
                time_context: 0.10,
            },
        );
        assert_weights(
            AdaptiveWeights::for_profile(&profile(false, true, 1.0)),
            AdaptiveWeights {
                content: 0.0,
                co_order: 0.65,
                popularity: 0.25,
                time_context: 0.10,
            },
        );
    }

    #[test]
    fn co_order_weight_grows_monotonically_with_evidence() {
        let low_a = AdaptiveWeights::for_profile(&profile(true, true, 0.0));
        let high_a = AdaptiveWeights::for_profile(&profile(true, true, 1.0));
        assert!(high_a.co_order >= low_a.co_order);
        assert!(high_a.content <= low_a.content);

        let low_c = AdaptiveWeights::for_profile(&profile(false, true, 0.0));
        let high_c = AdaptiveWeights::for_profile(&profile(false, true, 1.0));
        assert!(high_c.co_order >= low_c.co_order);
        assert!(high_c.popularity <= low_c.popularity);
    }

    #[test]
    fn no_context_or_observed_pair_has_zero_collaborative_confidence() {
        let orders = vec![
            Order {
                order_id: "O1".to_string(),
                session_user_id: "U1".to_string(),
                ordered_dishes: vec!["D01".to_string()],
                timestamp: "t".to_string(),
            };
            60
        ];
        let matrix = build_co_order_matrix(&orders);
        let without_context = RecommendationEvidenceProfile::build(
            &orders,
            &matrix,
            &[],
            true,
            false,
            AdaptiveScoringConfig::default(),
        );
        let without_pair = RecommendationEvidenceProfile::build(
            &orders,
            &matrix,
            &["D01".to_string()],
            true,
            false,
            AdaptiveScoringConfig::default(),
        );
        assert_eq!(without_context.collaborative_confidence, 0.0);
        assert_eq!(without_pair.collaborative_confidence, 0.0);
    }

    #[test]
    fn invalid_numbers_are_clamped_and_percentages_total_one_hundred() {
        let weights = AdaptiveWeights {
            content: f32::NAN,
            co_order: -2.0,
            popularity: 3.0,
            time_context: 0.5,
        }
        .normalised();
        assert!(weights.validate());
        assert_eq!(
            weights
                .as_percentages()
                .into_iter()
                .map(u16::from)
                .sum::<u16>(),
            100
        );
        assert!(weights.combine(f32::NAN, -1.0, 2.0, 0.5).is_finite());
    }

    #[test]
    fn controlled_five_twenty_and_fifty_order_profiles_grow_deterministically() {
        let build = |count: usize| {
            let orders = (0..count)
                .map(|index| Order {
                    order_id: format!("O{index}"),
                    session_user_id: format!("U{index}"),
                    ordered_dishes: if index == 0 {
                        vec!["D01".to_string(), "D02".to_string()]
                    } else {
                        vec!["D01".to_string()]
                    },
                    timestamp: "t".to_string(),
                })
                .collect::<Vec<_>>();
            let matrix = build_co_order_matrix(&orders);
            RecommendationEvidenceProfile::build(
                &orders,
                &matrix,
                &["D01".to_string()],
                true,
                false,
                AdaptiveScoringConfig::default(),
            )
        };
        let five = build(5);
        let twenty = build(20);
        let fifty = build(50);

        assert!(five.dataset_strength < twenty.dataset_strength);
        assert!(twenty.dataset_strength < fifty.dataset_strength);
        assert!(five.collaborative_confidence < twenty.collaborative_confidence);
        assert!(twenty.collaborative_confidence < fifty.collaborative_confidence);
        assert!(
            AdaptiveWeights::for_profile(&five).co_order
                < AdaptiveWeights::for_profile(&twenty).co_order
        );
        assert!(
            AdaptiveWeights::for_profile(&twenty).co_order
                < AdaptiveWeights::for_profile(&fifty).co_order
        );
    }
}
