//! Ranked menu search used by the customer dish locator.
//!
//! Search returns suggestions and match reasons only. It never mutates or
//! filters the server-rendered static Menu.

use crate::models::Dish;
use csv::ReaderBuilder;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::Path;

pub const SEARCH_ALIASES_PATH: &str = "data/search_aliases.csv";

/// Determines how multiple search terms are combined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchMode {
    /// A dish is shown if at least one search term matches it.
    Any,
    /// A dish is shown only if every search term matches it.
    All,
}

impl MatchMode {
    /// Parses browser/API values into the search service mode.
    ///
    /// Keeping this conversion in the search module avoids separate "any/all"
    /// interpretations in route handlers or JavaScript.
    pub fn from_query(value: Option<&str>) -> Self {
        match value.unwrap_or_default().trim().to_lowercase().as_str() {
            "any" | "match_any" | "match-any" => Self::Any,
            _ => Self::All,
        }
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
        .map(normalize_search_term)
        .filter(|term| !term.is_empty())
        .collect()
}

pub fn normalize_search_term(raw: &str) -> String {
    let collapsed = raw
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    match collapsed.as_str() {
        "noodles" => "noodle".to_string(),
        "fruits" => "fruit".to_string(),
        "chillies" | "chilli" => "chili".to_string(),
        value if value.ends_with('s') && value.len() > 4 => value.trim_end_matches('s').to_string(),
        value => value.to_string(),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchVocabulary {
    pub aliases: HashMap<String, Vec<String>>,
    pub concepts: HashMap<String, Vec<String>>,
}

pub fn build_search_vocabulary(dishes: &[Dish]) -> SearchVocabulary {
    let mut aliases = load_aliases(SEARCH_ALIASES_PATH);
    if aliases.is_empty() {
        aliases = fallback_aliases();
    }
    let vocabulary = dish_vocabulary(dishes);
    for (canonical, values) in fallback_aliases() {
        aliases.entry(canonical).or_insert(values);
    }
    let mut concepts = fallback_concepts();
    // Keep concept expansion explainable and menu-aware: unknown concept values
    // are harmless, but terms found in the current menu are retained first.
    for values in concepts.values_mut() {
        values.sort_by_key(|term| !vocabulary.contains(term));
        values.dedup();
    }
    SearchVocabulary { aliases, concepts }
}

/// Ranked result from the canonical menu search service.
///
/// The web UI uses these rows for both live suggestions and the full Menu grid,
/// so a query such as `mee, spicy` cannot produce one result set in the
/// dropdown and a different result set in the main menu.
#[derive(Debug, Clone, Serialize)]
pub struct RankedDishSearchMatch {
    pub dish_id: String,
    pub match_score: u32,
    pub match_reasons: Vec<String>,
}

/// Searches dishes using literal, alias, and concept-aware matching.
///
/// This is the single source of truth for customer menu search. JavaScript
/// should ask the `/api/search` route for these results instead of implementing
/// separate literal-only matching in the browser.
pub fn search_dishes(
    dishes: &[Dish],
    raw_query: &str,
    mode: MatchMode,
    vocabulary: &SearchVocabulary,
) -> Vec<RankedDishSearchMatch> {
    let terms = parse_filter_terms(raw_query);
    if terms.is_empty() {
        return dishes
            .iter()
            .map(|dish| RankedDishSearchMatch {
                dish_id: dish.dish_id.clone(),
                match_score: 0,
                match_reasons: Vec::new(),
            })
            .collect();
    }

    let groups = terms
        .iter()
        .map(|term| expanded_search_group(term, vocabulary))
        .collect::<Vec<_>>();

    let mut results = dishes
        .iter()
        .filter_map(|dish| {
            let group_matches = groups
                .iter()
                .map(|group| best_group_match(dish, group))
                .collect::<Vec<_>>();
            let matched = match mode {
                MatchMode::Any => group_matches.iter().any(Option::is_some),
                MatchMode::All => group_matches.iter().all(Option::is_some),
            };
            matched.then(|| {
                let mut reasons = group_matches
                    .iter()
                    .filter_map(|item| item.as_ref().map(|matched| matched.reason.clone()))
                    .collect::<Vec<_>>();
                reasons.sort();
                reasons.dedup();
                RankedDishSearchMatch {
                    dish_id: dish.dish_id.clone(),
                    match_score: group_matches
                        .iter()
                        .filter_map(|item| item.as_ref().map(|matched| matched.score))
                        .sum(),
                    match_reasons: reasons,
                }
            })
        })
        .collect::<Vec<_>>();

    results.sort_by(|a, b| {
        b.match_score
            .cmp(&a.match_score)
            .then_with(|| a.dish_id.cmp(&b.dish_id))
    });
    results
}

#[derive(Debug, Clone)]
struct ExpandedSearchGroup {
    raw: String,
    canonical_terms: HashSet<String>,
    alias_terms: HashSet<String>,
    concept_terms: HashSet<String>,
}

#[derive(Debug, Clone)]
struct FieldMatch {
    score: u32,
    reason: String,
}

fn expanded_search_group(term: &str, vocabulary: &SearchVocabulary) -> ExpandedSearchGroup {
    let raw = normalize_search_term(term);
    let mut canonical_terms = HashSet::from([raw.clone()]);
    let mut alias_terms = HashSet::new();
    let mut concept_terms = HashSet::new();

    for (canonical, aliases) in &vocabulary.aliases {
        let canonical = normalize_search_term(canonical);
        let aliases = aliases
            .iter()
            .map(|value| normalize_search_term(value))
            .collect::<Vec<_>>();
        if raw == canonical || aliases.iter().any(|alias| alias == &raw) {
            canonical_terms.insert(canonical);
            alias_terms.extend(aliases);
        }
    }

    for (concept, members) in &vocabulary.concepts {
        let concept = normalize_search_term(concept);
        let members = members
            .iter()
            .map(|value| normalize_search_term(value))
            .collect::<Vec<_>>();
        if raw == concept
            || canonical_terms.contains(&concept)
            || members.iter().any(|member| member == &raw)
            || members
                .iter()
                .any(|member| canonical_terms.contains(member))
        {
            concept_terms.insert(concept);
            concept_terms.extend(members);
        }
    }

    ExpandedSearchGroup {
        raw,
        canonical_terms,
        alias_terms,
        concept_terms,
    }
}

fn best_group_match(dish: &Dish, group: &ExpandedSearchGroup) -> Option<FieldMatch> {
    let normalized_id = normalize_search_term(&dish.dish_id);
    let normalized_name = normalize_search_term(&dish.name);
    let normalized_category = normalize_search_term(&dish.category);
    let normalized_ingredients = dish
        .ingredients
        .iter()
        .map(|value| normalize_search_term(value))
        .collect::<Vec<_>>();
    let normalized_tags = dish
        .tags
        .iter()
        .map(|value| normalize_search_term(value))
        .collect::<Vec<_>>();

    let mut best: Option<FieldMatch> = None;
    for term in group
        .canonical_terms
        .iter()
        .chain(group.alias_terms.iter())
        .chain(group.concept_terms.iter())
    {
        let origin = if group.canonical_terms.contains(term) {
            MatchOrigin::Literal
        } else if group.alias_terms.contains(term) {
            MatchOrigin::Alias
        } else {
            MatchOrigin::Concept
        };
        let candidate = field_match_for_term(
            dish,
            &group.raw,
            term,
            origin,
            &normalized_id,
            &normalized_name,
            &normalized_category,
            &normalized_ingredients,
            &normalized_tags,
        );
        if let Some(candidate) = candidate
            && best
                .as_ref()
                .is_none_or(|current| candidate.score > current.score)
        {
            best = Some(candidate);
        }
    }
    best
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchOrigin {
    Literal,
    Alias,
    Concept,
}

#[allow(clippy::too_many_arguments)]
fn field_match_for_term(
    dish: &Dish,
    raw: &str,
    term: &str,
    origin: MatchOrigin,
    normalized_id: &str,
    normalized_name: &str,
    normalized_category: &str,
    normalized_ingredients: &[String],
    normalized_tags: &[String],
) -> Option<FieldMatch> {
    if normalized_id == term || normalized_name == term {
        return Some(FieldMatch {
            score: 100,
            reason: format!("exact dish match: {}", dish.name),
        });
    }
    if normalized_name.starts_with(term) {
        return Some(FieldMatch {
            score: 85,
            reason: format!("dish name starts with {term}"),
        });
    }
    if normalized_name.contains(term) {
        return Some(FieldMatch {
            score: 70,
            reason: format!("dish name contains {term}"),
        });
    }
    if normalized_ingredients.iter().any(|value| value == term) {
        return Some(FieldMatch {
            score: match origin {
                MatchOrigin::Literal => 62,
                MatchOrigin::Alias => 55,
                MatchOrigin::Concept => 48,
            },
            reason: reason_for_origin(raw, term, origin, "ingredient"),
        });
    }
    if normalized_tags.iter().any(|value| value == term) {
        return Some(FieldMatch {
            score: match origin {
                MatchOrigin::Literal => 58,
                MatchOrigin::Alias => 52,
                MatchOrigin::Concept => 46,
            },
            reason: reason_for_origin(raw, term, origin, "tag"),
        });
    }
    if normalized_ingredients
        .iter()
        .any(|value| value.contains(term))
    {
        return Some(FieldMatch {
            score: 42,
            reason: reason_for_origin(raw, term, origin, "ingredient"),
        });
    }
    if normalized_tags.iter().any(|value| value.contains(term)) {
        return Some(FieldMatch {
            score: 40,
            reason: reason_for_origin(raw, term, origin, "tag"),
        });
    }
    if normalized_category.contains(term) {
        return Some(FieldMatch {
            score: 30,
            reason: format!("category: {}", dish.category),
        });
    }
    None
}

fn reason_for_origin(raw: &str, term: &str, origin: MatchOrigin, field: &str) -> String {
    match origin {
        MatchOrigin::Literal => format!("{field}: {term}"),
        MatchOrigin::Alias => format!("\"{raw}\" interpreted as {term}"),
        MatchOrigin::Concept => format!("matched {field} concept: {term}"),
    }
}

fn dish_vocabulary(dishes: &[Dish]) -> HashSet<String> {
    let mut terms = HashSet::new();
    for dish in dishes {
        terms.insert(normalize_search_term(&dish.name));
        terms.insert(normalize_search_term(&dish.category));
        for value in dish.ingredients.iter().chain(dish.tags.iter()) {
            terms.insert(normalize_search_term(value));
        }
    }
    terms
}

fn load_aliases(path: &str) -> HashMap<String, Vec<String>> {
    if !Path::new(path).exists() {
        return HashMap::new();
    }
    let Ok(file) = File::open(path) else {
        return HashMap::new();
    };
    let mut reader = ReaderBuilder::new().flexible(true).from_reader(file);
    let mut aliases = HashMap::new();
    for row in reader.records().flatten() {
        let Some(canonical) = row
            .get(0)
            .map(normalize_search_term)
            .filter(|v| !v.is_empty())
        else {
            continue;
        };
        let values = row
            .get(1)
            .unwrap_or_default()
            .split(',')
            .map(normalize_search_term)
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        aliases.insert(canonical, values);
    }
    aliases
}

fn fallback_aliases() -> HashMap<String, Vec<String>> {
    [
        ("noodle", vec!["mee", "mi", "noodles"]),
        ("banana", vec!["pisang"]),
        ("chicken", vec!["ayam"]),
        ("beef", vec!["daging"]),
        ("squid", vec!["sotong"]),
        ("prawn", vec!["shrimp", "udang"]),
        ("rice", vec!["nasi"]),
        ("spicy", vec!["pedas", "hot", "chili", "sambal"]),
        ("dessert", vec!["sweet", "kuih"]),
    ]
    .into_iter()
    .map(|(key, values)| {
        (
            key.to_string(),
            values.into_iter().map(str::to_string).collect(),
        )
    })
    .collect()
}

fn fallback_concepts() -> HashMap<String, Vec<String>> {
    [
        ("fruit", vec!["banana", "pisang", "mango", "pineapple"]),
        (
            "noodle",
            vec![
                "mee",
                "mi",
                "noodles",
                "yellow noodles",
                "rice noodles",
                "egg noodles",
                "flat rice noodles",
                "laksa noodles",
            ],
        ),
        (
            "seafood",
            vec![
                "fish",
                "squid",
                "sotong",
                "prawn",
                "shrimp",
                "anchovies",
                "mackerel",
            ],
        ),
        ("poultry", vec!["chicken", "ayam"]),
        (
            "spicy",
            vec!["chili", "sambal", "curry paste", "pedas", "spicy"],
        ),
    ]
    .into_iter()
    .map(|(key, values)| {
        (
            key.to_string(),
            values.into_iter().map(str::to_string).collect(),
        )
    })
    .collect()
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

    fn dish_with(id: &str, name: &str, ingredients: &[&str], tags: &[&str]) -> Dish {
        Dish {
            dish_id: id.to_string(),
            name: name.to_string(),
            ingredients: ingredients.iter().map(|value| value.to_string()).collect(),
            category: "main".to_string(),
            tags: tags.iter().map(|value| value.to_string()).collect(),
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
        let dishes = vec![
            sample_dish(),
            dish_with("D02", "Fruit Pudding", &["mango"], &["dessert"]),
        ];
        let vocabulary = build_search_vocabulary(&dishes);
        let results = search_dishes(
            &dishes,
            "dessert, coconut milk",
            MatchMode::Any,
            &vocabulary,
        );

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn match_all_requires_every_term() {
        let dishes = vec![
            sample_dish(),
            dish_with("D02", "Fruit Pudding", &["mango"], &["dessert"]),
        ];
        let vocabulary = build_search_vocabulary(&dishes);
        let matching = search_dishes(&dishes, "nasi; spicy; main", MatchMode::All, &vocabulary);
        let failing = search_dishes(&dishes, "nasi; dessert", MatchMode::All, &vocabulary);

        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].dish_id, "D01");
        assert!(failing.is_empty());
    }

    #[test]
    fn vocabulary_contains_menu_aware_aliases_and_concepts() {
        let dishes = vec![
            dish_with(
                "D04",
                "Laksa",
                &["noodles", "fish", "chili"],
                &["spicy", "noodle"],
            ),
            dish_with("D20", "Pisang Goreng", &["banana", "flour"], &["dessert"]),
        ];
        let vocabulary = build_search_vocabulary(&dishes);

        assert!(vocabulary.aliases["noodle"].contains(&"mee".to_string()));
        assert!(vocabulary.aliases["chicken"].contains(&"ayam".to_string()));
        assert!(vocabulary.concepts["fruit"].contains(&"banana".to_string()));
        assert!(vocabulary.concepts["spicy"].contains(&"sambal".to_string()));
    }

    #[test]
    fn normalizes_simple_food_plurals() {
        assert_eq!(normalize_search_term("Mee"), "mee");
        assert_eq!(normalize_search_term("Noodles"), "noodle");
        assert_eq!(normalize_search_term("Fruits"), "fruit");
        assert_eq!(normalize_search_term("Chillies"), "chili");
    }

    #[test]
    fn concept_search_matches_laksa_for_mee_and_spicy() {
        let dishes = vec![
            dish_with(
                "D04",
                "Laksa",
                &["rice noodles", "fish cake", "chili"],
                &["spicy", "local"],
            ),
            dish_with("D12", "Mee Soup", &["yellow noodles", "water"], &["mild"]),
        ];
        let vocabulary = build_search_vocabulary(&dishes);

        let results = search_dishes(&dishes, "mee, spicy", MatchMode::All, &vocabulary);

        assert_eq!(results[0].dish_id, "D04");
        assert!(
            results[0]
                .match_reasons
                .iter()
                .any(|reason| reason.contains("interpreted") || reason.contains("concept"))
        );
    }

    #[test]
    fn fruit_concept_matches_banana_and_mango_dishes() {
        let dishes = vec![
            dish_with("D20", "Pisang Goreng", &["banana", "flour"], &["dessert"]),
            dish_with("D21", "Mango Pudding", &["mango", "milk"], &["dessert"]),
            dish_with("D01", "Nasi Lemak", &["rice"], &["spicy"]),
        ];
        let vocabulary = build_search_vocabulary(&dishes);

        let ids = search_dishes(&dishes, "fruit", MatchMode::Any, &vocabulary)
            .into_iter()
            .map(|result| result.dish_id)
            .collect::<Vec<_>>();

        assert!(ids.contains(&"D20".to_string()));
        assert!(ids.contains(&"D21".to_string()));
        assert!(!ids.contains(&"D01".to_string()));
    }

    #[test]
    fn exact_name_match_ranks_above_concept_match() {
        let dishes = vec![
            dish_with("D04", "Laksa", &["rice noodles", "chili"], &["spicy"]),
            dish_with("D14", "Mee Goreng Mamak", &["yellow noodles"], &["fried"]),
        ];
        let vocabulary = build_search_vocabulary(&dishes);

        let results = search_dishes(&dishes, "mee", MatchMode::Any, &vocabulary);

        assert_eq!(results[0].dish_id, "D14");
        assert!(results.iter().any(|result| result.dish_id == "D04"));
    }
}
