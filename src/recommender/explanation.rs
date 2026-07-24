use crate::models::{Dish, UserPreference};
use crate::recommender::adaptive::AdaptiveWeights;
use crate::recommender::evidence::RecommendationEvidence;
use crate::recommender::ingredient_filter::build_ingredient_explanation;
use crate::recommender::time_context::{TimeContext, time_explanation};

/// Builds the production explanation from the scores, adaptive weights, and
/// evidence actually used for this request.
///
/// Formula text lives here instead of templates or JavaScript so customer and
/// admin views cannot drift away from the Rust calculation.
#[allow(clippy::too_many_arguments)]
pub fn build_adaptive_explanation(
    dish: &Dish,
    preference: &UserPreference,
    ingredient_score: f32,
    co_order_score: f32,
    popularity_score: f32,
    time_score: f32,
    time_context: TimeContext,
    association_lift: f32,
    weights: AdaptiveWeights,
    evidence: &RecommendationEvidence,
) -> String {
    let mut reasons = Vec::new();

    if ingredient_score > 0.0 {
        reasons.push(build_ingredient_explanation(dish, preference));
    }
    if co_order_score > 0.0 {
        reasons.push(format!(
            "appeared with the selected dish context in {} historical order(s)",
            evidence.candidate_pair_count
        ));
    }
    if association_lift > 0.0 {
        reasons.push(format!(
            "association lift {:.2} is shown as supporting context",
            association_lift
        ));
    }
    if popularity_score > 0.0
        && !preference.has_content_preferences()
        && !preference.has_selected_dishes()
    {
        reasons.push(format!(
            "used popularity fallback from {} historical appearance(s)",
            evidence.candidate_popularity_count
        ));
    } else if popularity_score > 0.0 && ingredient_score == 0.0 && co_order_score == 0.0 {
        reasons.push("has historical popularity evidence".to_string());
    }
    if let Some(reason) = time_explanation(dish, time_context, time_score) {
        reasons.push(reason);
    }
    if reasons.is_empty() {
        reasons.push("is shown as a deterministic fallback while evidence is limited".to_string());
    }

    let [content, co_order, popularity, time] = weights.as_percentages();
    format!(
        "{} is recommended because it {}. Adaptive weights used: content {}%, co-order {}%, popularity {}%, time/context {}%.",
        dish.name,
        reasons.join("; "),
        content,
        co_order,
        popularity,
        time
    )
}
