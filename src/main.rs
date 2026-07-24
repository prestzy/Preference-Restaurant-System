mod agent;
mod data_loader;
mod models;
mod persistence;
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
use persistence::learning_events::{LEARNING_EVENTS_PATH, load_learning_events};
use persistence::order_details::{ORDER_DETAILS_PATH, load_order_details};
use std::env;
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
    let order_details =
        load_order_details(ORDER_DETAILS_PATH).context("failed to load order_details.csv")?;
    // A missing or malformed explanatory timeline must not prevent ordering.
    // Durable order history remains authoritative and staff can rebuild the
    // timeline from the protected Recommendation Tester.
    let (learning_events, timeline_warning) = match load_learning_events(LEARNING_EVENTS_PATH) {
        Ok(events) => (events, None),
        Err(error) => {
            let warning = format!(
                "Recommendation timeline could not be loaded and should be rebuilt: {error}"
            );
            eprintln!("{warning}");
            (Vec::new(), Some(warning))
        }
    };
    let state = WebState::new_with_operational_data(
        dishes,
        orders,
        order_details,
        learning_events,
        timeline_warning,
    );
    let app = web::routes::router(state);

    // Loopback remains the safe local default. Setting APP_HOST=0.0.0.0 makes
    // the same server reachable from a phone on the local network.
    let host = env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("APP_PORT").unwrap_or_else(|_| "3000".to_string());
    let bind_address = format!("{host}:{port}");
    let listener = TcpListener::bind(&bind_address)
        .await
        .with_context(|| format!("failed to bind web server to {bind_address}"))?;

    println!("Preference-Driven Restaurant Ordering System");
    println!("Listening on:  http://{bind_address}/");
    println!("Customer menu: http://127.0.0.1:{port}/");
    println!("Admin page:    http://127.0.0.1:{port}/admin");
    if host == "0.0.0.0" {
        println!("Phone access:  open http://<this-computer-LAN-IP>:{port}/");
    }

    axum::serve(listener, app)
        .await
        .context("web server failed")?;

    Ok(())
}
