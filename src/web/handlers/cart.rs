use crate::web::state::{LiveOrder, WebState};
use crate::web::templates;
use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::{Deserialize, Serialize};

/// Renders the customer cart page.
pub async fn cart_page(State(state): State<WebState>, headers: HeaderMap) -> Response {
    let Some(session) = super::customer::current_customer_session(&state, &headers) else {
        return Redirect::to("/start").into_response();
    };
    let view = state.menu_view();
    Html(templates::cart_page(&view, &session)).into_response()
}

/// Creates a live in-memory order from the browser cart.
///
/// This is prototype checkout: it does not take payment or persist to a
/// database. It still produces a staff-visible order so the FYP can demonstrate
/// how customer actions create new operational data.
pub async fn create_order(
    State(state): State<WebState>,
    headers: HeaderMap,
    Json(payload): Json<CreateOrderRequest>,
) -> Json<CreateOrderResponse> {
    let Some(session_id) = super::customer::customer_session_id_from_headers(&headers) else {
        return Json(CreateOrderResponse {
            ok: false,
            order_id: None,
            order: None,
            message: "Your customer session expired. Please register again.".to_string(),
        });
    };

    match state.create_live_order_from_session(&session_id, payload.dish_ids, payload.note) {
        Ok(order) => Json(CreateOrderResponse {
            ok: true,
            order_id: Some(order.order_id.clone()),
            order: Some(order.clone()),
            message: format!(
                "Prototype order {} placed with {} item(s).",
                order.order_id,
                order.ordered_dishes.len()
            ),
        }),
        Err(message) => {
            eprintln!("checkout failed: {message}");
            Json(CreateOrderResponse {
                ok: false,
                order_id: None,
                order: None,
                message,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub dish_ids: Vec<String>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateOrderResponse {
    pub ok: bool,
    pub order_id: Option<String>,
    pub order: Option<LiveOrder>,
    pub message: String,
}
