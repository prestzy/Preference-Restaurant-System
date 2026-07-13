use crate::models::{Dish, Order, RecommendationResult, UserPreference};
use crate::recommender::collaborative_filter::{
    build_co_order_matrix, calculate_co_order_score, related_selected_dishes,
};
use crate::recommender::ingredient_filter::{
    build_ingredient_explanation, calculate_ingredient_score, check_disliked_ingredients,
    matched_disliked_ingredients, matched_liked_ingredients, matched_preferred_tags,
};
use std::collections::HashSet;

/// Default content-based weight used when both preference and co-order data exist.
const DEFAULT_ALPHA: f32 = 0.4;

/// Default collaborative filtering weight used when both signals exist.
const DEFAULT_BETA: f32 = 0.6;

/// Summary values displayed in the Evaluation / Prototype Testing section.
///
/// This is not a full academic evaluation metric. It provides lightweight demo
/// evidence: how many dishes passed filtering, how many were excluded, and how
/// diverse the top recommendations are by category.
#[derive(Debug, Clone, Default)]
pub struct RecommendationStats {
    pub filtered_dishes: usize,
    pub excluded_due_to_disliked: usize,
    pub skipped_selected_dishes: usize,
    pub diversity_count_top_5: usize,
}

/// Complete output from the recommendation engine.
#[derive(Debug, Clone, Default)]
pub struct RecommendationOutput {
    pub recommendations: Vec<RecommendationResult>,
    #[allow(dead_code)]
    pub stats: RecommendationStats,
}

/// Generates ranked recommendations using ingredient, co-order, and hybrid scoring.
///
/// Pipeline:
/// 1. Build an item-item co-order matrix from order logs.
/// 2. Exclude dishes containing disliked ingredients.
/// 3. Skip dishes already selected by the user.
/// 4. Calculate ingredient and co-order scores.
/// 5. Combine both scores into one hybrid score.
/// 6. Sort by final score descending.
pub fn generate_recommendations(
    dishes: &[Dish],
    orders: &[Order],
    preference: &UserPreference,
) -> RecommendationOutput {
    let co_order_matrix = build_co_order_matrix(orders);
    let selected_set: HashSet<String> = preference.selected_dish_ids.iter().cloned().collect();
    let mut recommendations = Vec::new();
    let mut stats = RecommendationStats::default();

    for dish in dishes {
        if selected_set.contains(&dish.dish_id) {
            stats.skipped_selected_dishes += 1;
            continue;
        }

        if check_disliked_ingredients(dish, preference) {
            stats.excluded_due_to_disliked += 1;
            continue;
        }

        stats.filtered_dishes += 1;

        let ingredient_score = calculate_ingredient_score(dish, preference);
        let co_order_score = calculate_co_order_score(
            &co_order_matrix,
            &preference.selected_dish_ids,
            &dish.dish_id,
        );
        let final_score = combine_scores(
            ingredient_score,
            co_order_score,
            preference.has_content_preferences(),
            preference.has_selected_dishes(),
        );

        // Results with no evidence are not useful as recommendations. If the
        // user has entered nothing, the GUI will explain how to start testing.
        if final_score <= 0.0 {
            continue;
        }

        let matched_liked_ingredients = matched_liked_ingredients(dish, preference);
        let matched_preferred_tags = matched_preferred_tags(dish, preference);
        let matched_disliked_ingredients = matched_disliked_ingredients(dish, preference);
        let related_selected_dish_ids = related_selected_dishes(
            &co_order_matrix,
            &preference.selected_dish_ids,
            &dish.dish_id,
        );

        recommendations.push(RecommendationResult {
            dish: dish.clone(),
            ingredient_score,
            co_order_score,
            final_score,
            matched_liked_ingredients,
            matched_preferred_tags,
            matched_disliked_ingredients,
            related_selected_dish_ids,
            explanation: build_hybrid_explanation(
                dish,
                preference,
                ingredient_score,
                co_order_score,
                &co_order_matrix,
            ),
        });
    }

    recommendations.sort_by(|a, b| {
        b.final_score
            .total_cmp(&a.final_score)
            .then_with(|| a.dish.name.cmp(&b.dish.name))
    });

    stats.diversity_count_top_5 = recommendations
        .iter()
        .take(5)
        .map(|result| result.dish.category.clone())
        .collect::<HashSet<_>>()
        .len();

    RecommendationOutput {
        recommendations,
        stats,
    }
}

/// Combines ingredient and co-order scores into one final hybrid score.
///
/// Normal case:
/// final_score = 0.4 * ingredient_score + 0.6 * co_order_score
///
/// Adaptive prototype logic:
/// - If no selected dishes are entered, co-ordering has no useful signal, so the
///   system relies fully on ingredient score.
/// - If no content preferences are entered, ingredient score has no useful
///   signal, so the system relies fully on co-order score.
/// - If both are missing, both component scores are zero and no recommendation
///   is produced until the user enters preferences or selected dishes.
pub fn combine_scores(
    ingredient_score: f32,
    co_order_score: f32,
    has_content_preferences: bool,
    has_selected_dishes: bool,
) -> f32 {
    let (alpha, beta) = match (has_content_preferences, has_selected_dishes) {
        (true, true) => (DEFAULT_ALPHA, DEFAULT_BETA),
        (true, false) => (1.0, 0.0),
        (false, true) => (0.0, 1.0),
        (false, false) => (DEFAULT_ALPHA, DEFAULT_BETA),
    };

    (alpha * ingredient_score + beta * co_order_score).clamp(0.0, 1.0)
}

/// Creates one readable explanation string for the hybrid recommendation.
fn build_hybrid_explanation(
    dish: &Dish,
    preference: &UserPreference,
    ingredient_score: f32,
    co_order_score: f32,
    co_order_matrix: &crate::recommender::collaborative_filter::CoOrderMatrix,
) -> String {
    let ingredient_explanation = build_ingredient_explanation(dish, preference);
    let related_dishes = related_selected_dishes(
        co_order_matrix,
        &preference.selected_dish_ids,
        &dish.dish_id,
    );

    let mut reasons = Vec::new();

    if ingredient_score > 0.0 {
        reasons.push(ingredient_explanation);
    }

    if co_order_score > 0.0 && !related_dishes.is_empty() {
        reasons.push(format!(
            "often ordered with selected dish(es): {}",
            related_dishes.join(", ")
        ));
    }

    if reasons.is_empty() {
        "Recommended by the hybrid score, but no strong individual reason was found.".to_string()
    } else {
        format!("Recommended because it {}.", reasons.join(" and "))
    }
}
