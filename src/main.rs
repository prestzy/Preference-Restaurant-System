mod data_loader;
mod models;
#[allow(dead_code)]
mod preferences;
mod recommender;
#[allow(dead_code)]
mod search;
#[allow(dead_code)]
mod simulation;
mod web;

use anyhow::{Context, Result};
use data_loader::{
    DISHES_PATH, ORDERS_PATH, generate_sample_data_if_missing, load_dishes, load_orders,
};
use tokio::net::TcpListener;
use web::state::WebState;

/// Program entry point for the web-based FYP prototype.
///
/// Startup flow:
/// 1. Ensure sample CSV files exist so the prototype runs immediately.
/// 2. Load dishes and historical orders from CSV using the existing data layer.
/// 3. Store loaded data in web state.
/// 4. Start an Axum server for the responsive QR-code restaurant menu.
#[tokio::main]
async fn main() -> Result<()> {
    generate_sample_data_if_missing().context("failed to create sample CSV data")?;

    let dishes = load_dishes(DISHES_PATH).context("failed to load dishes.csv")?;
    let orders = load_orders(ORDERS_PATH).context("failed to load orders.csv")?;
    let state = WebState::new(dishes, orders);
    let app = web::routes::router(state);

    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .context("failed to bind web server to 127.0.0.1:3000")?;

    println!("Preference-Driven Restaurant Ordering System");
    println!("Customer menu: http://127.0.0.1:3000/");
    println!("Admin page:    http://127.0.0.1:3000/admin");

    axum::serve(listener, app)
        .await
        .context("web server failed")?;

    Ok(())
}
