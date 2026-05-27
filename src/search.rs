//! Menu search and filtering utilities.
//!
//! This module is deliberately separate from egui rendering code. The GUI can
//! ask for parsed filter terms and matching dishes, while tests can verify the
//! search behaviour without constructing any desktop UI.

use crate::models::Dish;

/// Determines how multiple search terms are combined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchMode {
    /// A dish is shown if at least one search term matches it.
    Any,
    /// A dish is shown only if every search term matches it.
    All,
}

impl MatchMode {
    /// Human-readable label used by the GUI segmented control.
    pub fn label(self) -> &'static str {
        match self {
            Self::Any => "Match Any",
            Self::All => "Match All",
        }
    }
}

/// Parsed menu search filter.
///
/// The raw search field can contain comma, semicolon, pipe, or newline-separated
/// terms. Parsed terms are lowercase and empty values are removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchFilter {
    pub terms: Vec<String>,
    pub mode: MatchMode,
}

impl SearchFilter {
    /// Builds a search filter from raw user input and a selected match mode.
    pub fn parse(raw_input: &str, mode: MatchMode) -> Self {
        Self {
            terms: parse_filter_terms(raw_input),
            mode,
        }
    }

    /// Returns true if no active terms are present.
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }
}

/// Parses reusable multi-term filter text.
///
/// Delimiters intentionally include common characters users naturally type when
/// listing criteria: comma, semicolon, pipe, newline, and tab. Spaces are not
/// delimiters because menu values such as `coconut milk` and `nasi lemak` should
/// remain searchable as phrases.
pub fn parse_filter_terms(raw_input: &str) -> Vec<String> {
    raw_input
        .split([',', ';', '|', '\n', '\r', '\t'])
        .map(|term| term.trim().to_lowercase())
        .filter(|term| !term.is_empty())
        .collect()
}

/// Returns dishes that match the supplied search filter.
pub fn filter_dishes<'a>(dishes: &'a [Dish], filter: &SearchFilter) -> Vec<&'a Dish> {
    dishes
        .iter()
        .filter(|dish| dish_matches_filter(dish, filter))
        .collect()
}

/// Checks whether one dish satisfies the complete search filter.
pub fn dish_matches_filter(dish: &Dish, filter: &SearchFilter) -> bool {
    if filter.is_empty() {
        return true;
    }

    match filter.mode {
        MatchMode::Any => filter
            .terms
            .iter()
            .any(|term| dish_matches_single_term(dish, term)),
        MatchMode::All => filter
            .terms
            .iter()
            .all(|term| dish_matches_single_term(dish, term)),
    }
}

/// Checks whether one search term matches a dish.
///
/// Search covers the user-facing fields stakeholders expect: dish ID, dish
/// name, category, ingredients, and tags.
pub fn dish_matches_single_term(dish: &Dish, term: &str) -> bool {
    let term = term.trim().to_lowercase();
    if term.is_empty() {
        return true;
    }

    dish.dish_id.to_lowercase().contains(&term)
        || dish.name.to_lowercase().contains(&term)
        || dish.category.to_lowercase().contains(&term)
        || dish
            .ingredients
            .iter()
            .any(|ingredient| ingredient.contains(&term))
        || dish.tags.iter().any(|tag| tag.contains(&term))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_dish() -> Dish {
        Dish {
            dish_id: "D01".to_string(),
            name: "Nasi Lemak".to_string(),
            ingredients: vec!["rice".to_string(), "coconut milk".to_string()],
            category: "main".to_string(),
            tags: vec!["spicy".to_string(), "signature".to_string()],
            image_path: None,
            image_source_url: None,
        }
    }

    #[test]
    fn parses_multiple_delimiters() {
        let terms = parse_filter_terms(" chicken, rice;spicy\nD01 | main ");

        assert_eq!(terms, vec!["chicken", "rice", "spicy", "d01", "main"]);
    }

    #[test]
    fn match_any_accepts_one_matching_term() {
        let filter = SearchFilter::parse("dessert, coconut milk", MatchMode::Any);

        assert!(dish_matches_filter(&sample_dish(), &filter));
    }

    #[test]
    fn match_all_requires_every_term() {
        let matching_filter = SearchFilter::parse("nasi; spicy; main", MatchMode::All);
        let failing_filter = SearchFilter::parse("nasi; dessert", MatchMode::All);

        assert!(dish_matches_filter(&sample_dish(), &matching_filter));
        assert!(!dish_matches_filter(&sample_dish(), &failing_filter));
    }
}
