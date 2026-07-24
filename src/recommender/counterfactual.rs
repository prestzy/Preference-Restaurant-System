//! Side-effect-free comparison of production recommendation scenarios.
//!
//! Baseline and changed scenarios both call the exact production pipeline.
//! Temporary preference edits and simulated baskets are applied to clones only,
//! so this module cannot persist orders or create learning-timeline events.

use crate::models::{Dish, Order, RecommendationResult, UserPreference};
use crate::recommender::adaptive::AdaptiveWeights;
use crate::recommender::diversity_reranker::DiversityMode;
use crate::recommender::hybrid::generate_production_recommendations;
use crate::recommender::ingredient_filter::check_disliked_ingredients;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CounterfactualChanges {
    #[serde(default)]
    pub add_liked_ingredients: Vec<String>,
    #[serde(default)]
    pub remove_liked_ingredients: Vec<String>,
    #[serde(default)]
    pub add_disliked_ingredients: Vec<String>,
    #[serde(default)]
    pub remove_disliked_ingredients: Vec<String>,
    #[serde(default)]
    pub add_preferred_tags: Vec<String>,
    #[serde(default)]
    pub remove_preferred_tags: Vec<String>,
    #[serde(default)]
    pub add_context_dish_ids: Vec<String>,
    #[serde(default)]
    pub remove_context_dish_ids: Vec<String>,
    #[serde(default)]
    pub simulated_coorders: Vec<SimulatedCoOrderChange>,
    #[serde(default)]
    pub diversity_mode: Option<DiversityMode>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SimulatedCoOrderChange {
    pub anchor_dish_id: String,
    pub candidate_dish_id: String,
    pub additional_order_count: usize,
}

#[derive(Debug, Clone)]
pub struct CounterfactualInput {
    pub baseline: UserPreference,
    pub baseline_diversity_mode: DiversityMode,
    pub changes: CounterfactualChanges,
    pub top_k: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CounterfactualResult {
    pub baseline_summary: RecommendationScenarioSummary,
    pub changed_summary: RecommendationScenarioSummary,
    pub rank_changes: Vec<RecommendationRankChange>,
    pub entered_top_k: Vec<DishSummary>,
    pub left_top_k: Vec<DishSummary>,
    pub exclusions_added: Vec<ExclusionChange>,
    pub exclusions_removed: Vec<ExclusionChange>,
    pub adaptive_weight_change: AdaptiveWeightDelta,
    pub explanation: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecommendationScenarioSummary {
    pub top_k: usize,
    pub diversity_mode: DiversityMode,
    pub result_count: usize,
    pub top_dishes: Vec<DishSummary>,
    pub adaptive_weights: AdaptiveWeights,
}

#[derive(Debug, Clone, Serialize)]
pub struct DishSummary {
    pub dish_id: String,
    pub dish_name: String,
    pub rank: usize,
    pub score: f32,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecommendationRankChange {
    pub dish_id: String,
    pub dish_name: String,
    pub baseline_rank: Option<usize>,
    pub changed_rank: Option<usize>,
    pub baseline_score: Option<f32>,
    pub changed_score: Option<f32>,
    pub baseline_confidence: Option<f32>,
    pub changed_confidence: Option<f32>,
    pub rank_delta: Option<i32>,
    pub score_delta: Option<f32>,
    pub classification: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExclusionChange {
    pub dish_id: String,
    pub dish_name: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdaptiveWeightDelta {
    pub baseline: AdaptiveWeights,
    pub changed: AdaptiveWeights,
    pub content_delta: f32,
    pub co_order_delta: f32,
    pub popularity_delta: f32,
    pub time_context_delta: f32,
}

/// Runs an exact baseline/changed comparison after validating bounded changes.
pub fn compare_counterfactual(
    dishes: &[Dish],
    orders: &[Order],
    input: &CounterfactualInput,
) -> Result<CounterfactualResult, String> {
    validate_input(dishes, input)?;
    let baseline_preference = normalized_preference(input.baseline.clone());
    let changed_preference = apply_changes(&baseline_preference, &input.changes);
    let mut changed_orders = orders.to_vec();
    for (change_index, simulation) in input.changes.simulated_coorders.iter().enumerate() {
        for index in 0..simulation.additional_order_count {
            changed_orders.push(Order {
                order_id: format!("COUNTERFACTUAL-{change_index:02}-{index:03}"),
                session_user_id: "TEMPORARY-SCENARIO".to_string(),
                ordered_dishes: vec![
                    simulation.anchor_dish_id.trim().to_uppercase(),
                    simulation.candidate_dish_id.trim().to_uppercase(),
                ],
                timestamp: "temporary; not persisted".to_string(),
            });
        }
    }
    let changed_mode = input
        .changes
        .diversity_mode
        .unwrap_or(input.baseline_diversity_mode);
    let baseline = generate_production_recommendations(
        dishes,
        orders,
        &baseline_preference,
        input.baseline_diversity_mode,
    );
    let changed = generate_production_recommendations(
        dishes,
        &changed_orders,
        &changed_preference,
        changed_mode,
    );

    let baseline_by_id = result_lookup(&baseline.recommendations);
    let changed_by_id = result_lookup(&changed.recommendations);
    let names = dishes
        .iter()
        .map(|dish| (dish.dish_id.clone(), dish.name.clone()))
        .collect::<HashMap<_, _>>();
    let mut ids = baseline_by_id
        .keys()
        .chain(changed_by_id.keys())
        .cloned()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    ids.sort();
    let baseline_top = top_id_set(&baseline.recommendations, input.top_k);
    let changed_top = top_id_set(&changed.recommendations, input.top_k);
    let mut rank_changes = ids
        .into_iter()
        .map(|dish_id| {
            let before = baseline_by_id.get(&dish_id);
            let after = changed_by_id.get(&dish_id);
            let baseline_rank = before.map(|(rank, _)| *rank);
            let changed_rank = after.map(|(rank, _)| *rank);
            let rank_delta = match (baseline_rank, changed_rank) {
                (Some(before), Some(after)) => Some(before as i32 - after as i32),
                _ => None,
            };
            let classification = match (baseline_rank, changed_rank, rank_delta) {
                (None, Some(_), _) => "newly eligible",
                (Some(_), None, _) => "newly excluded",
                (_, _, Some(delta)) if delta > 0 => "moved up",
                (_, _, Some(delta)) if delta < 0 => "moved down",
                _ => "unchanged",
            }
            .to_string();
            RecommendationRankChange {
                dish_id: dish_id.clone(),
                dish_name: names.get(&dish_id).cloned().unwrap_or(dish_id),
                baseline_rank,
                changed_rank,
                baseline_score: before.map(|(_, item)| item.reranked_score),
                changed_score: after.map(|(_, item)| item.reranked_score),
                baseline_confidence: before.map(|(_, item)| item.evidence.overall_confidence),
                changed_confidence: after.map(|(_, item)| item.evidence.overall_confidence),
                rank_delta,
                score_delta: match (before, after) {
                    (Some((_, before)), Some((_, after))) => {
                        Some(after.reranked_score - before.reranked_score)
                    }
                    _ => None,
                },
                reasons: delta_reasons(&input.changes),
                classification,
            }
        })
        .collect::<Vec<_>>();
    rank_changes.sort_by(|left, right| {
        right
            .rank_delta
            .unwrap_or(i32::MIN)
            .cmp(&left.rank_delta.unwrap_or(i32::MIN))
            .then_with(|| left.dish_id.cmp(&right.dish_id))
    });

    let entered_top_k = dish_summaries(
        &changed.recommendations,
        input.top_k,
        Some(&baseline_top),
        false,
    );
    let left_top_k = dish_summaries(
        &baseline.recommendations,
        input.top_k,
        Some(&changed_top),
        false,
    );
    let baseline_excluded = explicit_exclusions(dishes, &baseline_preference);
    let changed_excluded = explicit_exclusions(dishes, &changed_preference);
    let exclusions_added = exclusion_changes(
        dishes,
        changed_excluded.difference(&baseline_excluded),
        "New disliked-ingredient hard exclusion.",
    );
    let exclusions_removed = exclusion_changes(
        dishes,
        baseline_excluded.difference(&changed_excluded),
        "Disliked-ingredient exclusion removed.",
    );

    let weight_delta = AdaptiveWeightDelta {
        baseline: baseline.adaptive_weights,
        changed: changed.adaptive_weights,
        content_delta: changed.adaptive_weights.content - baseline.adaptive_weights.content,
        co_order_delta: changed.adaptive_weights.co_order - baseline.adaptive_weights.co_order,
        popularity_delta: changed.adaptive_weights.popularity
            - baseline.adaptive_weights.popularity,
        time_context_delta: changed.adaptive_weights.time_context
            - baseline.adaptive_weights.time_context,
    };
    let mut explanation = delta_reasons(&input.changes);
    explanation.push(format!(
        "{} dish(es) entered Top-{} and {} left it. Temporary orders and preferences were not saved.",
        entered_top_k.len(),
        input.top_k,
        left_top_k.len()
    ));

    Ok(CounterfactualResult {
        baseline_summary: scenario_summary(
            &baseline.recommendations,
            input.top_k,
            input.baseline_diversity_mode,
            baseline.adaptive_weights,
        ),
        changed_summary: scenario_summary(
            &changed.recommendations,
            input.top_k,
            changed_mode,
            changed.adaptive_weights,
        ),
        rank_changes,
        entered_top_k,
        left_top_k,
        exclusions_added,
        exclusions_removed,
        adaptive_weight_change: weight_delta,
        explanation,
    })
}

fn validate_input(dishes: &[Dish], input: &CounterfactualInput) -> Result<(), String> {
    if !(1..=20).contains(&input.top_k) {
        return Err("Top-K must be between 1 and 20.".to_string());
    }
    let dish_ids = dishes
        .iter()
        .map(|dish| dish.dish_id.clone())
        .collect::<HashSet<_>>();
    let ingredients = dishes
        .iter()
        .flat_map(|dish| dish.ingredients.iter().cloned())
        .collect::<HashSet<_>>();
    let tags = dishes
        .iter()
        .flat_map(|dish| dish.tags.iter().cloned())
        .collect::<HashSet<_>>();
    validate_terms(
        &input.changes.add_liked_ingredients,
        &ingredients,
        "ingredient",
    )?;
    validate_terms(
        &input.changes.remove_liked_ingredients,
        &ingredients,
        "ingredient",
    )?;
    validate_terms(
        &input.changes.add_disliked_ingredients,
        &ingredients,
        "ingredient",
    )?;
    validate_terms(
        &input.changes.remove_disliked_ingredients,
        &ingredients,
        "ingredient",
    )?;
    validate_terms(&input.changes.add_preferred_tags, &tags, "tag")?;
    validate_terms(&input.changes.remove_preferred_tags, &tags, "tag")?;
    validate_no_conflict(
        &input.changes.add_liked_ingredients,
        &input.changes.remove_liked_ingredients,
        "liked ingredient",
    )?;
    validate_no_conflict(
        &input.changes.add_disliked_ingredients,
        &input.changes.remove_disliked_ingredients,
        "disliked ingredient",
    )?;
    validate_no_conflict(
        &input.changes.add_preferred_tags,
        &input.changes.remove_preferred_tags,
        "preferred tag",
    )?;
    validate_no_conflict(
        &input.changes.add_context_dish_ids,
        &input.changes.remove_context_dish_ids,
        "context dish",
    )?;
    validate_no_conflict(
        &input.changes.add_liked_ingredients,
        &input.changes.add_disliked_ingredients,
        "ingredient preference",
    )?;
    for id in input
        .changes
        .add_context_dish_ids
        .iter()
        .chain(input.changes.remove_context_dish_ids.iter())
    {
        if !dish_ids.contains(&id.trim().to_uppercase()) {
            return Err(format!("Dish ID '{id}' does not exist."));
        }
    }
    if input.changes.simulated_coorders.len() > 20 {
        return Err("At most 20 simulated co-order pairs may be compared at once.".to_string());
    }
    for simulation in &input.changes.simulated_coorders {
        let anchor = simulation.anchor_dish_id.trim().to_uppercase();
        let candidate = simulation.candidate_dish_id.trim().to_uppercase();
        if anchor == candidate {
            return Err("A simulated co-order needs two different dishes.".to_string());
        }
        if !dish_ids.contains(&anchor) || !dish_ids.contains(&candidate) {
            return Err("A simulated co-order contains an unknown dish ID.".to_string());
        }
        if simulation.additional_order_count > 100 {
            return Err("Each simulated co-order count must be between 0 and 100.".to_string());
        }
    }
    Ok(())
}

fn validate_terms(
    values: &[String],
    vocabulary: &HashSet<String>,
    label: &str,
) -> Result<(), String> {
    for value in values {
        if !vocabulary.contains(&value.trim().to_lowercase()) {
            return Err(format!("Unknown {label} '{}'.", value.trim()));
        }
    }
    Ok(())
}

fn validate_no_conflict(add: &[String], remove: &[String], label: &str) -> Result<(), String> {
    let added = add
        .iter()
        .map(|value| value.trim().to_lowercase())
        .collect::<HashSet<_>>();
    if let Some(conflict) = remove
        .iter()
        .map(|value| value.trim().to_lowercase())
        .find(|value| added.contains(value))
    {
        return Err(format!(
            "The {label} '{conflict}' cannot be added and removed together."
        ));
    }
    Ok(())
}

fn apply_changes(baseline: &UserPreference, changes: &CounterfactualChanges) -> UserPreference {
    UserPreference {
        liked_ingredients: edit_values(
            &baseline.liked_ingredients,
            &changes.add_liked_ingredients,
            &changes.remove_liked_ingredients,
            false,
        ),
        disliked_ingredients: edit_values(
            &baseline.disliked_ingredients,
            &changes.add_disliked_ingredients,
            &changes.remove_disliked_ingredients,
            false,
        ),
        preferred_tags: edit_values(
            &baseline.preferred_tags,
            &changes.add_preferred_tags,
            &changes.remove_preferred_tags,
            false,
        ),
        selected_dish_ids: edit_values(
            &baseline.selected_dish_ids,
            &changes.add_context_dish_ids,
            &changes.remove_context_dish_ids,
            true,
        ),
        time_context: baseline.time_context.clone(),
        ranking_method: baseline.ranking_method.clone(),
    }
}

fn normalized_preference(mut preference: UserPreference) -> UserPreference {
    preference.liked_ingredients = edit_values(&[], &preference.liked_ingredients, &[], false);
    preference.disliked_ingredients =
        edit_values(&[], &preference.disliked_ingredients, &[], false);
    preference.preferred_tags = edit_values(&[], &preference.preferred_tags, &[], false);
    preference.selected_dish_ids = edit_values(&[], &preference.selected_dish_ids, &[], true);
    preference
}

fn edit_values(
    existing: &[String],
    add: &[String],
    remove: &[String],
    uppercase: bool,
) -> Vec<String> {
    let normalize = |value: &str| {
        if uppercase {
            value.trim().to_uppercase()
        } else {
            value.trim().to_lowercase()
        }
    };
    let removed = remove
        .iter()
        .map(|value| normalize(value))
        .collect::<HashSet<_>>();
    let mut values = existing
        .iter()
        .chain(add.iter())
        .map(|value| normalize(value))
        .filter(|value| !value.is_empty() && !removed.contains(value))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    values.sort();
    values
}

fn result_lookup(
    results: &[RecommendationResult],
) -> HashMap<String, (usize, &RecommendationResult)> {
    results
        .iter()
        .enumerate()
        .map(|(index, result)| (result.dish.dish_id.clone(), (index + 1, result)))
        .collect()
}

fn top_id_set(results: &[RecommendationResult], top_k: usize) -> HashSet<String> {
    results
        .iter()
        .take(top_k)
        .map(|result| result.dish.dish_id.clone())
        .collect()
}

fn dish_summaries(
    results: &[RecommendationResult],
    top_k: usize,
    exclude: Option<&HashSet<String>>,
    include_excluded: bool,
) -> Vec<DishSummary> {
    results
        .iter()
        .take(top_k)
        .enumerate()
        .filter(|(_, result)| {
            include_excluded || !exclude.is_some_and(|ids| ids.contains(&result.dish.dish_id))
        })
        .map(|(index, result)| DishSummary {
            dish_id: result.dish.dish_id.clone(),
            dish_name: result.dish.name.clone(),
            rank: index + 1,
            score: result.reranked_score,
            confidence: result.evidence.overall_confidence,
        })
        .collect()
}

fn scenario_summary(
    results: &[RecommendationResult],
    top_k: usize,
    diversity_mode: DiversityMode,
    weights: AdaptiveWeights,
) -> RecommendationScenarioSummary {
    RecommendationScenarioSummary {
        top_k,
        diversity_mode,
        result_count: results.len(),
        top_dishes: dish_summaries(results, top_k, None, true),
        adaptive_weights: weights,
    }
}

fn explicit_exclusions(dishes: &[Dish], preference: &UserPreference) -> HashSet<String> {
    dishes
        .iter()
        .filter(|dish| check_disliked_ingredients(dish, preference))
        .map(|dish| dish.dish_id.clone())
        .collect()
}

fn exclusion_changes<'a>(
    dishes: &[Dish],
    ids: impl Iterator<Item = &'a String>,
    reason: &str,
) -> Vec<ExclusionChange> {
    let names = dishes
        .iter()
        .map(|dish| (dish.dish_id.clone(), dish.name.clone()))
        .collect::<HashMap<_, _>>();
    let mut rows = ids
        .map(|id| ExclusionChange {
            dish_id: id.clone(),
            dish_name: names.get(id).cloned().unwrap_or_else(|| id.clone()),
            reason: reason.to_string(),
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.dish_id.cmp(&right.dish_id));
    rows
}

fn delta_reasons(changes: &CounterfactualChanges) -> Vec<String> {
    let mut reasons = Vec::new();
    if !changes.add_liked_ingredients.is_empty() {
        reasons.push(format!(
            "Added liked ingredient(s): {}.",
            changes.add_liked_ingredients.join(", ")
        ));
    }
    if !changes.add_disliked_ingredients.is_empty() {
        reasons.push(format!(
            "Added hard exclusion(s): {}.",
            changes.add_disliked_ingredients.join(", ")
        ));
    }
    if !changes.add_context_dish_ids.is_empty() {
        reasons.push(format!(
            "Added selected dish context: {}.",
            changes.add_context_dish_ids.join(", ")
        ));
    }
    let simulated = changes
        .simulated_coorders
        .iter()
        .map(|item| item.additional_order_count)
        .sum::<usize>();
    if simulated > 0 {
        reasons.push(format!(
            "Added {simulated} temporary co-order basket(s) for this comparison only."
        ));
    }
    if reasons.is_empty() {
        reasons.push("No effective scenario change was supplied.".to_string());
    }
    reasons
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dish(id: &str, ingredients: &[&str]) -> Dish {
        Dish {
            dish_id: id.to_string(),
            name: id.to_string(),
            ingredients: ingredients.iter().map(|value| value.to_string()).collect(),
            category: "main".to_string(),
            tags: vec![],
            image_path: None,
            image_source_url: None,
        }
    }

    #[test]
    fn disliked_change_excludes_without_mutating_orders() {
        let dishes = vec![dish("D01", &["beef"]), dish("D02", &["rice"])];
        let orders = vec![Order {
            order_id: "O1".to_string(),
            session_user_id: "U1".to_string(),
            ordered_dishes: vec!["D01".to_string(), "D02".to_string()],
            timestamp: "t".to_string(),
        }];
        let original = orders.clone();
        let result = compare_counterfactual(
            &dishes,
            &orders,
            &CounterfactualInput {
                baseline: UserPreference::default(),
                baseline_diversity_mode: DiversityMode::Balanced,
                changes: CounterfactualChanges {
                    add_disliked_ingredients: vec!["beef".to_string()],
                    ..Default::default()
                },
                top_k: 2,
            },
        )
        .unwrap();
        assert_eq!(result.exclusions_added[0].dish_id, "D01");
        assert_eq!(orders[0].order_id, original[0].order_id);
        assert_eq!(orders.len(), original.len());
    }
}
