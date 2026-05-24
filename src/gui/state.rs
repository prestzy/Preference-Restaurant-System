use crate::data_loader::ORDERS_PATH;
use crate::models::{Dish, Order, UserPreference};
use crate::recommender::hybrid::{RecommendationOutput, generate_recommendations};
use crate::search::MatchMode;
use crate::simulation::{add_simulated_order, parse_dish_ids};
use std::collections::HashSet;

/// Main pages exposed by the application navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppPage {
    Dashboard,
    ExploreRecommend,
    Evaluation,
    AdminDemoTools,
}

/// Central non-rendering state for the GUI.
///
/// UI functions mutate this state through small methods. Recommendation refresh
/// logic lives here rather than inside rendering code, which keeps egui pages
/// focused on layout and event wiring.
pub struct AppState {
    pub dishes: Vec<Dish>,
    pub orders: Vec<Order>,
    pub active_page: AppPage,
    pub menu_search: String,
    pub search_match_mode: MatchMode,
    pub liked_ingredients_input: String,
    pub disliked_ingredients_input: String,
    pub preferred_tags_input: String,
    pub manual_selected_dishes_input: String,
    pub selected_dish_ids: HashSet<String>,
    pub simulated_order_input: String,
    pub append_simulated_orders_to_csv: bool,
    pub recommendation_output: RecommendationOutput,
    pub last_order_message: String,
    pub recommendation_version: u64,
}

impl AppState {
    /// Creates initial state using loaded dishes and orders.
    pub fn new(dishes: Vec<Dish>, orders: Vec<Order>) -> Self {
        let mut selected_dish_ids = HashSet::new();
        selected_dish_ids.insert("D01".to_string());
        selected_dish_ids.insert("D03".to_string());

        let mut state = Self {
            dishes,
            orders,
            active_page: AppPage::ExploreRecommend,
            menu_search: String::new(),
            search_match_mode: MatchMode::Any,
            liked_ingredients_input: "chicken, rice, egg".to_string(),
            disliked_ingredients_input: "beef, anchovies".to_string(),
            preferred_tags_input: "spicy, signature".to_string(),
            manual_selected_dishes_input: String::new(),
            selected_dish_ids,
            simulated_order_input: String::new(),
            append_simulated_orders_to_csv: false,
            recommendation_output: RecommendationOutput::default(),
            last_order_message: String::new(),
            recommendation_version: 0,
        };

        state.refresh_recommendations();
        state
    }

    /// Builds the current recommender input from text fields and selected cards.
    pub fn current_preference(&self) -> UserPreference {
        let selected_dish_ids = self.combined_selected_dish_ids().join(",");

        UserPreference::from_input_text(
            &self.liked_ingredients_input,
            &self.disliked_ingredients_input,
            &self.preferred_tags_input,
            &selected_dish_ids,
        )
    }

    /// Re-runs recommendation generation and increments a version counter.
    pub fn refresh_recommendations(&mut self) {
        let preference = self.current_preference();
        self.recommendation_output =
            generate_recommendations(&self.dishes, &self.orders, &preference);
        self.recommendation_version += 1;
    }

    /// Updates preference text fields and refreshes recommendations if changed.
    pub fn set_preference_inputs(
        &mut self,
        liked_ingredients: String,
        disliked_ingredients: String,
        preferred_tags: String,
        manual_selected_dishes: String,
    ) {
        if self.liked_ingredients_input != liked_ingredients
            || self.disliked_ingredients_input != disliked_ingredients
            || self.preferred_tags_input != preferred_tags
            || self.manual_selected_dishes_input != manual_selected_dishes
        {
            self.liked_ingredients_input = liked_ingredients;
            self.disliked_ingredients_input = disliked_ingredients;
            self.preferred_tags_input = preferred_tags;
            self.manual_selected_dishes_input = manual_selected_dishes;
            self.refresh_recommendations();
        }
    }

