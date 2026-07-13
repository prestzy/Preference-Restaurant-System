use crate::web::state::WebState;
use crate::web::templates;
use axum::Json;
use axum::extract::{Path, State};
use axum::response::Html;
use serde::Serialize;

/// Renders the customer orders page placeholder.
///
/// Live order tracking can be expanded later without touching cart checkout or
/// recommendation code.
pub async fn orders_page(State(state): State<WebState>) -> Html<String> {
    let view = state.menu_view();
    let orders = state.customer_orders();
    Html(templates::orders_page(&view, &orders))
}

/// Returns one in-memory checkout order for customer-side status tracking.
///
/// The endpoint intentionally reports only session memory. It does not pretend
/// to provide persistent customer history, which keeps the prototype honest for
/// FYP evaluation.
pub async fn order_status(
    State(state): State<WebState>,
    Path(order_id): Path<String>,
) -> Json<OrderStatusResponse> {
    match state.order_by_id(&order_id) {
        Some(order) => Json(OrderStatusResponse {
            ok: true,
            message: "Order found in this server session.".to_string(),
            order: Some(order),
        }),
        None => Json(OrderStatusResponse {
            ok: false,
            message: "Order was not found in this server session.".to_string(),
            order: None,
        }),
    }
}

#[derive(Debug, Serialize)]
pub struct OrderStatusResponse {
    pub ok: bool,
    pub message: String,
    pub order: Option<crate::web::state::LiveOrder>,
}
