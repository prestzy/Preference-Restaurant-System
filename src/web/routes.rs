use crate::web::handlers;
use crate::web::state::WebState;
use axum::Router;
use axum::routing::{get, post};
use tower_http::services::ServeDir;

/// Builds the complete Axum router.
///
/// The router intentionally only maps URLs to focused handler modules. Keeping
/// route declaration separate from menu, cart, admin, and recommendation logic
/// avoids turning `main.rs` or this file into another large application file.
pub fn router(state: WebState) -> Router {
    Router::new()
        .route("/", get(handlers::menu::customer_menu))
        .route("/cart", get(handlers::cart::cart_page))
        .route("/orders", get(handlers::orders::orders_page))
        .route("/admin", get(handlers::admin::admin_page))
        .route("/api/orders", post(handlers::cart::create_order))
        .route(
            "/api/recommendations",
            post(handlers::recommendations::recommendations),
        )
        .route(
            "/api/admin/orders/:order_id/status",
            post(handlers::admin::update_order_status),
        )
        .route("/api/admin/dishes", post(handlers::admin::upsert_dish))
        .route(
            "/api/admin/dishes/:dish_id/delete",
            post(handlers::admin::delete_dish),
        )
        .route(
            "/api/admin/dishes/:dish_id/availability",
            post(handlers::admin::set_dish_availability),
        )
        .route(
            "/api/admin/import/dishes",
            post(handlers::admin::import_dishes_csv),
        )
        .route(
            "/api/admin/import/orders",
            post(handlers::admin::import_orders_csv),
        )
        .route(
            "/admin/export/dishes.csv",
            get(handlers::admin::export_dishes_csv),
        )
        .route(
            "/admin/export/orders.csv",
            get(handlers::admin::export_orders_csv),
        )
        // Static files are local only. Dish images are served from `/assets`
        // so CSV image paths such as `assets/dishes/D01.jpg` become
        // `/assets/dishes/D01.jpg` in customer/admin pages.
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state)
}
