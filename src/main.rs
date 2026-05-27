mod data_loader;
mod gui;
mod image_loader;
mod models;
mod preferences;
mod recommender;
mod search;
mod simulation;

use anyhow::{Context, Result};
use data_loader::{
    DISHES_PATH, ORDERS_PATH, generate_sample_data_if_missing, load_dishes, load_orders,
};
use gui::RestaurantOrderingApp;
use image_loader::ensure_dish_image_folder;

/// Program entry point.
///
/// Startup flow:
/// 1. Ensure sample CSV files exist so the prototype can run immediately.
/// 2. Load and clean dish/order data.
/// 3. Start the eframe/egui desktop GUI.
fn main() -> Result<()> {
    generate_sample_data_if_missing().context("failed to create sample CSV data")?;
    ensure_dish_image_folder().context("failed to create dish image folder")?;

    let dishes = load_dishes(DISHES_PATH).context("failed to load dishes.csv")?;
    let orders = load_orders(ORDERS_PATH).context("failed to load orders.csv")?;

    // A larger default window makes the prototype usable immediately during a
    // demo: the menu, preference panel, and selected dishes can be seen without
    // the cramped first-launch layout. The minimum size keeps the two-column UI
    // from collapsing into an unreadable state.
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1500.0, 900.0])
            .with_min_inner_size([1100.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Preference-Driven Restaurant Ordering System",
        native_options,
        Box::new(move |creation_context| {
            Box::new(RestaurantOrderingApp::new(
                dishes,
                orders,
                &creation_context.egui_ctx,
            ))
        }),
    )
    .map_err(|error| anyhow::anyhow!("failed to start GUI: {error}"))?;

    Ok(())
}
