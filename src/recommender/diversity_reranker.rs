use crate::models::RecommendationResult;
use crate::recommender::similarity::{SimilarityConfig, dish_similarity};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiversityMode {
    Familiar,
    #[default]
    Balanced,
    Discover,
}

impl DiversityMode {
    pub fn from_label(value: Option<&str>) -> Self {
        match value.unwrap_or("balanced").trim().to_lowercase().as_str() {
            "familiar" => Self::Familiar,
            "discover" | "discovery" => Self::Discover,
            _ => Self::Balanced,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DiversityRerankerConfig {
    pub absolute_minimum: f32,
    pub relative_floor: f32,
    pub candidate_pool_size: usize,
}

impl Default for DiversityRerankerConfig {
    fn default() -> Self {
        Self {
            absolute_minimum: 0.10,
            relative_floor: 0.45,
            candidate_pool_size: 20,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DiversityMetrics {
    pub category_diversity: f32,
    pub intra_list_similarity: f32,
    pub novelty_average: f32,
    pub popularity_concentration: f32,
}

pub fn rerank_recommendations(
    mut candidates: Vec<RecommendationResult>,
    mode: DiversityMode,
    config: DiversityRerankerConfig,
) -> (Vec<RecommendationResult>, DiversityMetrics) {
    if candidates.is_empty() {
        return (candidates, DiversityMetrics::default());
    }
    candidates.sort_by(base_order);
    for (index, candidate) in candidates.iter_mut().enumerate() {
        candidate.base_rank = index + 1;
        candidate.base_score = candidate.final_score;
        candidate.reranked_score = candidate.final_score;
        candidate.novelty_score = (1.0 - candidate.popularity_score).clamp(0.0, 1.0);
        candidate.max_similarity = 0.0;
        candidate.category_bonus = 0.0;
        candidate.diversity_notes.clear();
    }

    let best = candidates[0].base_score;
    let floor = config.absolute_minimum.max(best * config.relative_floor);
    let pool_len = candidates.len().min(config.candidate_pool_size);
    let pool = candidates.drain(..pool_len).collect::<Vec<_>>();
    let (mut eligible, mut below_floor): (Vec<_>, Vec<_>) = pool
        .into_iter()
        .partition(|candidate| candidate.base_score >= floor);
    if eligible.is_empty() {
        // If every score is weak, preserve the base-ranked pool instead of
        // returning an empty top section. This is a cold-start fallback only.
        eligible = below_floor;
        below_floor = Vec::new();
    }

    let mut selected = Vec::new();
    let mut categories = HashSet::new();
    while !eligible.is_empty() {
        for candidate in &mut eligible {
            candidate.max_similarity = selected
                .iter()
                .map(|chosen: &RecommendationResult| {
                    dish_similarity(&candidate.dish, &chosen.dish, SimilarityConfig::default())
                })
                .fold(0.0_f32, f32::max);
            candidate.category_bonus = if categories.contains(&candidate.dish.category) {
                0.0
            } else {
                1.0
            };
            candidate.reranked_score = rerank_score(
                candidate.base_score,
                candidate.novelty_score,
                candidate.category_bonus,
                candidate.max_similarity,
                mode,
            );
        }
        eligible.sort_by(|left, right| {
            right
                .reranked_score
                .total_cmp(&left.reranked_score)
                .then_with(|| right.base_score.total_cmp(&left.base_score))
                .then_with(|| {
                    right
                        .evidence
                        .overall_confidence
                        .total_cmp(&left.evidence.overall_confidence)
                })
                .then_with(|| left.dish.dish_id.cmp(&right.dish.dish_id))
        });
        let chosen = eligible.remove(0);
        categories.insert(chosen.dish.category.clone());
        selected.push(chosen);
    }
    // Candidates below the relevance safeguard remain available after the
    // reranked pool; they are never silently removed from the full result set.
    selected.append(&mut below_floor);
    selected.extend(candidates);
    for (index, candidate) in selected.iter_mut().enumerate() {
        candidate.reranked_rank = index + 1;
        candidate.diversity_notes = diversity_notes(candidate, mode);
    }
    let metrics = calculate_metrics(&selected.iter().take(10).cloned().collect::<Vec<_>>());
    (selected, metrics)
}

fn rerank_score(
    base: f32,
    novelty: f32,
    category: f32,
    similarity: f32,
    mode: DiversityMode,
) -> f32 {
    let score = match mode {
        DiversityMode::Familiar => {
            0.85 * base + 0.05 * novelty + 0.05 * category - 0.05 * similarity
        }
        DiversityMode::Balanced => {
            0.70 * base + 0.10 * novelty + 0.10 * category - 0.10 * similarity
        }
        DiversityMode::Discover => {
            0.55 * base + 0.20 * novelty + 0.10 * category - 0.15 * similarity
        }
    };
    score.clamp(0.0, 1.0)
}

fn diversity_notes(candidate: &RecommendationResult, mode: DiversityMode) -> Vec<String> {
    let mut notes = Vec::new();
    if candidate.category_bonus > 0.0 && candidate.reranked_rank != candidate.base_rank {
        notes.push("Moved upward to improve category variety.".to_string());
    }
    if candidate.novelty_score >= 0.65 && mode == DiversityMode::Discover {
        notes.push("Less frequently ordered, but still above the relevance safeguard.".to_string());
    }
    if candidate.max_similarity >= 0.65 {
        notes.push(
            "Similarity to already selected recommendations reduced its diversity score."
                .to_string(),
        );
    }
    if notes.is_empty() {
        notes.push("Kept near its relevance position after the diversity check.".to_string());
    }
    notes
}

fn calculate_metrics(results: &[RecommendationResult]) -> DiversityMetrics {
    if results.is_empty() {
        return DiversityMetrics::default();
    }
    let categories = results
        .iter()
        .map(|candidate| candidate.dish.category.clone())
        .collect::<HashSet<_>>();
    let mut similarities = Vec::new();
    for left in 0..results.len() {
        for right in (left + 1)..results.len() {
            similarities.push(dish_similarity(
                &results[left].dish,
                &results[right].dish,
                SimilarityConfig::default(),
            ));
        }
    }
    let average = |values: &[f32]| {
        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<f32>() / values.len() as f32
        }
    };
    DiversityMetrics {
        category_diversity: categories.len() as f32 / results.len() as f32,
        intra_list_similarity: average(&similarities),
        novelty_average: results.iter().map(|item| item.novelty_score).sum::<f32>()
            / results.len() as f32,
        popularity_concentration: results
            .iter()
            .filter(|item| item.popularity_score >= 0.75)
            .count() as f32
            / results.len() as f32,
    }
}

fn base_order(left: &RecommendationResult, right: &RecommendationResult) -> std::cmp::Ordering {
    right
        .final_score
        .total_cmp(&left.final_score)
        .then_with(|| left.dish.dish_id.cmp(&right.dish.dish_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Dish, Order, UserPreference};
    use crate::recommender::hybrid::generate_recommendations;

    fn dish(id: &str, category: &str, ingredients: &[&str]) -> Dish {
        Dish {
            dish_id: id.to_string(),
            name: id.to_string(),
            ingredients: ingredients.iter().map(|value| value.to_string()).collect(),
            category: category.to_string(),
            tags: vec![category.to_string()],
            image_path: None,
            image_source_url: None,
        }
    }

    fn candidates() -> Vec<RecommendationResult> {
        let dishes = vec![
            dish("D01", "main", &["rice"]),
            dish("D02", "main", &["rice", "chicken"]),
            dish("D03", "side", &["vegetables"]),
            dish("D04", "dessert", &["banana"]),
        ];
        let orders = vec![
            Order {
                order_id: "O1".to_string(),
                session_user_id: "U1".to_string(),
                ordered_dishes: vec!["D01".to_string(), "D02".to_string()],
                timestamp: "t".to_string(),
            },
            Order {
                order_id: "O2".to_string(),
                session_user_id: "U2".to_string(),
                ordered_dishes: vec!["D01".to_string(), "D03".to_string()],
                timestamp: "t".to_string(),
            },
        ];
        generate_recommendations(&dishes, &orders, &UserPreference::default()).recommendations
    }

    #[test]
    fn all_modes_are_deterministic_and_preserve_candidates() {
        for mode in [
            DiversityMode::Familiar,
            DiversityMode::Balanced,
            DiversityMode::Discover,
        ] {
            let (first, metrics) =
                rerank_recommendations(candidates(), mode, DiversityRerankerConfig::default());
            let (second, _) =
                rerank_recommendations(candidates(), mode, DiversityRerankerConfig::default());
            assert_eq!(
                first
                    .iter()
                    .map(|item| &item.dish.dish_id)
                    .collect::<Vec<_>>(),
                second
                    .iter()
                    .map(|item| &item.dish.dish_id)
                    .collect::<Vec<_>>()
            );
            assert_eq!(first.len(), 4);
            assert!((0.0..=1.0).contains(&metrics.category_diversity));
            assert!((0.0..=1.0).contains(&metrics.intra_list_similarity));
        }
    }

    #[test]
    fn relevance_floor_keeps_weak_candidate_below_qualified_pool() {
        let mut values = candidates();
        values.sort_by(base_order);
        values[0].final_score = 1.0;
        values[1].final_score = 0.8;
        values[2].final_score = 0.05;
        values[3].final_score = 0.01;
        let (result, _) = rerank_recommendations(
            values,
            DiversityMode::Discover,
            DiversityRerankerConfig::default(),
        );
        assert!(result[0].base_score >= 0.45);
        assert!(result[1].base_score >= 0.45);
        assert!(result[2].base_score < 0.10);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn discover_rewards_novelty_more_than_familiar() {
        let familiar_gain = rerank_score(0.5, 1.0, 0.0, 0.0, DiversityMode::Familiar)
            - rerank_score(0.5, 0.0, 0.0, 0.0, DiversityMode::Familiar);
        let discover_gain = rerank_score(0.5, 1.0, 0.0, 0.0, DiversityMode::Discover)
            - rerank_score(0.5, 0.0, 0.0, 0.0, DiversityMode::Discover);
        assert!(discover_gain > familiar_gain);
    }
}
