use crate::data_loader::split_csv_field;
use crate::models::{Dish, Order, RecommendationResult, UserPreference};
use crate::preferences::{PreferenceOptions, extract_preference_options};
use crate::recommender::hybrid::{RecommendationOutput, generate_recommendations};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Local folder served by Axum under `/assets/dishes`.
const DISH_IMAGE_DIR: &str = "assets/dishes";

/// Shared application state used by Axum handlers.
///
/// The web prototype keeps loaded dishes, historical CSV orders, live checkout
/// orders, and availability flags separate. This makes the customer ordering
/// flow independent from historical training data while still allowing the
/// recommender to combine both when needed.
#[derive(Clone)]
pub struct WebState {
    dishes: Arc<RwLock<Vec<Dish>>>,
    historical_orders: Arc<RwLock<Vec<Order>>>,
    live_orders: Arc<RwLock<Vec<LiveOrder>>>,
    unavailable_dish_ids: Arc<RwLock<HashSet<String>>>,
}

impl WebState {
    /// Creates web state from data loaded by `data_loader`.
    pub fn new(dishes: Vec<Dish>, historical_orders: Vec<Order>) -> Self {
        Self {
            dishes: Arc::new(RwLock::new(dishes)),
            historical_orders: Arc::new(RwLock::new(historical_orders)),
            live_orders: Arc::new(RwLock::new(Vec::new())),
            unavailable_dish_ids: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Builds the complete customer menu view used by the home page.
    pub fn menu_view(&self) -> MenuView {
        let dishes = self.visible_dish_views();
        let recommendation_output = self.recommend(Default::default());
        let recommended_ids = recommendation_output
            .recommendations
            .iter()
            .map(|recommendation| recommendation.dish.dish_id.clone())
            .collect::<HashSet<_>>();

        let dishes = dishes
            .into_iter()
            .map(|mut dish| {
                dish.recommended = recommended_ids.contains(&dish.dish_id);
                dish
            })
            .collect::<Vec<_>>();

        MenuView {
            dishes_json: serde_json::to_string(&dishes).unwrap_or_else(|_| "[]".to_string()),
            recommendations_json: serde_json::to_string(&recommendation_output.recommendations)
                .unwrap_or_else(|_| "[]".to_string()),
            preference_options_json: serde_json::to_string(&self.preference_options())
                .unwrap_or_else(|_| "{}".to_string()),
            dishes,
            recommended: recommendation_output.recommendations,
            preference_options: self.preference_options(),
            order_count: self.historical_order_count() + self.live_order_count(),
        }
    }

    /// Builds a dashboard/admin view with summary counts and management data.
    pub fn admin_view(&self) -> AdminView {
        let dishes = self.all_dish_views();
        let all_live_orders = self
            .live_orders
            .read()
            .expect("live orders lock poisoned")
            .clone();
        let live_orders = all_live_orders
            .iter()
            .filter(|order| order.status != OrderStatus::Completed)
            .cloned()
            .collect::<Vec<_>>();
        let completed_session_orders = all_live_orders
            .iter()
            .filter(|order| order.status == OrderStatus::Completed)
            .cloned()
            .collect::<Vec<_>>();
        let historical_orders = self
            .historical_orders
            .read()
            .expect("historical orders lock poisoned")
            .iter()
            .rev()
            .take(10)
            .cloned()
            .collect::<Vec<_>>();

        let available_dishes = dishes.iter().filter(|dish| dish.available).count();
        let unavailable_dishes = dishes.len().saturating_sub(available_dishes);

        AdminView {
            total_dishes: dishes.len(),
            available_dishes,
            unavailable_dishes,
            historical_order_count: self.historical_order_count(),
            live_order_count: live_orders.len(),
            completed_session_order_count: completed_session_orders.len(),
            dishes,
            live_orders,
            completed_session_orders,
            historical_orders,
            frequent_dishes: self.frequent_dishes(6),
            co_order_pairs: self.common_co_order_pairs(6),
            preference_options: self.preference_options(),
        }
    }

    /// Generates recommendation output from web-selected preferences.
    ///
    /// Empty input falls back to a small demo preference profile so the home
    /// page has useful recommendations before the user opens preference chips.
    pub fn recommend(&self, request: RecommendationRequest) -> RecommendationApiResponse {
        let preference = request.into_user_preference_or_default();
        let dishes = self.available_dishes();
        let orders = self.combined_orders();
        let output = generate_recommendations(&dishes, &orders, &preference);

        RecommendationApiResponse {
            recommendations: output
                .recommendations
                .iter()
                .take(10)
                .map(|result| self.recommendation_view(result))
                .collect(),
            stats: RecommendationStatsView::from_output(&output),
        }
    }

    /// Creates a live web order from checkout dish IDs.
    pub fn create_live_order(&self, dish_ids: &[String]) -> Result<LiveOrder, String> {
        let dishes = self.available_dishes();
        let dish_lookup = dishes
            .iter()
            .map(|dish| (dish.dish_id.clone(), dish.clone()))
            .collect::<HashMap<_, _>>();

        let cleaned = dish_ids
            .iter()
            .map(|dish_id| dish_id.trim().to_uppercase())
            .filter(|dish_id| dish_lookup.contains_key(dish_id))
            .collect::<Vec<_>>();

        if cleaned.is_empty() {
            return Err("No valid available menu item was submitted.".to_string());
        }

        let dish_names = cleaned
            .iter()
            .filter_map(|dish_id| dish_lookup.get(dish_id))
            .map(|dish| dish.name.clone())
            .collect::<Vec<_>>();
        let total_price_amount = cleaned
            .iter()
            .map(|dish_id| placeholder_price_amount(dish_id))
            .sum();

        let mut live_orders = self.live_orders.write().expect("live orders lock poisoned");
        let order_number = live_orders.len() + 1;
        let order = LiveOrder {
            order_id: format!("WEB{order_number:03}"),
            session_user_id: "QR-CUSTOMER".to_string(),
            ordered_dishes: cleaned,
            dish_names,
            timestamp: unix_timestamp_label(),
            total_price: format!("RM {total_price_amount}"),
            total_price_amount,
            status: OrderStatus::Pending,
        };

        live_orders.push(order.clone());
        Ok(order)
    }

    /// Updates the status of a live order.
    pub fn update_order_status(
        &self,
        order_id: &str,
        status: OrderStatus,
    ) -> Result<LiveOrder, String> {
        let updated_order = {
            let mut live_orders = self.live_orders.write().expect("live orders lock poisoned");
            let Some(order) = live_orders
                .iter_mut()
                .find(|order| order.order_id.eq_ignore_ascii_case(order_id))
            else {
                return Err(format!("Live order {order_id} was not found."));
            };

            order.status = status;
            order.clone()
        };

        if status == OrderStatus::Completed {
            self.append_completed_order_to_history(&updated_order);
        }

        Ok(updated_order)
    }

    /// Adds a completed checkout order to the in-memory historical order log.
    ///
    /// This makes the Admin "Historical Orders" table update immediately when
    /// staff mark an order Completed. The append is guarded by `order_id` so
    /// repeated status updates do not duplicate the same order log.
    fn append_completed_order_to_history(&self, completed_order: &LiveOrder) {
        let mut historical_orders = self
            .historical_orders
            .write()
            .expect("historical orders lock poisoned");
        if historical_orders.iter().any(|order| {
            order
                .order_id
                .eq_ignore_ascii_case(&completed_order.order_id)
        }) {
            return;
        }

        historical_orders.push(completed_order.as_order());
    }

    /// Finds one live/session order for the customer order tracking page.
    pub fn order_by_id(&self, order_id: &str) -> Option<LiveOrder> {
        self.live_orders
            .read()
            .expect("live orders lock poisoned")
            .iter()
            .find(|order| order.order_id.eq_ignore_ascii_case(order_id))
            .cloned()
    }

    /// Adds or edits a dish in memory.
    ///
    /// CSV persistence is deliberately not mixed into this method. It keeps the
    /// admin management flow usable for demos while leaving a clear persistence
    /// seam for a later database/CSV writer.
    pub fn upsert_dish(&self, request: UpsertDishRequest) -> Result<DishView, String> {
        let name = request.name.trim().to_string();
        if name.is_empty() {
            return Err("Dish name is required.".to_string());
        }

        let dish_id = {
            let mut dishes = self.dishes.write().expect("dishes lock poisoned");
            let dish_id = request
                .normalized_id()
                .unwrap_or_else(|| next_dish_id(&dishes));

            let dish = Dish {
                dish_id: dish_id.clone(),
                name,
                category: request.category.trim().to_lowercase(),
                ingredients: split_csv_field(&request.ingredients),
                tags: split_csv_field(&request.tags),
                image_path: request.clean_image_path(),
                image_source_url: None,
            };

            if let Some(existing) = dishes.iter_mut().find(|dish| dish.dish_id == dish_id) {
                *existing = dish;
            } else {
                dishes.push(dish);
            }

            dish_id
        };

        // Availability is stored separately from dish data. The dish list write
        // lock is released before this call because availability validation
        // reads from the dish list; keeping both operations under one lock would
        // deadlock the prototype during admin add/edit.
        let available = request.available.unwrap_or(true);
        self.set_dish_availability(&dish_id, available)?;
        self.dish_view_by_id(&dish_id)
            .ok_or_else(|| "Dish was saved but could not be loaded.".to_string())
    }

    /// Deletes a dish from in-memory management state.
    pub fn delete_dish(&self, dish_id: &str) -> Result<(), String> {
        let mut dishes = self.dishes.write().expect("dishes lock poisoned");
        let before = dishes.len();
        dishes.retain(|dish| !dish.dish_id.eq_ignore_ascii_case(dish_id));
        self.unavailable_dish_ids
            .write()
            .expect("availability lock poisoned")
            .remove(&dish_id.trim().to_uppercase());

        if dishes.len() == before {
            Err(format!("Dish {dish_id} was not found."))
        } else {
            Ok(())
        }
    }

    /// Marks a dish as available or unavailable.
    pub fn set_dish_availability(&self, dish_id: &str, available: bool) -> Result<(), String> {
        let dish_id = dish_id.trim().to_uppercase();
        if !self
            .dishes
            .read()
            .expect("dishes lock poisoned")
            .iter()
            .any(|dish| dish.dish_id == dish_id)
        {
            return Err(format!("Dish {dish_id} was not found."));
        }

        let mut unavailable = self
            .unavailable_dish_ids
            .write()
            .expect("availability lock poisoned");
        if available {
            unavailable.remove(&dish_id);
        } else {
            unavailable.insert(dish_id);
        }
        Ok(())
    }

    /// Replaces the in-memory dish list with imported CSV dish data.
    pub fn replace_dishes_from_csv(&self, dishes: Vec<Dish>) -> usize {
        let count = dishes.len();
        *self.dishes.write().expect("dishes lock poisoned") = dishes;
        self.unavailable_dish_ids
            .write()
            .expect("availability lock poisoned")
            .clear();
        count
    }

    /// Merges imported dishes into the in-memory menu by `dish_id`.
    ///
    /// Existing records are replaced and new records are appended. This gives
    /// staff a safer CSV workflow when they only want to update part of the menu
    /// instead of replacing the whole dataset.
    pub fn merge_dishes_from_csv(&self, imported: Vec<Dish>) -> usize {
        let count = imported.len();
        let mut dishes = self.dishes.write().expect("dishes lock poisoned");
        for imported_dish in imported {
            if let Some(existing) = dishes
                .iter_mut()
                .find(|dish| dish.dish_id == imported_dish.dish_id)
            {
                *existing = imported_dish;
            } else {
                dishes.push(imported_dish);
            }
        }
        count
    }

    /// Replaces historical order logs in memory after admin CSV import.
    ///
    /// Live customer checkout orders are intentionally not touched because they
    /// represent the current browser session demo, while historical orders are
    /// the dataset used to rebuild collaborative filtering evidence.
    pub fn replace_historical_orders_from_csv(&self, orders: Vec<Order>) -> usize {
        let count = orders.len();
        *self
            .historical_orders
            .write()
            .expect("historical orders lock poisoned") = orders;
        count
    }

    /// Returns the current dish models for CSV export.
    pub fn dish_models_for_export(&self) -> Vec<Dish> {
        self.dishes.read().expect("dishes lock poisoned").clone()
    }

    /// Returns historical orders for CSV export.
    pub fn historical_orders_for_export(&self) -> Vec<Order> {
        self.historical_orders
            .read()
            .expect("historical orders lock poisoned")
            .clone()
    }

    /// Returns completed checkout orders as CSV-compatible order records.
    pub fn completed_session_orders_for_export(&self) -> Vec<Order> {
        self.live_orders
            .read()
            .expect("live orders lock poisoned")
            .iter()
            .filter(|order| order.status == OrderStatus::Completed)
            .map(LiveOrder::as_order)
            .collect()
    }

    fn preference_options(&self) -> PreferenceOptions {
        let dishes = self.dishes.read().expect("dishes lock poisoned");
        extract_preference_options(&dishes)
    }

    fn all_dish_views(&self) -> Vec<DishView> {
        let unavailable = self
            .unavailable_dish_ids
            .read()
            .expect("availability lock poisoned")
            .clone();
        self.dishes
            .read()
            .expect("dishes lock poisoned")
            .iter()
            .map(|dish| DishView::from_dish(dish, !unavailable.contains(&dish.dish_id), false))
            .collect()
    }

    fn visible_dish_views(&self) -> Vec<DishView> {
        self.all_dish_views()
            .into_iter()
            .filter(|dish| dish.available)
            .collect()
    }

    fn dish_view_by_id(&self, dish_id: &str) -> Option<DishView> {
        self.all_dish_views()
            .into_iter()
            .find(|dish| dish.dish_id.eq_ignore_ascii_case(dish_id))
    }

    fn available_dishes(&self) -> Vec<Dish> {
        let unavailable = self
            .unavailable_dish_ids
            .read()
            .expect("availability lock poisoned")
            .clone();
        self.dishes
            .read()
            .expect("dishes lock poisoned")
            .iter()
            .filter(|dish| !unavailable.contains(&dish.dish_id))
            .cloned()
            .collect()
    }

    fn combined_orders(&self) -> Vec<Order> {
        // Completed checkout orders are appended to `historical_orders` when
        // their status changes to Completed. Reading one source here avoids
        // double-counting the same completed order in collaborative filtering.
        self.historical_orders
            .read()
            .expect("historical orders lock poisoned")
            .clone()
    }

    fn historical_order_count(&self) -> usize {
        self.historical_orders
            .read()
            .expect("historical orders lock poisoned")
            .len()
    }

    fn live_order_count(&self) -> usize {
        self.live_orders
            .read()
            .expect("live orders lock poisoned")
            .len()
    }

    fn recommendation_view(&self, result: &RecommendationResult) -> RecommendationView {
        let related_selected_dishes = self.dish_labels_for_ids(&result.related_selected_dish_ids);
        RecommendationView {
            dish: DishView::from_dish(&result.dish, true, true),
            content_score: result.ingredient_score,
            co_order_score: result.co_order_score,
            popularity_score: result.popularity_score,
            business_rule_score: result.business_rule_score,
            hybrid_score: result.final_score,
            explanation: detailed_recommendation_reason(result, &related_selected_dishes),
            association_base_dish_id: result.association_base_dish_id.clone(),
            association_pair_count: result.association_pair_count,
            association_support: result.association_support,
            association_confidence: result.association_confidence,
            association_lift: result.association_lift,
            matched_liked_ingredients: result.matched_liked_ingredients.clone(),
            matched_preferred_tags: result.matched_preferred_tags.clone(),
            matched_disliked_ingredients: result.matched_disliked_ingredients.clone(),
            related_selected_dishes,
        }
    }

    fn dish_labels_for_ids(&self, dish_ids: &[String]) -> Vec<String> {
        let dishes = self.dishes.read().expect("dishes lock poisoned");
        let mut labels = dish_ids
            .iter()
            .map(|dish_id| {
                dishes
                    .iter()
                    .find(|dish| &dish.dish_id == dish_id)
                    .map(|dish| format!("{} ({})", dish.name, dish.dish_id))
                    .unwrap_or_else(|| dish_id.clone())
            })
            .collect::<Vec<_>>();
        labels.sort();
        labels
    }

    fn frequent_dishes(&self, limit: usize) -> Vec<FrequencyView> {
        let mut counts = HashMap::<String, usize>::new();
        for order in self.combined_orders() {
            for dish_id in order.ordered_dishes {
                *counts.entry(dish_id).or_default() += 1;
            }
        }

        let labels = self
            .dishes
            .read()
            .expect("dishes lock poisoned")
            .iter()
            .map(|dish| (dish.dish_id.clone(), dish.name.clone()))
            .collect::<HashMap<_, _>>();

        let mut values = counts
            .into_iter()
            .map(|(dish_id, count)| FrequencyView {
                label: labels
                    .get(&dish_id)
                    .map(|name| format!("{name} ({dish_id})"))
                    .unwrap_or(dish_id),
                count,
            })
            .collect::<Vec<_>>();
        values.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.label.cmp(&b.label)));
        values.truncate(limit);
        values
    }