    /// Toggles menu-card selection and refreshes recommendations immediately.
    pub fn toggle_dish_selection(&mut self, dish_id: &str) {
        if self.selected_dish_ids.contains(dish_id) {
            self.selected_dish_ids.remove(dish_id);
        } else {
            self.selected_dish_ids.insert(dish_id.to_string());
        }

        self.refresh_recommendations();
    }

    /// Returns selected dish IDs from both card selection and manual fallback.
    pub fn combined_selected_dish_ids(&self) -> Vec<String> {
        let mut selected = self.selected_dish_ids.iter().cloned().collect::<Vec<_>>();

        for manual_id in parse_dish_ids(&self.manual_selected_dishes_input) {
            if !selected.contains(&manual_id) {
                selected.push(manual_id);
            }
        }

        selected.sort();
        selected
    }

    /// Returns all known dish IDs for validation.
    pub fn known_dish_ids(&self) -> HashSet<String> {
        self.dishes
            .iter()
            .map(|dish| dish.dish_id.clone())
            .collect::<HashSet<_>>()
    }

    /// Creates a simulated order and refreshes recommendation output.
    pub fn create_simulated_order(&mut self) {
        let known_dish_ids = self.known_dish_ids();
        let outcome = add_simulated_order(
            &mut self.orders,
            &known_dish_ids,
            &self.simulated_order_input,
            self.append_simulated_orders_to_csv,
            ORDERS_PATH,
        );

        match outcome {
            Some(outcome) => {
                self.last_order_message = format!(
                    "Added {} with dish(es): {}. Collaborative filtering now includes this simulated behaviour.",
                    outcome.order.order_id,
                    outcome.order.ordered_dishes.join(", ")
                );

                if outcome.persisted_to_csv {
                    self.last_order_message
                        .push_str(" The order was also appended to data/orders.csv.");
                }

                if let Some(error) = outcome.csv_error {
                    self.last_order_message.push_str(&format!(
                        " The in-memory order was added, but CSV append failed: {error}."
                    ));
                }

                self.refresh_recommendations();
            }
            None => {
                self.last_order_message =
                    "No valid dish IDs found. Enter IDs that exist in the menu.".to_string();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dish(id: &str, name: &str, ingredients: &[&str]) -> Dish {
        Dish {
            dish_id: id.to_string(),
            name: name.to_string(),
            ingredients: ingredients.iter().map(|value| value.to_string()).collect(),
            category: "main".to_string(),
            tags: vec!["signature".to_string()],
        }
    }

    #[test]
    fn preference_change_refreshes_recommendations() {
        let dishes = vec![
            dish("D01", "Rice", &["rice"]),
            dish("D02", "Chicken", &["chicken"]),
        ];
        let orders = Vec::new();
        let mut state = AppState::new(dishes, orders);
        let before = state.recommendation_version;

        state.set_preference_inputs(
            "rice".to_string(),
            state.disliked_ingredients_input.clone(),
            state.preferred_tags_input.clone(),
            state.manual_selected_dishes_input.clone(),
        );

        assert!(state.recommendation_version > before);
    }

    #[test]
    fn selected_dishes_feed_recommendation_input() {
        let dishes = vec![
            dish("D01", "Rice", &["rice"]),
            dish("D02", "Chicken", &["chicken"]),
        ];
        let orders = Vec::new();
        let mut state = AppState::new(dishes, orders);
        state.selected_dish_ids.clear();

        state.toggle_dish_selection("D02");
        let preference = state.current_preference();

        assert_eq!(preference.selected_dish_ids, vec!["D02"]);
    }

    #[test]
    fn order_simulation_refreshes_recommendations() {
        let dishes = vec![
            dish("D01", "Rice", &["rice"]),
            dish("D02", "Chicken", &["chicken"]),
        ];
        let orders = Vec::new();
        let mut state = AppState::new(dishes, orders);
        state.simulated_order_input = "D01,D02".to_string();
        let before_version = state.recommendation_version;
        let before_orders = state.orders.len();

        state.create_simulated_order();

        assert_eq!(state.orders.len(), before_orders + 1);
        assert!(state.recommendation_version > before_version);
    }
}
