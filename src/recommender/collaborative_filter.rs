use crate::models::Order;
use std::collections::{HashMap, HashSet};

/// Item-item co-order matrix.
///
/// The outer key is a dish ID such as `D01`. The inner map stores other dishes
/// and how many historical orders contained both dishes together.
pub type CoOrderMatrix = HashMap<String, HashMap<String, u32>>;

/// Builds an item-item co-order frequency matrix from historical orders.
///
/// For every order, each pair of dishes receives +1 in both directions:
/// `D01 -> D03` and `D03 -> D01`. This symmetric matrix is enough for a
/// lightweight collaborative filtering demo without any heavy ML library.
pub fn build_co_order_matrix(orders: &[Order]) -> CoOrderMatrix {
    let mut matrix: CoOrderMatrix = HashMap::new();

    for order in orders {
        // A set prevents the same dish ID from being counted twice if it appears
        // more than once in a single order row.
        let unique_dishes: Vec<String> = order
            .ordered_dishes
            .iter()
            .map(|dish_id| dish_id.trim().to_uppercase())
            .filter(|dish_id| !dish_id.is_empty())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        for i in 0..unique_dishes.len() {
            for j in (i + 1)..unique_dishes.len() {
                let dish_a = &unique_dishes[i];
                let dish_b = &unique_dishes[j];

                *matrix
                    .entry(dish_a.clone())
                    .or_default()
                    .entry(dish_b.clone())
                    .or_default() += 1;

                *matrix
                    .entry(dish_b.clone())
                    .or_default()
                    .entry(dish_a.clone())
                    .or_default() += 1;
            }
        }
    }

    matrix
}

/// Calculates a co-order score with a request-scoped normalisation value.
///
/// The hybrid recommender computes `max_related_count` once per request and
/// reuses it for every candidate. The public wrapper above remains convenient
/// for isolated experiment calculations.
pub(crate) fn calculate_co_order_score_with_max(
    matrix: &CoOrderMatrix,
    selected_dish_ids: &[String],
    candidate_dish_id: &str,
    max_related_count: u32,
) -> f32 {
    if selected_dish_ids.is_empty() || selected_dish_ids.iter().any(|id| id == candidate_dish_id) {
        return 0.0;
    }

    let candidate_count = co_order_count(matrix, selected_dish_ids, candidate_dish_id);

    if max_related_count == 0 {
        0.0
    } else {
        (candidate_count as f32 / max_related_count as f32).clamp(0.0, 1.0)
    }
}

/// Lists selected dishes that have a historical co-order relationship with a candidate.
///
/// This function is used only for explanations, not for scoring.
pub fn related_selected_dishes(
    matrix: &CoOrderMatrix,
    selected_dish_ids: &[String],
    candidate_dish_id: &str,
) -> Vec<String> {
    let mut related = selected_dish_ids
        .iter()
        .filter(|selected_id| {
            matrix
                .get(*selected_id)
                .and_then(|related_dishes| related_dishes.get(candidate_dish_id))
                .copied()
                .unwrap_or(0)
                > 0
        })
        .cloned()
        .collect::<Vec<_>>();

    related.sort();
    related
}

/// Counts how often a candidate appeared with the selected dishes.
fn co_order_count(
    matrix: &CoOrderMatrix,
    selected_dish_ids: &[String],
    candidate_dish_id: &str,
) -> u32 {
    selected_dish_ids
        .iter()
        .map(|selected_id| {
            matrix
                .get(selected_id)
                .and_then(|related_dishes| related_dishes.get(candidate_dish_id))
                .copied()
                .unwrap_or(0)
        })
        .sum()
}

/// Finds the strongest candidate count available for the selected dishes.
///
/// This value is used as the normalisation denominator in
/// `calculate_co_order_score`.
pub(crate) fn strongest_related_count(matrix: &CoOrderMatrix, selected_dish_ids: &[String]) -> u32 {
    let selected_set: HashSet<&String> = selected_dish_ids.iter().collect();
    let mut totals: HashMap<String, u32> = HashMap::new();

    for selected_id in selected_dish_ids {
        if let Some(related_dishes) = matrix.get(selected_id) {
            for (candidate_id, count) in related_dishes {
                if !selected_set.contains(candidate_id) {
                    *totals.entry(candidate_id.clone()).or_default() += *count;
                }
            }
        }
    }

    totals.values().copied().max().unwrap_or(0)
}
