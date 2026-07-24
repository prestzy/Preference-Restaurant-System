use crate::models::{Dish, Order, UserPreference};
use crate::recommender::collaborative_filter::{CoOrderMatrix, build_co_order_matrix};
use crate::recommender::diversity_reranker::DiversityMode;
use crate::recommender::hybrid::generate_production_recommendations;
use crate::recommender::ingredient_filter::check_disliked_ingredients;
use crate::recommender::similarity::{SimilarityConfig, dish_similarity};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy)]
pub struct MealSetLimits {
    pub max_party_size: usize,
    pub max_dish_count: usize,
    pub max_set_count: usize,
    pub candidate_pool_size: usize,
    pub beam_width: usize,
}

impl Default for MealSetLimits {
    fn default() -> Self {
        Self {
            max_party_size: 12,
            max_dish_count: 8,
            max_set_count: 5,
            candidate_pool_size: 20,
            beam_width: 200,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MealSetScoringConfig {
    pub dish_utility_weight: f32,
    pub preference_coverage_weight: f32,
    pub category_coverage_weight: f32,
    pub pair_compatibility_weight: f32,
    pub diversity_weight: f32,
    pub budget_utilisation_weight: f32,
}

impl Default for MealSetScoringConfig {
    fn default() -> Self {
        Self {
            dish_utility_weight: 0.45,
            preference_coverage_weight: 0.15,
            category_coverage_weight: 0.15,
            pair_compatibility_weight: 0.10,
            diversity_weight: 0.10,
            budget_utilisation_weight: 0.05,
        }
    }
}

impl MealSetScoringConfig {
    pub fn validate(self) -> bool {
        let values = [
            self.dish_utility_weight,
            self.preference_coverage_weight,
            self.category_coverage_weight,
            self.pair_compatibility_weight,
            self.diversity_weight,
            self.budget_utilisation_weight,
        ];
        values
            .iter()
            .all(|value| value.is_finite() && *value >= 0.0)
            && (values.iter().sum::<f32>() - 1.0).abs() < 0.0001
    }
}

#[derive(Debug, Clone)]
pub struct MealSetInput {
    pub budget_cents: u32,
    pub party_size: usize,
    pub target_dish_count: Option<usize>,
    pub top_set_count: Option<usize>,
    pub preference: UserPreference,
    pub required_categories: Vec<String>,
    pub diversity_mode: DiversityMode,
}

#[derive(Debug, Clone, Serialize)]
pub struct MealSetDish {
    pub dish_id: String,
    pub name: String,
    pub category: String,
    pub price_cents: u32,
    pub price: String,
    pub base_score: f32,
    pub reranked_score: f32,
    pub evidence_confidence: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct PairEvidenceView {
    pub dish_a_id: String,
    pub dish_b_id: String,
    pub count: u32,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct MealSetRecommendation {
    pub set_id: String,
    pub dishes: Vec<MealSetDish>,
    pub total_price_cents: u32,
    pub budget_cents: u32,
    pub remaining_budget_cents: u32,
    pub average_dish_utility: f32,
    pub preference_coverage: f32,
    pub category_coverage: f32,
    pub pair_compatibility: f32,
    pub set_diversity: f32,
    pub budget_utilisation: f32,
    pub final_set_score: f32,
    pub matched_preferences: Vec<String>,
    pub represented_categories: Vec<String>,
    pub strongest_pair: Option<PairEvidenceView>,
    pub observed_pair_count: usize,
    pub unseen_pair_count: usize,
    pub explanation_notes: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Clone)]
struct Candidate {
    dish: Dish,
    price_cents: u32,
    base_score: f32,
    reranked_score: f32,
    confidence: f32,
    utility: f32,
}

#[derive(Clone)]
struct BeamState {
    indexes: Vec<usize>,
    total_price_cents: u32,
    next_index: usize,
    estimate: f32,
}

pub fn recommend_meal_sets(
    dishes: &[Dish],
    orders: &[Order],
    prices: &HashMap<String, u32>,
    input: &MealSetInput,
) -> Result<Vec<MealSetRecommendation>, String> {
    let limits = MealSetLimits::default();
    let scoring = MealSetScoringConfig::default();
    debug_assert!(scoring.validate());
    validate_input(dishes, prices, input, limits)?;
    let target = input
        .target_dish_count
        .unwrap_or_else(|| default_target_dish_count(input.party_size));
    let set_count = input
        .top_set_count
        .unwrap_or(3)
        .clamp(1, limits.max_set_count);
    let output = generate_production_recommendations(
        dishes,
        orders,
        &input.preference,
        input.diversity_mode,
    );
    let selected = input
        .preference
        .selected_dish_ids
        .iter()
        .map(|id| id.trim().to_uppercase())
        .collect::<HashSet<_>>();
    let result_by_id = output
        .recommendations
        .iter()
        .map(|result| (result.dish.dish_id.clone(), result))
        .collect::<HashMap<_, _>>();
    let mut candidates = dishes
        .iter()
        .filter_map(|dish| {
            if check_disliked_ingredients(dish, &input.preference) {
                return None;
            }
            let price_cents = prices.get(&dish.dish_id).copied()?;
            let result = result_by_id.get(&dish.dish_id);
            // A neutral dish may still be needed to satisfy dish-count,
            // category, or budget constraints. Give it a deliberately low
            // utility instead of removing it; hard restrictions were applied
            // above and therefore can never be relaxed by this fallback.
            let base = result.map(|item| item.base_score).unwrap_or(0.05);
            let reranked = result.map(|item| item.reranked_score).unwrap_or(base);
            let confidence = result
                .map(|item| item.evidence.overall_confidence)
                .unwrap_or(0.0);
            Some(Candidate {
                dish: dish.clone(),
                price_cents,
                base_score: base,
                reranked_score: reranked,
                confidence,
                utility: (0.80 * reranked + 0.20 * confidence).clamp(0.0, 1.0),
            })
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        selected
            .contains(&right.dish.dish_id)
            .cmp(&selected.contains(&left.dish.dish_id))
            .then_with(|| right.utility.total_cmp(&left.utility))
            .then_with(|| left.dish.dish_id.cmp(&right.dish.dish_id))
    });
    candidates.truncate(limits.candidate_pool_size + selected.len());

    let required_indexes = candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| selected.contains(&candidate.dish.dish_id))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let initial_price = required_indexes
        .iter()
        .map(|index| candidates[*index].price_cents)
        .sum();
    let mut states = vec![BeamState {
        indexes: required_indexes.clone(),
        total_price_cents: initial_price,
        next_index: 0,
        estimate: 0.0,
    }];
    while states
        .first()
        .is_some_and(|state| state.indexes.len() < target)
    {
        let mut expanded = Vec::new();
        for state in &states {
            for index in state.next_index..candidates.len() {
                if state.indexes.contains(&index) {
                    continue;
                }
                let total = state.total_price_cents + candidates[index].price_cents;
                if total > input.budget_cents {
                    continue;
                }
                let mut indexes = state.indexes.clone();
                indexes.push(index);
                indexes.sort_unstable();
                let estimate = indexes
                    .iter()
                    .map(|idx| candidates[*idx].utility)
                    .sum::<f32>()
                    / indexes.len() as f32;
                expanded.push(BeamState {
                    indexes,
                    total_price_cents: total,
                    next_index: index + 1,
                    estimate,
                });
            }
        }
        expanded.sort_by(|left, right| {
            right
                .estimate
                .total_cmp(&left.estimate)
                .then_with(|| left.total_price_cents.cmp(&right.total_price_cents))
                .then_with(|| left.indexes.cmp(&right.indexes))
        });
        expanded.dedup_by(|left, right| left.indexes == right.indexes);
        expanded.truncate(limits.beam_width);
        if expanded.is_empty() {
            break;
        }
        states = expanded;
    }

    let matrix = build_co_order_matrix(orders);
    let mut results = states
        .into_iter()
        .filter(|state| state.indexes.len() == target)
        .filter_map(|state| {
            let chosen = state
                .indexes
                .iter()
                .map(|index| &candidates[*index])
                .collect::<Vec<_>>();
            required_categories_satisfied(&chosen, &input.required_categories)
                .then(|| evaluate_set(&chosen, input, &matrix, scoring))
        })
        .collect::<Vec<_>>();
    results.sort_by(|left, right| {
        right
            .final_set_score
            .total_cmp(&left.final_set_score)
            .then_with(|| left.total_price_cents.cmp(&right.total_price_cents))
            .then_with(|| left.set_id.cmp(&right.set_id))
    });
    results.dedup_by(|left, right| left.set_id == right.set_id);
    results.truncate(set_count);
    if results.is_empty() {
        return Err(if input.required_categories.is_empty() {
            "No meal set fits the budget and selected dish constraints.".to_string()
        } else {
            format!(
                "No meal set within the budget can include the required category condition(s): {}.",
                input.required_categories.join(", ")
            )
        });
    }
    Ok(results)
}

fn validate_input(
    dishes: &[Dish],
    prices: &HashMap<String, u32>,
    input: &MealSetInput,
    limits: MealSetLimits,
) -> Result<(), String> {
    if input.budget_cents == 0 {
        return Err("Budget must be greater than zero.".to_string());
    }
    if input.party_size == 0 || input.party_size > limits.max_party_size {
        return Err("Party size must be between 1 and 12.".to_string());
    }
    let target = input
        .target_dish_count
        .unwrap_or_else(|| default_target_dish_count(input.party_size));
    if target == 0 || target > limits.max_dish_count {
        return Err("Target dish count must be between 1 and 8.".to_string());
    }
    if input.top_set_count.unwrap_or(3) == 0
        || input.top_set_count.unwrap_or(3) > limits.max_set_count
    {
        return Err("Top set count must be between 1 and 5.".to_string());
    }
    let categories = dishes
        .iter()
        .map(|dish| dish.category.to_lowercase())
        .collect::<HashSet<_>>();
    for category in &input.required_categories {
        if !categories.contains(&category.to_lowercase()) {
            return Err(format!("Required category '{category}' is not available."));
        }
    }
    let selected_total = input
        .preference
        .selected_dish_ids
        .iter()
        .map(|id| {
            let normalized = id.trim().to_uppercase();
            let dish = dishes
                .iter()
                .find(|dish| dish.dish_id == normalized)
                .ok_or_else(|| format!("Selected dish {id} does not exist."))?;
            if check_disliked_ingredients(dish, &input.preference) {
                return Err(format!(
                    "Selected dish {} contains a disliked ingredient.",
                    dish.name
                ));
            }
            prices
                .get(&normalized)
                .copied()
                .ok_or_else(|| format!("Selected dish {id} is unavailable."))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .sum::<u32>();
    if input.preference.selected_dish_ids.len() > target {
        return Err("Selected dishes exceed the target dish count.".to_string());
    }
    if selected_total > input.budget_cents {
        return Err("The selected dishes already exceed the requested budget.".to_string());
    }
    Ok(())
}

fn default_target_dish_count(party_size: usize) -> usize {
    match party_size {
        1 => 2,
        2 => 3,
        3..=4 => 4,
        5..=6 => 5,
        7..=8 => 6,
        _ => 8,
    }
}

fn required_categories_satisfied(chosen: &[&Candidate], required: &[String]) -> bool {
    required.iter().all(|required| {
        chosen
            .iter()
            .any(|candidate| candidate.dish.category.eq_ignore_ascii_case(required))
    })
}

fn evaluate_set(
    chosen: &[&Candidate],
    input: &MealSetInput,
    matrix: &CoOrderMatrix,
    config: MealSetScoringConfig,
) -> MealSetRecommendation {
    let total_price_cents = chosen.iter().map(|item| item.price_cents).sum::<u32>();
    let average_dish_utility =
        chosen.iter().map(|item| item.utility).sum::<f32>() / chosen.len() as f32;
    let requested = input
        .preference
        .liked_ingredients
        .iter()
        .chain(input.preference.preferred_tags.iter())
        .map(|value| value.to_lowercase())
        .collect::<HashSet<_>>();
    let matched = requested
        .iter()
        .filter(|term| {
            chosen.iter().any(|candidate| {
                candidate
                    .dish
                    .ingredients
                    .iter()
                    .chain(candidate.dish.tags.iter())
                    .any(|value| value.eq_ignore_ascii_case(term))
            })
        })
        .cloned()
        .collect::<HashSet<_>>();
    let preference_coverage = if requested.is_empty() {
        0.0
    } else {
        matched.len() as f32 / requested.len() as f32
    };
    let represented = chosen
        .iter()
        .map(|item| item.dish.category.clone())
        .collect::<HashSet<_>>();
    let useful_target = chosen.len().min(4).max(1);
    let category_coverage = (represented.len() as f32 / useful_target as f32).min(1.0);
    let pair = pair_metrics(chosen, matrix);
    let set_diversity = set_diversity(chosen);
    let budget_utilisation = (total_price_cents as f32 / input.budget_cents as f32).clamp(0.0, 1.0);
    let final_set_score = (config.dish_utility_weight * average_dish_utility
        + config.preference_coverage_weight * preference_coverage
        + config.category_coverage_weight * category_coverage
        + config.pair_compatibility_weight * pair.average
        + config.diversity_weight * set_diversity
        + config.budget_utilisation_weight * budget_utilisation)
        .clamp(0.0, 1.0);
    let mut ids = chosen
        .iter()
        .map(|candidate| candidate.dish.dish_id.clone())
        .collect::<Vec<_>>();
    ids.sort();
    let mut notes = vec![format!(
        "Stays within RM {:.2} for {} people and covers {} menu category/categories.",
        input.budget_cents as f32 / 100.0,
        input.party_size,
        represented.len()
    )];
    if pair.observed > 0 {
        notes.push(format!(
            "{} dish pair(s) have historical co-order evidence.",
            pair.observed
        ));
    }
    MealSetRecommendation {
        set_id: ids.join("-"),
        dishes: chosen
            .iter()
            .map(|candidate| MealSetDish {
                dish_id: candidate.dish.dish_id.clone(),
                name: candidate.dish.name.clone(),
                category: candidate.dish.category.clone(),
                price_cents: candidate.price_cents,
                price: format!("RM {:.2}", candidate.price_cents as f32 / 100.0),
                base_score: candidate.base_score,
                reranked_score: candidate.reranked_score,
                evidence_confidence: candidate.confidence,
            })
            .collect(),
        total_price_cents,
        budget_cents: input.budget_cents,
        remaining_budget_cents: input.budget_cents - total_price_cents,
        average_dish_utility,
        preference_coverage,
        category_coverage,
        pair_compatibility: pair.average,
        set_diversity,
        budget_utilisation,
        final_set_score,
        matched_preferences: sorted(matched),
        represented_categories: sorted(represented),
        strongest_pair: pair.strongest,
        observed_pair_count: pair.observed,
        unseen_pair_count: pair.unseen,
        explanation_notes: notes,
        warnings: Vec::new(),
    }
}

struct PairMetrics {
    average: f32,
    observed: usize,
    unseen: usize,
    strongest: Option<PairEvidenceView>,
}

fn pair_metrics(chosen: &[&Candidate], matrix: &CoOrderMatrix) -> PairMetrics {
    let max_count = matrix
        .values()
        .flat_map(|related| related.values())
        .copied()
        .max()
        .unwrap_or(0);
    let mut scores = Vec::new();
    let mut strongest: Option<PairEvidenceView> = None;
    let mut observed = 0;
    for left in 0..chosen.len() {
        for right in (left + 1)..chosen.len() {
            let count = matrix
                .get(&chosen[left].dish.dish_id)
                .and_then(|related| related.get(&chosen[right].dish.dish_id))
                .copied()
                .unwrap_or(0);
            let score = if max_count == 0 {
                0.0
            } else {
                count as f32 / max_count as f32
            };
            scores.push(score);
            if count > 0 {
                observed += 1;
            }
            if strongest.as_ref().is_none_or(|pair| count > pair.count) {
                strongest = Some(PairEvidenceView {
                    dish_a_id: chosen[left].dish.dish_id.clone(),
                    dish_b_id: chosen[right].dish.dish_id.clone(),
                    count,
                    score,
                });
            }
        }
    }
    PairMetrics {
        average: if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f32>() / scores.len() as f32
        },
        observed,
        unseen: scores.len().saturating_sub(observed),
        strongest: strongest.filter(|pair| pair.count > 0),
    }
}

fn set_diversity(chosen: &[&Candidate]) -> f32 {
    let mut values = Vec::new();
    for left in 0..chosen.len() {
        for right in (left + 1)..chosen.len() {
            values.push(dish_similarity(
                &chosen[left].dish,
                &chosen[right].dish,
                SimilarityConfig::default(),
            ));
        }
    }
    if values.is_empty() {
        0.0
    } else {
        (1.0 - values.iter().sum::<f32>() / values.len() as f32).clamp(0.0, 1.0)
    }
}

fn sorted(values: HashSet<String>) -> Vec<String> {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort();
    values
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn fixture() -> (Vec<Dish>, HashMap<String, u32>) {
        let dishes = vec![
            dish("D01", "main", &["rice", "chicken"]),
            dish("D02", "side", &["vegetables"]),
            dish("D03", "dessert", &["banana"]),
            dish("D04", "main", &["beef"]),
        ];
        let prices = [
            ("D01".to_string(), 2_000),
            ("D02".to_string(), 1_000),
            ("D03".to_string(), 800),
            ("D04".to_string(), 2_500),
        ]
        .into_iter()
        .collect();
        (dishes, prices)
    }

    fn input() -> MealSetInput {
        MealSetInput {
            budget_cents: 4_000,
            party_size: 2,
            target_dish_count: Some(3),
            top_set_count: Some(3),
            preference: UserPreference {
                liked_ingredients: vec!["chicken".to_string()],
                ..Default::default()
            },
            required_categories: vec!["dessert".to_string()],
            diversity_mode: DiversityMode::Balanced,
        }
    }

    #[test]
    fn valid_sets_respect_budget_category_uniqueness_and_determinism() {
        let (dishes, prices) = fixture();
        let first = recommend_meal_sets(&dishes, &[], &prices, &input()).unwrap();
        let second = recommend_meal_sets(&dishes, &[], &prices, &input()).unwrap();
        assert_eq!(first[0].set_id, second[0].set_id);
        for set in first {
            assert!(set.total_price_cents <= set.budget_cents);
            assert!(set.represented_categories.contains(&"dessert".to_string()));
            let ids = set
                .dishes
                .iter()
                .map(|dish| &dish.dish_id)
                .collect::<HashSet<_>>();
            assert_eq!(ids.len(), set.dishes.len());
            assert!((0.0..=1.0).contains(&set.final_set_score));
        }
    }

    #[test]
    fn disliked_selected_dish_is_rejected_as_hard_constraint() {
        let (dishes, prices) = fixture();
        let mut request = input();
        request.preference.disliked_ingredients = vec!["beef".to_string()];
        request.preference.selected_dish_ids = vec!["D04".to_string()];
        let error = recommend_meal_sets(&dishes, &[], &prices, &request).unwrap_err();
        assert!(error.contains("disliked ingredient"));
    }

    #[test]
    fn selected_dish_over_budget_is_rejected() {
        let (dishes, prices) = fixture();
        let mut request = input();
        request.budget_cents = 1_000;
        request.preference.selected_dish_ids = vec!["D01".to_string()];
        let error = recommend_meal_sets(&dishes, &[], &prices, &request).unwrap_err();
        assert!(error.contains("exceed"));
    }

    #[test]
    fn scoring_weights_are_valid() {
        assert!(MealSetScoringConfig::default().validate());
        assert_eq!(MealSetLimits::default().beam_width, 200);
    }
}
