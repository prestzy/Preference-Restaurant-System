//! HTTP adapters for meal sets, learning timeline, and counterfactual analysis.

use crate::recommender::counterfactual::CounterfactualResult;
use crate::recommender::meal_set::MealSetRecommendation;
use crate::web::state::{
    CounterfactualRequest, LearningTimelineClearResponse, LearningTimelineResponse, MealSetRequest,
    WebState,
};
use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;

use super::admin::{ApiResponse, is_admin_authenticated_for_handlers};

pub async fn meal_sets(
    State(state): State<WebState>,
    Json(payload): Json<MealSetRequest>,
) -> Json<ApiResponse<Vec<MealSetRecommendation>>> {
    match state.recommend_meal_sets(payload) {
        Ok(sets) => Json(ApiResponse::ok("Meal sets generated.", Some(sets))),
        Err(message) => Json(ApiResponse::error(message)),
    }
}

pub async fn counterfactual(
    State(state): State<WebState>,
    headers: HeaderMap,
    Json(payload): Json<CounterfactualRequest>,
) -> Json<ApiResponse<CounterfactualResult>> {
    if !is_admin_authenticated_for_handlers(&state, &headers) {
        return Json(ApiResponse::error("Admin login required."));
    }
    match state.counterfactual(payload) {
        Ok(result) => Json(ApiResponse::ok(
            "Temporary comparison completed. Production data was not changed.",
            Some(result),
        )),
        Err(message) => Json(ApiResponse::error(message)),
    }
}

pub async fn learning_timeline(
    State(state): State<WebState>,
    headers: HeaderMap,
) -> Json<ApiResponse<LearningTimelineResponse>> {
    if !is_admin_authenticated_for_handlers(&state, &headers) {
        return Json(ApiResponse::error("Admin login required."));
    }
    Json(ApiResponse::ok(
        "Learning timeline loaded.",
        Some(state.learning_timeline()),
    ))
}

pub async fn rebuild_learning_timeline(
    State(state): State<WebState>,
    headers: HeaderMap,
) -> Json<ApiResponse<LearningTimelineResponse>> {
    if !is_admin_authenticated_for_handlers(&state, &headers) {
        return Json(ApiResponse::error("Admin login required."));
    }
    match state.rebuild_learning_timeline() {
        Ok(timeline) => Json(ApiResponse::ok(
            "Learning timeline rebuilt from durable historical orders.",
            Some(timeline),
        )),
        Err(message) => Json(ApiResponse::error(message)),
    }
}

/// Removes only the explanatory learning timeline after admin authentication.
///
/// Historical orders remain loaded and continue to drive popularity,
/// co-ordering, association evidence, and hybrid recommendations.
pub async fn clear_learning_timeline(
    State(state): State<WebState>,
    headers: HeaderMap,
) -> Json<ApiResponse<LearningTimelineClearResponse>> {
    if !is_admin_authenticated_for_handlers(&state, &headers) {
        return Json(ApiResponse::error("Admin login required."));
    }
    match state.clear_learning_timeline() {
        Ok(result) => Json(ApiResponse::ok(
            "Learning timeline cleared. Historical orders and recommendation evidence were not changed.",
            Some(result),
        )),
        Err(message) => Json(ApiResponse::error(message)),
    }
}
