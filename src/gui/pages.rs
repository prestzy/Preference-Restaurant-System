use super::components::{chip_row, option_chip, recommendation_card, selected_dish_pills};
use super::state::AppState;
use crate::models::Dish;
use crate::search::{MatchMode, SearchFilter, filter_dishes};
use eframe::egui;

/// Width at which the Explore page moves from stacked sections to side-by-side panels.
const WIDE_LAYOUT_THRESHOLD: f32 = 980.0;
const NARROW_MENU_HEIGHT: f32 = 420.0;
const NARROW_OPTION_LIST_HEIGHT: f32 = 120.0;

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
        "Explore & Recommend collects menu selections and preferences, while Evaluation shows the generated recommendation output and reasoning.",
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
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    if available_width >= WIDE_LAYOUT_THRESHOLD {
        let content_height = ui.available_height().max(360.0);
        let option_list_height = (content_height * 0.16).clamp(96.0, 140.0);

        ui.columns(2, |columns| {
            show_menu_panel(&mut columns[0], state, None);
            egui::ScrollArea::vertical()
                .id_source("preference_cart_scroll")
                .max_height(content_height)
                .auto_shrink([false, false])
                .show(&mut columns[1], |ui| {
                    show_preference_and_cart_panel(ui, state, option_list_height);
                });
        });
    } else {
        // On narrow windows the panels stack vertically. This wrapper scrolls
        // the stacked layout while each heavy section still keeps its own
        // bounded scroll list, so no single option group dominates the page.
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                show_preference_and_cart_panel(ui, state, NARROW_OPTION_LIST_HEIGHT);
                ui.separator();
                ui.add_space(10.0);
                show_menu_panel(ui, state, Some(NARROW_MENU_HEIGHT));
            });
    }
}

/// Searchable menu panel with direct dish selection.
fn show_menu_panel(ui: &mut egui::Ui, state: &mut AppState, requested_list_height: Option<f32>) {
    ui.heading("Menu");
    ui.label("Search by name, ID, category, ingredient, or tag.");
    ui.add_space(6.0);

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
    let list_height = requested_list_height.unwrap_or_else(|| ui.available_height().max(240.0));

    egui::ScrollArea::vertical()
        .id_source("menu_cards_scroll")
        .max_height(list_height)
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

/// One readable dish card with metadata and a clear selection button.
///
/// The card uses bold labels for category, ingredients, and tags so viewers can
/// quickly understand what each line means. The action is styled as a real
/// button because plain text made the original Select affordance easy to miss.
fn dish_card(ui: &mut egui::Ui, dish: &Dish, selected: bool) -> bool {
    let mut clicked = false;

    egui::Frame::none()
        .fill(egui::Color32::WHITE)
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgb(226, 232, 240),
        ))
        .rounding(8.0)
        .inner_margin(egui::Margin::symmetric(14.0, 12.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new(&dish.name).strong().size(16.0));
                ui.monospace(format!("({})", dish.dish_id));
                ui.add_space(12.0);
                ui.strong("Category:");
                ui.label(display_category(&dish.category));
            });

            ui.add_space(8.0);
            labelled_metadata(ui, "Ingredients:", &dish.ingredients.join(", "));
            ui.add_space(4.0);
            labelled_metadata(ui, "Tags:", &dish.tags.join(", "));

            ui.add_space(10.0);
            // Clicking the selected state toggles the dish off because
            // `toggle_dish_selection` already supports select/unselect.
            if select_dish_button(ui, selected).clicked() {
                clicked = true;
            }
        });

    clicked
}

/// Draws a prominent Select/Selected button for a dish card.
///
/// Before selection it uses a filled accent button labeled "Select Dish". After
/// selection it switches to a visible selected state so users can clearly see
/// which dishes are already part of the recommendation input.
fn select_dish_button(ui: &mut egui::Ui, selected: bool) -> egui::Response {
    let (label, fill, text_color, stroke) = if selected {
        (
            "Selected",
            egui::Color32::from_rgb(219, 234, 254),
            egui::Color32::from_rgb(30, 64, 175),
            egui::Stroke::new(1.0, egui::Color32::from_rgb(37, 99, 235)),
        )
    } else {
        (
            "Select Dish",
            egui::Color32::from_rgb(37, 99, 235),
            egui::Color32::WHITE,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(37, 99, 235)),
        )
    };

    ui.add(
        egui::Button::new(egui::RichText::new(label).strong().color(text_color))
            .fill(fill)
            .stroke(stroke)
            .rounding(6.0)
            .min_size(egui::vec2(110.0, 32.0)),
    )
}

