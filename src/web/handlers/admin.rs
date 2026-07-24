use crate::data_loader::{
    DISHES_PATH, ORDERS_PATH, dishes_to_csv, load_dishes, load_orders, orders_to_csv,
    parse_dishes_from_reader, parse_orders_from_reader,
};
use crate::web::state::{
    DishView, EvaluationRequest, EvaluationResponse, ExperimentLabRequest, ExperimentLabResponse,
    OrderStatus, OrderStatusUpdate, OrderSyncResponse, SimulationRequest, SimulationResponse,
    UpsertDishRequest, WebState,
};
use crate::web::templates;
use axum::Json;
use axum::extract::{Form, Path, State};
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE, COOKIE, LOCATION, SET_COOKIE};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::{Deserialize, Serialize};
use std::env;
use std::io::Cursor;

pub const ADMIN_SESSION_COOKIE: &str = "admin_session";
const ADMIN_SESSION_MAX_AGE_SECONDS: u32 = 12 * 60 * 60;

/// Renders the staff/admin dashboard.
///
/// Admin pages share the same orange/white visual language as the customer app,
/// but use denser cards and tables because staff need to scan operational data.
pub async fn admin_login_page() -> Html<String> {
    Html(templates::admin_login_page(None, None))
}

pub async fn admin_login_submit(
    State(state): State<WebState>,
    Form(payload): Form<AdminLoginRequest>,
) -> Response {
    println!("Admin login request received.");
    process_admin_login(&state, payload, admin_credentials())
}

fn process_admin_login(
    state: &WebState,
    payload: AdminLoginRequest,
    credentials: Result<(String, String), &'static str>,
) -> Response {
    let credentials = match credentials {
        Ok(credentials) => credentials,
        Err(message) => {
            println!("Admin authentication unavailable: {message}");
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Html(templates::admin_login_page(
                    Some(&payload.username),
                    Some(message),
                )),
            )
                .into_response();
        }
    };

    if payload.username.trim() == credentials.0 && payload.password == credentials.1 {
        let session_id = state.create_admin_session();
        let cookie = admin_session_cookie(&session_id);
        println!("Admin authentication succeeded; admin session created.");
        return (
            [(SET_COOKIE, cookie.as_str()), (LOCATION, "/admin")],
            StatusCode::SEE_OTHER,
        )
            .into_response();
    }

    println!("Admin authentication failed.");
    (
        StatusCode::UNAUTHORIZED,
        Html(templates::admin_login_page(
            Some(&payload.username),
            Some("Invalid username or password."),
        )),
    )
        .into_response()
}

pub async fn admin_logout(State(state): State<WebState>, headers: HeaderMap) -> Response {
    if let Some(session_id) = admin_session_id_from_headers(&headers) {
        state.clear_admin_session(&session_id);
    }
    (
        [(SET_COOKIE, expired_admin_session_cookie().as_str())],
        [(LOCATION, "/admin/login")],
        StatusCode::SEE_OTHER,
    )
        .into_response()
}

pub async fn admin_page(State(state): State<WebState>, headers: HeaderMap) -> Response {
    if !is_admin_authenticated(&state, &headers) {
        return redirect_to_login();
    }
    let view = state.menu_view();
    let admin = state.admin_view();
    Html(templates::admin_page(&view, &admin)).into_response()
}

pub async fn admin_orders_page(State(state): State<WebState>, headers: HeaderMap) -> Response {
    if !is_admin_authenticated(&state, &headers) {
        return redirect_to_login();
    }
    let view = state.menu_view();
    let admin = state.admin_view();
    Html(templates::admin_orders_page(&view, &admin)).into_response()
}

pub async fn admin_dishes_page(State(state): State<WebState>, headers: HeaderMap) -> Response {
    if !is_admin_authenticated(&state, &headers) {
        return redirect_to_login();
    }
    let view = state.menu_view();
    let admin = state.admin_view();
    Html(templates::admin_dishes_page(&view, &admin)).into_response()
}

pub async fn admin_recommendations_page(
    State(state): State<WebState>,
    headers: HeaderMap,
) -> Response {
    if !is_admin_authenticated(&state, &headers) {
        return redirect_to_login();
    }
    let view = state.menu_view();
    let admin = state.admin_view();
    Html(templates::admin_recommendations_page(&view, &admin)).into_response()
}

pub async fn admin_evaluation_page(State(state): State<WebState>, headers: HeaderMap) -> Response {
    if !is_admin_authenticated(&state, &headers) {
        return redirect_to_login();
    }
    redirect_to_admin_recommendations()
}

