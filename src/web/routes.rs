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
        .route(
            "/start",
            get(handlers::customer::start_page).post(handlers::customer::start_submit),
        )
        .route(
            "/profile",
            get(handlers::customer::profile_page).post(handlers::customer::profile_submit),
        )
        .route("/profile/end", post(handlers::customer::end_session))
        .route("/cart", get(handlers::cart::cart_page))
        .route("/orders", get(handlers::orders::orders_page))
        .route(
            "/admin/login",
            get(handlers::admin::admin_login_page).post(handlers::admin::admin_login_submit),
        )
        .route("/admin/logout", get(handlers::admin::admin_logout))
        .route("/admin", get(handlers::admin::admin_page))
        .route("/admin/orders", get(handlers::admin::admin_orders_page))
        .route("/admin/dishes", get(handlers::admin::admin_dishes_page))
        .route(
            "/admin/recommendations",
            get(handlers::admin::admin_recommendations_page),
        )
        .route(
            "/admin/evaluation",
            get(handlers::admin::admin_evaluation_page),
        )
        .route("/admin/maintenance", get(handlers::admin::admin_data_page))
        .route("/admin/insights", get(handlers::admin::admin_insights_page))
        .route("/api/orders", post(handlers::cart::create_order))
        .route("/api/orders/my", get(handlers::orders::my_orders))
        .route("/api/search", get(handlers::search::menu_search))
        .route("/api/profile/orders", get(handlers::orders::profile_orders))
        .route(
            "/api/customer/orders",
            get(handlers::orders::profile_orders),
        )
        .route(
            "/api/recommendations",
            post(handlers::recommendations::recommendations),
        )
        .route(
            "/api/recommendations/meal-set",
            post(handlers::advanced_recommendations::meal_sets),
        )
        .route(
            "/api/assistant/recommendations",
            post(handlers::assistant::smart_menu_assistant),
        )
        .route(
            "/api/admin/insights",
            get(handlers::assistant::admin_insights),
        )
        .route(
            "/api/admin/simulation",
            post(handlers::admin::run_simulation),
        )
        .route(
            "/api/admin/experiment-lab",
            post(handlers::admin::run_experiment_lab),
        )
        .route(
            "/api/admin/recommendations/counterfactual",
            post(handlers::advanced_recommendations::counterfactual),
        )
        .route(
            "/api/admin/recommendations/learning-timeline",
            get(handlers::advanced_recommendations::learning_timeline),
        )
        .route(
            "/api/admin/recommendations/learning-timeline/rebuild",
            post(handlers::advanced_recommendations::rebuild_learning_timeline),
        )
        .route("/api/admin/orders", get(handlers::admin::admin_orders_sync))
        .route(
            "/api/admin/orders/:order_id/status",
            post(handlers::admin::update_order_status),
        )
        .route("/api/admin/dishes", post(handlers::admin::upsert_dish))
        .route(
            "/api/admin/dishes/:dish_id",
            get(handlers::admin::get_dish).put(handlers::admin::update_dish),
        )
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
            "/api/admin/reload/dishes",
            post(handlers::admin::reload_dishes_from_file),
        )
        .route(
            "/api/admin/import/orders",
            post(handlers::admin::import_orders_csv),
        )
        .route(
            "/api/admin/reload/orders",
            post(handlers::admin::reload_orders_from_file),
        )
        .route(
            "/admin/export/dishes.csv",
            get(handlers::admin::export_dishes_csv),
        )
        .route(
            "/admin/export/orders.csv",
            get(handlers::admin::export_orders_csv),
        )
        .route(
            "/admin/export/completed-session-orders.csv",
            get(handlers::admin::export_completed_session_orders_csv),
        )
        .route("/api/orders/:order_id", get(handlers::orders::order_status))
        // Static files are local only. Dish images are served from `/assets`
        // so CSV image paths such as `assets/dishes/D01.jpg` become
        // `/assets/dishes/D01.jpg` in customer/admin pages.
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state)
}
