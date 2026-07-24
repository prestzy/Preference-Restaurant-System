//! Explainable snapshots showing how completed orders change recommendation data.
//!
//! This module is deliberately independent from web state and persistence. It
//! compares an order history before and after one real completed basket, then
//! returns factual popularity, pair-association, and co-order-rank changes.

use crate::models::{Dish, Order};
use crate::recommender::association_metrics::calculate_association_metric;
use crate::recommender::collaborative_filter::build_co_order_matrix;
use crate::recommender::popularity::build_popularity_counts;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// One append-only explanation event keyed by the durable historical order ID.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecommendationLearningEvent {
    pub event_id: String,
    pub historical_order_id: String,
    pub completed_at: String,
    pub dish_ids: Vec<String>,
    pub total_orders_before: usize,
    pub total_orders_after: usize,
    pub popularity_changes: Vec<PopularityChange>,
    pub pair_changes: Vec<PairChange>,
    pub rank_changes: Vec<AnchorRankChange>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PopularityChange {
    pub dish_id: String,
    pub dish_name: String,
    pub before_count: usize,
    pub after_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PairChange {
    pub dish_a_id: String,
    pub dish_a_name: String,
    pub dish_b_id: String,
    pub dish_b_name: String,
    pub before_count: usize,
    pub after_count: usize,
    pub support_before: f32,
    pub support_after: f32,
    pub confidence_a_to_b_before: f32,
    pub confidence_a_to_b_after: f32,
    pub lift_before: f32,
    pub lift_after: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnchorRankChange {
    pub anchor_dish_id: String,
    pub candidate_dish_id: String,
    pub before_rank: Option<usize>,
    pub after_rank: Option<usize>,
}

/// Builds one learning event without changing either input collection.
pub fn build_learning_event(
    dishes: &[Dish],
    before_orders: &[Order],
    completed_order: &Order,
) -> RecommendationLearningEvent {
    let mut dish_ids = completed_order
        .ordered_dishes
        .iter()
        .map(|id| id.trim().to_uppercase())
        .filter(|id| !id.is_empty())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    dish_ids.sort();

    let names = dishes
        .iter()
        .map(|dish| (dish.dish_id.clone(), dish.name.clone()))
        .collect::<HashMap<_, _>>();
    let before_popularity = build_popularity_counts(before_orders);
    let mut after_orders = before_orders.to_vec();
    after_orders.push(completed_order.clone());
    let after_popularity = build_popularity_counts(&after_orders);

    let popularity_changes = dish_ids
        .iter()
        .map(|dish_id| PopularityChange {
            dish_id: dish_id.clone(),
            dish_name: dish_name(&names, dish_id),
            before_count: before_popularity.get(dish_id).copied().unwrap_or(0) as usize,
            after_count: after_popularity.get(dish_id).copied().unwrap_or(0) as usize,
        })
        .collect::<Vec<_>>();

    let mut pair_changes = Vec::new();
    for left in 0..dish_ids.len() {
        for right in (left + 1)..dish_ids.len() {
            let a = &dish_ids[left];
            let b = &dish_ids[right];
            let before = calculate_association_metric(before_orders, a, b).unwrap_or_default();
            let after = calculate_association_metric(&after_orders, a, b).unwrap_or_default();
            pair_changes.push(PairChange {
                dish_a_id: a.clone(),
                dish_a_name: dish_name(&names, a),
                dish_b_id: b.clone(),
                dish_b_name: dish_name(&names, b),
                before_count: before.pair_count as usize,
                after_count: after.pair_count as usize,
                support_before: before.support,
                support_after: after.support,
                confidence_a_to_b_before: before.confidence,
                confidence_a_to_b_after: after.confidence,
                lift_before: before.lift,
                lift_after: after.lift,
            });
        }
    }

    let before_matrix = build_co_order_matrix(before_orders);
    let after_matrix = build_co_order_matrix(&after_orders);
    let catalogue_ids = dishes
        .iter()
        .map(|dish| dish.dish_id.clone())
        .collect::<Vec<_>>();
    let mut rank_changes = Vec::new();
    for anchor in &dish_ids {
        let before_ranks = co_order_ranks(&before_matrix, anchor, &catalogue_ids);
        let after_ranks = co_order_ranks(&after_matrix, anchor, &catalogue_ids);
        for candidate in dish_ids.iter().filter(|candidate| *candidate != anchor) {
            let before_rank = before_ranks.get(candidate).copied();
            let after_rank = after_ranks.get(candidate).copied();
            if before_rank != after_rank {
                rank_changes.push(AnchorRankChange {
                    anchor_dish_id: anchor.clone(),
                    candidate_dish_id: candidate.clone(),
                    before_rank,
                    after_rank,
                });
            }
        }
    }
    rank_changes.sort_by(|left, right| {
        left.anchor_dish_id
            .cmp(&right.anchor_dish_id)
            .then_with(|| left.candidate_dish_id.cmp(&right.candidate_dish_id))
    });

    let summary = if pair_changes.is_empty() {
        format!(
            "This order increased popularity evidence for {} dish(es); no co-order pair was created.",
            popularity_changes.len()
        )
    } else if rank_changes.is_empty() {
        format!(
            "This order strengthened {} co-order relationship(s). No tracked pair rank changed.",
            pair_changes.len()
        )
    } else {
        format!(
            "This order strengthened {} co-order relationship(s) and changed {} tracked pair rank(s).",
            pair_changes.len(),
            rank_changes.len()
        )
    };

    RecommendationLearningEvent {
        event_id: format!("LEARN-{}", completed_order.order_id),
        historical_order_id: completed_order.order_id.clone(),
        completed_at: completed_order.timestamp.clone(),
        dish_ids,
        total_orders_before: before_orders.len(),
        total_orders_after: after_orders.len(),
        popularity_changes,
        pair_changes,
        rank_changes,
        summary,
    }
}

/// Replays durable orders in stable chronological order to recover the timeline.
pub fn rebuild_learning_timeline(
    historical_orders: &[Order],
    dishes: &[Dish],
) -> Vec<RecommendationLearningEvent> {
    let mut ordered = historical_orders.to_vec();
    ordered.sort_by(|left, right| {
        left.timestamp
            .cmp(&right.timestamp)
            .then_with(|| left.order_id.cmp(&right.order_id))
    });
    let mut prior = Vec::new();
    let mut events = Vec::with_capacity(ordered.len());
    for order in ordered {
        events.push(build_learning_event(dishes, &prior, &order));
        prior.push(order);
    }
    events
}

fn dish_name(names: &HashMap<String, String>, dish_id: &str) -> String {
    names
        .get(dish_id)
        .cloned()
        .unwrap_or_else(|| dish_id.to_string())
}

fn co_order_ranks(
    matrix: &HashMap<String, HashMap<String, u32>>,
    anchor: &str,
    catalogue_ids: &[String],
) -> HashMap<String, usize> {
    let related = matrix.get(anchor);
    let mut rows = catalogue_ids
        .iter()
        .filter(|candidate| candidate.as_str() != anchor)
        .filter_map(|candidate| {
            let count = related
                .and_then(|values| values.get(candidate))
                .copied()
                .unwrap_or(0);
            (count > 0).then(|| (candidate.clone(), count))
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    rows.into_iter()
        .enumerate()
        .map(|(index, (dish_id, _))| (dish_id, index + 1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dish(id: &str, name: &str) -> Dish {
        Dish {
            dish_id: id.to_string(),
            name: name.to_string(),
            ingredients: vec![],
            category: "main".to_string(),
            tags: vec![],
            image_path: None,
            image_source_url: None,
        }
    }

    fn order(id: &str, dishes: &[&str]) -> Order {
        Order {
            order_id: id.to_string(),
            session_user_id: "U1".to_string(),
            ordered_dishes: dishes.iter().map(|id| id.to_string()).collect(),
            timestamp: id.to_string(),
        }
    }

    #[test]
    fn completed_order_records_unique_popularity_and_pair_deltas() {
        let dishes = vec![dish("D01", "One"), dish("D02", "Two")];
        let event = build_learning_event(&dishes, &[], &order("O001", &["D01", "D01", "D02"]));
        assert_eq!(event.popularity_changes.len(), 2);
        assert_eq!(event.pair_changes.len(), 1);
        assert_eq!(event.pair_changes[0].after_count, 1);
        assert_eq!(event.total_orders_after, 1);
    }

    #[test]
    fn rebuild_is_deterministic_and_chronological() {
        let dishes = vec![dish("D01", "One"), dish("D02", "Two")];
        let orders = vec![order("O002", &["D01"]), order("O001", &["D01", "D02"])];
        let first = rebuild_learning_timeline(&orders, &dishes);
        let second = rebuild_learning_timeline(&orders, &dishes);
        assert_eq!(first, second);
        assert_eq!(first[0].historical_order_id, "O001");
    }
}