pub async fn admin_data_page(State(state): State<WebState>, headers: HeaderMap) -> Response {
    if !is_admin_authenticated(&state, &headers) {
        return redirect_to_login();
    }
    let view = state.menu_view();
    Html(templates::admin_data_page(&view)).into_response()
}

pub async fn admin_insights_page(State(state): State<WebState>, headers: HeaderMap) -> Response {
    if !is_admin_authenticated(&state, &headers) {
        return redirect_to_login();
    }
    Redirect::to("/admin").into_response()
}

#[allow(dead_code)]
pub async fn run_evaluation(
    State(state): State<WebState>,
    headers: HeaderMap,
    Json(payload): Json<EvaluationRequest>,
) -> Json<ApiResponse<EvaluationResponse>> {
    if !is_admin_authenticated(&state, &headers) {
        return Json(ApiResponse::error("Admin login required."));
    }
    Json(ApiResponse::ok(
        "Evaluation completed.",
        Some(state.evaluation_report(payload)),
    ))
}

pub async fn run_simulation(
    State(state): State<WebState>,
    headers: HeaderMap,
    Json(payload): Json<SimulationRequest>,
) -> Json<ApiResponse<SimulationResponse>> {
    if !is_admin_authenticated(&state, &headers) {
        return Json(ApiResponse::error("Admin login required."));
    }
    Json(ApiResponse::ok(
        "Simulation completed in memory. Real data/orders.csv was not changed.",
        Some(state.simulation_report(payload)),
    ))
}

pub async fn run_experiment_lab(
    State(state): State<WebState>,
    headers: HeaderMap,
    Json(payload): Json<ExperimentLabRequest>,
) -> Json<ApiResponse<ExperimentLabResponse>> {
    if !is_admin_authenticated(&state, &headers) {
        return Json(ApiResponse::error("Admin login required."));
    }
    match state.experiment_lab(payload) {
        Ok(response) => Json(ApiResponse::ok("Experiment completed.", Some(response))),
        Err(message) => Json(ApiResponse::error(message)),
    }
}

pub async fn admin_orders_sync(
    State(state): State<WebState>,
    headers: HeaderMap,
) -> Json<ApiResponse<OrderSyncResponse>> {
    if !is_admin_authenticated(&state, &headers) {
        return Json(ApiResponse::error("Admin login required."));
    }
    Json(ApiResponse::ok(
        "Admin orders loaded.",
        Some(state.admin_order_sync()),
    ))
}

/// Updates the workflow status for a live customer checkout order.
pub async fn update_order_status(
    State(state): State<WebState>,
    headers: HeaderMap,
    Path(order_id): Path<String>,
    Json(payload): Json<UpdateStatusRequest>,
) -> Json<ApiResponse<OrderStatusUpdate>> {
    if !is_admin_authenticated(&state, &headers) {
        return Json(ApiResponse::error("Admin login required."));
    }
    let Some(status) = OrderStatus::from_label(&payload.status) else {
        return Json(ApiResponse::error("Unknown order status."));
    };

    match state.update_order_status(&order_id, status) {
        Ok(update) => {
            let message = if update.saved_to_csv {
                format!(
                    "Order marked Completed and saved to data/orders.csv as {}.",
                    update
                        .historical_order_id
                        .as_deref()
                        .unwrap_or("a historical order")
                )
            } else if status == OrderStatus::Completed {
                "Order marked Completed. This live order was already saved to data/orders.csv."
                    .to_string()
            } else {
                "Order status updated.".to_string()
            };
            Json(ApiResponse::ok(message, Some(update)))
        }
        Err(message) => Json(ApiResponse::error(message)),
    }
}

/// Adds or updates a dish in memory.
///
/// This gives the FYP admin flow real behaviour while keeping persistence
/// lightweight. CSV export can be used to save the current in-memory menu.
pub async fn upsert_dish(
    State(state): State<WebState>,
    headers: HeaderMap,
    Json(payload): Json<UpsertDishRequest>,
) -> Json<ApiResponse<DishView>> {
    if !is_admin_authenticated(&state, &headers) {
        return Json(ApiResponse::error("Admin login required."));
    }
    if let Some(dish_id) = payload.dish_id.as_deref()
        && !dish_id.trim().is_empty()
        && state.admin_dish_by_id(dish_id).is_some()
    {
        return Json(ApiResponse::error(format!(
            "Dish {} already exists. Use Edit instead.",
            dish_id.trim().to_uppercase()
        )));
    }
    match state.upsert_dish(payload) {
        Ok(dish) => Json(ApiResponse::ok("Dish saved in memory.", Some(dish))),
        Err(message) => Json(ApiResponse::error(message)),
    }
}

