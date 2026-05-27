use crate::data_loader::ORDERS_PATH;
use crate::models::{Dish, Order, UserPreference};
use crate::preferences::{PreferenceOptions, extract_preference_options};
use crate::recommender::hybrid::{RecommendationOutput, generate_recommendations};
use crate::search::MatchMode;
use crate::simulation::add_simulated_order;
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
    pub preference_options: PreferenceOptions,
    pub selected_liked_ingredients: HashSet<String>,
    pub selected_disliked_ingredients: HashSet<String>,
    pub selected_preferred_tags: HashSet<String>,
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
        let preference_options = extract_preference_options(&dishes);

        let mut state = Self {
            dishes,
            orders,
            active_page: AppPage::ExploreRecommend,
            menu_search: String::new(),
            search_match_mode: MatchMode::Any,
            preference_options,
            selected_liked_ingredients: HashSet::new(),
            selected_disliked_ingredients: HashSet::new(),
            selected_preferred_tags: HashSet::new(),
            selected_dish_ids: HashSet::new(),
            simulated_order_input: String::new(),
            append_simulated_orders_to_csv: false,
            recommendation_output: RecommendationOutput::default(),
            last_order_message: String::new(),
            recommendation_version: 0,
        };

        state.refresh_recommendations();
        state
    }

    /// Builds the current recommender input from selected GUI options.
    ///
    /// Manual selected dish ID entry was removed from the end-user workflow.
    /// Dish IDs now come only from menu card Select buttons, which prevents
    /// typing mistakes and keeps co-ordering input aligned with visible menu
    /// choices.
    pub fn current_preference(&self) -> UserPreference {
        UserPreference {
            liked_ingredients: sorted_set_values(&self.selected_liked_ingredients),
            disliked_ingredients: sorted_set_values(&self.selected_disliked_ingredients),
            preferred_tags: sorted_set_values(&self.selected_preferred_tags),
            selected_dish_ids: self.selected_dish_ids(),
        }
    }

    /// Re-runs recommendation generation and increments a version counter.
    pub fn refresh_recommendations(&mut self) {
        let preference = self.current_preference();
        self.recommendation_output =
            generate_recommendations(&self.dishes, &self.orders, &preference);
        self.recommendation_version += 1;
    }

    /// Toggles one liked ingredient option.
    ///
    /// Conflict rule: an ingredient cannot be liked and disliked at the same
    /// time. Selecting it as liked removes it from disliked preferences.
    pub fn toggle_liked_ingredient(&mut self, ingredient: &str) {
        if self.selected_liked_ingredients.contains(ingredient) {
            self.selected_liked_ingredients.remove(ingredient);
        } else {
            self.selected_liked_ingredients
                .insert(ingredient.to_string());
            self.selected_disliked_ingredients.remove(ingredient);
        }

        self.refresh_recommendations();
    }

    /// Toggles one disliked ingredient option.
    ///
    /// Conflict rule: selecting an ingredient as disliked removes it from liked
    /// preferences. This keeps the recommendation input unambiguous.
    pub fn toggle_disliked_ingredient(&mut self, ingredient: &str) {
        if self.selected_disliked_ingredients.contains(ingredient) {
            self.selected_disliked_ingredients.remove(ingredient);
        } else {
            self.selected_disliked_ingredients
                .insert(ingredient.to_string());
            self.selected_liked_ingredients.remove(ingredient);
        }

        self.refresh_recommendations();
    }

    /// Toggles one preferred tag option and refreshes recommendations.
    pub fn toggle_preferred_tag(&mut self, tag: &str) {
        if self.selected_preferred_tags.contains(tag) {
            self.selected_preferred_tags.remove(tag);
        } else {
            self.selected_preferred_tags.insert(tag.to_string());
        }

        self.refresh_recommendations();
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

    /// Returns selected dish IDs collected from menu Select buttons.
    ///
    /// This is the only end-user source of selected dishes. The old manual
    /// selected-dish ID fallback was removed to keep the normal workflow simpler
    /// and reduce invalid input.
    pub fn selected_dish_ids(&self) -> Vec<String> {
        let mut selected = self.selected_dish_ids.iter().cloned().collect::<Vec<_>>();
        selected.sort();
        selected
    }

    /// Returns selected dish names for the cart panel.
    pub fn selected_dish_labels(&self) -> Vec<String> {
        let mut labels = self
            .dishes
            .iter()
            .filter(|dish| self.selected_dish_ids.contains(&dish.dish_id))
            .map(|dish| format!("{} ({})", dish.name, dish.dish_id))
            .collect::<Vec<_>>();
        labels.sort();
        labels
    }

    /// Converts dish IDs into `Name (ID)` labels for explanation text.
    pub fn dish_labels_for_ids(&self, dish_ids: &[String]) -> Vec<String> {
        let mut labels = dish_ids
            .iter()
            .map(|dish_id| {
                self.dishes
                    .iter()
                    .find(|dish| &dish.dish_id == dish_id)
                    .map(|dish| format!("{} ({})", dish.name, dish.dish_id))
                    .unwrap_or_else(|| dish_id.clone())
            })
            .collect::<Vec<_>>();
        labels.sort();
        labels
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

/// Converts a `HashSet` into a sorted vector for deterministic recommender input.
fn sorted_set_values(values: &HashSet<String>) -> Vec<String> {
    let mut values = values.iter().cloned().collect::<Vec<_>>();
    values.sort();
    values
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
            image_path: None,
            image_source_url: None,
        }
    }

    #[test]
    fn preference_selection_refreshes_recommendations() {
        let dishes = vec![
            dish("D01", "Rice", &["rice"]),
            dish("D02", "Chicken", &["chicken"]),
        ];
        let orders = Vec::new();
        let mut state = AppState::new(dishes, orders);
        let before = state.recommendation_version;

        state.toggle_liked_ingredient("rice");

        assert!(state.recommendation_version > before);
    }

    #[test]
    fn ingredient_cannot_be_liked_and_disliked_at_same_time() {
        let dishes = vec![dish("D01", "Rice", &["rice"])];
        let orders = Vec::new();
        let mut state = AppState::new(dishes, orders);

        state.toggle_liked_ingredient("rice");
        state.toggle_disliked_ingredient("rice");

        assert!(!state.selected_liked_ingredients.contains("rice"));
        assert!(state.selected_disliked_ingredients.contains("rice"));
    }

    #[test]
    fn selected_dishes_feed_recommendation_input() {
        let dishes = vec![
            dish("D01", "Rice", &["rice"]),
            dish("D02", "Chicken", &["chicken"]),
        ];
        let orders = Vec::new();
        let mut state = AppState::new(dishes, orders);

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
