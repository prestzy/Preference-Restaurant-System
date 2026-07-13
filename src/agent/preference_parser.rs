use crate::models::{Dish, UserPreference};
use serde::Serialize;
use std::collections::HashSet;

/// Parsed output from the Smart Menu Assistant prompt.
///
/// The assistant keeps both machine-readable preference vectors and a
/// human-readable summary. This makes the feature useful for customers while
/// staying explainable for an FYP demonstration.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ParsedPreference {
    pub liked_ingredients: Vec<String>,
    pub disliked_ingredients: Vec<String>,
    pub preferred_tags: Vec<String>,
    pub preferred_categories: Vec<String>,
    pub matched_dish_names: Vec<String>,
    pub avoidance_terms: Vec<String>,
    pub understood_summary: String,
}

impl ParsedPreference {
    /// Converts parsed natural-language intent into recommender input.
    ///
    /// Categories are appended to `preferred_tags` because the existing
    /// content-based scorer treats tags/categories as small explainable boosts.
    /// This avoids a separate recommendation API just for the assistant.
    pub fn to_user_preference(&self, selected_dish_ids: Vec<String>) -> UserPreference {
        let mut preferred_tags = self.preferred_tags.clone();
        preferred_tags.extend(self.preferred_categories.clone());
        preferred_tags.sort();
        preferred_tags.dedup();

        UserPreference {
            liked_ingredients: self.liked_ingredients.clone(),
            disliked_ingredients: self.disliked_ingredients.clone(),
            preferred_tags,
            selected_dish_ids,
            time_context: None,
            ranking_method: Some("hybrid".to_string()),
        }
    }
}

/// Parses a customer prompt using only vocabulary found in the current menu.
///
/// This rule-based parser deliberately avoids external LLM/API dependencies.
/// It supports simple positive phrases ("chicken and rice") and simple
/// negation ("no beef", "without peanuts", "avoid seafood"). Unknown terms are
/// ignored so the recommender never invents ingredients or tags.
pub fn parse_preference_prompt(prompt: &str, dishes: &[Dish]) -> ParsedPreference {
    let text = normalize_text(prompt);
    let vocabulary = AssistantVocabulary::from_dishes(dishes);

    let mut liked_ingredients = HashSet::new();
    let mut disliked_ingredients = HashSet::new();
    let mut preferred_tags = HashSet::new();
    let mut preferred_categories = HashSet::new();
    let mut matched_dish_names = HashSet::new();
    let mut avoidance_terms = HashSet::new();

    for ingredient in &vocabulary.ingredients {
        if contains_term(&text, ingredient) {
            if is_negated(&text, ingredient) {
                disliked_ingredients.insert(ingredient.clone());
                avoidance_terms.insert(ingredient.clone());
            } else {
                liked_ingredients.insert(ingredient.clone());
            }
        }
    }

    for tag in &vocabulary.tags {
        if contains_term(&text, tag) {
            if is_negated(&text, tag) {
                // The recommender's disliked field is used as an avoidance
                // vector. The ingredient filter also checks tags/categories, so
                // phrases like "no spicy" are respected without adding a new UI
                // field just for disliked tags.
                disliked_ingredients.insert(tag.clone());
                avoidance_terms.insert(tag.clone());
            } else {
                preferred_tags.insert(tag.clone());
            }
        }
    }

    for category in &vocabulary.categories {
        if contains_term(&text, category) {
            if is_negated(&text, category) {
                avoidance_terms.insert(category.clone());
            } else {
                preferred_categories.insert(category.clone());
            }
        }
    }

    for dish_name in &vocabulary.dish_names {
        if contains_term(&text, dish_name) && !is_negated(&text, dish_name) {
            matched_dish_names.insert(dish_name.clone());
        }
    }

    let mut parsed = ParsedPreference {
        liked_ingredients: sorted(liked_ingredients),
        disliked_ingredients: sorted(disliked_ingredients),
        preferred_tags: sorted(preferred_tags),
        preferred_categories: sorted(preferred_categories),
        matched_dish_names: sorted(matched_dish_names),
        avoidance_terms: sorted(avoidance_terms),
        understood_summary: String::new(),
    };
    parsed.understood_summary = build_summary(&parsed);
    parsed
}

#[derive(Debug)]
struct AssistantVocabulary {
    ingredients: Vec<String>,
    tags: Vec<String>,
    categories: Vec<String>,
    dish_names: Vec<String>,
}

