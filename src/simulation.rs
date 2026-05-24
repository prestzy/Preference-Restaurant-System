//! Order simulation service.
//!
//! Simulation is an admin/demo feature, not part of normal customer browsing.
//! It lets evaluators add new in-memory order behaviour and immediately observe
//! how collaborative filtering changes.

use crate::data_loader::append_order_to_csv;
use crate::models::Order;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

/// Result of adding a simulated order.
#[derive(Debug, Clone)]
pub struct SimulationOutcome {
    pub order: Order,
    pub persisted_to_csv: bool,
    pub csv_error: Option<String>,
}

/// Parses, validates, stores, and optionally persists one simulated order.
pub fn add_simulated_order(
    orders: &mut Vec<Order>,
    known_dish_ids: &HashSet<String>,
    raw_dish_ids: &str,
    append_to_csv_enabled: bool,
    csv_path: &str,
) -> Option<SimulationOutcome> {
    let ordered_dishes = parse_dish_ids(raw_dish_ids)
        .into_iter()
        .filter(|dish_id| known_dish_ids.contains(dish_id))
        .collect::<Vec<_>>();

    if ordered_dishes.is_empty() {
        return None;
    }

    let order = Order {
        order_id: format!("SIM{:03}", orders.len() + 1),
        session_user_id: "SIM_USER".to_string(),
        ordered_dishes,
        timestamp: prototype_timestamp(),
    };

    orders.push(order.clone());

    let csv_error = if append_to_csv_enabled {
        append_order_to_csv(&order, csv_path)
            .err()
            .map(|error| error.to_string())
    } else {
        None
    };
    let persisted_to_csv = append_to_csv_enabled && csv_error.is_none();

    Some(SimulationOutcome {
        order,
        persisted_to_csv,
        csv_error,
    })
}

/// Parses manually entered dish IDs for the admin/demo simulation tool.
pub fn parse_dish_ids(raw_dish_ids: &str) -> Vec<String> {
    raw_dish_ids
        .split([',', ';', '|', '\n', '\r', '\t', ' '])
        .map(|dish_id| dish_id.trim().to_uppercase())
        .filter(|dish_id| !dish_id.is_empty())
        .collect()
}

/// Produces a lightweight timestamp string for simulated orders.
fn prototype_timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();

    format!("simulated-{seconds}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulated_order_updates_in_memory_orders() {
        let known_dish_ids = ["D01".to_string(), "D02".to_string()]
            .into_iter()
            .collect::<HashSet<_>>();
        let mut orders = Vec::new();

        let outcome = add_simulated_order(
            &mut orders,
            &known_dish_ids,
            "D01, D99, d02",
            false,
            "unused",
        )
        .expect("valid dish IDs should create an order");

        assert_eq!(orders.len(), 1);
        assert_eq!(outcome.order.ordered_dishes, vec!["D01", "D02"]);
        assert!(!outcome.persisted_to_csv);
        assert!(outcome.csv_error.is_none());
    }
}
