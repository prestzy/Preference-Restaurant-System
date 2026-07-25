use crate::web::state::WebState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
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
    headers: HeaderMap,
    Path(order_id): Path<String>,
) -> Json<OrderStatusResponse> {
    let Some(session) = super::customer::current_customer_session(&state, &headers) else {
        return Json(OrderStatusResponse {
            ok: false,
            message: "Your customer session expired. Please register again.".to_string(),
            order: None,
        });
    };

    match state.customer_order_by_id(&order_id, &session.customer_phone) {
        Ok(Some(order)) => Json(OrderStatusResponse {
            ok: true,
            message: "Order found in this server session.".to_string(),
            order: Some(order),
        }),
        Ok(None) => Json(OrderStatusResponse {
            ok: false,
            message: "Order was not found in this customer session.".to_string(),
            order: None,
        }),
        Err(message) => Json(OrderStatusResponse {
            ok: false,
            message,
            order: None,
        }),
    }
}

pub async fn my_orders(
    State(state): State<WebState>,
    headers: HeaderMap,
) -> Json<MyOrdersResponse> {
    let Some(session) = super::customer::current_customer_session(&state, &headers) else {
        return Json(MyOrdersResponse {
            ok: false,
            message: "Your customer session expired. Please register again.".to_string(),
            orders: Vec::new(),
        });
    };
    match state.customer_orders_by_phone(&session.customer_phone) {
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