/// Returns the current dish values used to prefill the admin edit form.
pub async fn get_dish(
    State(state): State<WebState>,
    headers: HeaderMap,
    Path(dish_id): Path<String>,
) -> Json<ApiResponse<DishView>> {
    if !is_admin_authenticated(&state, &headers) {
        return Json(ApiResponse::error("Admin login required."));
    }
    match state.admin_dish_by_id(&dish_id) {
        Some(dish) => Json(ApiResponse::ok("Dish loaded.", Some(dish))),
        None => Json(ApiResponse::error(format!("Dish {dish_id} was not found."))),
    }
}

/// Updates one existing dish while keeping its stable dish ID unchanged.
pub async fn update_dish(
    State(state): State<WebState>,
    headers: HeaderMap,
    Path(dish_id): Path<String>,
    Json(mut payload): Json<UpsertDishRequest>,
) -> Json<ApiResponse<DishView>> {
    if !is_admin_authenticated(&state, &headers) {
        return Json(ApiResponse::error("Admin login required."));
    }
    if state.admin_dish_by_id(&dish_id).is_none() {
        return Json(ApiResponse::error(format!("Dish {dish_id} was not found.")));
    }
    payload.dish_id = Some(dish_id);
    match state.upsert_dish(payload) {
        Ok(dish) => Json(ApiResponse::ok("Dish updated in memory.", Some(dish))),
        Err(message) => Json(ApiResponse::error(message)),
    }
}

/// Deletes a dish from the current in-memory menu.
pub async fn delete_dish(
    State(state): State<WebState>,
    headers: HeaderMap,
    Path(dish_id): Path<String>,
) -> Json<ApiResponse<()>> {
    if !is_admin_authenticated(&state, &headers) {
        return Json(ApiResponse::error("Admin login required."));
    }
    match state.delete_dish(&dish_id) {
        Ok(()) => Json(ApiResponse::ok("Dish removed from memory.", None)),
        Err(message) => Json(ApiResponse::error(message)),
    }
}

