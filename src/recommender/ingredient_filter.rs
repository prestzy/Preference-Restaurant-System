use crate::models::{Dish, UserPreference};
use std::collections::HashSet;

/// Checks whether a dish contains any ingredient/tag/category the user avoids.
///
/// In this prototype, disliked terms are used as a hard exclusion rule. The
/// field is still named `disliked_ingredients` for backward compatibility with
/// the original preference flow, but the Smart Menu Assistant may also place negated tags
/// such as `spicy` here so phrases like "no spicy" can be respected.
pub fn check_disliked_ingredients(dish: &Dish, preference: &UserPreference) -> bool {
    !matched_disliked_ingredients(dish, preference).is_empty()
}

/// Calculates the ingredient-based filtering score for one dish.
///
/// Formula used for the prototype:
/// liked ingredient match ratio = matched liked ingredients / total ingredients
/// preferred tag bonus = 0.15 per matched tag, capped by the final 1.0 limit
///
/// The score is intentionally simple and transparent. It is not meant to be a
/// production nutrition or taste model; it is a lightweight content-based signal
/// for demonstrating preference-driven ordering.
pub fn calculate_ingredient_score(dish: &Dish, preference: &UserPreference) -> f32 {
    if check_disliked_ingredients(dish, preference) {
        return 0.0;
    }

    let matched_ingredients = matched_liked_ingredients(dish, preference).len() as f32;
    let total_ingredients = dish.ingredients.len().max(1) as f32;
    let ingredient_ratio = matched_ingredients / total_ingredients;

    let matched_tags = matched_preferred_tags(dish, preference).len() as f32;
    let tag_bonus = matched_tags * 0.15;

    (ingredient_ratio + tag_bonus).min(1.0)
}

/// Builds a human-readable explanation for the ingredient filtering result.
///
/// The web interface displays this text beside each recommendation so the user can see
/// why the dish was ranked. Explainability is one of the main goals of this FYP
/// prototype.
pub fn build_ingredient_explanation(dish: &Dish, preference: &UserPreference) -> String {
    if check_disliked_ingredients(dish, preference) {
        return format!(
            "excluded because it contains disliked ingredient(s): {}",
            matched_disliked_ingredients(dish, preference).join(", ")
        );
    }

    let matched_ingredients = matched_liked_ingredients(dish, preference);
    let matched_tags = matched_preferred_tags(dish, preference);
    let mut parts = Vec::new();

    if !matched_ingredients.is_empty() {
        parts.push(format!(
            "contains preferred ingredient(s): {}",
            matched_ingredients.join(", ")
        ));
    }

    if !matched_tags.is_empty() {
        parts.push(format!(
            "matches preferred tag(s): {}",
            matched_tags.join(", ")
        ));
    }

    if parts.is_empty() {
        "No direct ingredient or tag preference match.".to_string()
    } else {
        parts.join("; ")
    }
}

/// Returns the liked ingredients that match a dish.
///
/// A `HashSet` removes duplicates so the same user term is only explained once.
pub fn matched_liked_ingredients(dish: &Dish, preference: &UserPreference) -> Vec<String> {
    let mut matches = HashSet::new();

    for liked in &preference.liked_ingredients {
        if dish
            .ingredients
            .iter()
            .any(|ingredient| term_matches(ingredient, liked))
        {
            matches.insert(liked.clone());
        }
    }

    sorted_values(matches)
}

/// Returns preferred tag/category values that match a dish.
pub fn matched_preferred_tags(dish: &Dish, preference: &UserPreference) -> Vec<String> {
    let mut matches = HashSet::new();

    for preferred_tag in &preference.preferred_tags {
        if dish.tags.iter().any(|tag| term_matches(tag, preferred_tag))
            || term_matches(&dish.category, preferred_tag)
        {
            matches.insert(preferred_tag.clone());
        }
    }

    sorted_values(matches)
}

/// Returns disliked ingredient/tag/category terms that match a dish.
///
/// Recommended dishes should normally have an empty result here because the
/// hybrid recommender excludes disliked dishes before ranking. Keeping this as
/// a public helper lets the UI explain that exclusion explicitly.
pub fn matched_disliked_ingredients(dish: &Dish, preference: &UserPreference) -> Vec<String> {
    let mut matches = HashSet::new();

    for disliked in &preference.disliked_ingredients {
        if dish
            .ingredients
            .iter()
            .any(|ingredient| term_matches(ingredient, disliked))
            || dish.tags.iter().any(|tag| term_matches(tag, disliked))
            || term_matches(&dish.category, disliked)
        {
            matches.insert(disliked.clone());
        }
    }

    sorted_values(matches)
}

/// Compares preference terms against ingredient/tag text.
///
/// The exact string comparison handles values like `coconut milk`. The token
/// comparison handles compound menu data such as `chicken curry` when a user
/// enters only `chicken`, while still avoiding a bad match between `egg` and
/// `eggplant`.
fn term_matches(candidate: &str, preference_term: &str) -> bool {
    if candidate == preference_term {
        return true;
    }

    let candidate_words = words(candidate);
    let preference_words = words(preference_term);

    !preference_words.is_empty()
        && preference_words.iter().all(|word| {
            candidate_words
                .iter()
                .any(|candidate_word| candidate_word == word)
        })
}

/// Splits a string into lowercase alphanumeric words for safer matching.
fn words(value: &str) -> Vec<String> {
    value
        .split(|character: char| !character.is_alphanumeric())
        .map(str::trim)
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Converts a set into a sorted vector so explanations are deterministic.
fn sorted_values(values: HashSet<String>) -> Vec<String> {
    let mut values: Vec<String> = values.into_iter().collect();
    values.sort();
    values
}