    fn common_co_order_pairs(&self, limit: usize) -> Vec<FrequencyView> {
        let mut counts = HashMap::<String, usize>::new();
        for order in self.combined_orders() {
            let mut ids = order.ordered_dishes;
            ids.sort();
            ids.dedup();
            for i in 0..ids.len() {
                for j in (i + 1)..ids.len() {
                    *counts
                        .entry(format!("{} + {}", ids[i], ids[j]))
                        .or_default() += 1;
                }
            }
        }

        let mut values = counts
            .into_iter()
            .map(|(label, count)| FrequencyView { label, count })
            .collect::<Vec<_>>();
        values.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.label.cmp(&b.label)));
        values.truncate(limit);
        values
    }
}

/// Data prepared for the customer home page template.
pub struct MenuView {
    pub dishes: Vec<DishView>,
    pub recommended: Vec<RecommendationView>,
    pub preference_options: PreferenceOptions,
    pub dishes_json: String,
    pub recommendations_json: String,
    pub preference_options_json: String,
    pub order_count: usize,
}

/// Data prepared for the admin page template.
pub struct AdminView {
    pub total_dishes: usize,
    pub available_dishes: usize,
    pub unavailable_dishes: usize,
    pub historical_order_count: usize,
    pub live_order_count: usize,
    pub completed_session_order_count: usize,
    pub dishes: Vec<DishView>,
    pub live_orders: Vec<LiveOrder>,
    pub completed_session_orders: Vec<LiveOrder>,
    pub historical_orders: Vec<Order>,
    pub frequent_dishes: Vec<FrequencyView>,
    pub co_order_pairs: Vec<FrequencyView>,
    pub preference_options: PreferenceOptions,
}

