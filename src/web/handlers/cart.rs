use crate::web::state::{LiveOrder, WebState};
use crate::web::templates;
use axum::Json;
use axum::extract::State;
use axum::response::Html;
use serde::{Deserialize, Serialize};

/// Renders the customer cart page.
pub async fn cart_page(State(state): State<WebState>) -> Html<String> {
    let view = state.menu_view();
    Html(templates::cart_page(&view))
}

/// Creates a live in-memory order from the browser cart.
///
/// This is prototype checkout: it does not take payment or persist to a
/// database. It still produces a staff-visible order so the FYP can demonstrate
/// how customer actions create new operational data.
pub async fn create_order(
    State(state): State<WebState>,
    Json(payload): Json<CreateOrderRequest>,
) -> Json<CreateOrderResponse> {
    match state.create_live_order(&payload.dish_ids) {
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
        Err(message) => Json(CreateOrderResponse {
            ok: false,
            order_id: None,
            order: None,
            message,
        }),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub dish_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateOrderResponse {
    pub ok: bool,
    pub order_id: Option<String>,
    pub order: Option<LiveOrder>,
    pub message: String,
}
