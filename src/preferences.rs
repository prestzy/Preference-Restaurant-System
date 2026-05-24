//! Preference option extraction from dish data.
//!
//! The GUI should not ask users to guess valid ingredient or tag names. This
//! module derives the available selectable options from the loaded CSV dataset,
//! keeping the extraction logic separate from rendering code.

use crate::models::Dish;
use std::collections::BTreeSet;

/// Selectable preference options generated from the menu dataset.
#[derive(Debug, Clone, Default)]
pub struct PreferenceOptions {
    pub ingredients: Vec<String>,
    pub tags: Vec<String>,
}

/// Extracts unique, sorted ingredient and tag options from all loaded dishes.
///
/// `BTreeSet` is used because it removes duplicates and keeps values sorted.
/// The dish loader already lowercases ingredients and tags, but this function
/// normalizes again so it remains correct if future code builds dishes directly.
pub fn extract_preference_options(dishes: &[Dish]) -> PreferenceOptions {
    let mut ingredients = BTreeSet::new();
    let mut tags = BTreeSet::new();

    for dish in dishes {
        for ingredient in &dish.ingredients {
            if let Some(normalized) = normalize_option(ingredient) {
                ingredients.insert(normalized);
            }
        }

        for tag in &dish.tags {
            if let Some(normalized) = normalize_option(tag) {
                tags.insert(normalized);
            }
        }
    }

    PreferenceOptions {
        ingredients: ingredients.into_iter().collect(),
        tags: tags.into_iter().collect(),
    }
}

/// Normalizes one selectable option.
///
/// Options are trimmed and lowercased so `Chicken`, ` chicken `, and `chicken`
/// appear as one clear choice in the GUI.
fn normalize_option(value: &str) -> Option<String> {
    let normalized = value.trim().to_lowercase();
    (!normalized.is_empty()).then_some(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_unique_sorted_options_from_dishes() {
        let dishes = vec![
            Dish {
                dish_id: "D01".to_string(),
                name: "A".to_string(),
                ingredients: vec!["Rice".to_string(), " chicken ".to_string()],
                category: "main".to_string(),
                tags: vec!["Spicy".to_string()],
            },
            Dish {
                dish_id: "D02".to_string(),
                name: "B".to_string(),
                ingredients: vec!["rice".to_string(), "egg".to_string()],
                category: "main".to_string(),
                tags: vec!["signature".to_string(), "spicy".to_string()],
            },
        ];

        let options = extract_preference_options(&dishes);

        assert_eq!(options.ingredients, vec!["chicken", "egg", "rice"]);
        assert_eq!(options.tags, vec!["signature", "spicy"]);
    }
}
