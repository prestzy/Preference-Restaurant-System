mod data_loader;
mod gui;
mod models;
mod recommender;
mod search;
mod simulation;

use anyhow::{Context, Result};
use data_loader::{
    DISHES_PATH, ORDERS_PATH, generate_sample_data_if_missing, load_dishes, load_orders,
};
use gui::RestaurantOrderingApp;

/// Program entry point.
///
/// Startup flow:
/// 1. Ensure sample CSV files exist so the prototype can run immediately.
/// 2. Load and clean dish/order data.
/// 3. Start the eframe/egui desktop GUI.
fn main() -> Result<()> {
    generate_sample_data_if_missing().context("failed to create sample CSV data")?;

    let dishes = load_dishes(DISHES_PATH).context("failed to load dishes.csv")?;
    let orders = load_orders(ORDERS_PATH).context("failed to load orders.csv")?;

    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Preference-Driven Restaurant Ordering System",
        native_options,
        Box::new(move |_creation_context| Box::new(RestaurantOrderingApp::new(dishes, orders))),
    )
    .map_err(|error| anyhow::anyhow!("failed to start GUI: {error}"))?;

    Ok(())
}
