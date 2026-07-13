use crate::web::state::WebState;
use crate::web::templates;
use axum::extract::State;
use axum::response::Html;

/// Renders the customer QR menu home page.
///
/// The handler only prepares the view model and delegates HTML generation to
/// `templates`, keeping HTTP wiring separate from page markup.
pub async fn customer_menu(State(state): State<WebState>) -> Html<String> {
    let view = state.menu_view();
    Html(templates::customer_menu_page(&view))
}
