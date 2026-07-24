use crate::web::state::WebState;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use serde::Serialize;

/// Renders the customer orders page placeholder.
///
/// Live order tracking can be expanded later without touching cart checkout or
/// recommendation code.
pub async fn orders_page() -> Response {
    Redirect::to("/profile").into_response()
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

pub async fn my_orders(
    State(state): State<WebState>,
    headers: HeaderMap,
    Query(query): Query<MyOrdersQuery>,
) -> Json<MyOrdersResponse> {
    let phone = super::customer::current_customer_session(&state, &headers)
        .map(|session| session.customer_phone)
        .or(query.phone);
    let Some(phone) = phone else {
        return Json(MyOrdersResponse {
            ok: false,
            message: "Your customer session expired. Please register again.".to_string(),
            orders: Vec::new(),
        });
    };
    match state.customer_orders_by_phone(&phone) {
        Ok(orders) => Json(MyOrdersResponse {
            ok: true,
            message: "Customer orders loaded.".to_string(),
            orders,
        }),
        Err(message) => Json(MyOrdersResponse {
            ok: false,
            message,
            orders: Vec::new(),
        }),
    }
}

pub async fn profile_orders(
    State(state): State<WebState>,
    headers: HeaderMap,
) -> Json<MyOrdersSyncResponse> {
    let Some(session) = super::customer::current_customer_session(&state, &headers) else {
        return Json(MyOrdersSyncResponse {
            ok: false,
            message: "Your customer session expired. Please register again.".to_string(),
            data: None,
        });
    };
    match state.customer_order_sync_by_phone(&session.customer_phone) {
        Ok(data) => Json(MyOrdersSyncResponse {
            ok: true,
            message: "Customer orders loaded.".to_string(),
            data: Some(data),
        }),
        Err(message) => Json(MyOrdersSyncResponse {
            ok: false,
            message,
            data: None,
        }),
    }
}

#[derive(Debug, Deserialize)]
pub struct MyOrdersQuery {
    #[serde(default)]
    pub phone: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MyOrdersResponse {
    pub ok: bool,
    pub message: String,
    pub orders: Vec<crate::web::state::LiveOrder>,
}

#[derive(Debug, Serialize)]
pub struct MyOrdersSyncResponse {
    pub ok: bool,
    pub message: String,
    pub data: Option<crate::web::state::OrderSyncResponse>,
}

#[derive(Debug, Serialize)]
pub struct OrderStatusResponse {
    pub ok: bool,
    pub message: String,
    pub order: Option<crate::web::state::LiveOrder>,
}
