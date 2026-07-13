use crate::web::state::{RecommendationApiResponse, RecommendationRequest, WebState};
use axum::Json;
use axum::extract::State;

/// Runs content-based, collaborative, and hybrid recommendation scoring.
///
/// The web layer receives simple JSON preference arrays. The actual scoring
/// remains inside the recommender modules so this handler is only an API bridge.
pub async fn recommendations(
    State(state): State<WebState>,
    Json(payload): Json<RecommendationRequest>,
) -> Json<RecommendationApiResponse> {
    Json(state.recommend(payload))
}
