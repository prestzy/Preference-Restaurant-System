use crate::web::state::WebState;
use crate::web::templates;
use axum::extract::State;
use axum::response::Html;

/// Renders the customer orders page placeholder.
///
/// Live order tracking can be expanded later without touching cart checkout or
/// recommendation code.
pub async fn orders_page(State(state): State<WebState>) -> Html<String> {
    let view = state.menu_view();
    Html(templates::orders_page(&view))
}
