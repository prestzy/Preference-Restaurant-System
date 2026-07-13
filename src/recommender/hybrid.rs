use crate::models::{Dish, Order, RecommendationResult, UserPreference};
use crate::recommender::association_metrics::best_association_metric;
use crate::recommender::collaborative_filter::{
    build_co_order_matrix, calculate_co_order_score, related_selected_dishes,
};
use crate::recommender::ingredient_filter::{
    build_ingredient_explanation, calculate_ingredient_score, check_disliked_ingredients,
    matched_disliked_ingredients, matched_liked_ingredients, matched_preferred_tags,
};
use crate::recommender::popularity::{build_popularity_counts, calculate_popularity_score};
use crate::recommender::time_context::{TimeContext, calculate_time_score, time_explanation};
use std::collections::HashSet;

const CONTENT_WEIGHT: f32 = 0.45;
const CO_ORDER_WEIGHT: f32 = 0.25;
const POPULARITY_WEIGHT: f32 = 0.20;
const BUSINESS_RULE_WEIGHT: f32 = 0.10;

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

/// Generates ranked recommendations using content, co-ordering, popularity, and
/// simple time/business-rule scoring.
///
/// Pipeline:
/// 1. Build co-order and popularity evidence from order logs.
/// 2. Exclude dishes containing disliked ingredients.
/// 3. Skip dishes already selected by the user.
/// 4. Calculate all explainable component scores.
/// 5. Combine scores according to ranking method.
/// 6. Sort by final score descending.
pub fn generate_recommendations(
    dishes: &[Dish],
    orders: &[Order],
    preference: &UserPreference,
) -> RecommendationOutput {
    let co_order_matrix = build_co_order_matrix(orders);
    let popularity_counts = build_popularity_counts(orders);
    let time_context = TimeContext::from_label(preference.time_context.as_deref().unwrap_or("Any"));
    let ranking_method = RankingMethod::from_value(preference.ranking_method.as_deref());
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
        let popularity_score = calculate_popularity_score(&popularity_counts, &dish.dish_id);
        let business_rule_score = calculate_time_score(dish, time_context);
        let mut final_score = combine_scores(
            ingredient_score,
            co_order_score,
            popularity_score,
            business_rule_score,
            ranking_method,
        );

        if final_score <= 0.0 {
            if !preference.has_content_preferences() && !preference.has_selected_dishes() {
                // Last-resort fallback: when a brand-new dataset has no order
                // history yet, still show available dishes instead of an empty
                // customer recommendation section.
                final_score = 0.01;
            } else {
                continue;
            }
        }

        let matched_liked_ingredients = matched_liked_ingredients(dish, preference);
        let matched_preferred_tags = matched_preferred_tags(dish, preference);
        let matched_disliked_ingredients = matched_disliked_ingredients(dish, preference);
        let related_selected_dish_ids = related_selected_dishes(
            &co_order_matrix,
            &preference.selected_dish_ids,
            &dish.dish_id,
        );
        let association =
            best_association_metric(orders, &preference.selected_dish_ids, &dish.dish_id);

        recommendations.push(RecommendationResult {
            dish: dish.clone(),
            ingredient_score,
            co_order_score,
            popularity_score,
            business_rule_score,
            final_score,
            association_base_dish_id: association
                .as_ref()
                .map(|metric| metric.base_dish_id.clone()),
            association_pair_count: association
                .as_ref()
                .map(|metric| metric.pair_count)
                .unwrap_or(0),
            association_support: association
                .as_ref()
                .map(|metric| metric.support)
                .unwrap_or(0.0),
            association_confidence: association
                .as_ref()
                .map(|metric| metric.confidence)
                .unwrap_or(0.0),
            association_lift: association
                .as_ref()
                .map(|metric| metric.lift)
                .unwrap_or(0.0),
            matched_liked_ingredients,
            matched_preferred_tags,
            matched_disliked_ingredients,
            related_selected_dish_ids,
            explanation: build_hybrid_explanation(
                dish,
                preference,
                ingredient_score,
                co_order_score,
                popularity_score,
                business_rule_score,
                time_context,
                association
                    .as_ref()
                    .map(|metric| metric.lift)
                    .unwrap_or(0.0),
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

/// Combines all component scores.
///
/// Default hybrid formula:
/// final = 0.45 content + 0.25 co-order + 0.20 popularity + 0.10 time/business
///
/// Admin tester method overrides:
/// - content-based: content score only
/// - co-ordering: co-order score, with popularity as fallback
/// - hybrid: full formula above
pub fn combine_scores(
    ingredient_score: f32,
    co_order_score: f32,
    popularity_score: f32,
    business_rule_score: f32,
    ranking_method: RankingMethod,
) -> f32 {
    let score = match ranking_method {
        RankingMethod::ContentBased => ingredient_score,
        RankingMethod::CoOrdering => {
            if co_order_score > 0.0 {
                co_order_score
            } else {
                popularity_score * 0.7 + business_rule_score * 0.3
            }
        }
        RankingMethod::Hybrid => {
            CONTENT_WEIGHT * ingredient_score
                + CO_ORDER_WEIGHT * co_order_score
                + POPULARITY_WEIGHT * popularity_score
                + BUSINESS_RULE_WEIGHT * business_rule_score
        }
    };

    score.clamp(0.0, 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankingMethod {
    ContentBased,
    CoOrdering,
    Hybrid,
}

impl RankingMethod {
    fn from_value(value: Option<&str>) -> Self {
        match value.unwrap_or("hybrid").trim().to_lowercase().as_str() {
            "content" | "content-based" | "content_based" => Self::ContentBased,
            "co-ordering" | "co_ordering" | "collaborative" => Self::CoOrdering,
            _ => Self::Hybrid,
        }
    }
}

fn build_hybrid_explanation(
    dish: &Dish,
    preference: &UserPreference,
    ingredient_score: f32,
    co_order_score: f32,
    popularity_score: f32,
    business_rule_score: f32,
    time_context: TimeContext,
    association_lift: f32,
) -> String {
    let mut reasons = Vec::new();

    if ingredient_score > 0.0 {
        reasons.push(build_ingredient_explanation(dish, preference));
    }

    if co_order_score > 0.0 {
        reasons.push("often appears with the selected/current dish context".to_string());
    }

    if association_lift > 0.0 {
        reasons.push(format!("association lift {:.2}", association_lift));
    }

    if popularity_score > 0.0
        && !preference.has_content_preferences()
        && !preference.has_selected_dishes()
    {
        reasons.push(
            "popular dish based on historical orders; used as fallback due to limited preference input"
                .to_string(),
        );
    } else if popularity_score > 0.0 && ingredient_score == 0.0 && co_order_score == 0.0 {
        reasons.push("popular dish based on historical orders".to_string());
    }

    if let Some(explanation) = time_explanation(dish, time_context, business_rule_score) {
        reasons.push(explanation);
    }

    if reasons.is_empty() {
        format!(
            "{} is recommended by the hybrid score formula: 0.45 content + 0.25 co-order + 0.20 popularity + 0.10 time/business.",
            dish.name
        )
    } else {
        format!(
            "{} is recommended because it {}. Formula: 0.45 content + 0.25 co-order + 0.20 popularity + 0.10 time/business.",
            dish.name,
            reasons.join("; ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dish(id: &str, name: &str, category: &str, ingredients: &[&str], tags: &[&str]) -> Dish {
        Dish {
            dish_id: id.to_string(),
            name: name.to_string(),
            ingredients: ingredients.iter().map(|value| value.to_string()).collect(),
            category: category.to_string(),
            tags: tags.iter().map(|value| value.to_string()).collect(),
            image_path: None,
            image_source_url: None,
        }
    }

    #[test]
    fn hybrid_formula_uses_all_component_scores() {
        let score = combine_scores(1.0, 0.5, 0.25, 1.0, RankingMethod::Hybrid);

        assert!((score - 0.725).abs() < 0.0001);
    }

    #[test]
    fn popularity_fallback_produces_recommendations_without_preferences() {
        let dishes = vec![
            dish("D01", "Nasi Lemak", "main", &["rice"], &["signature"]),
            dish("D02", "Satay", "main", &["chicken"], &["grilled"]),
        ];
        let orders = vec![Order {
            order_id: "O1".to_string(),
            session_user_id: "U1".to_string(),
            ordered_dishes: vec!["D02".to_string(), "D02".to_string()],
            timestamp: "t".to_string(),
        }];

        let output = generate_recommendations(&dishes, &orders, &UserPreference::default());

        assert!(!output.recommendations.is_empty());
        assert_eq!(output.recommendations[0].dish.dish_id, "D02");
        assert!(output.recommendations[0].popularity_score > 0.0);
    }
}
