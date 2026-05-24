use super::components::{chip_row, display_list, labelled_text_input, recommendation_card};
use super::state::AppState;
use crate::models::Dish;
use crate::search::{MatchMode, SearchFilter, filter_dishes};
use eframe::egui;

/// Width at which the Explore page moves from stacked sections to side-by-side panels.
const WIDE_LAYOUT_THRESHOLD: f32 = 980.0;

/// Dashboard with the prototype purpose and loaded data counts.
pub fn show_dashboard(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("Preference-Driven Restaurant Ordering System");
    ui.separator();

    ui.label(format!("Loaded dishes: {}", state.dishes.len()));
    ui.label(format!("Loaded historical orders: {}", state.orders.len()));
    ui.add_space(8.0);
    ui.label(
        "This Rust desktop prototype recommends dishes for one restaurant by combining explicit food preferences with historical co-order patterns.",
    );
    ui.label(
        "The main workflow is in Explore & Recommend, where menu browsing, preference entry, dish selection, and recommendation output stay visible together.",
    );
}

/// Main end-user workflow.
///
/// On wide windows this shows menu browsing beside preferences and results. On
/// narrow windows the same sections stack vertically so the UI remains readable
/// on laptop resolutions.
pub fn show_explore(ui: &mut egui::Ui, state: &mut AppState, available_width: f32) {
    ui.heading("Explore & Recommend");
    ui.label("Browse dishes, select what the customer is considering, and watch recommendations update automatically.");
    ui.separator();

    if available_width >= WIDE_LAYOUT_THRESHOLD {
        ui.columns(2, |columns| {
            show_menu_panel(&mut columns[0], state);
            show_preference_and_results_panel(&mut columns[1], state);
        });
    } else {
        show_preference_and_results_panel(ui, state);
        ui.separator();
        show_menu_panel(ui, state);
    }
}

/// Searchable menu panel with direct dish selection.
fn show_menu_panel(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("Menu");
    ui.label("Search by name, ID, category, ingredient, or tag.");

    let filter_changed = ui
        .add(
            egui::TextEdit::multiline(&mut state.menu_search)
                .desired_rows(2)
                .hint_text("Example: chicken, spicy; D01"),
        )
        .changed();

    ui.horizontal_wrapped(|ui| {
        ui.label("Mode:");
        ui.selectable_value(
            &mut state.search_match_mode,
            MatchMode::Any,
            MatchMode::Any.label(),
        );
        ui.selectable_value(
            &mut state.search_match_mode,
            MatchMode::All,
            MatchMode::All.label(),
        );
    });

    let filter = SearchFilter::parse(&state.menu_search, state.search_match_mode);
    if !filter.terms.is_empty() {
        ui.horizontal_wrapped(|ui| {
            ui.label("Active filters:");
            chip_row(ui, &filter.terms);
        });
    }

    // Match mode changes affect filtering only. They do not need to refresh the
    // recommendation engine because recommendations depend on preferences and
    // selected dishes, not on the visible menu filter.
    if filter_changed {
        ui.ctx().request_repaint();
    }

    let filtered_dishes = filter_dishes(&state.dishes, &filter);
    ui.label(format!("Showing {} dish(es)", filtered_dishes.len()));

    let mut toggled_dish_id = None;

    egui::ScrollArea::vertical()
        .id_source("menu_cards_scroll")
        .max_height(520.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for dish in filtered_dishes {
                if dish_card(ui, dish, state.selected_dish_ids.contains(&dish.dish_id)) {
                    toggled_dish_id = Some(dish.dish_id.clone());
                }
                ui.add_space(8.0);
            }
        });

    if let Some(dish_id) = toggled_dish_id {
        state.toggle_dish_selection(&dish_id);
    }
}

/// One readable dish card with metadata and a selection button.
fn dish_card(ui: &mut egui::Ui, dish: &Dish, selected: bool) -> bool {
    let mut clicked = false;

    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.strong(format!("{} ({})", dish.name, dish.dish_id));
            ui.label(format!("Category: {}", dish.category));
        });

        ui.label(format!("Ingredients: {}", dish.ingredients.join(", ")));
        ui.label(format!("Tags: {}", dish.tags.join(", ")));

        let label = if selected { "Selected" } else { "Select" };
        if ui.selectable_label(selected, label).clicked() {
            clicked = true;
        }
    });

    clicked
}

/// Combined preference input and recommendation output panel.
fn show_preference_and_results_panel(ui: &mut egui::Ui, state: &mut AppState) {
    show_preference_panel(ui, state);
    ui.separator();
    show_recommendation_panel(ui, state);
}

