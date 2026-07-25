use crate::models::Dish;
use serde::{Deserialize, Serialize};

/// Simple restaurant time context used for explainable rule-based boosting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeContext {
    Any,
    Breakfast,
    Lunch,
    Dinner,
    Snack,
}

impl TimeContext {
    pub fn from_label(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            "breakfast" => Self::Breakfast,
            "lunch" => Self::Lunch,
            "dinner" => Self::Dinner,
            "snack" | "dessert" | "dessert/snack" => Self::Snack,
            _ => Self::Any,
        }
    }
}

/// Calculates a small rule-based time score.
///
/// This is deliberately not ML. It uses dish category, tags, ingredients, and
/// dish name keywords so the FYP demo can explain exactly why a boost happened.
pub fn calculate_time_score(dish: &Dish, context: TimeContext) -> f32 {
    match context {
        TimeContext::Any => 0.0,
        TimeContext::Breakfast => {
            let text = searchable_text(dish);
            if text.contains("nasi lemak")
                || text.contains("roti")
                || text.contains("egg")
                || text.contains("breakfast")
            {
                1.0
            } else if dish.category == "main" {
                0.35
            } else {
                0.0
            }
        }
        TimeContext::Lunch => {
            if dish.category == "main" {
                1.0
            } else if has_any(dish, &["rice", "noodle", "noodles"]) {
                0.65
            } else {
                0.0
            }
        }
        TimeContext::Dinner => {
            if dish.category == "main" || has_any(dish, &["grilled", "spicy", "signature"]) {
                1.0
            } else {
                0.0
            }
        }
        TimeContext::Snack => {
            if dish.category == "dessert" || dish.category == "side" {
                1.0
            } else if has_any(dish, &["sweet", "snack", "kuih"]) {
                0.85
            } else {
                0.0
            }
        }
    }
}

pub fn time_explanation(dish: &Dish, context: TimeContext, score: f32) -> Option<String> {
    if score <= 0.0 || context == TimeContext::Any {
        return None;
    }

    Some(match context {
        TimeContext::Breakfast => format!("Boosted because {} fits breakfast context", dish.name),
        TimeContext::Lunch => format!("Boosted because {} fits lunch menu context", dish.name),
        TimeContext::Dinner => format!("Boosted because {} fits dinner context", dish.name),
        TimeContext::Snack => "Suggested as a dessert/snack item".to_string(),
        TimeContext::Any => unreachable!(),
    })
}

fn has_any(dish: &Dish, terms: &[&str]) -> bool {
    let text = searchable_text(dish);
    terms.iter().any(|term| text.contains(term))
}

fn searchable_text(dish: &Dish) -> String {
    format!(
        "{} {} {} {}",
        dish.name,
        dish.category,
        dish.ingredients.join(" "),
        dish.tags.join(" ")
    )
    .to_lowercase()
}