/// Frontend-friendly dish shape.
#[derive(Debug, Clone, Serialize)]
pub struct DishView {
    pub dish_id: String,
    pub name: String,
    pub category: String,
    pub tags: Vec<String>,
    pub ingredients: Vec<String>,
    pub price: String,
    pub price_amount: u32,
    pub recommended: bool,
    pub available: bool,
    pub image_url: Option<String>,
    pub image_path: Option<String>,
}

impl DishView {
    fn from_dish(dish: &Dish, available: bool, recommended: bool) -> Self {
        let price_amount = placeholder_price_amount(&dish.dish_id);
        Self {
            dish_id: dish.dish_id.clone(),
            name: dish.name.clone(),
            category: title_case(&dish.category),
            tags: dish.tags.clone(),
            ingredients: dish.ingredients.clone(),
            price: format!("RM {price_amount}"),
            price_amount,
            recommended,
            available,
            image_url: dish_image_url(dish),
            image_path: dish.image_path.clone(),
        }
    }
}

/// Explainable recommendation result sent to the browser.
#[derive(Debug, Clone, Serialize)]
pub struct RecommendationView {
    pub dish: DishView,
    pub content_score: f32,
    pub co_order_score: f32,
    pub popularity_score: f32,
    pub business_rule_score: f32,
    pub hybrid_score: f32,
    pub explanation: String,
    pub association_base_dish_id: Option<String>,
    pub association_pair_count: u32,
    pub association_support: f32,
    pub association_confidence: f32,
    pub association_lift: f32,
    pub matched_liked_ingredients: Vec<String>,
    pub matched_preferred_tags: Vec<String>,
    pub matched_disliked_ingredients: Vec<String>,
    pub related_selected_dishes: Vec<String>,
}

