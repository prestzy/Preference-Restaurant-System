use crate::data_loader::{ORDERS_PATH, append_order_to_csv};
use crate::models::{Dish, Order, RecommendationResult, UserPreference};
use crate::recommender::hybrid::{RecommendationOutput, generate_recommendations};
use eframe::egui;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

/// Main pages in the prototype.
///
/// A simple side navigation keeps the GUI easy to present: each FYP requirement
/// maps to a visible section of the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppPage {
    Dashboard,
    MenuViewer,
    Preferences,
    Recommendations,
    OrderSimulation,
    Evaluation,
}

/// eframe/egui application state.
///
/// The app keeps dishes and orders in memory. When a simulated order is added,
/// the order vector is updated and recommendations are recalculated, showing how
/// new behavioural data can immediately affect collaborative filtering.
pub struct RestaurantOrderingApp {
    dishes: Vec<Dish>,
    orders: Vec<Order>,
    active_page: AppPage,
    menu_search: String,
    liked_ingredients_input: String,
    disliked_ingredients_input: String,
    preferred_tags_input: String,
    selected_dishes_input: String,
    simulated_order_input: String,
    append_simulated_orders_to_csv: bool,
    recommendation_output: RecommendationOutput,
    last_order_message: String,
}

impl RestaurantOrderingApp {
    /// Creates the GUI app after `main.rs` has loaded CSV data.
    pub fn new(dishes: Vec<Dish>, orders: Vec<Order>) -> Self {
        let mut app = Self {
            dishes,
            orders,
            active_page: AppPage::Dashboard,
            menu_search: String::new(),
            liked_ingredients_input: "chicken, rice, egg".to_string(),
            disliked_ingredients_input: "beef, anchovies".to_string(),
            preferred_tags_input: "spicy, signature".to_string(),
            selected_dishes_input: "D01, D03".to_string(),
            simulated_order_input: String::new(),
            append_simulated_orders_to_csv: false,
            recommendation_output: RecommendationOutput::default(),
            last_order_message: String::new(),
        };

        app.refresh_recommendations();
        app
    }

    /// Converts the current text fields into a clean `UserPreference`.
    fn current_preference(&self) -> UserPreference {
        UserPreference::from_input_text(
            &self.liked_ingredients_input,
            &self.disliked_ingredients_input,
            &self.preferred_tags_input,
            &self.selected_dishes_input,
        )
    }

    /// Re-runs the recommendation engine using current inputs and order data.
    fn refresh_recommendations(&mut self) {
        let preference = self.current_preference();
        self.recommendation_output =
            generate_recommendations(&self.dishes, &self.orders, &preference);
    }

    /// Draws the persistent side navigation.
    fn show_navigation(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("navigation")
            .resizable(false)
            .default_width(210.0)
            .show(ctx, |ui| {
                ui.heading("FYP Prototype");
                ui.label("Restaurant Ordering");
                ui.separator();

                ui.selectable_value(&mut self.active_page, AppPage::Dashboard, "Dashboard");
                ui.selectable_value(&mut self.active_page, AppPage::MenuViewer, "Menu Viewer");
                ui.selectable_value(
                    &mut self.active_page,
                    AppPage::Preferences,
                    "User Preference Input",
                );
                ui.selectable_value(
                    &mut self.active_page,
                    AppPage::Recommendations,
                    "Recommendation Output",
                );
                ui.selectable_value(
                    &mut self.active_page,
                    AppPage::OrderSimulation,
                    "Order Simulation",
                );
                ui.selectable_value(&mut self.active_page, AppPage::Evaluation, "Evaluation");

                ui.separator();
                ui.label(format!("Dishes: {}", self.dishes.len()));
                ui.label(format!("Orders: {}", self.orders.len()));
            });
    }

    /// Dashboard section with the main purpose of the prototype.
    fn show_dashboard(&self, ui: &mut egui::Ui) {
        ui.heading("Preference-Driven Restaurant Ordering System");
        ui.separator();

        ui.label(format!("Loaded dishes: {}", self.dishes.len()));
        ui.label(format!("Loaded historical orders: {}", self.orders.len()));
        ui.add_space(8.0);

        ui.label(
            "This lightweight single-restaurant prototype recommends dishes by combining ingredient-based filtering with co-ordering-based collaborative filtering.",
        );
        ui.label(
            "Ingredient filtering compares liked ingredients, disliked ingredients, and preferred tags against each dish. Collaborative filtering counts dishes that frequently appear together in previous orders.",
        );
        ui.label(
            "The hybrid score uses alpha = 0.4 for ingredient score and beta = 0.6 for co-order score when both signals are available.",
        );
    }

