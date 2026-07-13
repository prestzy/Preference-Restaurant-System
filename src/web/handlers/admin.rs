use crate::data_loader::{
    dishes_to_csv, orders_to_csv, parse_dishes_from_reader, parse_orders_from_reader,
};
use crate::web::state::{DishView, LiveOrder, OrderStatus, UpsertDishRequest, WebState};
use crate::web::templates;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::response::{Html, IntoResponse};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

/// Renders the staff/admin dashboard.
///
/// Admin pages share the same orange/white visual language as the customer app,
/// but use denser cards and tables because staff need to scan operational data.
pub async fn admin_page(State(state): State<WebState>) -> Html<String> {
    let view = state.menu_view();
    let admin = state.admin_view();
    Html(templates::admin_page(&view, &admin))
}

/// Updates the workflow status for a live customer checkout order.
pub async fn update_order_status(
    State(state): State<WebState>,
    Path(order_id): Path<String>,
    Json(payload): Json<UpdateStatusRequest>,
) -> Json<ApiResponse<LiveOrder>> {
    let Some(status) = OrderStatus::from_label(&payload.status) else {
        return Json(ApiResponse::error("Unknown order status."));
    };

    match state.update_order_status(&order_id, status) {
        Ok(order) => Json(ApiResponse::ok("Order status updated.", Some(order))),
        Err(message) => Json(ApiResponse::error(message)),
    }
}

/// Adds or updates a dish in memory.
///
/// This gives the FYP admin flow real behaviour while keeping persistence
/// lightweight. CSV export can be used to save the current in-memory menu.
pub async fn upsert_dish(
    State(state): State<WebState>,
    Json(payload): Json<UpsertDishRequest>,
) -> Json<ApiResponse<DishView>> {
    match state.upsert_dish(payload) {
        Ok(dish) => Json(ApiResponse::ok("Dish saved in memory.", Some(dish))),
        Err(message) => Json(ApiResponse::error(message)),
    }
}

/// Deletes a dish from the current in-memory menu.
pub async fn delete_dish(
    State(state): State<WebState>,
    Path(dish_id): Path<String>,
) -> Json<ApiResponse<()>> {
    match state.delete_dish(&dish_id) {
        Ok(()) => Json(ApiResponse::ok("Dish removed from memory.", None)),
        Err(message) => Json(ApiResponse::error(message)),
    }
}

/// Marks a dish available or unavailable in the customer menu.
pub async fn set_dish_availability(
    State(state): State<WebState>,
    Path(dish_id): Path<String>,
    Json(payload): Json<AvailabilityRequest>,
) -> Json<ApiResponse<()>> {
    match state.set_dish_availability(&dish_id, payload.available) {
        Ok(()) => Json(ApiResponse::ok("Dish availability updated.", None)),
        Err(message) => Json(ApiResponse::error(message)),
    }
}

/// Imports dish CSV text pasted into the admin tool.
///
/// Import uses the same parser as startup loading, so older five-column CSV
/// files and newer image-aware CSV files are both accepted.
pub async fn import_dishes_csv(
    State(state): State<WebState>,
    Json(payload): Json<CsvImportRequest>,
) -> Json<ApiResponse<usize>> {
    match parse_dishes_from_reader(Cursor::new(payload.csv)) {
        Ok(dishes) if dishes.is_empty() => Json(ApiResponse::error("CSV contained no dishes.")),
        Ok(dishes) => {
            let count = state.replace_dishes_from_csv(dishes);
            Json(ApiResponse::ok(
                format!("Imported {count} dish record(s) into memory."),
                Some(count),
            ))
        }
        Err(error) => Json(ApiResponse::error(format!(
            "Dish CSV import failed: {error}"
        ))),
    }
}

/// Imports historical order CSV text pasted into the admin tool.
pub async fn import_orders_csv(
    State(state): State<WebState>,
    Json(payload): Json<CsvImportRequest>,
) -> Json<ApiResponse<usize>> {
    match parse_orders_from_reader(Cursor::new(payload.csv)) {
        Ok(orders) if orders.is_empty() => Json(ApiResponse::error("CSV contained no orders.")),
        Ok(orders) => {
            let count = state.replace_historical_orders_from_csv(orders);
            Json(ApiResponse::ok(
                format!("Imported {count} historical order record(s) into memory."),
                Some(count),
            ))
        }
        Err(error) => Json(ApiResponse::error(format!(
            "Order CSV import failed: {error}"
        ))),
    }
}

/// Downloads the current in-memory dishes as CSV.
pub async fn export_dishes_csv(State(state): State<WebState>) -> impl IntoResponse {
    match dishes_to_csv(&state.dish_models_for_export()) {
        Ok(csv) => (
            [
                (CONTENT_TYPE, "text/csv; charset=utf-8"),
                (CONTENT_DISPOSITION, "attachment; filename=\"dishes.csv\""),
            ],
            csv,
        )
            .into_response(),
        Err(error) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Dish CSV export failed: {error}"),
        )
            .into_response(),
    }
}

/// Downloads historical order logs as CSV.
pub async fn export_orders_csv(State(state): State<WebState>) -> impl IntoResponse {
    match orders_to_csv(&state.historical_orders_for_export()) {
        Ok(csv) => (
            [
                (CONTENT_TYPE, "text/csv; charset=utf-8"),
                (CONTENT_DISPOSITION, "attachment; filename=\"orders.csv\""),
            ],
            csv,
        )
            .into_response(),
        Err(error) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Order CSV export failed: {error}"),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct AvailabilityRequest {
    pub available: bool,
}

#[derive(Debug, Deserialize)]
pub struct CsvImportRequest {
    pub csv: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub ok: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    fn ok(message: impl Into<String>, data: Option<T>) -> Self {
        Self {
            ok: true,
            message: message.into(),
            data,
        }
    }

    fn error(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            message: message.into(),
            data: None,
        }
    }
}
