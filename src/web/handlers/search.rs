use crate::search::MatchMode;
use crate::web::state::{MenuSearchResponse, WebState};
use axum::Json;
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};

/// Customer-facing menu search endpoint.
///
/// The browser calls this route for both live suggestions and the full Menu
/// grid. All alias/concept matching stays in Rust (`src/search.rs`) so search
/// behaviour is testable and not duplicated in JavaScript.
pub async fn menu_search(
    State(state): State<WebState>,
    Query(query): Query<MenuSearchQuery>,
) -> Json<MenuSearchApiResponse> {
    let mode = MatchMode::from_query(query.mode.as_deref());
    Json(MenuSearchApiResponse {
        ok: true,
        message: "Search completed.".to_string(),
        data: state.search_menu(query.q.as_deref().unwrap_or_default(), mode),
    })
}

#[derive(Debug, Deserialize)]
pub struct MenuSearchQuery {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub mode: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MenuSearchApiResponse {
    pub ok: bool,
    pub message: String,
    pub data: MenuSearchResponse,
}