/// Lightweight evaluation stats returned with recommendation results.
#[derive(Debug, Clone, Serialize)]
pub struct RecommendationStatsView {
    pub filtered_dishes: usize,
    pub excluded_due_to_disliked: usize,
    pub skipped_selected_dishes: usize,
    pub diversity_count_top_5: usize,
}

impl RecommendationStatsView {
    fn from_output(output: &RecommendationOutput) -> Self {
        Self {
            filtered_dishes: output.stats.filtered_dishes,
            excluded_due_to_disliked: output.stats.excluded_due_to_disliked,
            skipped_selected_dishes: output.stats.skipped_selected_dishes,
            diversity_count_top_5: output.stats.diversity_count_top_5,
        }
    }
}

/// JSON request accepted by `/api/recommendations`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RecommendationRequest {
    #[serde(default)]
    pub liked_ingredients: Vec<String>,
    #[serde(default)]
    pub disliked_ingredients: Vec<String>,
    #[serde(default)]
    pub preferred_tags: Vec<String>,
    #[serde(default)]
    pub selected_dish_ids: Vec<String>,
    #[serde(default)]
    pub time_context: Option<String>,
    #[serde(default)]
    pub ranking_method: Option<String>,
}

impl RecommendationRequest {
    fn into_user_preference_or_default(self) -> UserPreference {
        UserPreference {
            liked_ingredients: normalize_list(self.liked_ingredients, false),
            disliked_ingredients: normalize_list(self.disliked_ingredients, false),
            preferred_tags: normalize_list(self.preferred_tags, false),
            selected_dish_ids: normalize_list(self.selected_dish_ids, true),
            time_context: self.time_context,
            ranking_method: self.ranking_method,
        }
    }
}

