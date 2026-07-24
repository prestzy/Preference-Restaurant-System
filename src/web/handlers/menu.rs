use crate::web::state::WebState;
use crate::web::templates;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Redirect, Response};

/// Renders the customer QR menu home page.
///
/// The handler only prepares the view model and delegates HTML generation to
/// `templates`, keeping HTTP wiring separate from page markup.
pub async fn customer_menu(State(state): State<WebState>, headers: HeaderMap) -> Response {
    let Some(session) = super::customer::current_customer_session(&state, &headers) else {
        return Redirect::to("/start").into_response();
    };
    let view = state.menu_view();
    Html(templates::customer_menu_page(&view, &session)).into_response()
}