impl AssistantVocabulary {
    fn from_dishes(dishes: &[Dish]) -> Self {
        let mut ingredients = HashSet::new();
        let mut tags = HashSet::new();
        let mut categories = HashSet::new();
        let mut dish_names = HashSet::new();

        for dish in dishes {
            ingredients.extend(dish.ingredients.iter().cloned());
            tags.extend(dish.tags.iter().cloned());
            categories.insert(dish.category.to_lowercase());
            dish_names.insert(dish.name.to_lowercase());
        }

        Self {
            ingredients: sorted(ingredients),
            tags: sorted(tags),
            categories: sorted(categories),
            dish_names: sorted(dish_names),
        }
    }
}

fn build_summary(parsed: &ParsedPreference) -> String {
    let mut parts = Vec::new();

    if !parsed.liked_ingredients.is_empty() {
        parts.push(format!(
            "prefer ingredient(s): {}",
            parsed.liked_ingredients.join(", ")
        ));
    }
    if !parsed.preferred_tags.is_empty() {
        parts.push(format!(
            "prefer tag(s): {}",
            parsed.preferred_tags.join(", ")
        ));
    }
    if !parsed.preferred_categories.is_empty() {
        parts.push(format!(
            "prefer category: {}",
            parsed.preferred_categories.join(", ")
        ));
    }
    if !parsed.disliked_ingredients.is_empty() {
        parts.push(format!("avoid: {}", parsed.disliked_ingredients.join(", ")));
    }

    if parts.is_empty() {
        "We could not match specific menu terms, so recommendations use popularity and ordering patterns.".to_string()
    } else {
        format!("We understood that you {}.", parts.join("; "))
    }
}

fn normalize_text(value: &str) -> String {
    format!(
        " {} ",
        value
            .to_lowercase()
            .replace(['.', ',', ';', ':', '!', '?', '(', ')'], " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    )
}

fn contains_term(text: &str, term: &str) -> bool {
    let normalized_term = normalize_text(term);
    text.contains(normalized_term.trim())
}

fn is_negated(text: &str, term: &str) -> bool {
    let term = normalize_text(term);
    let term = term.trim();
    [
        format!(" no {term} "),
        format!(" without {term} "),
        format!(" avoid {term} "),
        format!(" not {term} "),
        format!(" don't want {term} "),
        format!(" dont want {term} "),
        format!(" do not want {term} "),
    ]
    .iter()
    .any(|pattern| text.contains(pattern))
}

fn sorted(values: HashSet<String>) -> Vec<String> {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort();
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dishes() -> Vec<Dish> {
        vec![
            Dish {
                dish_id: "D01".to_string(),
                name: "Nasi Lemak".to_string(),
                ingredients: vec!["rice".to_string(), "egg".to_string(), "sambal".to_string()],
                category: "main".to_string(),
                tags: vec!["spicy".to_string(), "signature".to_string()],
                image_path: None,
                image_source_url: None,
            },
            Dish {
                dish_id: "D02".to_string(),
                name: "Beef Satay".to_string(),
                ingredients: vec!["beef".to_string(), "peanuts".to_string()],
                category: "main".to_string(),
                tags: vec!["grilled".to_string()],
                image_path: None,
                image_source_url: None,
            },
            Dish {
                dish_id: "D03".to_string(),
                name: "Kuih".to_string(),
                ingredients: vec!["coconut milk".to_string(), "sugar".to_string()],
                category: "dessert".to_string(),
                tags: vec!["sweet".to_string()],
                image_path: None,
                image_source_url: None,
            },
        ]
    }

    #[test]
    fn extracts_liked_ingredient_and_preferred_tag() {
        let parsed = parse_preference_prompt("I want spicy rice", &dishes());

        assert_eq!(parsed.liked_ingredients, vec!["rice"]);
        assert_eq!(parsed.preferred_tags, vec!["spicy"]);
    }

    #[test]
    fn extracts_disliked_ingredient_from_simple_negation() {
        let parsed = parse_preference_prompt("I want something spicy but no beef", &dishes());

        assert_eq!(parsed.disliked_ingredients, vec!["beef"]);
        assert_eq!(parsed.preferred_tags, vec!["spicy"]);
    }

    #[test]
    fn extracts_preferred_category() {
        let parsed = parse_preference_prompt("I want dessert, no peanuts", &dishes());

        assert_eq!(parsed.preferred_categories, vec!["dessert"]);
        assert_eq!(parsed.disliked_ingredients, vec!["peanuts"]);
    }

    #[test]
    fn ignores_unknown_terms_outside_menu_vocabulary() {
        let parsed = parse_preference_prompt("I want truffle pasta", &dishes());

        assert!(parsed.liked_ingredients.is_empty());
        assert!(parsed.preferred_tags.is_empty());
        assert!(parsed.preferred_categories.is_empty());
    }
}