/// Preference input panel.
fn show_preference_panel(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("Preferences");
    ui.label("Comma-separated values are accepted. Changes refresh recommendations automatically.");

    let mut liked_ingredients = state.liked_ingredients_input.clone();
    let mut disliked_ingredients = state.disliked_ingredients_input.clone();
    let mut preferred_tags = state.preferred_tags_input.clone();
    let mut manual_selected_dishes = state.manual_selected_dishes_input.clone();

    let mut changed = false;
    changed |= labelled_text_input(
        ui,
        "Liked ingredients",
        "chicken, rice, egg",
        &mut liked_ingredients,
    );
    changed |= labelled_text_input(
        ui,
        "Disliked ingredients",
        "beef, anchovies",
        &mut disliked_ingredients,
    );
    changed |= labelled_text_input(
        ui,
        "Preferred tags",
        "spicy, signature",
        &mut preferred_tags,
    );
    changed |= labelled_text_input(
        ui,
        "Manual selected dish IDs (fallback)",
        "D01, D03",
        &mut manual_selected_dishes,
    );

    if changed {
        state.set_preference_inputs(
            liked_ingredients,
            disliked_ingredients,
            preferred_tags,
            manual_selected_dishes,
        );
    }

    let selected = state.combined_selected_dish_ids();
    ui.add_space(6.0);
    ui.label(format!("Selected dishes: {}", display_list(&selected)));
    if !selected.is_empty() {
        chip_row(ui, &selected);
    }
}

/// Recommendation output displayed close to preferences and menu items.
fn show_recommendation_panel(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("Recommendations");
    ui.label("Hybrid score = ingredient preference evidence + co-order evidence.");

    if state.recommendation_output.recommendations.is_empty() {
        ui.label("No recommendations yet. Enter preferences or select dishes from the menu.");
        return;
    }

    egui::ScrollArea::vertical()
        .id_source("recommendation_scroll")
        .max_height(520.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (rank, recommendation) in state
                .recommendation_output
                .recommendations
                .iter()
                .take(8)
                .enumerate()
            {
                recommendation_card(ui, rank + 1, recommendation);
                ui.add_space(8.0);
            }
        });
}

/// Evaluation page with demo metrics.
pub fn show_evaluation(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("Evaluation / Prototype Testing");
    ui.label("Simple demonstration metrics for the current Explore & Recommend state.");
    ui.separator();

    let stats = &state.recommendation_output.stats;
    ui.label(format!(
        "Top recommendations available: {}",
        state.recommendation_output.recommendations.len()
    ));
    ui.label(format!(
        "Dishes evaluated after filters: {}",
        stats.filtered_dishes
    ));
    ui.label(format!(
        "Dishes excluded due to disliked ingredients: {}",
        stats.excluded_due_to_disliked
    ));
    ui.label(format!(
        "Already selected dishes skipped: {}",
        stats.skipped_selected_dishes
    ));
    ui.label(format!(
        "Category diversity count in top 5: {}",
        stats.diversity_count_top_5
    ));

    ui.separator();
    ui.strong("Top 5 recommendations");
    for (rank, recommendation) in state
        .recommendation_output
        .recommendations
        .iter()
        .take(5)
        .enumerate()
    {
        recommendation_card(ui, rank + 1, recommendation);
        ui.add_space(8.0);
    }
}

/// Admin/demo section for order simulation.
pub fn show_admin_demo_tools(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("Admin / Demo Tools");
    ui.label(
        "Order simulation is for demonstrations and testing. It creates new behavioural data so evaluators can see collaborative filtering respond to added order history.",
    );
    ui.separator();

    ui.label("Simulated order dish IDs");
    ui.add(
        egui::TextEdit::singleline(&mut state.simulated_order_input)
            .hint_text("Example: D01, D09, D30"),
    );
    ui.checkbox(
        &mut state.append_simulated_orders_to_csv,
        "Also append simulated order to data/orders.csv",
    );

    ui.horizontal_wrapped(|ui| {
        if ui.button("Use selected dishes").clicked() {
            state.simulated_order_input = state.combined_selected_dish_ids().join(", ");
        }
        if ui.button("Create simulated order").clicked() {
            state.create_simulated_order();
        }
        if ui.button("Clear").clicked() {
            state.simulated_order_input.clear();
            state.last_order_message.clear();
        }
    });

    if !state.last_order_message.is_empty() {
        ui.separator();
        ui.label(&state.last_order_message);
    }

    ui.separator();
    ui.strong("Most recent orders in memory");
    egui::ScrollArea::vertical()
        .max_height(260.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for order in state.orders.iter().rev().take(10) {
                ui.label(format!(
                    "{} | {} | {} | {}",
                    order.order_id,
                    order.session_user_id,
                    order.ordered_dishes.join(", "),
                    order.timestamp
                ));
            }
        });
}