/// Marks a dish available or unavailable in the customer menu.
pub async fn set_dish_availability(
    State(state): State<WebState>,
    headers: HeaderMap,
    Path(dish_id): Path<String>,
    Json(payload): Json<AvailabilityRequest>,
) -> Json<ApiResponse<()>> {
    if !is_admin_authenticated(&state, &headers) {
        return Json(ApiResponse::error("Admin login required."));
    }
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
    headers: HeaderMap,
    Json(payload): Json<CsvImportRequest>,
) -> Json<ApiResponse<usize>> {
    if !is_admin_authenticated(&state, &headers) {
        return Json(ApiResponse::error("Admin login required."));
    }
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
pub async fn reload_dishes_from_file(
    State(state): State<WebState>,
    headers: HeaderMap,
) -> Json<ApiResponse<usize>> {
    if !is_admin_authenticated(&state, &headers) {
        return Json(ApiResponse::error("Admin login required."));
    }
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
    headers: HeaderMap,
    Json(payload): Json<CsvImportRequest>,
) -> Json<ApiResponse<usize>> {
    if !is_admin_authenticated(&state, &headers) {
        return Json(ApiResponse::error("Admin login required."));
    }
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
pub async fn reload_orders_from_file(
    State(state): State<WebState>,
    headers: HeaderMap,
) -> Json<ApiResponse<usize>> {
    if !is_admin_authenticated(&state, &headers) {
        return Json(ApiResponse::error("Admin login required."));
    }
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
pub async fn export_dishes_csv(State(state): State<WebState>, headers: HeaderMap) -> Response {
    if !is_admin_authenticated(&state, &headers) {
        return redirect_to_login();
    }
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
pub async fn export_orders_csv(State(state): State<WebState>, headers: HeaderMap) -> Response {
    if !is_admin_authenticated(&state, &headers) {
        return redirect_to_login();
    }
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
    headers: HeaderMap,
) -> Response {
    if !is_admin_authenticated(&state, &headers) {
        return redirect_to_login();
    }
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
pub struct AdminLoginRequest {
    pub username: String,
    pub password: String,
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

fn is_admin_authenticated(state: &WebState, headers: &HeaderMap) -> bool {
    admin_session_id_from_headers(headers)
        .is_some_and(|session_id| state.is_admin_session(&session_id))
}

pub(crate) fn is_admin_authenticated_for_handlers(state: &WebState, headers: &HeaderMap) -> bool {
    is_admin_authenticated(state, headers)
}

fn admin_session_id_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get(COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let (name, value) = cookie.trim().split_once('=')?;
                (name == ADMIN_SESSION_COOKIE).then(|| value.to_string())
            })
        })
}

fn admin_session_cookie(session_id: &str) -> String {
    format!(
        "{ADMIN_SESSION_COOKIE}={session_id}; HttpOnly; SameSite=Lax; Path=/; Max-Age={ADMIN_SESSION_MAX_AGE_SECONDS}{}",
        super::customer::secure_cookie_attribute()
    )
}

fn expired_admin_session_cookie() -> String {
    format!(
        "{ADMIN_SESSION_COOKIE}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0{}",
        super::customer::secure_cookie_attribute()
    )
}

fn admin_credentials() -> Result<(String, String), &'static str> {
    let username = env::var("ADMIN_USERNAME").ok();
    let password = env::var("ADMIN_PASSWORD").ok();
    match (username, password) {
        (Some(username), Some(password)) if !username.trim().is_empty() && !password.is_empty() => {
            Ok((username.trim().to_string(), password))
        }
        (None, None) if cfg!(debug_assertions) => {
            println!(
                "ADMIN_USERNAME and ADMIN_PASSWORD are unset; using documented local debug credentials."
            );
            Ok(("admin".to_string(), "admin".to_string()))
        }
        _ => Err("Admin credentials are not configured on the server."),
    }
}

fn redirect_to_login() -> Response {
    ([(LOCATION, "/admin/login")], StatusCode::FOUND).into_response()
}

fn redirect_to_admin_recommendations() -> Response {
    ([(LOCATION, "/admin/recommendations")], StatusCode::FOUND).into_response()
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
    pub(crate) fn ok(message: impl Into<String>, data: Option<T>) -> Self {
        Self {
            ok: true,
            message: message.into(),
            data,
        }
    }

    pub(crate) fn error(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            message: message.into(),
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::HeaderValue;

    fn login(username: &str, password: &str) -> AdminLoginRequest {
        AdminLoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    async fn response_body(response: Response) -> String {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        String::from_utf8(bytes.to_vec()).expect("response should be UTF-8")
    }

    #[tokio::test]
    async fn valid_admin_login_sets_separate_cookie_and_unlocks_dashboard() {
        let state = WebState::new(Vec::new(), Vec::new());
        let response = process_admin_login(
            &state,
            login("staff", "secret"),
            Ok(("staff".to_string(), "secret".to_string())),
        );

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get(LOCATION).unwrap(), "/admin");
        let set_cookie = response
            .headers()
            .get(SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(set_cookie.starts_with("admin_session="));
        assert!(!set_cookie.starts_with("customer_session="));
        assert!(set_cookie.contains("HttpOnly"));
        assert!(set_cookie.contains("SameSite=Lax"));
        assert!(set_cookie.contains("Path=/"));
        assert!(set_cookie.contains("Max-Age="));
        assert!(!set_cookie.contains("Secure"));

        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_str(set_cookie.split(';').next().unwrap()).unwrap(),
        );
        let dashboard = admin_page(State(state), headers).await;
        assert_eq!(dashboard.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn invalid_login_preserves_username_but_not_password() {
        let state = WebState::new(Vec::new(), Vec::new());
        let response = process_admin_login(
            &state,
            login("staff", "wrong-secret"),
            Ok(("staff".to_string(), "correct-secret".to_string())),
        );

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = response_body(response).await;
        assert!(body.contains(r#"value="staff""#));
        assert!(body.contains("Invalid username or password."));
        assert!(!body.contains("wrong-secret"));
    }

    #[tokio::test]
    async fn missing_credentials_returns_clear_configuration_error() {
        let state = WebState::new(Vec::new(), Vec::new());
        let response = process_admin_login(
            &state,
            login("staff", "secret"),
            Err("Admin credentials are not configured on the server."),
        );

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = response_body(response).await;
        assert!(body.contains("Admin credentials are not configured on the server."));
        assert!(body.contains(r#"value="staff""#));
        assert!(!body.contains(r#"value="secret""#));
    }

    #[tokio::test]
    async fn protected_admin_route_rejects_missing_cookie() {
        let response = admin_page(
            State(WebState::new(Vec::new(), Vec::new())),
            HeaderMap::new(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(response.headers().get(LOCATION).unwrap(), "/admin/login");
    }

    #[tokio::test]
    async fn admin_logout_expires_only_admin_cookie() {
        let state = WebState::new(Vec::new(), Vec::new());
        let session_id = state.create_admin_session();
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_str(&format!(
                "customer_session=customer-1; admin_session={session_id}"
            ))
            .unwrap(),
        );

        let response = admin_logout(State(state.clone()), headers).await;
        let set_cookie = response
            .headers()
            .get(SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(set_cookie.starts_with("admin_session="));
        assert!(!set_cookie.contains("customer_session"));
        assert!(!state.is_admin_session(&session_id));
    }
}