/// JSON response from `/api/recommendations`.
#[derive(Debug, Clone, Serialize)]
pub struct RecommendationApiResponse {
    pub recommendations: Vec<RecommendationView>,
    pub stats: RecommendationStatsView,
}

/// Live session order created by customer checkout.
#[derive(Debug, Clone, Serialize)]
pub struct LiveOrder {
    pub order_id: String,
    pub session_user_id: String,
    pub ordered_dishes: Vec<String>,
    pub dish_names: Vec<String>,
    pub timestamp: String,
    pub total_price: String,
    pub total_price_amount: u32,
    pub status: OrderStatus,
}

impl LiveOrder {
    fn as_order(&self) -> Order {
        Order {
            order_id: self.order_id.clone(),
            session_user_id: self.session_user_id.clone(),
            ordered_dishes: self.ordered_dishes.clone(),
            timestamp: self.timestamp.clone(),
        }
    }
}

/// Staff-visible status for live orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Preparing,
    Ready,
    Completed,
    Cancelled,
}

impl OrderStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Preparing => "Preparing",
            Self::Ready => "Ready",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
        }
    }

    /// Converts staff form values into the enum used by live order state.
    pub fn from_label(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "pending" => Some(Self::Pending),
            "preparing" => Some(Self::Preparing),
            "ready" => Some(Self::Ready),
            "completed" => Some(Self::Completed),
            "cancelled" | "canceled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

/// Admin add/edit dish request.
#[derive(Debug, Clone, Deserialize)]
pub struct UpsertDishRequest {
    #[serde(default)]
    pub dish_id: Option<String>,
    pub name: String,
    pub category: String,
    pub ingredients: String,
    pub tags: String,
    #[serde(default)]
    pub image_path: Option<String>,
    #[serde(default)]
    pub available: Option<bool>,
}

impl UpsertDishRequest {
    fn normalized_id(&self) -> Option<String> {
        self.dish_id
            .as_deref()
            .map(|value| value.trim().to_uppercase())
            .filter(|value| !value.is_empty())
    }

    fn clean_image_path(&self) -> Option<String> {
        self.image_path
            .as_deref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }
}

/// Simple frequency row for dashboard summaries.
#[derive(Debug, Clone)]
pub struct FrequencyView {
    pub label: String,
    pub count: usize,
}

fn normalize_list(values: Vec<String>, uppercase: bool) -> Vec<String> {
    let mut values = values
        .into_iter()
        .map(|value| {
            if uppercase {
                value.trim().to_uppercase()
            } else {
                value.trim().to_lowercase()
            }
        })
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
        None => "Uncategorised".to_string(),
    }
}

