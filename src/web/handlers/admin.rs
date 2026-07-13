use crate::data_loader::{
    DISHES_PATH, ORDERS_PATH, dishes_to_csv, load_dishes, load_orders, orders_to_csv,
    parse_dishes_from_reader, parse_orders_from_reader,
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
            let mode = ImportMode::from_value(payload.mode.as_deref());
            let count = match mode {
                ImportMode::Replace => state.replace_dishes_from_csv(dishes),
                ImportMode::Merge => state.merge_dishes_from_csv(dishes),
            };
            Json(ApiResponse::ok(
                format!(
                    "{} {count} dish record(s) into memory.",
                    mode.past_tense_label()
                ),
                Some(count),
            ))
        }
        Err(error) => Json(ApiResponse::error(format!(
            "Dish CSV import failed: {error}"
        ))),
    }
}

/// Reloads dishes directly from `data/dishes.csv`.
///
/// This is the simplest practical admin workflow for the FYP demo: staff can
/// edit the CSV file in a spreadsheet and click reload instead of pasting rows
/// into a browser text area.
pub async fn reload_dishes_from_file(State(state): State<WebState>) -> Json<ApiResponse<usize>> {
    match load_dishes(DISHES_PATH) {
        Ok(dishes) if dishes.is_empty() => {
            Json(ApiResponse::error("data/dishes.csv contained no dishes."))
        }
        Ok(dishes) => {
            let count = state.replace_dishes_from_csv(dishes);
            Json(ApiResponse::ok(
                format!("Reloaded {count} dish record(s) from {DISHES_PATH}."),
                Some(count),
            ))
        }
        Err(error) => Json(ApiResponse::error(format!(
            "Reload from {DISHES_PATH} failed: {error}"
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

/// Reloads historical orders directly from `data/orders.csv`.
pub async fn reload_orders_from_file(State(state): State<WebState>) -> Json<ApiResponse<usize>> {
    match load_orders(ORDERS_PATH) {
        Ok(orders) if orders.is_empty() => {
            Json(ApiResponse::error("data/orders.csv contained no orders."))
        }
        Ok(orders) => {
            let count = state.replace_historical_orders_from_csv(orders);
            Json(ApiResponse::ok(
                format!("Reloaded {count} historical order record(s) from {ORDERS_PATH}."),
                Some(count),
            ))
        }
        Err(error) => Json(ApiResponse::error(format!(
            "Reload from {ORDERS_PATH} failed: {error}"
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

/// Downloads completed checkout orders from the current server session.
pub async fn export_completed_session_orders_csv(
    State(state): State<WebState>,
) -> impl IntoResponse {
    match orders_to_csv(&state.completed_session_orders_for_export()) {
        Ok(csv) => (
            [
                (CONTENT_TYPE, "text/csv; charset=utf-8"),
                (
                    CONTENT_DISPOSITION,
                    "attachment; filename=\"completed_session_orders.csv\"",
                ),
            ],
            csv,
        )
            .into_response(),
        Err(error) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Completed session order export failed: {error}"),
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
    #[serde(default)]
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum ImportMode {
    Replace,
    Merge,
}

impl ImportMode {
    fn from_value(value: Option<&str>) -> Self {
        match value.unwrap_or_default().trim().to_lowercase().as_str() {
            "merge" => Self::Merge,
            _ => Self::Replace,
        }
    }

    fn past_tense_label(self) -> &'static str {
        match self {
            Self::Replace => "Imported",
            Self::Merge => "Merged",
        }
    }
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