/// Displays one dish metadata row with a bold label for readability.
fn labelled_metadata(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.strong(label);
        ui.label(value);
    });
}

/// Converts the CSV category value into a friendlier display label.
fn display_category(category: &str) -> String {
    let mut chars = category.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
        None => "-".to_string(),
    }
}

/// Combined preference and cart panel.
///
/// Recommendations intentionally live on the Evaluation page now. Explore stays
/// focused on choosing menu items and preference inputs, making it less crowded
/// and easier to scan during a demo.
/// The right column is scrollable so long option lists stay readable instead of
/// pushing Selected Dishes out of view.
fn show_preference_and_cart_panel(
    ui: &mut egui::Ui,
    state: &mut AppState,
    option_list_height: f32,
) {
    show_preference_panel(ui, state, option_list_height);
    ui.add_space(16.0);
    ui.separator();
    ui.add_space(12.0);
    show_cart_panel(ui, state);
}

/// Preference input panel.
fn show_preference_panel(ui: &mut egui::Ui, state: &mut AppState, option_list_height: f32) {
    ui.heading("Preference Panel");
    ui.label("Choose from ingredients and tags found in dishes.csv. Recommendations update automatically.");
    ui.add_space(12.0);

    if let Some(action) = preference_option_sections(ui, state, option_list_height) {
        match action {
            PreferenceAction::LikedIngredient(ingredient) => {
                state.toggle_liked_ingredient(&ingredient)
            }
            PreferenceAction::DislikedIngredient(ingredient) => {
                state.toggle_disliked_ingredient(&ingredient)
            }
            PreferenceAction::PreferredTag(tag) => state.toggle_preferred_tag(&tag),
        }
    }
}

/// Selected dish display kept near preferences on Explore.
///
/// This section was renamed from "Selected Dishes / Cart" because it is not a
/// checkout cart. It shows the dishes selected as recommendation input.
fn show_cart_panel(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("Selected Dishes");
    let selected = state.selected_dish_labels();
    ui.add_space(8.0);
    if selected.is_empty() {
        ui.label("No dishes selected yet. Use the Select Dish buttons in the menu.");
    } else {
        ui.label(format!(
            "{} dish(es) selected for recommendation input:",
            selected.len()
        ));
        ui.add_space(8.0);
        selected_dish_pills(ui, &selected);
    }
}

/// User preference changes emitted from selectable option chips.
enum PreferenceAction {
    LikedIngredient(String),
    DislikedIngredient(String),
    PreferredTag(String),
}

/// Renders all generated preference option sections.
///
/// Ingredient and tag options come from the loaded dish dataset, so the user can
/// select known values instead of guessing free-text terms.
fn preference_option_sections(
    ui: &mut egui::Ui,
    state: &AppState,
    option_list_height: f32,
) -> Option<PreferenceAction> {
    let mut action = None;

    ui.add_space(8.0);
    if let Some(clicked_action) = option_section(
        ui,
        "Liked Ingredients",
        "Ingredients the customer wants to see more often.",
        option_list_height,
        &state.preference_options.ingredients,
        |ingredient| state.selected_liked_ingredients.contains(ingredient),
        |ingredient| PreferenceAction::LikedIngredient(ingredient.to_string()),
    ) {
        action = Some(clicked_action);
    }

    ui.add_space(14.0);
    if let Some(clicked_action) = option_section(
        ui,
        "Disliked Ingredients",
        "Dishes containing these ingredients are excluded from recommendations.",
        option_list_height,
        &state.preference_options.ingredients,
        |ingredient| state.selected_disliked_ingredients.contains(ingredient),
        |ingredient| PreferenceAction::DislikedIngredient(ingredient.to_string()),
    ) {
        action = Some(clicked_action);
    }

    ui.add_space(14.0);
    if let Some(clicked_action) = option_section(
        ui,
        "Preferred Tags",
        "Tags add a small bonus to the ingredient-based score.",
        option_list_height,
        &state.preference_options.tags,
        |tag| state.selected_preferred_tags.contains(tag),
        |tag| PreferenceAction::PreferredTag(tag.to_string()),
    ) {
        action = Some(clicked_action);
    }

    action
}