fn placeholder_price_amount(dish_id: &str) -> u32 {
    let number = dish_id
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect::<String>()
        .parse::<u32>()
        .unwrap_or(1);
    8 + (number % 9) * 2
}

/// Resolves the local image URL for web rendering.
///
/// Runtime never hotlinks external images. The lookup mirrors the previous
/// desktop prototype: explicit `image_path`, then `assets/dishes/{dish_id}` with
/// jpg/png/jpeg fallbacks. The returned value is a URL under Axum's `/assets`
/// static service.
fn dish_image_url(dish: &Dish) -> Option<String> {
    if let Some(path) = dish.image_path.as_deref() {
        let path = PathBuf::from(path);
        if path.exists() {
            return asset_url_from_path(&path);
        }
    }

    ["jpg", "png", "jpeg"].into_iter().find_map(|extension| {
        let path = PathBuf::from(DISH_IMAGE_DIR).join(format!("{}.{}", dish.dish_id, extension));
        path.exists().then(|| asset_url_from_path(&path)).flatten()
    })
}

fn asset_url_from_path(path: &Path) -> Option<String> {
    let normalized = path.to_string_lossy().replace('\\', "/");
    normalized
        .strip_prefix("assets/")
        .map(|relative| format!("/assets/{relative}"))
}

