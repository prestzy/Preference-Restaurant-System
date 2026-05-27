use super::components::{ImagePreviewState, render_image_preview_modal};
use super::pages::{show_admin_demo_tools, show_dashboard, show_evaluation, show_explore};
use super::state::{AppPage, AppState};
use crate::image_loader::DishImageCache;
use crate::models::{Dish, Order};
use eframe::egui;

/// Top-level eframe application object.
///
/// Rendering work is delegated to page functions so this file remains focused on
/// window-level application flow.
pub struct RestaurantOrderingApp {
    state: AppState,
    image_cache: DishImageCache,
    image_preview: Option<ImagePreviewState>,
}

impl RestaurantOrderingApp {
    /// Creates the GUI after CSV data has been loaded in `main.rs`.
    pub fn new(dishes: Vec<Dish>, orders: Vec<Order>, ctx: &egui::Context) -> Self {
        apply_white_theme(ctx);

        Self {
            state: AppState::new(dishes, orders),
            image_cache: DishImageCache::new(),
            image_preview: None,
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

/// Applies a clean white restaurant-ordering theme to egui.
///
/// `Visuals::light` gives the app a white/light-gray base. The overrides keep
/// cards and panels bright, text dark, and selected controls visible with a
/// restrained blue accent.
fn apply_white_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = egui::Color32::from_rgb(248, 250, 252);
    visuals.window_fill = egui::Color32::WHITE;
    visuals.extreme_bg_color = egui::Color32::from_rgb(241, 245, 249);
    visuals.selection.bg_fill = egui::Color32::from_rgb(219, 234, 254);
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(37, 99, 235));
    visuals.widgets.inactive.bg_fill = egui::Color32::WHITE;
    visuals.widgets.inactive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(226, 232, 240));
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(239, 246, 255);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(219, 234, 254);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    ctx.set_style(style);
}

impl eframe::App for RestaurantOrderingApp {
    /// Main egui frame update.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.show_navigation(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            let active_page = self.state.active_page;
            let available_width = ui.available_width();

            match active_page {
                // Explore has its own internal scroll regions for menu,
                // preferences, and selected dishes. Avoiding a page-level
                // scroll area lets the menu panel fill the bottom of the
                // window instead of leaving unused vertical space.
                AppPage::ExploreRecommend => show_explore(
                    ui,
                    &mut self.state,
                    &mut self.image_cache,
                    &mut self.image_preview,
                    available_width,
                ),
                _ => {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| match active_page {
                            AppPage::ExploreRecommend => unreachable!(),
                            AppPage::Dashboard => show_dashboard(ui, &self.state),
                            AppPage::Evaluation => show_evaluation(
                                ui,
                                &self.state,
                                &mut self.image_cache,
                                &mut self.image_preview,
                            ),
                            AppPage::AdminDemoTools => show_admin_demo_tools(ui, &mut self.state),
                        });
                }
            };
        });

        // Image preview state is owned once at app level and rendered after the
        // active page. Both menu thumbnails and recommendation thumbnails set
        // the same state, so the modal behaviour stays consistent everywhere.
        render_image_preview_modal(
            ctx,
            &self.state.dishes,
            &mut self.image_cache,
            &mut self.image_preview,
        );
    }
}
