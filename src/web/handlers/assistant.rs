use crate::web::state::{AdminInsightResponse, AssistantRequest, AssistantResponse, WebState};
use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;

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
pub async fn admin_insights(
    State(state): State<WebState>,
    headers: HeaderMap,
) -> Json<AdminInsightResponse> {
    if !super::admin::is_admin_authenticated_for_handlers(&state, &headers) {
        return Json(AdminInsightResponse {
            summary: "Admin login required.".to_string(),
            popular: Vec::new(),
            co_order_pairs: Vec::new(),
            low_exposure: Vec::new(),
        });
    }
    Json(state.admin_insights())
}