fn detailed_recommendation_reason(
    result: &RecommendationResult,
    related_selected_dishes: &[String],
) -> String {
    let mut reasons = Vec::new();

    if !result.matched_liked_ingredients.is_empty() {
        reasons.push(format!(
            "matched preferred ingredient(s): {}",
            result.matched_liked_ingredients.join(", ")
        ));
    }

    if !result.matched_preferred_tags.is_empty() {
        reasons.push(format!(
            "matched preferred tag(s): {}",
            result.matched_preferred_tags.join(", ")
        ));
    }

    if !related_selected_dishes.is_empty() {
        reasons.push(format!(
            "often ordered with {}",
            related_selected_dishes.join(", ")
        ));
    }

    if result.association_lift > 0.0 {
        reasons.push(format!(
            "association metrics show pair count {}, support {:.2}, confidence {:.2}, lift {:.2}",
            result.association_pair_count,
            result.association_support,
            result.association_confidence,
            result.association_lift
        ));
    }

    if result.popularity_score > 0.0 {
        reasons.push("popular dish based on historical orders".to_string());
    }

    if result.business_rule_score > 0.0 {
        reasons.push("matches the selected time/menu context".to_string());
    }

    if reasons.is_empty() {
        format!(
            "{} is recommended by hybrid score {:.2}. Formula: 0.45 content + 0.25 co-order + 0.20 popularity + 0.10 time/business.",
            result.dish.name, result.final_score
        )
    } else {
        format!(
            "{} is recommended because it {}. Hybrid score {:.2} = content {:.2}, co-order {:.2}, popularity {:.2}, time/business {:.2}.",
            result.dish.name,
            reasons.join("; "),
            result.final_score,
            result.ingredient_score,
            result.co_order_score,
            result.popularity_score,
            result.business_rule_score
        )
    }
}