    /// Menu Viewer section with searchable dish list.
    fn show_menu_viewer(&mut self, ui: &mut egui::Ui) {
        ui.heading("Menu Viewer");
        ui.label("Search by dish name, ingredient, category, or tag.");

        ui.add(
            egui::TextEdit::singleline(&mut self.menu_search)
                .hint_text("Example: chicken, rice, spicy, dessert"),
        );
        ui.separator();

        let query = self.menu_search.trim().to_lowercase();
        let filtered_dishes = self
            .dishes
            .iter()
            .filter(|dish| dish_matches_search(dish, &query))
            .collect::<Vec<_>>();

        ui.label(format!("Showing {} dish(es)", filtered_dishes.len()));

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("menu_grid")
                .striped(true)
                .num_columns(5)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.strong("ID");
                    ui.strong("Name");
                    ui.strong("Category");
                    ui.strong("Ingredients");
                    ui.strong("Tags");
                    ui.end_row();

                    for dish in filtered_dishes {
                        ui.monospace(&dish.dish_id);
                        ui.label(&dish.name);
                        ui.label(&dish.category);
                        ui.label(dish.ingredients.join(", "));
                        ui.label(dish.tags.join(", "));
                        ui.end_row();
                    }
                });
        });
    }

    /// User Preference Input section.
    ///
    /// The text fields intentionally accept comma-separated values to mirror the
    /// CSV format and keep the prototype simple for demonstration.
    fn show_preferences(&mut self, ui: &mut egui::Ui) {
        ui.heading("User Preference Input");
        ui.label("Enter comma-separated values. Recommendations update when the fields change.");
        ui.separator();

        let mut changed = false;

        changed |= preference_text_field(
            ui,
            "Liked ingredients",
            "chicken, rice, egg",
            &mut self.liked_ingredients_input,
        );
        changed |= preference_text_field(
            ui,
            "Disliked ingredients",
            "beef, anchovies",
            &mut self.disliked_ingredients_input,
        );
        changed |= preference_text_field(
            ui,
            "Preferred tags",
            "spicy, signature",
            &mut self.preferred_tags_input,
        );
        changed |= preference_text_field(
            ui,
            "Selected dish IDs",
            "D01, D03",
            &mut self.selected_dishes_input,
        );

        if changed {
            self.refresh_recommendations();
        }

        ui.add_space(8.0);
        if ui.button("Refresh recommendations").clicked() {
            self.refresh_recommendations();
        }

        ui.separator();
        let preference = self.current_preference();
        ui.label(format!(
            "Cleaned liked ingredients: {}",
            display_list(&preference.liked_ingredients)
        ));
        ui.label(format!(
            "Cleaned disliked ingredients: {}",
            display_list(&preference.disliked_ingredients)
        ));
        ui.label(format!(
            "Cleaned preferred tags: {}",
            display_list(&preference.preferred_tags)
        ));
        ui.label(format!(
            "Cleaned selected dish IDs: {}",
            display_list(&preference.selected_dish_ids)
        ));
    }

    /// Recommendation Output section.
    fn show_recommendations(&mut self, ui: &mut egui::Ui) {
        ui.heading("Recommendation Output");
        ui.label("Top dishes ranked by final hybrid score.");

        if ui.button("Recalculate").clicked() {
            self.refresh_recommendations();
        }

        ui.separator();
        self.show_recommendation_cards(ui, 10);
    }

    /// Draws recommendation rows used by both Recommendation and Evaluation pages.
    fn show_recommendation_cards(&self, ui: &mut egui::Ui, limit: usize) {
        if self.recommendation_output.recommendations.is_empty() {
            ui.label("No recommendations yet. Enter liked ingredients, preferred tags, or selected dish IDs to generate results.");
            return;
        }

        for (rank, recommendation) in self
            .recommendation_output
            .recommendations
            .iter()
            .take(limit)
            .enumerate()
        {
            recommendation_card(ui, rank + 1, recommendation);
            ui.add_space(8.0);
        }
    }

    /// Order Simulation section.
    ///
    /// Adding an order appends it to the in-memory order log, then refreshes the
    /// co-order matrix indirectly by regenerating recommendations. This simulates
    /// how new behaviour data can improve collaborative recommendations.
    fn show_order_simulation(&mut self, ui: &mut egui::Ui) {
        ui.heading("Order Simulation");
        ui.label("Create a simulated order using comma-separated dish IDs.");
        ui.add(
            egui::TextEdit::singleline(&mut self.simulated_order_input)
                .hint_text("Example: D01, D09, D30"),
        );

        ui.checkbox(
            &mut self.append_simulated_orders_to_csv,
            "Also append simulated order to data/orders.csv",
        );

        ui.horizontal(|ui| {
            if ui.button("Use selected dish IDs").clicked() {
                self.simulated_order_input = self.selected_dishes_input.clone();
            }

            if ui.button("Create simulated order").clicked() {
                self.create_simulated_order();
            }

            if ui.button("Clear").clicked() {
                self.simulated_order_input.clear();
                self.last_order_message.clear();
            }
        });

        if !self.last_order_message.is_empty() {
            ui.separator();
            ui.label(&self.last_order_message);
        }

        ui.separator();
        ui.label("Most recent orders in memory:");
        egui::ScrollArea::vertical()
            .max_height(220.0)
            .show(ui, |ui| {
                for order in self.orders.iter().rev().take(8) {
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

    /// Creates and stores a simulated order from GUI input.
    fn create_simulated_order(&mut self) {
        let known_dish_ids = self
            .dishes
            .iter()
            .map(|dish| dish.dish_id.clone())
            .collect::<HashSet<_>>();
        let ordered_dishes =
            UserPreference::from_input_text("", "", "", &self.simulated_order_input)
                .selected_dish_ids
                .into_iter()
                .filter(|dish_id| known_dish_ids.contains(dish_id))
                .collect::<Vec<_>>();

        if ordered_dishes.is_empty() {
            self.last_order_message =
                "No valid dish IDs found. Enter IDs that exist in the Menu Viewer.".to_string();
            return;
        }

        let order = Order {
            order_id: format!("SIM{:03}", self.orders.len() + 1),
            session_user_id: "SIM_USER".to_string(),
            ordered_dishes,
            timestamp: prototype_timestamp(),
        };

        self.orders.push(order.clone());

        let mut message = format!(
            "Added {} with dish(es): {}. Collaborative filtering now includes this simulated behaviour.",
            order.order_id,
            order.ordered_dishes.join(", ")
        );

        if self.append_simulated_orders_to_csv {
            match append_order_to_csv(&order, ORDERS_PATH) {
                Ok(()) => message.push_str(" The order was also appended to data/orders.csv."),
                Err(error) => message.push_str(&format!(
                    " The in-memory order was added, but CSV append failed: {error}."
                )),
            }
        }

        self.last_order_message = message;
        self.refresh_recommendations();
    }

    /// Evaluation / Prototype Testing section.
    ///
    /// These values are simple demonstration metrics. They support the FYP demo
    /// by showing filtering impact and category diversity without claiming to be
    /// a full offline recommender-system evaluation.
    fn show_evaluation(&self, ui: &mut egui::Ui) {
        ui.heading("Evaluation / Prototype Testing");
        ui.label("Simple demo metrics for the current preference input.");
        ui.separator();

        let stats = &self.recommendation_output.stats;
        ui.label(format!(
            "Top recommendations available: {}",
            self.recommendation_output.recommendations.len()
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
        ui.strong("Decision support explanation");
        ui.label(
            "The system ranks dishes by combining explicit preference evidence with behavioural co-order evidence. Excluded dishes are removed before scoring when they contain disliked ingredients.",
        );

        ui.separator();
        ui.strong("Top 5 recommendations");
        self.show_recommendation_cards(ui, 5);
    }
}

impl eframe::App for RestaurantOrderingApp {
    /// Main egui update loop.
    ///
    /// egui redraws the interface every frame. The app draws navigation first,
    /// then displays the selected FYP section in the central panel.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.show_navigation(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| match self.active_page {
                AppPage::Dashboard => self.show_dashboard(ui),
                AppPage::MenuViewer => self.show_menu_viewer(ui),
                AppPage::Preferences => self.show_preferences(ui),
                AppPage::Recommendations => self.show_recommendations(ui),
                AppPage::OrderSimulation => self.show_order_simulation(ui),
                AppPage::Evaluation => self.show_evaluation(ui),
            });
        });
    }
}

