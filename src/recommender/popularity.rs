use crate::models::Order;
use std::collections::HashMap;

/// Dish popularity counts derived from historical/completed order baskets.
///
/// This is intentionally simple: every appearance of a dish ID in an order log
/// adds one count. It gives the recommender a fallback signal when the customer
/// has not selected preferences or a cart context yet.
pub type PopularityCounts = HashMap<String, u32>;

/// Builds per-dish order frequency counts.
pub fn build_popularity_counts(orders: &[Order]) -> PopularityCounts {
    let mut counts = PopularityCounts::new();

    for order in orders {
        for dish_id in &order.ordered_dishes {
            let normalized = dish_id.trim().to_uppercase();
            if !normalized.is_empty() {
                *counts.entry(normalized).or_default() += 1;
            }
        }
    }

    counts
}

/// Converts a dish count into a normalized popularity score from 0.0 to 1.0.
///
/// The most frequent dish receives 1.0. Dishes without historical appearances
/// receive 0.0, but can still be recommended by content/time signals.
pub fn calculate_popularity_score(counts: &PopularityCounts, dish_id: &str) -> f32 {
    let max_count = counts.values().copied().max().unwrap_or(0);
    if max_count == 0 {
        return 0.0;
    }

    let count = counts
        .get(&dish_id.trim().to_uppercase())
        .copied()
        .unwrap_or(0);
    (count as f32 / max_count as f32).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popularity_score_is_normalized_by_most_frequent_dish() {
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
                ordered_dishes: vec!["D01".to_string()],
                timestamp: "t".to_string(),
            },
        ];

        let counts = build_popularity_counts(&orders);

        assert_eq!(calculate_popularity_score(&counts, "D01"), 1.0);
        assert_eq!(calculate_popularity_score(&counts, "D02"), 0.5);
        assert_eq!(calculate_popularity_score(&counts, "D99"), 0.0);
    }
}