fn next_dish_id(dishes: &[Dish]) -> String {
    let next_number = dishes
        .iter()
        .filter_map(|dish| dish.dish_id.strip_prefix('D'))
        .filter_map(|number| number.parse::<u32>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    format!("D{next_number:02}")
}

fn unix_timestamp_label() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    format!("unix:{seconds}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dish(id: &str, name: &str, category: &str, ingredients: &[&str], tags: &[&str]) -> Dish {
        Dish {
            dish_id: id.to_string(),
            name: name.to_string(),
            ingredients: ingredients.iter().map(|value| value.to_string()).collect(),
            category: category.to_string(),
            tags: tags.iter().map(|value| value.to_string()).collect(),
            image_path: None,
            image_source_url: None,
        }
    }

    #[test]
    fn menu_view_marks_recommended_dishes() {
        let dishes = vec![
            dish(
                "D01",
                "Chicken Rice",
                "main",
                &["chicken", "rice"],
                &["signature"],
            ),
            dish("D02", "Plain Rice", "side", &["rice"], &["simple"]),
        ];
        let state = WebState::new(dishes, Vec::new());

        let view = state.menu_view();

        assert_eq!(view.dishes.len(), 2);
        assert!(!view.recommended.is_empty());
        assert!(view.dishes.iter().any(|dish| dish.recommended));
    }

    #[test]
    fn checkout_adds_valid_live_order_to_memory() {
        let dishes = vec![dish(
            "D01",
            "Chicken Rice",
            "main",
            &["chicken"],
            &["signature"],
        )];
        let state = WebState::new(dishes, Vec::new());

        let order = state
            .create_live_order(&["D01".to_string(), "UNKNOWN".to_string()])
            .expect("valid dish should create an order");

        assert_eq!(order.ordered_dishes, vec!["D01"]);
        assert_eq!(order.status, OrderStatus::Pending);
        assert_eq!(state.live_order_count(), 1);
    }

    #[test]
    fn dish_availability_removes_dish_from_customer_menu() {
        let dishes = vec![dish(
            "D01",
            "Chicken Rice",
            "main",
            &["chicken"],
            &["signature"],
        )];
        let state = WebState::new(dishes, Vec::new());

        state.set_dish_availability("D01", false).unwrap();

        assert!(state.menu_view().dishes.is_empty());
        assert_eq!(state.admin_view().unavailable_dishes, 1);
    }

    #[test]
    fn selected_dishes_drive_collaborative_recommendations() {
        let dishes = vec![
            dish(
                "D01",
                "Nasi Lemak",
                "main",
                &["rice", "egg"],
                &["signature"],
            ),
            dish(
                "D02",
                "Chicken Satay",
                "main",
                &["chicken", "peanut sauce"],
                &["grilled"],
            ),
        ];
        let orders = vec![Order {
            order_id: "O001".to_string(),
            session_user_id: "U01".to_string(),
            ordered_dishes: vec!["D01".to_string(), "D02".to_string()],
            timestamp: "2026-01-01 12:30".to_string(),
        }];
        let state = WebState::new(dishes, orders);

        let response = state.recommend(RecommendationRequest {
            selected_dish_ids: vec!["D01".to_string()],
            ..RecommendationRequest::default()
        });

        let top = response
            .recommendations
            .first()
            .expect("co-order data should produce one recommendation");
        assert_eq!(top.dish.dish_id, "D02");
        assert!(top.co_order_score > 0.0);
        assert_eq!(top.related_selected_dishes, vec!["Nasi Lemak (D01)"]);
    }

    #[test]
    fn updating_live_order_status_changes_admin_view() {
        let dishes = vec![dish(
            "D01",
            "Chicken Rice",
            "main",
            &["chicken"],
            &["signature"],
        )];
        let state = WebState::new(dishes, Vec::new());
        let order = state.create_live_order(&["D01".to_string()]).unwrap();

        state
            .update_order_status(&order.order_id, OrderStatus::Ready)
            .unwrap();

        assert_eq!(state.admin_view().live_orders[0].status, OrderStatus::Ready);
    }

    #[test]
    fn completed_order_moves_to_completed_session_section() {
        let dishes = vec![
            dish("D01", "Nasi Lemak", "main", &["rice"], &["signature"]),
            dish("D02", "Satay", "main", &["chicken"], &["grilled"]),
        ];
        let state = WebState::new(dishes, Vec::new());
        let order = state
            .create_live_order(&["D01".to_string(), "D02".to_string()])
            .unwrap();

        state
            .update_order_status(&order.order_id, OrderStatus::Completed)
            .unwrap();

        let admin = state.admin_view();
        assert!(admin.live_orders.is_empty());
        assert_eq!(admin.completed_session_orders.len(), 1);
        assert_eq!(
            admin.completed_session_orders[0].dish_names,
            vec!["Nasi Lemak", "Satay"]
        );
        assert_eq!(admin.historical_orders[0].order_id, order.order_id);
        assert!(
            state.completed_session_orders_for_export()[0]
                .ordered_dishes
                .contains(&"D01".to_string())
        );
    }

    #[test]
    fn checkout_order_keeps_duplicate_items_for_total_price() {
        let dishes = vec![dish("D01", "Nasi Lemak", "main", &["rice"], &["signature"])];
        let state = WebState::new(dishes, Vec::new());

        let order = state
            .create_live_order(&["D01".to_string(), "D01".to_string()])
            .unwrap();

        assert_eq!(order.ordered_dishes, vec!["D01", "D01"]);
        assert_eq!(order.dish_names, vec!["Nasi Lemak", "Nasi Lemak"]);
        assert!(order.total_price_amount > placeholder_price_amount("D01"));
    }

    #[test]
    fn menu_view_uses_dish_id_image_fallback_when_available() {
        let dishes = vec![dish("D01", "Nasi Lemak", "main", &["rice"], &["signature"])];
        let state = WebState::new(dishes, Vec::new());

        let view = state.menu_view();

        assert_eq!(
            view.dishes[0].image_url.as_deref(),
            Some("/assets/dishes/D01.jpg")
        );
    }

    #[test]
    fn upsert_dish_adds_generated_id_and_preference_options() {
        let state = WebState::new(Vec::new(), Vec::new());

        let dish = state
            .upsert_dish(UpsertDishRequest {
                dish_id: None,
                name: "Tofu Bowl".to_string(),
                category: "main".to_string(),
                ingredients: "tofu, rice".to_string(),
                tags: "vegetarian".to_string(),
                image_path: None,
                available: Some(true),
            })
            .unwrap();

        assert_eq!(dish.dish_id, "D01");
        let options = state.menu_view().preference_options;
        assert!(options.ingredients.contains(&"tofu".to_string()));
        assert!(options.tags.contains(&"vegetarian".to_string()));
    }
}
