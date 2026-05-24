//! egui desktop interface for the FYP prototype.
//!
//! Rendering is split across focused modules:
//! - `state` owns GUI state and recommendation refresh behaviour.
//! - `app` connects state to the eframe application loop.
//! - `pages` renders the main screens.
//! - `components` contains reusable visual helpers.

mod app;
mod components;
mod pages;
mod state;

pub use app::RestaurantOrderingApp;
