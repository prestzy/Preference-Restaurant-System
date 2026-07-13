use crate::web::state::{AdminInsightResponse, AssistantRequest, AssistantResponse, WebState};
use axum::Json;
use axum::extract::State;

/// Parses a customer natural-language prompt and returns recommendations.
///
/// The handler is intentionally thin: `WebState` coordinates parser and
/// recommender access, while this function only maps HTTP JSON to JSON.
pub async fn smart_menu_assistant(
    State(state): State<WebState>,
    Json(payload): Json<AssistantRequest>,
) -> Json<AssistantResponse> {
    Json(state.assistant_recommend(payload))
}

/// Returns rule-based admin insight summaries from order history.
pub async fn admin_insights(State(state): State<WebState>) -> Json<AdminInsightResponse> {
    Json(state.admin_insights())
}
