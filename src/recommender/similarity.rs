use crate::models::Dish;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub struct SimilarityConfig {
    pub ingredient_weight: f32,
    pub tag_weight: f32,
    pub category_weight: f32,
}

impl Default for SimilarityConfig {
    fn default() -> Self {
        Self {
            ingredient_weight: 0.50,
            tag_weight: 0.30,
            category_weight: 0.20,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DishFeatureProfile {
    pub ingredient_tokens: HashSet<String>,
    pub tag_tokens: HashSet<String>,
    pub category: String,
}

impl DishFeatureProfile {
    pub fn from_dish(dish: &Dish) -> Self {
        Self {
            ingredient_tokens: normalized_features(&dish.ingredients),
            tag_tokens: normalized_features(&dish.tags),
            category: dish.category.trim().to_lowercase(),
        }
    }
}

pub fn dish_similarity(left: &Dish, right: &Dish, config: SimilarityConfig) -> f32 {
    let left = DishFeatureProfile::from_dish(left);
    let right = DishFeatureProfile::from_dish(right);
    let ingredient = jaccard(&left.ingredient_tokens, &right.ingredient_tokens);
    let tag = jaccard(&left.tag_tokens, &right.tag_tokens);
    let category = if !left.category.is_empty() && left.category == right.category {
        1.0
    } else {
        0.0
    };
    (config.ingredient_weight * ingredient
        + config.tag_weight * tag
        + config.category_weight * category)
        .clamp(0.0, 1.0)
}

fn normalized_features(values: &[String]) -> HashSet<String> {
    values
        .iter()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .collect()
}

fn jaccard(left: &HashSet<String>, right: &HashSet<String>) -> f32 {
    if left.is_empty() && right.is_empty() {
        return 0.0;
    }
    let union = left.union(right).count();
    if union == 0 {
        0.0
    } else {
        (left.intersection(right).count() as f32 / union as f32).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dish(id: &str, ingredients: &[&str], tags: &[&str], category: &str) -> Dish {
        Dish {
            dish_id: id.to_string(),
            name: id.to_string(),
            ingredients: ingredients.iter().map(|value| value.to_string()).collect(),
            tags: tags.iter().map(|value| value.to_string()).collect(),
            category: category.to_string(),
            image_path: None,
            image_source_url: None,
        }
    }

    #[test]
    fn identical_features_are_highly_similar() {
        let left = dish("D01", &["rice", "egg"], &["spicy"], "main");
        let right = dish("D02", &["rice", "egg"], &["spicy"], "main");
        assert!((dish_similarity(&left, &right, SimilarityConfig::default()) - 1.0).abs() < 0.0001);
    }

    #[test]
    fn disjoint_and_empty_features_are_safe() {
        let left = dish("D01", &["rice"], &["spicy"], "main");
        let right = dish("D02", &["banana"], &["sweet"], "dessert");
        let empty = dish("D03", &[], &[], "");
        assert_eq!(
            dish_similarity(&left, &right, SimilarityConfig::default()),
            0.0
        );
        assert!(dish_similarity(&empty, &empty, SimilarityConfig::default()).is_finite());
    }
}