/// Draws one labelled text input and returns whether it changed.
fn preference_text_field(ui: &mut egui::Ui, label: &str, hint: &str, value: &mut String) -> bool {
    ui.label(label);
    ui.add(egui::TextEdit::singleline(value).hint_text(hint))
        .changed()
}

/// Draws one recommendation in an explainable score format.
fn recommendation_card(ui: &mut egui::Ui, rank: usize, recommendation: &RecommendationResult) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.heading(format!(
                "{}. {} ({})",
                rank, recommendation.dish.name, recommendation.dish.dish_id
            ));
            ui.label(format!("Category: {}", recommendation.dish.category));
        });

        ui.label(format!(
            "Ingredient score: {:.2} | Co-order score: {:.2} | Final hybrid score: {:.2}",
            recommendation.ingredient_score,
            recommendation.co_order_score,
            recommendation.final_score
        ));
        ui.label(&recommendation.explanation);
    });
}

/// Checks whether a dish should appear for the current menu search query.
fn dish_matches_search(dish: &Dish, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    dish.dish_id.to_lowercase().contains(query)
        || dish.name.to_lowercase().contains(query)
        || dish.category.to_lowercase().contains(query)
        || dish
            .ingredients
            .iter()
            .any(|ingredient| ingredient.contains(query))
        || dish.tags.iter().any(|tag| tag.contains(query))
}

/// Displays a vector in a compact way for cleaned preference previews.
fn display_list(values: &[String]) -> String {
    if values.is_empty() {
        "(none)".to_string()
    } else {
        values.join(", ")
    }
}

/// Produces a simple timestamp string for simulated orders.
///
/// The standard library does not format local dates directly. For this
/// prototype, Unix seconds are sufficient because the value only needs to show
/// when simulated behavioural data was created during the demo.
fn prototype_timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();

    format!("simulated-{seconds}")
}
