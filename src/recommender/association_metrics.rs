use crate::models::Order;
use serde::Serialize;
use std::collections::HashSet;

/// Association-rule metrics for one selected/context dish -> candidate dish.
///
/// These values make the collaborative filtering explanation stronger for an
/// FYP demo because it can show not only "often ordered together", but also
/// support, confidence, and lift.
#[derive(Debug, Clone, Default, Serialize)]
pub struct AssociationMetric {
    pub base_dish_id: String,
    pub candidate_dish_id: String,
    pub pair_count: u32,
    pub support: f32,
    pub confidence: f32,
    pub lift: f32,
}

/// Finds the strongest association metric from any selected dish to a candidate.
///
/// When the customer has multiple selected/cart dishes, the recommender uses the
/// relationship with the highest lift, then highest confidence, then pair count.
pub fn best_association_metric(
    orders: &[Order],
    selected_dish_ids: &[String],
    candidate_dish_id: &str,
) -> Option<AssociationMetric> {
    selected_dish_ids
        .iter()
        .filter_map(|base| calculate_association_metric(orders, base, candidate_dish_id))
        .max_by(|a, b| {
            a.lift
                .total_cmp(&b.lift)
                .then_with(|| a.confidence.total_cmp(&b.confidence))
                .then_with(|| a.pair_count.cmp(&b.pair_count))
        })
}

/// Calculates support, confidence, and lift for base dish A -> candidate dish B.
///
/// support(A,B) = orders containing both A and B / total orders
/// confidence(A -> B) = orders containing both A and B / orders containing A
/// lift(A -> B) = confidence(A -> B) / support(B)
pub fn calculate_association_metric(
    orders: &[Order],
    base_dish_id: &str,
    candidate_dish_id: &str,
) -> Option<AssociationMetric> {
    let base = base_dish_id.trim().to_uppercase();
    let candidate = candidate_dish_id.trim().to_uppercase();
    if base.is_empty() || candidate.is_empty() || base == candidate || orders.is_empty() {
        return None;
    }

    let total_orders = orders.len() as f32;
    let mut base_count = 0_u32;
    let mut candidate_count = 0_u32;
    let mut pair_count = 0_u32;

    for order in orders {
        let basket = order
            .ordered_dishes
            .iter()
            .map(|dish_id| dish_id.trim().to_uppercase())
            .collect::<HashSet<_>>();
        let has_base = basket.contains(&base);
        let has_candidate = basket.contains(&candidate);

        if has_base {
            base_count += 1;
        }
        if has_candidate {
            candidate_count += 1;
        }
        if has_base && has_candidate {
            pair_count += 1;
        }
    }

    if pair_count == 0 || base_count == 0 || candidate_count == 0 {
        return None;
    }

    let support = pair_count as f32 / total_orders;
    let confidence = pair_count as f32 / base_count as f32;
    let candidate_support = candidate_count as f32 / total_orders;
    let lift = if candidate_support > 0.0 {
        confidence / candidate_support
    } else {
        0.0
    };

    Some(AssociationMetric {
        base_dish_id: base,
        candidate_dish_id: candidate,
        pair_count,
        support,
        confidence,
        lift,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order(id: &str, dishes: &[&str]) -> Order {
        Order {
            order_id: id.to_string(),
            session_user_id: "U".to_string(),
            ordered_dishes: dishes.iter().map(|dish| dish.to_string()).collect(),
            timestamp: "t".to_string(),
        }
    }

    #[test]
    fn calculates_support_confidence_and_lift() {
        let orders = vec![
            order("O1", &["D01", "D02"]),
            order("O2", &["D01", "D02"]),
            order("O3", &["D01"]),
            order("O4", &["D02"]),
        ];

        let metric = calculate_association_metric(&orders, "D01", "D02").unwrap();

        assert_eq!(metric.pair_count, 2);
        assert!((metric.support - 0.5).abs() < f32::EPSILON);
        assert!((metric.confidence - 0.6666667).abs() < 0.0001);
        assert!((metric.lift - 0.8888889).abs() < 0.0001);
    }
}
