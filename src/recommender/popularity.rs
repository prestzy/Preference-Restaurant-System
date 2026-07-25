use crate::models::Order;
use std::collections::{HashMap, HashSet};

/// Dish popularity counts derived from historical/completed order baskets.
///
/// This is intentionally simple: every basket containing a dish adds one count.
/// Duplicate dish IDs inside one basket are ignored because evidence maturity
/// represents independent order baskets, not item quantity.
pub type PopularityCounts = HashMap<String, u32>;

/// Builds per-dish order frequency counts.
pub fn build_popularity_counts(orders: &[Order]) -> PopularityCounts {
    let mut counts = PopularityCounts::new();

    for order in orders {
        let unique_dishes = order
            .ordered_dishes
            .iter()
            .map(|dish_id| dish_id.trim().to_uppercase())
            .filter(|dish_id| !dish_id.is_empty())
            .collect::<HashSet<_>>();
        for dish_id in unique_dishes {
            *counts.entry(dish_id).or_default() += 1;
        }
    }

    counts
}

/// Returns the strongest basket frequency used to normalize all candidates.
pub(crate) fn maximum_popularity_count(counts: &PopularityCounts) -> u32 {
    counts.values().copied().max().unwrap_or(0)
}

/// Calculates popularity using a request-scoped maximum count.
pub(crate) fn calculate_popularity_score_with_max(
    counts: &PopularityCounts,
    dish_id: &str,
    max_count: u32,
) -> f32 {
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

        let maximum = maximum_popularity_count(&counts);
        assert_eq!(
            calculate_popularity_score_with_max(&counts, "D01", maximum),
            1.0
        );
        assert_eq!(
            calculate_popularity_score_with_max(&counts, "D02", maximum),
            0.5
        );
        assert_eq!(
            calculate_popularity_score_with_max(&counts, "D99", maximum),
            0.0
        );
    }
}
