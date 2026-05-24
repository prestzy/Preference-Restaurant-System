use super::pages::{show_admin_demo_tools, show_dashboard, show_evaluation, show_explore};
use super::state::{AppPage, AppState};
use crate::models::{Dish, Order};
use eframe::egui;

/// Top-level eframe application object.
///
/// Rendering work is delegated to page functions so this file remains focused on
/// window-level application flow.
pub struct RestaurantOrderingApp {
    state: AppState,
}

impl RestaurantOrderingApp {
    /// Creates the GUI after CSV data has been loaded in `main.rs`.
    pub fn new(dishes: Vec<Dish>, orders: Vec<Order>) -> Self {
        Self {
            state: AppState::new(dishes, orders),
        }
    }

    /// Draws the compact navigation used across all pages.
    fn show_navigation(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("navigation")
            .resizable(false)
            .default_width(190.0)
            .width_range(160.0..=220.0)
            .show(ctx, |ui| {
                ui.heading("FYP Prototype");
                ui.label("Restaurant Ordering");
                ui.separator();

                ui.selectable_value(&mut self.state.active_page, AppPage::Dashboard, "Dashboard");
                ui.selectable_value(
                    &mut self.state.active_page,
                    AppPage::ExploreRecommend,
                    "Explore & Recommend",
                );
                ui.selectable_value(
                    &mut self.state.active_page,
                    AppPage::Evaluation,
                    "Evaluation",
                );
                ui.selectable_value(
                    &mut self.state.active_page,
                    AppPage::AdminDemoTools,
                    "Admin / Demo Tools",
                );

                ui.separator();
                ui.label(format!("Dishes: {}", self.state.dishes.len()));
                ui.label(format!("Orders: {}", self.state.orders.len()));
            });
    }
}

impl eframe::App for RestaurantOrderingApp {
    /// Main egui frame update.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.show_navigation(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            let available_width = ui.available_width();
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| match self.state.active_page {
                    AppPage::Dashboard => show_dashboard(ui, &self.state),
                    AppPage::ExploreRecommend => show_explore(ui, &mut self.state, available_width),
                    AppPage::Evaluation => show_evaluation(ui, &self.state),
                    AppPage::AdminDemoTools => show_admin_demo_tools(ui, &mut self.state),
                });
        });
    }
}