/// Renders one bounded scrollable option list inside a card.
///
/// Each preference group can contain many values from `dishes.csv`. Giving every
/// group its own scroll area keeps liked ingredients, disliked ingredients, and
/// preferred tags readable without pushing the recommendations off-screen.
///
/// The card boundary makes the three preference concepts visually distinct:
/// liked ingredients increase content score, disliked ingredients exclude
/// dishes, and preferred tags add a smaller bonus.
fn option_section(
    ui: &mut egui::Ui,
    title: &str,
    description: &str,
    list_height: f32,
    options: &[String],
    is_selected: impl Fn(&str) -> bool,
    to_action: impl Fn(&str) -> PreferenceAction,
) -> Option<PreferenceAction> {
    let mut action = None;

    egui::Frame::none()
        .fill(egui::Color32::WHITE)
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgb(226, 232, 240),
        ))
        .rounding(8.0)
        .inner_margin(egui::Margin::symmetric(14.0, 12.0))
        .show(ui, |ui| {
            ui.strong(title);
            ui.label(description);
            ui.add_space(8.0);
            egui::ScrollArea::vertical()
                .id_source(format!("option-list-{title}"))
                .max_height(list_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for option in options {
                            if option_chip(ui, option, is_selected(option)) {
                                action = Some(to_action(option));
                            }
                        }
                    });
                });
        });

    action
}

/// Evaluation page with demo metrics.
pub fn show_evaluation(ui: &mut egui::Ui, state: &AppState) {
    // The page is now primarily for recommendation review, so the shorter title
    // is clearer than the older "Evaluation / Recommendation Results" label.
    ui.heading("Recommendation Results");
    ui.label(
        "Review generated recommendations, score breakdowns, and the reasoning behind each result.",
    );
    ui.separator();
    ui.add_space(8.0);

    show_recommendation_results(ui, state);

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);
    ui.heading("Prototype Testing Summary");

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
}

/// Recommendation output moved to Evaluation.
///
/// Keeping this section here separates the input workflow from output analysis:
/// Explore is for selecting menu/preferences, while Evaluation explains the
/// generated results and how each score was produced.
fn show_recommendation_results(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("Recommendation Results");
    ui.label("Each card shows the hybrid score plus the ingredient and co-order evidence used to rank the dish.");

    if state.recommendation_output.recommendations.is_empty() {
        ui.label(
            "No recommendations yet. Select preferences or cart items on Explore & Recommend.",
        );
        return;
    }

    for (rank, recommendation) in state
        .recommendation_output
        .recommendations
        .iter()
        .take(10)
        .enumerate()
    {
        let related_labels = state.dish_labels_for_ids(&recommendation.related_selected_dish_ids);
        recommendation_card(ui, rank + 1, recommendation, &related_labels);
        ui.add_space(12.0);
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
            state.simulated_order_input = state.selected_dish_ids().join(", ");
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
    ui.add_space(6.0);
    show_recent_orders_table(ui, state);
}

/// Displays recent orders as an aligned table.
///
/// The ordered-dishes column is given a stable width and wraps long dish ID
/// lists. This prevents long simulated orders from shifting later columns or
/// breaking row alignment.
fn show_recent_orders_table(ui: &mut egui::Ui, state: &AppState) {
    let dishes_column_width = (ui.available_width() * 0.48).clamp(220.0, 560.0);

    egui::ScrollArea::vertical()
        .max_height(260.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new("recent_orders_grid")
                .striped(true)
                .num_columns(4)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.strong("Order ID");
                    ui.strong("Session/User ID");
                    ui.strong("Ordered Dishes");
                    ui.strong("Timestamp");
                    ui.end_row();

                    for order in state.orders.iter().rev().take(10) {
                        ui.monospace(&order.order_id);
                        ui.monospace(&order.session_user_id);
                        ui.add_sized(
                            [dishes_column_width, 18.0],
                            egui::Label::new(order.ordered_dishes.join(", ")).wrap(true),
                        );
                        ui.label(&order.timestamp);
                        ui.end_row();
                    }
                });
        });
}
