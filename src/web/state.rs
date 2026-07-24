use crate::agent::preference_parser::{ParsedPreference, parse_preference_prompt};
use crate::data_loader::{ORDERS_PATH, append_completed_order_to_csv, split_csv_field};
use crate::models::{Dish, Order, RecommendationResult, UserPreference};
use crate::persistence::learning_events::{
    LEARNING_EVENTS_PATH, append_learning_event, rewrite_learning_events,
};
use crate::persistence::order_details::{
    ORDER_DETAILS_PATH, OrderDetailRecord, append_order_detail, rewrite_order_details,
};
use crate::preferences::{PreferenceOptions, extract_preference_options};
use crate::recommender::adaptive::{
    AdaptiveScoringConfig, AdaptiveWeights, RecommendationEvidenceProfile,
};
use crate::recommender::association_metrics::{AssociationMetric, calculate_association_metric};
use crate::recommender::counterfactual::{
    CounterfactualChanges, CounterfactualInput, CounterfactualResult, compare_counterfactual,
};
use crate::recommender::diversity_reranker::{DiversityMetrics, DiversityMode};
use crate::recommender::evidence::RecommendationEvidence;
use crate::recommender::hybrid::{
    RecommendationOutput, generate_production_recommendations, generate_recommendations,
};
use crate::recommender::learning_timeline::{
    RecommendationLearningEvent, build_learning_event, rebuild_learning_timeline,
};
use crate::recommender::meal_set::{MealSetInput, MealSetRecommendation, recommend_meal_sets};
use crate::search::{MatchMode, build_search_vocabulary, search_dishes};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Local folder served by Axum under `/assets/dishes`.
const DISH_IMAGE_DIR: &str = "assets/dishes";
static SESSION_SEQUENCE: AtomicU64 = AtomicU64::new(1);

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
    order_details: Arc<RwLock<Vec<OrderDetailRecord>>>,
    customer_sessions: Arc<RwLock<HashMap<String, CustomerSession>>>,
    admin_sessions: Arc<RwLock<HashSet<String>>>,
    unavailable_dish_ids: Arc<RwLock<HashSet<String>>>,
    dish_price_overrides: Arc<RwLock<HashMap<String, u32>>>,
    order_version: Arc<RwLock<u64>>,
    order_csv_path: Arc<PathBuf>,
    order_details_path: Arc<PathBuf>,
    learning_events: Arc<RwLock<Vec<RecommendationLearningEvent>>>,
    learning_events_path: Arc<PathBuf>,
    learning_timeline_warning: Arc<RwLock<Option<String>>>,
}

impl WebState {
    /// Creates web state from data loaded by `data_loader`.
    #[allow(dead_code)]
    pub fn new(dishes: Vec<Dish>, historical_orders: Vec<Order>) -> Self {
        Self {
            dishes: Arc::new(RwLock::new(dishes)),
            historical_orders: Arc::new(RwLock::new(historical_orders)),
            live_orders: Arc::new(RwLock::new(Vec::new())),
            order_details: Arc::new(RwLock::new(Vec::new())),
            customer_sessions: Arc::new(RwLock::new(HashMap::new())),
            admin_sessions: Arc::new(RwLock::new(HashSet::new())),
            unavailable_dish_ids: Arc::new(RwLock::new(HashSet::new())),
            dish_price_overrides: Arc::new(RwLock::new(HashMap::new())),
            order_version: Arc::new(RwLock::new(0)),
            order_csv_path: Arc::new(PathBuf::from(ORDERS_PATH)),
            order_details_path: Arc::new(PathBuf::new()),
            learning_events: Arc::new(RwLock::new(Vec::new())),
            learning_events_path: Arc::new(PathBuf::new()),
            learning_timeline_warning: Arc::new(RwLock::new(None)),
        }
    }

    pub fn new_with_operational_data(
        dishes: Vec<Dish>,
        historical_orders: Vec<Order>,
        order_details: Vec<OrderDetailRecord>,
        learning_events: Vec<RecommendationLearningEvent>,
        timeline_warning: Option<String>,
    ) -> Self {
        let live_orders = live_orders_from_details(&dishes, &order_details);
        Self {
            dishes: Arc::new(RwLock::new(dishes)),
            historical_orders: Arc::new(RwLock::new(historical_orders)),
            live_orders: Arc::new(RwLock::new(live_orders)),
            order_details: Arc::new(RwLock::new(order_details)),
            customer_sessions: Arc::new(RwLock::new(HashMap::new())),
            admin_sessions: Arc::new(RwLock::new(HashSet::new())),
            unavailable_dish_ids: Arc::new(RwLock::new(HashSet::new())),
            dish_price_overrides: Arc::new(RwLock::new(HashMap::new())),
            order_version: Arc::new(RwLock::new(0)),
            order_csv_path: Arc::new(PathBuf::from(ORDERS_PATH)),
            order_details_path: Arc::new(PathBuf::from(ORDER_DETAILS_PATH)),
            learning_events: Arc::new(RwLock::new(learning_events)),
            learning_events_path: Arc::new(PathBuf::from(LEARNING_EVENTS_PATH)),
            learning_timeline_warning: Arc::new(RwLock::new(timeline_warning)),
        }
    }

    /// Creates web state with a custom historical order CSV path.
    ///
    /// Production uses `WebState::new`, which points at `data/orders.csv`.
    /// Tests use this constructor so completion-persistence checks do not
    /// modify the real FYP dataset.
    #[cfg(test)]
    fn new_with_order_csv_path(
        dishes: Vec<Dish>,
        historical_orders: Vec<Order>,
        order_csv_path: PathBuf,
    ) -> Self {
        Self {
            dishes: Arc::new(RwLock::new(dishes)),
            historical_orders: Arc::new(RwLock::new(historical_orders)),
            live_orders: Arc::new(RwLock::new(Vec::new())),
            order_details: Arc::new(RwLock::new(Vec::new())),
            customer_sessions: Arc::new(RwLock::new(HashMap::new())),
            admin_sessions: Arc::new(RwLock::new(HashSet::new())),
            unavailable_dish_ids: Arc::new(RwLock::new(HashSet::new())),
            dish_price_overrides: Arc::new(RwLock::new(HashMap::new())),
            order_version: Arc::new(RwLock::new(0)),
            order_csv_path: Arc::new(order_csv_path),
            order_details_path: Arc::new(PathBuf::new()),
            learning_events: Arc::new(RwLock::new(Vec::new())),
            learning_events_path: Arc::new(PathBuf::new()),
            learning_timeline_warning: Arc::new(RwLock::new(None)),
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
            search_vocabulary_json: serde_json::to_string(&build_search_vocabulary(
                &self.dishes.read().expect("dishes lock poisoned"),
            ))
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
        let diversity_mode = DiversityMode::from_label(request.diversity_mode.as_deref());
        let preference = request.into_user_preference_or_default();
        let dishes = self.available_dishes();
        let orders = self.combined_orders();
        let output =
            generate_production_recommendations(&dishes, &orders, &preference, diversity_mode);

        RecommendationApiResponse {
            recommendations: output
                .recommendations
                .iter()
                .take(10)
                .map(|result| self.recommendation_view(result))
                .collect(),
            stats: RecommendationStatsView::from_output(&output),
            evidence_profile: output.evidence_profile,
            adaptive_weights: output.adaptive_weights,
            scoring_config: output.scoring_config,
            diversity_mode: output.diversity_mode,
            diversity_metrics: output.diversity_metrics,
        }
    }

    /// Builds bounded, explainable meal sets from current prices, availability,
    /// production recommendation scores, and the customer's selected context.
    pub fn recommend_meal_sets(
        &self,
        request: MealSetRequest,
    ) -> Result<Vec<MealSetRecommendation>, String> {
        let diversity_mode = DiversityMode::from_label(request.diversity_mode.as_deref());
        let preference = UserPreference {
            liked_ingredients: normalize_list(request.liked_ingredients, false),
            disliked_ingredients: normalize_list(request.disliked_ingredients, false),
            preferred_tags: normalize_list(request.preferred_tags, false),
            selected_dish_ids: normalize_list(request.selected_dish_ids, true),
            time_context: request.time_context,
            ranking_method: Some("hybrid".to_string()),
        };
        let dishes = self.available_dishes();
        // The current prototype stores whole-RM prices. Converting once to
        // integer cents keeps all budget comparisons exact.
        let prices = dishes
            .iter()
            .map(|dish| {
                let amount = self
                    .price_override_for(&dish.dish_id)
                    .unwrap_or_else(|| placeholder_price_amount(&dish.dish_id));
                (dish.dish_id.clone(), amount.saturating_mul(100))
            })
            .collect::<HashMap<_, _>>();
        recommend_meal_sets(
            &dishes,
            &self.combined_orders(),
            &prices,
            &MealSetInput {
                budget_cents: request.budget_cents,
                party_size: request.party_size,
                target_dish_count: request.target_dish_count,
                top_set_count: request.top_set_count,
                preference,
                required_categories: normalize_list(request.required_categories, false),
                diversity_mode,
            },
        )
    }

    /// Compares two production recommendation runs using cloned scenario data.
    pub fn counterfactual(
        &self,
        request: CounterfactualRequest,
    ) -> Result<CounterfactualResult, String> {
        let baseline_mode = DiversityMode::from_label(request.baseline.diversity_mode.as_deref());
        let baseline = request.baseline.into_user_preference_or_default();
        compare_counterfactual(
            &self.available_dishes(),
            &self.combined_orders(),
            &CounterfactualInput {
                baseline,
                baseline_diversity_mode: baseline_mode,
                changes: request.changes,
                top_k: request.top_k,
            },
        )
    }

    /// Returns persisted timeline events newest first with any recoverable
    /// persistence warning collected during order completion.
    pub fn learning_timeline(&self) -> LearningTimelineResponse {
        let mut events = self
            .learning_events
            .read()
            .expect("learning events lock poisoned")
            .clone();
        events.reverse();
        LearningTimelineResponse {
            event_count: events.len(),
            events,
            warning: self
                .learning_timeline_warning
                .read()
                .expect("timeline warning lock poisoned")
                .clone(),
        }
    }

    /// Deterministically rebuilds explanatory events from durable order history.
    pub fn rebuild_learning_timeline(&self) -> Result<LearningTimelineResponse, String> {
        // Holding the event lock through reconstruction and replacement keeps a
        // newly completed order from appending between the file rewrite and
        // in-memory replacement.
        let mut stored_events = self
            .learning_events
            .write()
            .expect("learning events lock poisoned");
        let events = rebuild_learning_timeline(
            &self
                .historical_orders
                .read()
                .expect("historical orders lock poisoned"),
            &self.dishes.read().expect("dishes lock poisoned"),
        );
        if !self.learning_events_path.as_os_str().is_empty() {
            rewrite_learning_events(&events, &self.learning_events_path.to_string_lossy())
                .map_err(|error| format!("Could not rebuild the learning timeline: {error}"))?;
        }
        *stored_events = events;
        drop(stored_events);
        *self
            .learning_timeline_warning
            .write()
            .expect("timeline warning lock poisoned") = None;
        Ok(self.learning_timeline())
    }

    /// Clears only explanatory learning events and leaves every order-based
    /// recommendation data source untouched.
    ///
    /// The write lock spans the durable replacement and the in-memory clear so
    /// a concurrently completed order cannot be lost between those two steps.
    /// If persistence fails, the existing in-memory events remain available.
    pub fn clear_learning_timeline(&self) -> Result<LearningTimelineClearResponse, String> {
        let mut events = self
            .learning_events
            .write()
            .expect("learning events lock poisoned");
        let removed_event_count = events.len();
        if !self.learning_events_path.as_os_str().is_empty() {
            rewrite_learning_events(&[], &self.learning_events_path.to_string_lossy())
                .map_err(|error| format!("Could not clear the learning timeline: {error}"))?;
        }
        events.clear();
        *self
            .learning_timeline_warning
            .write()
            .expect("timeline warning lock poisoned") = None;
        Ok(LearningTimelineClearResponse {
            removed_event_count,
        })
    }

    /// Runs canonical customer menu search for the locator suggestion list.
    ///
    /// Search is intentionally resolved on the Rust side because the same
    /// concept vocabulary must drive dropdown suggestions and tests. Search
    /// results are deliberately separate from the static customer Menu view.
    pub fn search_menu(&self, query: &str, mode: MatchMode) -> MenuSearchResponse {
        let dishes = self.available_dishes();
        let vocabulary = build_search_vocabulary(&dishes);
        let views = dishes
            .iter()
            .map(|dish| {
                (
                    dish.dish_id.clone(),
                    DishView::from_dish(dish, true, false, self.price_override_for(&dish.dish_id)),
                )
            })
            .collect::<HashMap<_, _>>();

        let results = search_dishes(&dishes, query, mode, &vocabulary)
            .into_iter()
            .filter_map(|result| {
                views
                    .get(&result.dish_id)
                    .cloned()
                    .map(|dish| MenuSearchItem {
                        dish,
                        match_score: result.match_score,
                        match_reasons: result.match_reasons,
                    })
            })
            .collect::<Vec<_>>();

        MenuSearchResponse {
            query: query.to_string(),
            mode: match mode {
                MatchMode::Any => "any".to_string(),
                MatchMode::All => "all".to_string(),
            },
            result_count: results.len(),
            results,
        }
    }

    /// Runs the rule-based Smart Menu Assistant.
    ///
    /// The assistant only converts natural language into structured
    /// preferences. The actual ranking still uses the existing recommender, so
    /// this feature stays lightweight and explainable instead of becoming a
    /// separate black-box algorithm.
    pub fn assistant_recommend(&self, request: AssistantRequest) -> AssistantResponse {
        let dishes = self.available_dishes();
        let parsed = parse_preference_prompt(&request.prompt, &dishes);
        let selected_dish_ids = normalize_list(request.selected_dish_ids, true);
        let preference = parsed.to_user_preference(selected_dish_ids.clone());
        let orders = self.combined_orders();
        let output = generate_production_recommendations(
            &dishes,
            &orders,
            &preference,
            DiversityMode::Balanced,
        );

        AssistantResponse {
            understood: parsed.understood_summary.clone(),
            parsed,
            recommendations: output
                .recommendations
                .iter()
                .take(8)
                .map(|result| self.recommendation_view(result))
                .collect(),
            upsells: self.cart_upsells(&selected_dish_ids),
            stats: RecommendationStatsView::from_output(&output),
            evidence_profile: output.evidence_profile,
            adaptive_weights: output.adaptive_weights,
            scoring_config: output.scoring_config,
            diversity_mode: output.diversity_mode,
            diversity_metrics: output.diversity_metrics,
        }
    }

    /// Produces rule-based admin insights from persisted order baskets.
    pub fn admin_insights(&self) -> AdminInsightResponse {
        let popular = self
            .frequent_dishes(5)
            .into_iter()
            .map(|item| format!("{} appeared in {} order(s).", item.label, item.count))
            .collect::<Vec<_>>();
        let co_order_pairs = self
            .common_co_order_pairs(5)
            .into_iter()
            .map(|item| {
                format!(
                    "{} were ordered together {} time(s).",
                    item.label, item.count
                )
            })
            .collect::<Vec<_>>();
        let low_exposure = self
            .low_exposure_dishes(5)
            .into_iter()
            .map(|dish| {
                format!(
                    "{} ({}) has low exposure in order logs.",
                    dish.name, dish.dish_id
                )
            })
            .collect::<Vec<_>>();

        let order_count = self.historical_order_count();
        let summary = if order_count < 20 {
            format!(
                "Limited-data warning: only {order_count} historical basket(s) are available. Insights include CSV history and completed checkout orders, but patterns should be treated as early evidence."
            )
        } else {
            format!(
                "Recommendation data summary: {order_count} historical basket(s), including CSV history and completed checkout orders, support popularity and co-order analysis."
            )
        };

        AdminInsightResponse {
            summary,
            popular,
            co_order_pairs,
            low_exposure,
        }
    }

    #[allow(dead_code)]
    pub fn evaluation_report(&self, request: EvaluationRequest) -> EvaluationResponse {
        let start = Instant::now();
        let dishes = self.available_dishes();
        let all_orders = self.combined_orders();
        let orders = match request.dataset_size.as_deref() {
            Some("5") => all_orders.into_iter().take(5).collect::<Vec<_>>(),
            Some("20") => all_orders.into_iter().take(20).collect::<Vec<_>>(),
            _ => all_orders,
        };
        let preference = RecommendationRequest {
            liked_ingredients: request.liked_ingredients,
            disliked_ingredients: request.disliked_ingredients,
            preferred_tags: request.preferred_tags,
            selected_dish_ids: request.selected_dish_ids,
            time_context: request.time_context,
            ranking_method: Some("hybrid".to_string()),
            diversity_mode: None,
        }
        .into_user_preference_or_default();

        let static_results = dishes
            .iter()
            .take(5)
            .map(|dish| EvaluationRecommendation {
                dish_id: dish.dish_id.clone(),
                dish_name: dish.name.clone(),
                score: 0.0,
                reason: "Static menu order baseline.".to_string(),
            })
            .collect::<Vec<_>>();
        let content_results = evaluation_method_results(
            &dishes,
            &orders,
            &preference,
            "content-based",
            |mut pref| {
                pref.ranking_method = Some("content-based".to_string());
                pref
            },
        );
        let co_order_results =
            evaluation_method_results(&dishes, &orders, &preference, "co-ordering", |mut pref| {
                pref.ranking_method = Some("co-ordering".to_string());
                pref
            });
        let popularity_results =
            evaluation_method_results(&dishes, &orders, &preference, "popularity", |mut pref| {
                pref.liked_ingredients.clear();
                pref.preferred_tags.clear();
                pref.selected_dish_ids.clear();
                pref.ranking_method = Some("co-ordering".to_string());
                pref
            });
        let hybrid_results =
            evaluation_method_results(&dishes, &orders, &preference, "hybrid", |mut pref| {
                pref.ranking_method = Some("hybrid".to_string());
                pref
            });
        let methods = vec![
            EvaluationMethodOutput::new("Static", static_results, &dishes),
            EvaluationMethodOutput::new("Content-Based", content_results, &dishes),
            EvaluationMethodOutput::new("Co-Ordering", co_order_results, &dishes),
            EvaluationMethodOutput::new("Popularity", popularity_results, &dishes),
            EvaluationMethodOutput::new("Hybrid", hybrid_results, &dishes),
        ];
        EvaluationResponse {
            dataset_size: orders.len(),
            dish_count: dishes.len(),
            unique_co_order_pairs: count_unique_pairs(&orders),
            average_dishes_per_order: average_dishes_per_order(&orders),
            response_time_ms: start.elapsed().as_millis(),
            methods,
        }
    }

    /// Runs an in-memory co-order simulation for the admin Recommendation
    /// Tester. Generated baskets are combined with the current historical
    /// orders only for this response; real `data/orders.csv` is never modified.
    pub fn simulation_report(&self, request: SimulationRequest) -> SimulationResponse {
        let dishes = self.available_dishes();
        let base_orders = self.combined_orders();
        let preference = RecommendationRequest {
            liked_ingredients: request.liked_ingredients.clone(),
            disliked_ingredients: request.disliked_ingredients.clone(),
            preferred_tags: request.preferred_tags.clone(),
            selected_dish_ids: request.selected_dish_ids.clone(),
            time_context: request.time_context.clone(),
            ranking_method: Some("hybrid".to_string()),
            diversity_mode: None,
        }
        .into_user_preference_or_default();

        let before = generate_recommendations(&dishes, &base_orders, &preference)
            .recommendations
            .into_iter()
            .take(10)
            .collect::<Vec<_>>();
        let generated_orders = generate_simulated_orders(&dishes, &base_orders, &request);
        let mut simulated_dataset = base_orders.clone();
        simulated_dataset.extend(generated_orders.clone());
        let after = generate_recommendations(&dishes, &simulated_dataset, &preference)
            .recommendations
            .into_iter()
            .take(10)
            .collect::<Vec<_>>();

        let before_rank = before
            .iter()
            .enumerate()
            .map(|(index, result)| (result.dish.dish_id.clone(), (index + 1, result.final_score)))
            .collect::<HashMap<_, _>>();
        let after_rank = after
            .iter()
            .enumerate()
            .map(|(index, result)| (result.dish.dish_id.clone(), (index + 1, result.final_score)))
            .collect::<HashMap<_, _>>();

        let mut rank_changes = after
            .iter()
            .map(|result| {
                let before_value = before_rank.get(&result.dish.dish_id).copied();
                let after_value = after_rank.get(&result.dish.dish_id).copied();
                SimulationRankChange {
                    dish_id: result.dish.dish_id.clone(),
                    dish_name: result.dish.name.clone(),
                    before_rank: before_value.map(|(rank, _)| rank),
                    after_rank: after_value.map(|(rank, _)| rank),
                    before_score: before_value.map(|(_, score)| score).unwrap_or(0.0),
                    after_score: after_value.map(|(_, score)| score).unwrap_or(0.0),
                    explanation: simulation_rank_reason(
                        &result.dish.dish_id,
                        before_value,
                        after_value,
                    ),
                }
            })
            .collect::<Vec<_>>();
        rank_changes.sort_by(|a, b| {
            let a_change = a.after_score - a.before_score;
            let b_change = b.after_score - b.before_score;
            b_change.total_cmp(&a_change)
        });

        let changed_pairs = top_changed_pairs(&dishes, &base_orders, &simulated_dataset, 8);
        let preview = generated_orders
            .iter()
            .take(8)
            .map(|order| SimulationOrderPreview {
                order_id: order.order_id.clone(),
                dish_ids: order.ordered_dishes.clone(),
                dish_names: self.dish_labels_for_ids(&order.ordered_dishes),
            })
            .collect();

        SimulationResponse {
            generated_order_count: generated_orders.len(),
            preview,
            changed_pairs,
            rank_changes: rank_changes.into_iter().take(10).collect(),
            note:
                "Simulation used an in-memory dataset only. Real data/orders.csv was not changed."
                    .to_string(),
        }
    }

    pub fn experiment_lab(
        &self,
        request: ExperimentLabRequest,
    ) -> Result<ExperimentLabResponse, String> {
        match request.experiment_type.as_str() {
            "ingredient" => Ok(self.ingredient_experiment(request)),
            "coorder" => self.coorder_impact_experiment(request),
            "method" => self.method_comparison_experiment(request),
            _ => Err("Unknown experiment type.".to_string()),
        }
    }

    fn ingredient_experiment(&self, request: ExperimentLabRequest) -> ExperimentLabResponse {
        let top_k = request.top_k.unwrap_or(5).clamp(3, 10);
        let disliked = normalize_list(request.disliked_ingredients.clone(), false);
        let preference = RecommendationRequest {
            liked_ingredients: request.liked_ingredients,
            disliked_ingredients: request.disliked_ingredients,
            preferred_tags: request.preferred_tags,
            selected_dish_ids: request.selected_dish_ids,
            ranking_method: Some("content-based".to_string()),
            time_context: None,
            diversity_mode: None,
        }
        .into_user_preference_or_default();
        let dishes = self.available_dishes();
        let orders = self.combined_orders();
        let baseline = generate_recommendations(&dishes, &orders, &UserPreference::default());
        let output = generate_recommendations(&dishes, &orders, &preference);
        let mut rows = baseline
            .recommendations
            .into_iter()
            .take(top_k)
            .enumerate()
            .map(|(index, item)| ExperimentRow {
                method: "Before (no preferences)".to_string(),
                rank: Some(index + 1),
                dish_id: item.dish.dish_id,
                dish_name: item.dish.name,
                ingredient_score: item.ingredient_score,
                co_order_score: 0.0,
                final_score: item.ingredient_score,
                matched: item.matched_liked_ingredients.join(", "),
                excluded: "-".to_string(),
                hidden_match: false,
            })
            .collect::<Vec<_>>();
        rows.extend(
            output
                .recommendations
                .into_iter()
                .take(top_k)
                .enumerate()
                .map(|(index, item)| ExperimentRow {
                    method: "After (selected preferences)".to_string(),
                    rank: Some(index + 1),
                    dish_id: item.dish.dish_id,
                    dish_name: item.dish.name,
                    ingredient_score: item.ingredient_score,
                    co_order_score: 0.0,
                    final_score: item.ingredient_score,
                    matched: if item.matched_liked_ingredients.is_empty() {
                        "-".to_string()
                    } else {
                        item.matched_liked_ingredients.join(", ")
                    },
                    excluded: "-".to_string(),
                    hidden_match: false,
                }),
        );

        let excluded_count = dishes
            .iter()
            .filter(|dish| {
                dish.ingredients
                    .iter()
                    .any(|ingredient| disliked.contains(&ingredient.to_lowercase()))
            })
            .count();
        let conclusion = rows
            .iter()
            .find(|row| row.method.starts_with("After"))
            .map(|row| {
                format!(
                    "{} became the top preference-shaped result with score {:.2}. {} dish(es) were excluded by disliked ingredients.",
                    row.dish_name, row.ingredient_score, excluded_count
                )
            })
            .unwrap_or_else(|| "No ingredient-compatible dishes were found.".to_string());
        ExperimentLabResponse {
            experiment_type: "ingredient".to_string(),
            conclusion,
            csv: "rank,dish_id,dish_name,matched_ingredients,excluded,ingredient_score\n"
                .to_string(),
            rows,
            simulation: None,
        }
    }

    fn coorder_impact_experiment(
        &self,
        request: ExperimentLabRequest,
    ) -> Result<ExperimentLabResponse, String> {
        let anchor = request
            .anchor_dish_id
            .as_deref()
            .map(str::trim)
            .map(str::to_uppercase)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "Anchor dish is required for Co-Order Impact.".to_string())?;
        let candidate = request
            .candidate_dish_id
            .as_deref()
            .map(str::trim)
            .map(str::to_uppercase)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "Candidate dish is required for Co-Order Impact.".to_string())?;
        if anchor == candidate {
            return Err("Anchor and candidate dishes must be different.".to_string());
        }

        let additional = request.additional_coorders.unwrap_or(10).min(200);
        let dishes = self.available_dishes();
        validate_dish_exists(&dishes, &anchor)?;
        validate_dish_exists(&dishes, &candidate)?;
        let base_orders = self.combined_orders();
        let mut after_orders = base_orders.clone();
        // Co-Order Impact uses exact temporary baskets containing the selected
        // anchor/candidate pair. It never calls the broader random simulation
        // helper, so zero additional orders really means no change.
        for index in 0..additional {
            after_orders.push(Order {
                order_id: format!("EXP{:03}", index + 1),
                session_user_id: "EXPERIMENT".to_string(),
                ordered_dishes: vec![anchor.clone(), candidate.clone()],
                timestamp: "in-memory experiment".to_string(),
            });
        }

        let preference = RecommendationRequest {
            liked_ingredients: request.liked_ingredients,
            disliked_ingredients: request.disliked_ingredients,
            preferred_tags: request.preferred_tags,
            selected_dish_ids: vec![anchor.clone()],
            ranking_method: Some("hybrid".to_string()),
            time_context: None,
            diversity_mode: None,
        }
        .into_user_preference_or_default();

        let before_rank =
            controlled_candidate_rank(&dishes, &base_orders, &preference, &candidate, 0.0, 1.0);
        let after_rank =
            controlled_candidate_rank(&dishes, &after_orders, &preference, &candidate, 0.0, 1.0);
        let before_metric = pair_metric_or_default(&base_orders, &anchor, &candidate);
        let after_metric = pair_metric_or_default(&after_orders, &anchor, &candidate);
        let anchor_label = self.dish_labels_for_ids(std::slice::from_ref(&anchor));
        let candidate_label = self.dish_labels_for_ids(std::slice::from_ref(&candidate));
        let anchor_label = anchor_label.first().cloned().unwrap_or(anchor.clone());
        let candidate_label = candidate_label
            .first()
            .cloned()
            .unwrap_or(candidate.clone());

        let rows = vec![
            ExperimentRow {
                method: "Co-Order Impact Before".to_string(),
                rank: before_rank.map(|rank| rank.0),
                dish_id: candidate.clone(),
                dish_name: candidate_label.clone(),
                ingredient_score: 0.0,
                co_order_score: before_rank.map(|rank| rank.1).unwrap_or(0.0),
                final_score: before_rank.map(|rank| rank.1).unwrap_or(0.0),
                matched: format!(
                    "Pair count: {}; support {:.2}; confidence {:.2}; lift {:.2}.",
                    before_metric.pair_count,
                    before_metric.support,
                    before_metric.confidence,
                    before_metric.lift
                ),
                excluded: "-".to_string(),
                hidden_match: false,
            },
            ExperimentRow {
                method: "Co-Order Impact After".to_string(),
                rank: after_rank.map(|rank| rank.0),
                dish_id: candidate.clone(),
                dish_name: candidate_label.clone(),
                ingredient_score: 0.0,
                co_order_score: after_rank.map(|rank| rank.1).unwrap_or(0.0),
                final_score: after_rank.map(|rank| rank.1).unwrap_or(0.0),
                matched: format!(
                    "Pair count: {}; support {:.2}; confidence {:.2}; lift {:.2}; added temporary co-orders: {}.",
                    after_metric.pair_count,
                    after_metric.support,
                    after_metric.confidence,
                    after_metric.lift,
                    additional
                ),
                excluded: "-".to_string(),
                hidden_match: false,
            },
        ];

        let rank_change = match (before_rank, after_rank) {
            (Some((before, _)), Some((after, _))) => before as isize - after as isize,
            (None, Some((after, _))) => after as isize,
            _ => 0,
        };
        let conclusion = format!(
            "{} -> {} pair count changed from {} to {}. Candidate rank change: {}. Real data/orders.csv was not modified.",
            anchor_label,
            candidate_label,
            before_metric.pair_count,
            after_metric.pair_count,
            rank_change
        );

        Ok(ExperimentLabResponse {
            experiment_type: "coorder".to_string(),
            conclusion,
            csv: "phase,dish_id,dish_name,rank,coorder_score,pair_count,support,confidence,lift\n"
                .to_string(),
            rows,
            simulation: None,
        })
    }

    fn method_comparison_experiment(
        &self,
        request: ExperimentLabRequest,
    ) -> Result<ExperimentLabResponse, String> {
        let top_k = request.top_k.unwrap_or(3).clamp(1, 10);
        let historical_order_id = request
            .historical_order_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "Historical order is required for Method Comparison.".to_string())?;
        let hidden = request
            .hidden_dish_id
            .or(request.candidate_dish_id)
            .unwrap_or_default()
            .trim()
            .to_uppercase();
        if hidden.is_empty() {
            return Err("Hidden target dish is required for Method Comparison.".to_string());
        }

        let historical_order = self
            .combined_orders()
            .into_iter()
            .find(|order| order.order_id.eq_ignore_ascii_case(historical_order_id))
            .ok_or_else(|| format!("Historical order {historical_order_id} was not found."))?;
        if historical_order.ordered_dishes.len() < 2 {
            return Err("Selected historical order must contain at least two dishes.".to_string());
        }
        if !historical_order
            .ordered_dishes
            .iter()
            .any(|dish_id| dish_id.eq_ignore_ascii_case(&hidden))
        {
            return Err("Hidden target must belong to the selected historical order.".to_string());
        }

        let selected = historical_order
            .ordered_dishes
            .iter()
            .map(|dish_id| dish_id.trim().to_uppercase())
            .filter(|dish_id| !dish_id.is_empty() && dish_id != &hidden)
            .collect::<Vec<_>>();
        let preference = RecommendationRequest {
            liked_ingredients: request.liked_ingredients,
            disliked_ingredients: request.disliked_ingredients,
            preferred_tags: request.preferred_tags,
            selected_dish_ids: selected,
            ranking_method: Some("hybrid".to_string()),
            time_context: None,
            diversity_mode: None,
        }
        .into_user_preference_or_default();
        let dishes = self.available_dishes();
        let orders = self.combined_orders();
        let mut all_rows = Vec::new();
        for (method, iw, cw) in [
            ("Ingredient-only", 1.0_f32, 0.0_f32),
            ("Co-order-only", 0.0, 1.0),
            ("Hybrid 0.4/0.6", 0.4, 0.6),
        ] {
            let rows = controlled_ranked_results(&dishes, &orders, &preference, iw, cw);
            let hidden_rank = rows
                .iter()
                .position(|(item, _)| item.dish.dish_id == hidden)
                .map(|index| index + 1);
            let hit_at_k = hidden_rank.is_some_and(|rank| rank <= top_k);
            let preference_match_rate = preference_match_rate(&rows, top_k, &preference);
            let restriction_violations = restriction_violations(&rows, top_k);
            all_rows.extend(rows.into_iter().take(top_k).enumerate().map(
                |(index, (item, score))| {
                    let hidden_match = item.dish.dish_id == hidden;
                    ExperimentRow {
                    method: method.to_string(),
                    rank: Some(index + 1),
                    dish_id: item.dish.dish_id,
                    dish_name: item.dish.name,
                    ingredient_score: item.ingredient_score,
                    co_order_score: item.co_order_score,
                    final_score: score,
                    matched: format!(
                        "{}Hit@{}: {}; hidden rank: {}; preference match rate: {:.2}; violations: {}.",
                        if hidden_match { "Hidden target recovered. " } else { "" },
                        top_k,
                        if hit_at_k { "yes" } else { "no" },
                        hidden_rank
                            .map(|rank| rank.to_string())
                            .unwrap_or_else(|| "Not recovered".to_string()),
                        preference_match_rate,
                        restriction_violations
                    ),
                    excluded: "-".to_string(),
                    hidden_match,
                    }
                },
            ));
        }
        let recovered = all_rows
            .iter()
            .filter(|row| row.hidden_match)
            .map(|row| format!("{} at rank {}", row.method, row.rank.unwrap_or(0)))
            .collect::<Vec<_>>();
        let conclusion = if recovered.is_empty() {
            format!(
                "No controlled method recovered {} in Top-{} for historical order {}.",
                self.dish_labels_for_ids(std::slice::from_ref(&hidden))
                    .first()
                    .cloned()
                    .unwrap_or(hidden.clone()),
                top_k,
                historical_order_id
            )
        } else {
            format!(
                "Recovered hidden target for order {}: {}.",
                historical_order_id,
                recovered.join("; ")
            )
        };
        Ok(ExperimentLabResponse {
            experiment_type: "method".to_string(),
            conclusion,
            csv: "method,rank,dish_id,dish_name,ingredient_score,coorder_score,final_score,hidden_dish_match\n".to_string(),
            rows: all_rows,
            simulation: None,
        })
    }

    /// Registers a temporary dining session before the customer sees the menu.
    ///
    /// This keeps customer identity handling server-side instead of relying on
    /// cart JavaScript fields. The session is short-lived and belongs only to
    /// the current QR ordering visit; it is not a permanent customer account.
    pub fn register_customer_session(
        &self,
        request: CustomerRegistrationRequest,
    ) -> Result<CustomerSession, String> {
        let customer_name = validate_customer_name(&request.customer_name)?;
        let customer_phone = normalize_phone(&request.customer_phone)?;
        let table_number = validate_table_number(&request.table_number)?;
        let session_id = new_opaque_session_id("customer");
        let session = CustomerSession {
            session_id: session_id.clone(),
            customer_name,
            customer_phone,
            table_number,
        };
        self.customer_sessions
            .write()
            .expect("customer sessions lock poisoned")
            .insert(session_id, session.clone());
        Ok(session)
    }

    /// Edits details for the current temporary dining session.
    pub fn update_customer_session(
        &self,
        session_id: &str,
        request: CustomerRegistrationRequest,
    ) -> Result<CustomerSession, String> {
        let customer_name = validate_customer_name(&request.customer_name)?;
        let customer_phone = normalize_phone(&request.customer_phone)?;
        let table_number = validate_table_number(&request.table_number)?;
        let mut sessions = self
            .customer_sessions
            .write()
            .expect("customer sessions lock poisoned");
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| "Your customer session expired. Please register again.".to_string())?;
        session.customer_name = customer_name;
        session.customer_phone = customer_phone;
        session.table_number = table_number;
        Ok(session.clone())
    }

    /// Reads a registered customer session by opaque cookie value.
    pub fn customer_session(&self, session_id: &str) -> Option<CustomerSession> {
        self.customer_sessions
            .read()
            .expect("customer sessions lock poisoned")
            .get(session_id)
            .cloned()
    }

    /// Ends the temporary customer session without touching admin login.
    pub fn clear_customer_session(&self, session_id: &str) {
        self.customer_sessions
            .write()
            .expect("customer sessions lock poisoned")
            .remove(session_id);
    }

    /// Creates an opaque staff session after credentials have been checked.
    ///
    /// Admin sessions are stored separately from customer dining sessions so
    /// staff login and logout can never overwrite a customer's session.
    pub fn create_admin_session(&self) -> String {
        let session_id = new_opaque_session_id("admin");
        self.admin_sessions
            .write()
            .expect("admin sessions lock poisoned")
            .insert(session_id.clone());
        session_id
    }

    /// Returns whether an admin cookie refers to a currently active session.
    pub fn is_admin_session(&self, session_id: &str) -> bool {
        self.admin_sessions
            .read()
            .expect("admin sessions lock poisoned")
            .contains(session_id)
    }

    /// Invalidates only the selected admin session.
    pub fn clear_admin_session(&self, session_id: &str) {
        self.admin_sessions
            .write()
            .expect("admin sessions lock poisoned")
            .remove(session_id);
    }

    /// Creates a live web order from checkout dish IDs.
    #[allow(dead_code)]
    pub fn create_live_order(&self, dish_ids: &[String]) -> Result<LiveOrder, String> {
        self.create_live_order_with_customer(CreateLiveOrderRequest {
            dish_ids: dish_ids.to_vec(),
            customer_name: "Demo Customer".to_string(),
            customer_phone: "0123456789".to_string(),
            table_number: None,
            note: None,
        })
    }

    pub fn create_live_order_with_customer(
        &self,
        request: CreateLiveOrderRequest,
    ) -> Result<LiveOrder, String> {
        let customer_name = validate_customer_name(&request.customer_name)?;
        let customer_phone = normalize_phone(&request.customer_phone)?;
        let table_number = request
            .table_number
            .as_ref()
            .map(|value| validate_table_number(value))
            .transpose()?;
        let note = clean_optional_short(&request.note, 120, "Order note")?;
        let dishes = self.available_dishes();
        let dish_lookup = dishes
            .iter()
            .map(|dish| (dish.dish_id.clone(), dish.clone()))
            .collect::<HashMap<_, _>>();

        let cleaned = request
            .dish_ids
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
            .map(|dish_id| {
                self.price_override_for(dish_id)
                    .unwrap_or_else(|| placeholder_price_amount(dish_id))
            })
            .sum();

        let mut live_orders = self.live_orders.write().expect("live orders lock poisoned");
        let order_number = live_orders.len() + 1;
        let timestamp = human_timestamp_label();
        let order = LiveOrder {
            order_id: format!("WEB{order_number:03}"),
            session_user_id: format!("PHONE-{}", masked_phone_suffix(&customer_phone)),
            ordered_dishes: cleaned,
            dish_names,
            timestamp: timestamp.clone(),
            total_price: format!("RM {total_price_amount}"),
            total_price_amount,
            status: OrderStatus::Pending,
            historical_order_id: None,
            customer_name,
            customer_phone,
            table_number,
            note,
        };

        let detail = order.to_detail_record();
        // Persist before mutating live-order memory. If the operational CSV is
        // unavailable, staff should not see a partial order that the customer
        // was told had failed.
        if !self.order_details_path.as_os_str().is_empty() {
            append_order_detail(&detail, &self.order_details_path.to_string_lossy()).map_err(
                |error| format!("We could not save your order. Please try again. ({error})"),
            )?;
        }
        self.order_details
            .write()
            .expect("order details lock poisoned")
            .push(detail);
        live_orders.push(order.clone());
        self.bump_order_version();
        Ok(order)
    }

    /// Creates a checkout order using the server-side customer session created
    /// at `/start`. The cart only submits dish IDs and an order-specific note.
    pub fn create_live_order_from_session(
        &self,
        session_id: &str,
        dish_ids: Vec<String>,
        note: Option<String>,
    ) -> Result<LiveOrder, String> {
        let session = self
            .customer_session(session_id)
            .ok_or_else(|| "Your customer session expired. Please register again.".to_string())?;
        self.create_live_order_with_customer(CreateLiveOrderRequest {
            dish_ids,
            customer_name: session.customer_name,
            customer_phone: session.customer_phone,
            table_number: Some(session.table_number),
            note,
        })
    }

    /// Updates the status of a live order.
    pub fn update_order_status(
        &self,
        order_id: &str,
        status: OrderStatus,
    ) -> Result<OrderStatusUpdate, String> {
        let previous_order = {
            let live_orders = self.live_orders.read().expect("live orders lock poisoned");
            live_orders
                .iter()
                .find(|order| order.order_id.eq_ignore_ascii_case(order_id))
                .cloned()
                .ok_or_else(|| format!("Live order {order_id} was not found."))?
        };

        let should_persist = status == OrderStatus::Completed
            && previous_order.status != OrderStatus::Completed
            && previous_order.historical_order_id.is_none();
        let history_before_completion = should_persist.then(|| {
            self.historical_orders
                .read()
                .expect("historical orders lock poisoned")
                .clone()
        });
        let persisted_order = if should_persist {
            // Completed orders become durable behavioural data. Persisting
            // before mutating status means a disk error is visible to staff and
            // does not silently mark an unsaved order as completed.
            match append_completed_order_to_csv(
                &previous_order.ordered_dishes,
                &human_timestamp_label(),
                self.order_csv_path.to_string_lossy().as_ref(),
            ) {
                Ok(order) => Some(order),
                Err(error) => {
                    return Err(format!(
                        "Order was not marked Completed because saving to data/orders.csv failed: {error}"
                    ));
                }
            }
        } else {
            None
        };

        let updated_order = {
            let mut live_orders = self.live_orders.write().expect("live orders lock poisoned");
            let Some(order) = live_orders
                .iter_mut()
                .find(|order| order.order_id.eq_ignore_ascii_case(order_id))
            else {
                return Err(format!("Live order {order_id} was not found."));
            };

            order.status = status;
            if let Some(persisted_order) = &persisted_order {
                order.historical_order_id = Some(persisted_order.order_id.clone());
            }
            order.clone()
        };

        let mut timeline_warning = None;
        if let Some(persisted_order) = &persisted_order {
            self.append_completed_order_to_history(persisted_order);
            // Timeline generation occurs only after the real order row has
            // been appended. A timeline failure is explanatory-data failure,
            // not an order-completion failure, and remains recoverable through
            // the authenticated rebuild action.
            let event = build_learning_event(
                &self.dishes.read().expect("dishes lock poisoned"),
                history_before_completion.as_deref().unwrap_or(&[]),
                persisted_order,
            );
            if let Err(error) = self.persist_learning_event(event) {
                timeline_warning = Some(error.clone());
                *self
                    .learning_timeline_warning
                    .write()
                    .expect("timeline warning lock poisoned") = Some(error);
            }
        }
        self.update_order_detail_record(&updated_order)
            .map_err(|error| {
                format!("Order status changed, but detail persistence failed: {error}")
            })?;
        self.bump_order_version();

        Ok(OrderStatusUpdate {
            order: updated_order,
            saved_to_csv: persisted_order.is_some(),
            historical_order_id: persisted_order.as_ref().map(|order| order.order_id.clone()),
            timeline_warning,
        })
    }

    fn update_order_detail_record(&self, order: &LiveOrder) -> Result<(), String> {
        let mut details = self
            .order_details
            .write()
            .expect("order details lock poisoned");
        if let Some(detail) = details
            .iter_mut()
            .find(|detail| detail.web_order_id.eq_ignore_ascii_case(&order.order_id))
        {
            detail.status = order.status.label().to_string();
            detail.historical_order_id = order.historical_order_id.clone().unwrap_or_default();
        }

        if !self.order_details_path.as_os_str().is_empty() {
            rewrite_order_details(&details, &self.order_details_path.to_string_lossy())
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    /// Adds a completed checkout order to the in-memory historical order log.
    ///
    /// This makes the Admin "Historical Orders" table update immediately when
    /// staff mark an order Completed. The append is guarded by `order_id` so
    /// repeated status updates do not duplicate the same order log.
    fn append_completed_order_to_history(&self, completed_order: &Order) {
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

        historical_orders.push(completed_order.clone());
    }

    fn persist_learning_event(&self, event: RecommendationLearningEvent) -> Result<(), String> {
        let mut events = self
            .learning_events
            .write()
            .expect("learning events lock poisoned");
        if events
            .iter()
            .any(|item| item.historical_order_id == event.historical_order_id)
        {
            return Ok(());
        }
        if !self.learning_events_path.as_os_str().is_empty() {
            append_learning_event(&event, &self.learning_events_path.to_string_lossy())
                .map_err(|error| format!("Order completed, but timeline append failed: {error}"))?;
        }
        events.push(event);
        Ok(())
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

    /// Returns all live/session orders for the customer Orders page.
    #[allow(dead_code)]
    pub fn customer_orders(&self) -> Vec<LiveOrder> {
        let mut orders = self
            .live_orders
            .read()
            .expect("live orders lock poisoned")
            .clone();
        orders.reverse();
        orders
    }

    pub fn customer_orders_by_phone(&self, phone: &str) -> Result<Vec<LiveOrder>, String> {
        let phone = normalize_phone(phone)?;
        let mut orders = self
            .live_orders
            .read()
            .expect("live orders lock poisoned")
            .iter()
            .filter(|order| order.customer_phone == phone)
            .cloned()
            .collect::<Vec<_>>();
        orders.reverse();
        Ok(orders)
    }

    pub fn customer_order_sync_by_phone(&self, phone: &str) -> Result<OrderSyncResponse, String> {
        Ok(OrderSyncResponse {
            version: self.order_version(),
            updated_at: human_timestamp_label(),
            orders: self.customer_orders_by_phone(phone)?,
        })
    }

    pub fn admin_order_sync(&self) -> OrderSyncResponse {
        OrderSyncResponse {
            version: self.order_version(),
            updated_at: human_timestamp_label(),
            orders: self
                .live_orders
                .read()
                .expect("live orders lock poisoned")
                .clone(),
        }
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
        if request.category.trim().is_empty() {
            return Err("Dish category is required.".to_string());
        }
        if split_csv_field(&request.ingredients).is_empty() {
            return Err("At least one ingredient is required.".to_string());
        }
        // Validate before mutating the dish list so an invalid price cannot
        // leave a partially updated in-memory record.
        let parsed_price = request.parsed_price()?;

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
        if let Some(price) = parsed_price {
            self.dish_price_overrides
                .write()
                .expect("dish price lock poisoned")
                .insert(dish_id.clone(), price);
        }
        self.set_dish_availability(&dish_id, available)?;
        self.dish_view_by_id(&dish_id)
            .ok_or_else(|| "Dish was saved but could not be loaded.".to_string())
    }

    /// Returns one dish for the admin edit form.
    pub fn admin_dish_by_id(&self, dish_id: &str) -> Option<DishView> {
        self.dish_view_by_id(dish_id)
    }

    /// Deletes a dish from in-memory management state.
    ///
    /// Historical order IDs are behavioural evidence used by collaborative
    /// filtering. Removing their dish metadata would make admin reports and
    /// experiments ambiguous, so referenced dishes must be made unavailable.
    pub fn delete_dish(&self, dish_id: &str) -> Result<(), String> {
        let dish_id = dish_id.trim().to_uppercase();
        if self
            .combined_orders()
            .iter()
            .any(|order| order.ordered_dishes.iter().any(|id| id == &dish_id))
        {
            return Err(
                "This dish exists in historical orders. Mark it unavailable instead.".to_string(),
            );
        }

        let mut dishes = self.dishes.write().expect("dishes lock poisoned");
        let before = dishes.len();
        dishes.retain(|dish| !dish.dish_id.eq_ignore_ascii_case(&dish_id));
        self.unavailable_dish_ids
            .write()
            .expect("availability lock poisoned")
            .remove(&dish_id);
        self.dish_price_overrides
            .write()
            .expect("dish price lock poisoned")
            .remove(&dish_id);

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
            .map(|dish| {
                DishView::from_dish(
                    dish,
                    !unavailable.contains(&dish.dish_id),
                    false,
                    self.price_override_for(&dish.dish_id),
                )
            })
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
            dish: DishView::from_dish(
                &result.dish,
                true,
                true,
                self.price_override_for(&result.dish.dish_id),
            ),
            content_score: result.ingredient_score,
            co_order_score: result.co_order_score,
            popularity_score: result.popularity_score,
            business_rule_score: result.business_rule_score,
            hybrid_score: result.final_score,
            base_score: result.base_score,
            reranked_score: result.reranked_score,
            base_rank: result.base_rank,
            reranked_rank: result.reranked_rank,
            novelty_score: result.novelty_score,
            max_similarity: result.max_similarity,
            category_bonus: result.category_bonus,
            diversity_notes: result.diversity_notes.clone(),
            adaptive_weights: result.adaptive_weights,
            evidence: result.evidence.clone(),
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

    fn cart_upsells(&self, selected_dish_ids: &[String]) -> Vec<String> {
        if selected_dish_ids.is_empty() {
            return Vec::new();
        }

        let context = self.dish_labels_for_ids(selected_dish_ids).join(", ");
        let response = self.recommend(RecommendationRequest {
            selected_dish_ids: selected_dish_ids.to_vec(),
            ranking_method: Some("co-ordering".to_string()),
            ..RecommendationRequest::default()
        });

        response
            .recommendations
            .iter()
            .filter(|recommendation| recommendation.co_order_score > 0.0)
            .take(3)
            .map(|recommendation| {
                format!(
                    "{} is often ordered together with {}.",
                    recommendation.dish.name, context
                )
            })
            .collect()
    }

    fn low_exposure_dishes(&self, limit: usize) -> Vec<DishView> {
        let counts = self
            .combined_orders()
            .into_iter()
            .flat_map(|order| order.ordered_dishes)
            .fold(HashMap::<String, usize>::new(), |mut counts, dish_id| {
                *counts.entry(dish_id).or_default() += 1;
                counts
            });

        let mut dishes = self.all_dish_views();
        dishes.sort_by_key(|dish| counts.get(&dish.dish_id).copied().unwrap_or(0));
        dishes.into_iter().take(limit).collect()
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
        let labels = self.dish_label_lookup();
        for order in self.combined_orders() {
            let mut ids = order.ordered_dishes;
            ids.sort();
            ids.dedup();
            for i in 0..ids.len() {
                for j in (i + 1)..ids.len() {
                    let pair_label = format!(
                        "{} + {}",
                        labels
                            .get(&ids[i])
                            .cloned()
                            .unwrap_or_else(|| ids[i].clone()),
                        labels
                            .get(&ids[j])
                            .cloned()
                            .unwrap_or_else(|| ids[j].clone())
                    );
                    *counts.entry(pair_label).or_default() += 1;
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

    fn dish_label_lookup(&self) -> HashMap<String, String> {
        self.dishes
            .read()
            .expect("dishes lock poisoned")
            .iter()
            .map(|dish| {
                (
                    dish.dish_id.clone(),
                    format!("{} ({})", dish.name, dish.dish_id),
                )
            })
            .collect()
    }

    fn bump_order_version(&self) {
        *self
            .order_version
            .write()
            .expect("order version lock poisoned") += 1;
    }

    fn order_version(&self) -> u64 {
        *self
            .order_version
            .read()
            .expect("order version lock poisoned")
    }

    fn price_override_for(&self, dish_id: &str) -> Option<u32> {
        self.dish_price_overrides
            .read()
            .expect("dish price lock poisoned")
            .get(dish_id)
            .copied()
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
    pub search_vocabulary_json: String,
    #[allow(dead_code)]
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
    fn from_dish(
        dish: &Dish,
        available: bool,
        recommended: bool,
        price_override: Option<u32>,
    ) -> Self {
        let price_amount =
            price_override.unwrap_or_else(|| placeholder_price_amount(&dish.dish_id));
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
    pub base_score: f32,
    pub reranked_score: f32,
    pub base_rank: usize,
    pub reranked_rank: usize,
    pub novelty_score: f32,
    pub max_similarity: f32,
    pub category_bonus: f32,
    pub diversity_notes: Vec<String>,
    pub adaptive_weights: AdaptiveWeights,
    pub evidence: RecommendationEvidence,
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
    pub eligible_dishes: usize,
    pub matched_preferences: usize,
    pub excluded_due_to_disliked: usize,
    pub skipped_selected_dishes: usize,
    pub recommended_shown: usize,
    pub diversity_count_top_5: usize,
}

impl RecommendationStatsView {
    fn from_output(output: &RecommendationOutput) -> Self {
        Self {
            filtered_dishes: output.stats.filtered_dishes,
            eligible_dishes: output.stats.filtered_dishes,
            matched_preferences: output.stats.matched_preference_dishes,
            excluded_due_to_disliked: output.stats.excluded_due_to_disliked,
            skipped_selected_dishes: output.stats.skipped_selected_dishes,
            recommended_shown: output.recommendations.len().min(10),
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
    #[serde(default)]
    pub diversity_mode: Option<String>,
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

/// Customer request for a bounded budget-aware meal-set search.
#[derive(Debug, Clone, Deserialize)]
pub struct MealSetRequest {
    pub budget_cents: u32,
    pub party_size: usize,
    #[serde(default)]
    pub target_dish_count: Option<usize>,
    #[serde(default)]
    pub top_set_count: Option<usize>,
    #[serde(default)]
    pub liked_ingredients: Vec<String>,
    #[serde(default)]
    pub disliked_ingredients: Vec<String>,
    #[serde(default)]
    pub preferred_tags: Vec<String>,
    #[serde(default)]
    pub required_categories: Vec<String>,
    #[serde(default)]
    pub selected_dish_ids: Vec<String>,
    #[serde(default)]
    pub time_context: Option<String>,
    #[serde(default)]
    pub diversity_mode: Option<String>,
}

/// Admin-only, side-effect-free baseline/change comparison request.
#[derive(Debug, Clone, Deserialize)]
pub struct CounterfactualRequest {
    pub baseline: RecommendationRequest,
    #[serde(default)]
    pub changes: CounterfactualChanges,
    pub top_k: usize,
}

/// Admin timeline response. Events deliberately contain no customer identity.
#[derive(Debug, Clone, Serialize)]
pub struct LearningTimelineResponse {
    pub event_count: usize,
    pub events: Vec<RecommendationLearningEvent>,
    pub warning: Option<String>,
}

/// Result of deleting explanatory timeline records.
#[derive(Debug, Clone, Serialize)]
pub struct LearningTimelineClearResponse {
    pub removed_event_count: usize,
}

/// JSON response from `/api/recommendations`.
#[derive(Debug, Clone, Serialize)]
pub struct RecommendationApiResponse {
    pub recommendations: Vec<RecommendationView>,
    pub stats: RecommendationStatsView,
    pub evidence_profile: RecommendationEvidenceProfile,
    pub adaptive_weights: AdaptiveWeights,
    pub scoring_config: AdaptiveScoringConfig,
    pub diversity_mode: DiversityMode,
    pub diversity_metrics: DiversityMetrics,
}

/// JSON response from `/api/search`.
///
/// It carries the same ranked result set used for both the suggestion dropdown
/// and locator behavior. Search results never replace, hide, or reorder the
/// server-rendered static customer Menu cards.
#[derive(Debug, Clone, Serialize)]
pub struct MenuSearchResponse {
    pub query: String,
    pub mode: String,
    pub result_count: usize,
    pub results: Vec<MenuSearchItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MenuSearchItem {
    pub dish: DishView,
    pub match_score: u32,
    pub match_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderSyncResponse {
    pub version: u64,
    pub updated_at: String,
    pub orders: Vec<LiveOrder>,
}

/// JSON request accepted by the Smart Menu Assistant endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AssistantRequest {
    pub prompt: String,
    #[serde(default)]
    pub selected_dish_ids: Vec<String>,
}

/// JSON response returned by the Smart Menu Assistant endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct AssistantResponse {
    pub understood: String,
    pub parsed: ParsedPreference,
    pub recommendations: Vec<RecommendationView>,
    pub upsells: Vec<String>,
    pub stats: RecommendationStatsView,
    pub evidence_profile: RecommendationEvidenceProfile,
    pub adaptive_weights: AdaptiveWeights,
    pub scoring_config: AdaptiveScoringConfig,
    pub diversity_mode: DiversityMode,
    pub diversity_metrics: DiversityMetrics,
}

/// Lightweight admin insight response calculated from order history.
#[derive(Debug, Clone, Serialize)]
pub struct AdminInsightResponse {
    pub summary: String,
    pub popular: Vec<String>,
    pub co_order_pairs: Vec<String>,
    pub low_exposure: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[allow(dead_code)]
pub struct EvaluationRequest {
    #[serde(default)]
    pub dataset_size: Option<String>,
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
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct EvaluationResponse {
    pub dataset_size: usize,
    pub dish_count: usize,
    pub unique_co_order_pairs: usize,
    pub average_dishes_per_order: f32,
    pub response_time_ms: u128,
    pub methods: Vec<EvaluationMethodOutput>,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct EvaluationMethodOutput {
    pub method: String,
    pub recommendations: Vec<EvaluationRecommendation>,
    pub diversity: f32,
    pub coverage: f32,
    pub novelty_proxy: f32,
}

impl EvaluationMethodOutput {
    #[allow(dead_code)]
    fn new(method: &str, recommendations: Vec<EvaluationRecommendation>, dishes: &[Dish]) -> Self {
        let top_n = recommendations.len().max(1) as f32;
        let recommended_ids = recommendations
            .iter()
            .map(|item| item.dish_id.clone())
            .collect::<HashSet<_>>();
        let categories = dishes
            .iter()
            .filter(|dish| recommended_ids.contains(&dish.dish_id))
            .map(|dish| dish.category.clone())
            .collect::<HashSet<_>>();
        let coverage = if dishes.is_empty() {
            0.0
        } else {
            (recommended_ids.len() as f32 / dishes.len() as f32) * 100.0
        };
        let novelty_proxy = recommendations
            .iter()
            .filter(|item| item.reason.to_lowercase().contains("popular"))
            .count();

        Self {
            method: method.to_string(),
            recommendations,
            diversity: categories.len() as f32 / top_n,
            coverage,
            novelty_proxy: 1.0 - (novelty_proxy as f32 / top_n),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct EvaluationRecommendation {
    pub dish_id: String,
    pub dish_name: String,
    pub score: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SimulationRequest {
    #[serde(default)]
    pub order_count: usize,
    #[serde(default)]
    pub min_dishes: usize,
    #[serde(default)]
    pub max_dishes: usize,
    #[serde(default)]
    pub seed: u64,
    #[serde(default)]
    pub popularity_skew: String,
    #[serde(default)]
    pub forced_dish_a: Option<String>,
    #[serde(default)]
    pub forced_dish_b: Option<String>,
    #[serde(default)]
    pub pair_probability: u8,
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
}

#[derive(Debug, Clone, Serialize)]
pub struct SimulationResponse {
    pub generated_order_count: usize,
    pub preview: Vec<SimulationOrderPreview>,
    pub changed_pairs: Vec<SimulationPairChange>,
    pub rank_changes: Vec<SimulationRankChange>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SimulationOrderPreview {
    pub order_id: String,
    pub dish_ids: Vec<String>,
    pub dish_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SimulationPairChange {
    pub label: String,
    pub before_count: usize,
    pub after_count: usize,
    pub support_before: f32,
    pub support_after: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SimulationRankChange {
    pub dish_id: String,
    pub dish_name: String,
    pub before_rank: Option<usize>,
    pub after_rank: Option<usize>,
    pub before_score: f32,
    pub after_score: f32,
    pub explanation: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ExperimentLabRequest {
    pub experiment_type: String,
    #[serde(default)]
    pub liked_ingredients: Vec<String>,
    #[serde(default)]
    pub disliked_ingredients: Vec<String>,
    #[serde(default)]
    pub preferred_tags: Vec<String>,
    #[serde(default)]
    pub selected_dish_ids: Vec<String>,
    #[serde(default)]
    pub anchor_dish_id: Option<String>,
    #[serde(default)]
    pub candidate_dish_id: Option<String>,
    #[serde(default)]
    pub hidden_dish_id: Option<String>,
    #[serde(default)]
    pub historical_order_id: Option<String>,
    #[serde(default)]
    pub top_k: Option<usize>,
    #[serde(default)]
    pub additional_coorders: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExperimentLabResponse {
    pub experiment_type: String,
    pub conclusion: String,
    pub rows: Vec<ExperimentRow>,
    pub csv: String,
    pub simulation: Option<SimulationResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExperimentRow {
    pub method: String,
    pub rank: Option<usize>,
    pub dish_id: String,
    pub dish_name: String,
    pub ingredient_score: f32,
    pub co_order_score: f32,
    pub final_score: f32,
    pub matched: String,
    pub excluded: String,
    pub hidden_match: bool,
}

/// Temporary customer details collected before entering the customer menu.
///
/// This is deliberately smaller than a real account model. It supports the FYP
/// QR flow by tying cart checkout and profile order tracking to the current
/// dining visit without mixing customer identity into recommendation history.
#[derive(Debug, Clone, Serialize)]
pub struct CustomerSession {
    pub session_id: String,
    pub customer_name: String,
    pub customer_phone: String,
    pub table_number: String,
}

impl CustomerSession {
    pub fn masked_phone(&self) -> String {
        let suffix = masked_phone_suffix(&self.customer_phone);
        format!("***-***-{suffix}")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomerRegistrationRequest {
    pub customer_name: String,
    pub customer_phone: String,
    pub table_number: String,
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
    pub historical_order_id: Option<String>,
    pub customer_name: String,
    pub customer_phone: String,
    pub table_number: Option<String>,
    pub note: Option<String>,
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

    fn to_detail_record(&self) -> OrderDetailRecord {
        OrderDetailRecord {
            web_order_id: self.order_id.clone(),
            historical_order_id: self.historical_order_id.clone().unwrap_or_default(),
            customer_name: self.customer_name.clone(),
            customer_phone: self.customer_phone.clone(),
            table_number: self.table_number.clone().unwrap_or_default(),
            note: self.note.clone().unwrap_or_default(),
            dish_ids: self.ordered_dishes.join(","),
            total: self.total_price.clone(),
            status: self.status.label().to_string(),
            created_at: self.timestamp.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateLiveOrderRequest {
    pub dish_ids: Vec<String>,
    pub customer_name: String,
    pub customer_phone: String,
    #[serde(default)]
    pub table_number: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

/// Result returned after staff update a live order status.
///
/// `saved_to_csv` is true only when this status update appended a new historical
/// row to `data/orders.csv`.
#[derive(Debug, Clone, Serialize)]
pub struct OrderStatusUpdate {
    pub order: LiveOrder,
    pub saved_to_csv: bool,
    pub historical_order_id: Option<String>,
    pub timeline_warning: Option<String>,
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
    pub price: Option<String>,
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

    fn parsed_price(&self) -> Result<Option<u32>, String> {
        let Some(raw) = self.price.as_deref() else {
            return Ok(None);
        };
        let cleaned = raw
            .trim()
            .trim_start_matches("RM")
            .trim()
            .parse::<u32>()
            .map_err(|_| "Price must be a positive whole number, for example 16.".to_string())?;
        if cleaned == 0 {
            return Err("Price must be greater than zero.".to_string());
        }
        Ok(Some(cleaned))
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
        reasons.push("is shown as a deterministic low-evidence fallback".to_string());
    }

    let [content, co_order, popularity, time] = result.adaptive_weights.as_percentages();
    format!(
        "{} is recommended because it {}. Score {:.2}; evidence confidence {:.0}%. Adaptive weights: content {}%, co-order {}%, popularity {}%, time/context {}%.",
        result.dish.name,
        reasons.join("; "),
        result.final_score,
        result.evidence.overall_confidence * 100.0,
        content,
        co_order,
        popularity,
        time
    )
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

fn validate_dish_exists(dishes: &[Dish], dish_id: &str) -> Result<(), String> {
    if dishes
        .iter()
        .any(|dish| dish.dish_id.eq_ignore_ascii_case(dish_id))
    {
        Ok(())
    } else {
        Err(format!(
            "Dish {dish_id} is not available for this experiment."
        ))
    }
}

fn pair_metric_or_default(orders: &[Order], anchor: &str, candidate: &str) -> AssociationMetric {
    calculate_association_metric(orders, anchor, candidate).unwrap_or_else(|| AssociationMetric {
        base_dish_id: anchor.to_string(),
        candidate_dish_id: candidate.to_string(),
        pair_count: 0,
        support: 0.0,
        confidence: 0.0,
        lift: 0.0,
    })
}

fn controlled_candidate_rank(
    dishes: &[Dish],
    orders: &[Order],
    preference: &UserPreference,
    candidate: &str,
    ingredient_weight: f32,
    co_order_weight: f32,
) -> Option<(usize, f32)> {
    let rows = controlled_ranked_results(
        dishes,
        orders,
        preference,
        ingredient_weight,
        co_order_weight,
    );
    rows.iter()
        .position(|(item, _)| item.dish.dish_id.eq_ignore_ascii_case(candidate))
        .map(|index| (index + 1, rows[index].1))
}

fn controlled_ranked_results(
    dishes: &[Dish],
    orders: &[Order],
    preference: &UserPreference,
    ingredient_weight: f32,
    co_order_weight: f32,
) -> Vec<(RecommendationResult, f32)> {
    let mut rows = generate_recommendations(dishes, orders, preference)
        .recommendations
        .into_iter()
        .map(|item| {
            let final_score =
                ingredient_weight * item.ingredient_score + co_order_weight * item.co_order_score;
            (item, final_score.clamp(0.0, 1.0))
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b.1.total_cmp(&a.1)
            .then_with(|| a.0.dish.name.cmp(&b.0.dish.name))
    });
    rows
}

fn preference_match_rate(
    rows: &[(RecommendationResult, f32)],
    top_k: usize,
    preference: &UserPreference,
) -> f32 {
    if preference.liked_ingredients.is_empty() {
        return 0.0;
    }
    let top = rows.iter().take(top_k).collect::<Vec<_>>();
    if top.is_empty() {
        return 0.0;
    }
    let matching = top
        .iter()
        .filter(|(item, _)| !item.matched_liked_ingredients.is_empty())
        .count();
    matching as f32 / top.len() as f32
}

fn restriction_violations(rows: &[(RecommendationResult, f32)], top_k: usize) -> usize {
    rows.iter()
        .take(top_k)
        .filter(|(item, _)| !item.matched_disliked_ingredients.is_empty())
        .count()
}

fn human_timestamp_label() -> String {
    Local::now().format("%Y-%m-%d %H:%M").to_string()
}

fn live_orders_from_details(dishes: &[Dish], details: &[OrderDetailRecord]) -> Vec<LiveOrder> {
    let dish_lookup = dishes
        .iter()
        .map(|dish| (dish.dish_id.clone(), dish.name.clone()))
        .collect::<HashMap<_, _>>();

    details
        .iter()
        .map(|detail| {
            let ordered_dishes = detail
                .dish_ids
                .split(',')
                .map(|dish_id| dish_id.trim().to_uppercase())
                .filter(|dish_id| !dish_id.is_empty())
                .collect::<Vec<_>>();
            let dish_names = ordered_dishes
                .iter()
                .map(|dish_id| {
                    dish_lookup
                        .get(dish_id)
                        .cloned()
                        .unwrap_or_else(|| dish_id.clone())
                })
                .collect::<Vec<_>>();

            LiveOrder {
                order_id: detail.web_order_id.clone(),
                session_user_id: format!("PHONE-{}", masked_phone_suffix(&detail.customer_phone)),
                ordered_dishes,
                dish_names,
                timestamp: detail.created_at.clone(),
                total_price: detail.total.clone(),
                total_price_amount: numeric_total(&detail.total),
                status: OrderStatus::from_label(&detail.status).unwrap_or(OrderStatus::Pending),
                historical_order_id: (!detail.historical_order_id.trim().is_empty())
                    .then(|| detail.historical_order_id.clone()),
                customer_name: detail.customer_name.clone(),
                customer_phone: detail.customer_phone.clone(),
                table_number: (!detail.table_number.trim().is_empty())
                    .then(|| detail.table_number.clone()),
                note: (!detail.note.trim().is_empty()).then(|| detail.note.clone()),
            }
        })
        .collect()
}

fn numeric_total(total: &str) -> u32 {
    total
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or(0)
}

fn validate_customer_name(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.len() < 2 {
        return Err("Customer name is required.".to_string());
    }
    if value.len() > 60 {
        return Err("Customer name is too long.".to_string());
    }
    Ok(value.to_string())
}

/// Generates an opaque, cookie-safe identifier without exposing customer or
/// staff details. The timestamp and process-local sequence avoid collisions in
/// this lightweight in-memory prototype.
fn new_opaque_session_id(kind: &str) -> String {
    let sequence = SESSION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!(
        "{kind}-{:x}-{sequence:x}",
        Local::now().timestamp_nanos_opt().unwrap_or_default()
    )
}

#[allow(dead_code)]
pub fn normalize_customer_phone(value: &str) -> Result<String, String> {
    normalize_phone(value)
}

fn normalize_phone(value: &str) -> Result<String, String> {
    let raw = value.trim().replace([' ', '-'], "");
    let digits = if let Some(rest) = raw.strip_prefix("+60") {
        format!("60{rest}")
    } else if let Some(rest) = raw.strip_prefix('0') {
        format!("60{rest}")
    } else {
        raw
    };

    if digits.len() < 10 || digits.len() > 13 || !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err("Enter a valid Malaysian phone number.".to_string());
    }
    Ok(digits)
}

fn validate_table_number(value: &str) -> Result<String, String> {
    let value = value.trim().to_uppercase();
    if value.is_empty() {
        return Err("Table number is required for this dine-in prototype.".to_string());
    }
    if value.len() > 16 {
        return Err("Table number is too long.".to_string());
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-' || character == '_')
    {
        return Err("Table number can use letters, numbers, dash, or underscore only.".to_string());
    }
    Ok(value)
}

fn clean_optional_short(
    value: &Option<String>,
    max_len: usize,
    label: &str,
) -> Result<Option<String>, String> {
    let Some(value) = value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    if value.len() > max_len {
        return Err(format!("{label} is too long."));
    }
    Ok(Some(value.to_string()))
}

fn masked_phone_suffix(phone: &str) -> String {
    phone
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}

#[allow(dead_code)]
fn evaluation_method_results(
    dishes: &[Dish],
    orders: &[Order],
    preference: &UserPreference,
    _method: &str,
    configure: impl FnOnce(UserPreference) -> UserPreference,
) -> Vec<EvaluationRecommendation> {
    let output = generate_recommendations(dishes, orders, &configure(preference.clone()));
    output
        .recommendations
        .into_iter()
        .take(5)
        .map(|item| EvaluationRecommendation {
            dish_id: item.dish.dish_id,
            dish_name: item.dish.name,
            score: item.final_score,
            reason: item.explanation,
        })
        .collect()
}

#[allow(dead_code)]
fn count_unique_pairs(orders: &[Order]) -> usize {
    let mut pairs = HashSet::new();
    for order in orders {
        let mut ids = order.ordered_dishes.clone();
        ids.sort();
        ids.dedup();
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                pairs.insert((ids[i].clone(), ids[j].clone()));
            }
        }
    }
    pairs.len()
}

#[allow(dead_code)]
fn average_dishes_per_order(orders: &[Order]) -> f32 {
    if orders.is_empty() {
        0.0
    } else {
        orders
            .iter()
            .map(|order| order.ordered_dishes.len())
            .sum::<usize>() as f32
            / orders.len() as f32
    }
}

fn generate_simulated_orders(
    dishes: &[Dish],
    historical_orders: &[Order],
    request: &SimulationRequest,
) -> Vec<Order> {
    let available_ids = dishes
        .iter()
        .map(|dish| dish.dish_id.clone())
        .collect::<Vec<_>>();
    if available_ids.is_empty() {
        return Vec::new();
    }
    let popularity = dish_popularity_weight_lookup(historical_orders);
    let mut rng = DeterministicRng::new(if request.seed == 0 { 42 } else { request.seed });
    let order_count = request.order_count.clamp(1, 200);
    let min_dishes = request.min_dishes.clamp(1, 10);
    let max_dishes = request.max_dishes.clamp(min_dishes, 10);
    let pair_probability = request.pair_probability.min(100) as usize;
    let forced_a = request
        .forced_dish_a
        .as_deref()
        .map(str::trim)
        .map(str::to_uppercase)
        .filter(|dish_id| available_ids.contains(dish_id));
    let forced_b = request
        .forced_dish_b
        .as_deref()
        .map(str::trim)
        .map(str::to_uppercase)
        .filter(|dish_id| available_ids.contains(dish_id));

    (0..order_count)
        .map(|index| {
            let target_size = rng.range(min_dishes, max_dishes);
            let mut basket = Vec::new();
            if let (Some(a), Some(b)) = (&forced_a, &forced_b) {
                if a != b && rng.range(1, 100) <= pair_probability {
                    basket.push(a.clone());
                    basket.push(b.clone());
                }
            }
            while basket.len() < target_size {
                let candidate = choose_simulated_dish(
                    &available_ids,
                    &popularity,
                    &request.popularity_skew,
                    &mut rng,
                );
                if !basket.contains(&candidate) {
                    basket.push(candidate);
                }
                if basket.len() == available_ids.len() {
                    break;
                }
            }
            Order {
                order_id: format!("SIM{:03}", index + 1),
                session_user_id: "SIMULATION".to_string(),
                ordered_dishes: basket,
                timestamp: format!("simulation-seed-{}", request.seed),
            }
        })
        .collect()
}

fn choose_simulated_dish(
    dish_ids: &[String],
    popularity: &HashMap<String, usize>,
    skew: &str,
    rng: &mut DeterministicRng,
) -> String {
    if skew.eq_ignore_ascii_case("uniform") || popularity.is_empty() {
        return dish_ids[rng.range(0, dish_ids.len() - 1)].clone();
    }
    let multiplier = if skew.eq_ignore_ascii_case("strong") {
        3
    } else {
        1
    };
    let weights = dish_ids
        .iter()
        .map(|dish_id| 1 + popularity.get(dish_id).copied().unwrap_or(0) * multiplier)
        .collect::<Vec<_>>();
    let total = weights.iter().sum::<usize>().max(1);
    let mut pick = rng.range(1, total);
    for (dish_id, weight) in dish_ids.iter().zip(weights) {
        if pick <= weight {
            return dish_id.clone();
        }
        pick -= weight;
    }
    dish_ids[0].clone()
}

fn dish_popularity_weight_lookup(orders: &[Order]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for order in orders {
        for dish_id in &order.ordered_dishes {
            *counts.entry(dish_id.clone()).or_insert(0) += 1;
        }
    }
    counts
}

fn top_changed_pairs(
    dishes: &[Dish],
    before: &[Order],
    after: &[Order],
    limit: usize,
) -> Vec<SimulationPairChange> {
    let before_counts = pair_counts(before);
    let after_counts = pair_counts(after);
    let names = dishes
        .iter()
        .map(|dish| (dish.dish_id.clone(), dish.name.clone()))
        .collect::<HashMap<_, _>>();
    let mut changes = after_counts
        .iter()
        .filter_map(|(pair, after_count)| {
            let before_count = before_counts.get(pair).copied().unwrap_or(0);
            (*after_count > before_count).then(|| SimulationPairChange {
                label: format!(
                    "{} ({}) + {} ({})",
                    names.get(&pair.0).unwrap_or(&pair.0),
                    pair.0,
                    names.get(&pair.1).unwrap_or(&pair.1),
                    pair.1
                ),
                before_count,
                after_count: *after_count,
                support_before: support_for_count(before_count, before.len()),
                support_after: support_for_count(*after_count, after.len()),
            })
        })
        .collect::<Vec<_>>();
    changes.sort_by(|a, b| {
        let a_delta = a.after_count.saturating_sub(a.before_count);
        let b_delta = b.after_count.saturating_sub(b.before_count);
        b_delta.cmp(&a_delta).then_with(|| a.label.cmp(&b.label))
    });
    changes.into_iter().take(limit).collect()
}

fn pair_counts(orders: &[Order]) -> HashMap<(String, String), usize> {
    let mut counts = HashMap::new();
    for order in orders {
        let mut ids = order.ordered_dishes.clone();
        ids.sort();
        ids.dedup();
        for left in 0..ids.len() {
            for right in (left + 1)..ids.len() {
                *counts
                    .entry((ids[left].clone(), ids[right].clone()))
                    .or_insert(0) += 1;
            }
        }
    }
    counts
}

fn support_for_count(count: usize, order_count: usize) -> f32 {
    if order_count == 0 {
        0.0
    } else {
        count as f32 / order_count as f32
    }
}

fn simulation_rank_reason(
    dish_id: &str,
    before: Option<(usize, f32)>,
    after: Option<(usize, f32)>,
) -> String {
    match (before, after) {
        (Some((before_rank, _)), Some((after_rank, _))) if after_rank < before_rank => format!(
            "{dish_id} moved up because simulated co-order baskets increased its hybrid evidence."
        ),
        (None, Some((after_rank, _))) => {
            format!("{dish_id} newly entered the top results at rank {after_rank}.")
        }
        _ => format!("{dish_id} remained stable under the simulated dataset."),
    }
}

struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }

    fn range(&mut self, min: usize, max: usize) -> usize {
        if max <= min {
            return min;
        }
        min + (self.next() as usize % (max - min + 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_loader::load_orders;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_order_csv_path(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!("fyp_web_state_{label}_{unique}.csv"))
    }

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
    fn registered_customer_can_checkout_from_session() {
        let dishes = vec![dish(
            "D01",
            "Chicken Rice",
            "main",
            &["chicken"],
            &["signature"],
        )];
        let state = WebState::new(dishes, Vec::new());
        let session = state
            .register_customer_session(CustomerRegistrationRequest {
                customer_name: "Preston".to_string(),
                customer_phone: "0123456789".to_string(),
                table_number: "T05".to_string(),
            })
            .expect("valid registration should create a session");

        let order = state
            .create_live_order_from_session(
                &session.session_id,
                vec!["D01".to_string()],
                Some("Less spicy".to_string()),
            )
            .expect("registered customer should check out");

        assert_eq!(order.customer_name, "Preston");
        assert_eq!(order.table_number.as_deref(), Some("T05"));
        assert_eq!(order.note.as_deref(), Some("Less spicy"));
    }

    #[test]
    fn unregistered_customer_cannot_checkout_from_session() {
        let dishes = vec![dish(
            "D01",
            "Chicken Rice",
            "main",
            &["chicken"],
            &["signature"],
        )];
        let state = WebState::new(dishes, Vec::new());

        let error = state
            .create_live_order_from_session("missing", vec!["D01".to_string()], None)
            .expect_err("missing session should fail");

        assert!(error.contains("session expired"));
        assert_eq!(state.live_order_count(), 0);
    }

    #[test]
    fn registration_rejects_invalid_phone_and_missing_table() {
        let state = WebState::new(Vec::new(), Vec::new());

        let invalid_phone = state.register_customer_session(CustomerRegistrationRequest {
            customer_name: "Ali".to_string(),
            customer_phone: "abc".to_string(),
            table_number: "T01".to_string(),
        });
        assert!(invalid_phone.is_err());

        let missing_table = state.register_customer_session(CustomerRegistrationRequest {
            customer_name: "Ali".to_string(),
            customer_phone: "0123456789".to_string(),
            table_number: " ".to_string(),
        });
        assert!(missing_table.is_err());
    }

    #[test]
    fn customer_order_lookup_is_scoped_by_phone() {
        let dishes = vec![dish(
            "D01",
            "Chicken Rice",
            "main",
            &["chicken"],
            &["signature"],
        )];
        let state = WebState::new(dishes, Vec::new());
        let first = state
            .register_customer_session(CustomerRegistrationRequest {
                customer_name: "Ali".to_string(),
                customer_phone: "0123456789".to_string(),
                table_number: "T01".to_string(),
            })
            .unwrap();
        let second = state
            .register_customer_session(CustomerRegistrationRequest {
                customer_name: "Mei".to_string(),
                customer_phone: "0199999999".to_string(),
                table_number: "T02".to_string(),
            })
            .unwrap();
        state
            .create_live_order_from_session(&first.session_id, vec!["D01".to_string()], None)
            .unwrap();

        assert_eq!(
            state
                .customer_orders_by_phone(&first.customer_phone)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            state
                .customer_orders_by_phone(&second.customer_phone)
                .unwrap()
                .len(),
            0
        );
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
    fn menu_search_filters_available_dishes_with_concepts() {
        let dishes = vec![
            dish(
                "D04",
                "Laksa",
                "main",
                &["rice noodles", "chili"],
                &["spicy"],
            ),
            dish(
                "D14",
                "Mee Goreng Mamak",
                "main",
                &["yellow noodles"],
                &["fried"],
            ),
            dish("D20", "Pisang Goreng", "dessert", &["banana"], &["sweet"]),
        ];
        let state = WebState::new(dishes, Vec::new());

        let response = state.search_menu("mee, spicy", MatchMode::All);

        assert_eq!(response.result_count, 1);
        assert_eq!(response.results[0].dish.dish_id, "D04");
        assert!(
            response.results[0]
                .match_reasons
                .iter()
                .any(|reason| reason.contains("concept") || reason.contains("interpreted"))
        );
    }

    #[test]
    fn menu_search_excludes_unavailable_dishes() {
        let dishes = vec![
            dish(
                "D04",
                "Laksa",
                "main",
                &["rice noodles", "chili"],
                &["spicy"],
            ),
            dish(
                "D14",
                "Mee Goreng Mamak",
                "main",
                &["yellow noodles"],
                &["fried"],
            ),
        ];
        let state = WebState::new(dishes, Vec::new());
        state.set_dish_availability("D04", false).unwrap();

        let response = state.search_menu("laksa", MatchMode::Any);

        assert!(response.results.is_empty());
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
    fn recommendation_api_exposes_adaptive_profile_weights_and_candidate_evidence() {
        let dishes = vec![
            dish("D01", "Nasi Lemak", "main", &["rice"], &["signature"]),
            dish("D02", "Chicken Satay", "main", &["chicken"], &["grilled"]),
        ];
        let orders = vec![Order {
            order_id: "O001".to_string(),
            session_user_id: "U01".to_string(),
            ordered_dishes: vec!["D01".to_string(), "D02".to_string()],
            timestamp: "2026-01-01 12:30".to_string(),
        }];
        let state = WebState::new(dishes, orders);

        let response = state.recommend(RecommendationRequest {
            liked_ingredients: vec!["chicken".to_string()],
            selected_dish_ids: vec!["D01".to_string()],
            ..RecommendationRequest::default()
        });

        assert_eq!(response.evidence_profile.total_order_count, 1);
        assert!(response.adaptive_weights.validate());
        assert_eq!(response.recommendations[0].evidence.candidate_pair_count, 1);
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"adaptive_weights\""));
        assert!(json.contains("\"evidence\""));
        assert!(json.contains("\"confidence_level\""));
    }

    #[test]
    fn simulation_is_reproducible_and_does_not_mutate_history() {
        let dishes = vec![
            dish("D01", "Nasi Lemak", "main", &["rice"], &["signature"]),
            dish("D02", "Chicken Satay", "main", &["chicken"], &["grilled"]),
            dish("D03", "Kuih", "dessert", &["coconut"], &["sweet"]),
        ];
        let orders = vec![Order {
            order_id: "O001".to_string(),
            session_user_id: "U01".to_string(),
            ordered_dishes: vec!["D01".to_string(), "D02".to_string()],
            timestamp: "2026-01-01 12:30".to_string(),
        }];
        let state = WebState::new(dishes, orders);
        let request = SimulationRequest {
            order_count: 5,
            min_dishes: 2,
            max_dishes: 3,
            seed: 77,
            popularity_skew: "uniform".to_string(),
            forced_dish_a: Some("D01".to_string()),
            forced_dish_b: Some("D02".to_string()),
            pair_probability: 100,
            selected_dish_ids: vec!["D01".to_string()],
            ..SimulationRequest::default()
        };

        let first = state.simulation_report(request.clone());
        let second = state.simulation_report(request);

        assert_eq!(first.generated_order_count, 5);
        assert_eq!(first.preview[0].dish_ids, second.preview[0].dish_ids);
        assert_eq!(state.combined_orders().len(), 1);
        assert!(
            first
                .changed_pairs
                .iter()
                .any(|pair| pair.label.contains("D01"))
        );
    }

    #[test]
    fn coorder_impact_zero_orders_keeps_before_and_after_equal() {
        let dishes = vec![
            dish("D01", "Nasi Lemak", "main", &["rice"], &["signature"]),
            dish("D02", "Chicken Satay", "main", &["chicken"], &["grilled"]),
        ];
        let orders = vec![Order {
            order_id: "O001".to_string(),
            session_user_id: "U01".to_string(),
            ordered_dishes: vec!["D01".to_string(), "D02".to_string()],
            timestamp: "2026-01-01 12:30".to_string(),
        }];
        let state = WebState::new(dishes, orders);

        let response = state
            .experiment_lab(ExperimentLabRequest {
                experiment_type: "coorder".to_string(),
                anchor_dish_id: Some("D01".to_string()),
                candidate_dish_id: Some("D02".to_string()),
                additional_coorders: Some(0),
                ..ExperimentLabRequest::default()
            })
            .expect("valid pair should run");

        assert_eq!(response.rows.len(), 2);
        assert_eq!(
            response.rows[0].co_order_score,
            response.rows[1].co_order_score
        );
        assert!(response.rows[0].matched.contains("Pair count: 1"));
        assert!(response.rows[1].matched.contains("Pair count: 1"));
        assert_eq!(state.combined_orders().len(), 1);
    }

    #[test]
    fn coorder_impact_added_orders_increase_pair_count_without_mutating_history() {
        let dishes = vec![
            dish("D01", "Nasi Lemak", "main", &["rice"], &["signature"]),
            dish("D02", "Chicken Satay", "main", &["chicken"], &["grilled"]),
        ];
        let orders = vec![Order {
            order_id: "O001".to_string(),
            session_user_id: "U01".to_string(),
            ordered_dishes: vec!["D01".to_string(), "D02".to_string()],
            timestamp: "2026-01-01 12:30".to_string(),
        }];
        let state = WebState::new(dishes, orders);

        let response = state
            .experiment_lab(ExperimentLabRequest {
                experiment_type: "coorder".to_string(),
                anchor_dish_id: Some("D01".to_string()),
                candidate_dish_id: Some("D02".to_string()),
                additional_coorders: Some(10),
                ..ExperimentLabRequest::default()
            })
            .expect("valid pair should run");

        assert!(response.rows[1].matched.contains("Pair count: 11"));
        assert_eq!(state.combined_orders().len(), 1);
    }

    #[test]
    fn coorder_impact_rejects_same_anchor_and_candidate() {
        let dishes = vec![dish("D01", "Nasi Lemak", "main", &["rice"], &["signature"])];
        let state = WebState::new(dishes, Vec::new());

        let error = state
            .experiment_lab(ExperimentLabRequest {
                experiment_type: "coorder".to_string(),
                anchor_dish_id: Some("D01".to_string()),
                candidate_dish_id: Some("D01".to_string()),
                ..ExperimentLabRequest::default()
            })
            .expect_err("same pair should be invalid");

        assert!(error.contains("must be different"));
    }

    #[test]
    fn method_comparison_removes_hidden_target_from_context() {
        let dishes = vec![
            dish("D01", "Nasi Lemak", "main", &["rice"], &["signature"]),
            dish("D02", "Chicken Satay", "main", &["chicken"], &["grilled"]),
            dish(
                "D03",
                "Laksa",
                "main",
                &["rice noodles", "chili"],
                &["spicy"],
            ),
            dish("D04", "Kuih", "dessert", &["coconut"], &["sweet"]),
        ];
        let orders = vec![Order {
            order_id: "O001".to_string(),
            session_user_id: "U01".to_string(),
            ordered_dishes: vec!["D01".to_string(), "D02".to_string(), "D03".to_string()],
            timestamp: "2026-01-01 12:30".to_string(),
        }];
        let state = WebState::new(dishes, orders);

        let response = state
            .experiment_lab(ExperimentLabRequest {
                experiment_type: "method".to_string(),
                historical_order_id: Some("O001".to_string()),
                hidden_dish_id: Some("D03".to_string()),
                liked_ingredients: vec!["chili".to_string()],
                top_k: Some(3),
                ..ExperimentLabRequest::default()
            })
            .expect("valid historical order should run");

        assert!(
            response
                .rows
                .iter()
                .any(|row| row.method == "Ingredient-only")
        );
        assert!(
            response
                .rows
                .iter()
                .any(|row| row.method == "Co-order-only")
        );
        assert!(
            response
                .rows
                .iter()
                .any(|row| row.method == "Hybrid 0.4/0.6")
        );
        assert!(response.rows.iter().all(|row| row.dish_id != "D01"));
        assert!(response.rows.iter().all(|row| row.dish_id != "D02"));
        assert!(response.rows.iter().any(|row| row.dish_id == "D03"));
        for row in &response.rows {
            let expected = match row.method.as_str() {
                "Ingredient-only" => row.ingredient_score,
                "Co-order-only" => row.co_order_score,
                "Hybrid 0.4/0.6" => 0.4 * row.ingredient_score + 0.6 * row.co_order_score,
                method => panic!("unexpected method {method}"),
            };
            assert!((row.final_score - expected).abs() < 0.0001);
        }
    }

    #[test]
    fn admin_co_order_pairs_include_dish_names() {
        let dishes = vec![
            dish("D01", "Nasi Lemak", "main", &["rice"], &["signature"]),
            dish("D02", "Chicken Satay", "main", &["chicken"], &["grilled"]),
        ];
        let orders = vec![Order {
            order_id: "O001".to_string(),
            session_user_id: "U01".to_string(),
            ordered_dishes: vec!["D01".to_string(), "D02".to_string()],
            timestamp: "2026-01-01 12:30".to_string(),
        }];
        let state = WebState::new(dishes, orders);

        let pairs = state.admin_view().co_order_pairs;

        assert_eq!(pairs[0].label, "Nasi Lemak (D01) + Chicken Satay (D02)");
        assert_eq!(pairs[0].count, 1);
    }

    #[test]
    fn completed_order_updates_collaborative_recommendations_immediately() {
        let dishes = vec![
            dish("D01", "Nasi Lemak", "main", &["rice"], &["signature"]),
            dish("D02", "Chicken Satay", "main", &["chicken"], &["grilled"]),
        ];
        let csv_path = temp_order_csv_path("completed_recommendation");
        let state = WebState::new_with_order_csv_path(dishes, Vec::new(), csv_path.clone());
        let request = RecommendationRequest {
            selected_dish_ids: vec!["D01".to_string()],
            ranking_method: Some("co-ordering".to_string()),
            ..RecommendationRequest::default()
        };
        let before = state.recommend(request.clone());
        let order = state
            .create_live_order(&["D01".to_string(), "D02".to_string()])
            .unwrap();

        let update = state
            .update_order_status(&order.order_id, OrderStatus::Completed)
            .unwrap();
        let response = state.recommend(request);
        let timeline = state.learning_timeline();
        let _ = fs::remove_file(&csv_path);

        assert!(update.timeline_warning.is_none());
        assert_eq!(timeline.event_count, 1);
        assert_eq!(timeline.events[0].historical_order_id, "O001");
        assert_eq!(response.recommendations[0].dish.dish_id, "D02");
        assert!(response.recommendations[0].co_order_score > 0.0);
        assert!(
            response.evidence_profile.collaborative_confidence
                > before.evidence_profile.collaborative_confidence
        );
        assert!(response.adaptive_weights.co_order > before.adaptive_weights.co_order);
    }

    #[test]
    fn assistant_recommendations_exclude_negated_menu_terms() {
        let dishes = vec![
            dish("D01", "Beef Satay", "main", &["beef"], &["grilled"]),
            dish(
                "D02",
                "Chicken Rice",
                "main",
                &["chicken", "rice"],
                &["signature"],
            ),
        ];
        let state = WebState::new(dishes, Vec::new());

        let response = state.assistant_recommend(AssistantRequest {
            prompt: "I want chicken but no beef".to_string(),
            selected_dish_ids: Vec::new(),
        });

        assert!(
            response
                .parsed
                .disliked_ingredients
                .contains(&"beef".to_string())
        );
        assert!(
            response
                .recommendations
                .iter()
                .all(|recommendation| recommendation.dish.dish_id != "D01")
        );
        assert!(response.adaptive_weights.validate());
        assert!(
            response
                .recommendations
                .iter()
                .all(|recommendation| recommendation.evidence.requested_preference_count > 0)
        );
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
        let csv_path = temp_order_csv_path("completed_order");
        let state = WebState::new_with_order_csv_path(dishes, Vec::new(), csv_path.clone());
        let order = state
            .create_live_order(&["D01".to_string(), "D02".to_string()])
            .unwrap();

        let update = state
            .update_order_status(&order.order_id, OrderStatus::Completed)
            .unwrap();
        let duplicate_update = state
            .update_order_status(&order.order_id, OrderStatus::Completed)
            .unwrap();

        let admin = state.admin_view();
        let reloaded_orders =
            load_orders(csv_path.to_str().unwrap()).expect("completed order should reload");
        let raw = fs::read_to_string(&csv_path).expect("CSV should be readable");
        let _ = fs::remove_file(&csv_path);

        assert!(update.saved_to_csv);
        assert_eq!(update.historical_order_id.as_deref(), Some("O001"));
        assert!(!duplicate_update.saved_to_csv);
        assert_eq!(state.learning_timeline().event_count, 1);
        assert!(admin.live_orders.is_empty());
        assert_eq!(admin.completed_session_orders.len(), 1);
        assert_eq!(
            admin.completed_session_orders[0].dish_names,
            vec!["Nasi Lemak", "Satay"]
        );
        assert_eq!(admin.historical_orders[0].order_id, "O001");
        assert_eq!(admin.historical_orders[0].session_user_id, "U1");
        assert_eq!(reloaded_orders.len(), 1);
        assert_eq!(reloaded_orders[0].order_id, "O001");
        assert_eq!(reloaded_orders[0].session_user_id, "U1");
        assert_eq!(reloaded_orders[0].ordered_dishes, vec!["D01", "D02"]);
        assert_eq!(raw.matches("O001").count(), 1);
        assert!(!raw.contains("WEB"));
        assert!(!raw.contains("QR-CUSTOMER"));
        assert!(!raw.contains("unix:"));
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
                price: Some("18".to_string()),
                image_path: None,
                available: Some(true),
            })
            .unwrap();

        assert_eq!(dish.dish_id, "D01");
        assert_eq!(dish.price_amount, 18);
        let options = state.menu_view().preference_options;
        assert!(options.ingredients.contains(&"tofu".to_string()));
        assert!(options.tags.contains(&"vegetarian".to_string()));
    }

    #[test]
    fn customer_search_does_not_mutate_static_menu_dataset() {
        let dishes = vec![
            dish("D01", "Nasi Kandar", "main", &["rice"], &["spicy"]),
            dish("D02", "Nasi Kerabu", "main", &["blue rice"], &["local"]),
            dish("D03", "Chicken Satay", "main", &["chicken"], &["grilled"]),
        ];
        let state = WebState::new(dishes, Vec::new());
        let before_ids = state
            .menu_view()
            .dishes
            .into_iter()
            .map(|dish| dish.dish_id)
            .collect::<Vec<_>>();

        let suggestions = state.search_menu("Nasi K", MatchMode::Any);
        let after_ids = state
            .menu_view()
            .dishes
            .into_iter()
            .map(|dish| dish.dish_id)
            .collect::<Vec<_>>();

        assert_eq!(suggestions.results.len(), 2);
        assert_eq!(before_ids, after_ids);
        assert_eq!(after_ids.len(), 3);
    }

    #[test]
    fn customer_order_sync_is_scoped_and_version_changes_after_admin_update() {
        let dishes = vec![dish("D01", "Nasi Lemak", "main", &["rice"], &["signature"])];
        let state = WebState::new(dishes, Vec::new());
        let first = state
            .create_live_order_with_customer(CreateLiveOrderRequest {
                dish_ids: vec!["D01".to_string()],
                customer_name: "Aina".to_string(),
                customer_phone: "0121111111".to_string(),
                table_number: Some("T01".to_string()),
                note: None,
            })
            .unwrap();
        state
            .create_live_order_with_customer(CreateLiveOrderRequest {
                dish_ids: vec!["D01".to_string()],
                customer_name: "Ben".to_string(),
                customer_phone: "0122222222".to_string(),
                table_number: Some("T02".to_string()),
                note: None,
            })
            .unwrap();

        let before = state.customer_order_sync_by_phone("0121111111").unwrap();
        state
            .update_order_status(&first.order_id, OrderStatus::Preparing)
            .unwrap();
        let after = state.customer_order_sync_by_phone("0121111111").unwrap();

        assert_eq!(after.orders.len(), 1);
        assert_eq!(after.orders[0].customer_name, "Aina");
        assert_eq!(after.orders[0].status, OrderStatus::Preparing);
        assert!(after.version > before.version);
    }

    #[test]
    fn referenced_dish_cannot_be_deleted() {
        let dishes = vec![dish("D01", "Nasi Lemak", "main", &["rice"], &["signature"])];
        let orders = vec![Order {
            order_id: "O001".to_string(),
            session_user_id: "U01".to_string(),
            ordered_dishes: vec!["D01".to_string()],
            timestamp: "2026-01-01 12:00".to_string(),
        }];
        let state = WebState::new(dishes, orders);

        let error = state.delete_dish("D01").unwrap_err();

        assert!(error.contains("historical orders"));
        assert!(state.admin_dish_by_id("D01").is_some());
    }

    #[test]
    fn ingredient_experiment_returns_before_and_after_rankings() {
        let dishes = vec![
            dish("D01", "Rice Bowl", "main", &["rice"], &["local"]),
            dish("D02", "Chicken Satay", "main", &["chicken"], &["grilled"]),
            dish("D03", "Beef Soup", "main", &["beef"], &["soup"]),
        ];
        let state = WebState::new(dishes, Vec::new());

        let result = state
            .experiment_lab(ExperimentLabRequest {
                experiment_type: "ingredient".to_string(),
                liked_ingredients: vec!["chicken".to_string()],
                disliked_ingredients: vec!["beef".to_string()],
                top_k: Some(3),
                ..ExperimentLabRequest::default()
            })
            .unwrap();

        assert!(
            result
                .rows
                .iter()
                .any(|row| row.method == "Before (no preferences)")
        );
        assert!(
            result
                .rows
                .iter()
                .any(|row| row.method == "After (selected preferences)")
        );
        assert!(result.conclusion.contains("excluded"));
    }

    #[test]
    fn advanced_scenarios_do_not_mutate_history_or_timeline() {
        let dishes = vec![
            dish("D01", "Rice Bowl", "main", &["rice"], &["local"]),
            dish("D02", "Chicken Satay", "side", &["chicken"], &["grilled"]),
            dish("D03", "Banana", "dessert", &["banana"], &["sweet"]),
        ];
        let orders = vec![Order {
            order_id: "O001".to_string(),
            session_user_id: "U01".to_string(),
            ordered_dishes: vec!["D01".to_string(), "D02".to_string()],
            timestamp: "2026-01-01 12:00".to_string(),
        }];
        let state = WebState::new(dishes, orders);
        let history_before = state.combined_orders();

        let comparison = state
            .counterfactual(CounterfactualRequest {
                baseline: RecommendationRequest::default(),
                changes: CounterfactualChanges {
                    add_liked_ingredients: vec!["rice".to_string()],
                    simulated_coorders: vec![
                        crate::recommender::counterfactual::SimulatedCoOrderChange {
                            anchor_dish_id: "D01".to_string(),
                            candidate_dish_id: "D03".to_string(),
                            additional_order_count: 10,
                        },
                    ],
                    ..Default::default()
                },
                top_k: 3,
            })
            .unwrap();

        assert!(!comparison.rank_changes.is_empty());
        assert_eq!(state.combined_orders().len(), history_before.len());
        assert_eq!(state.learning_timeline().event_count, 0);
    }

    #[test]
    fn clearing_learning_timeline_preserves_orders_and_recommendation_evidence() {
        let dishes = vec![
            dish("D01", "Rice Bowl", "main", &["rice"], &["local"]),
            dish("D02", "Chicken Satay", "side", &["chicken"], &["grilled"]),
        ];
        let orders = vec![Order {
            order_id: "O001".to_string(),
            session_user_id: "U01".to_string(),
            ordered_dishes: vec!["D01".to_string(), "D02".to_string()],
            timestamp: "2026-01-01 12:00".to_string(),
        }];
        let path = temp_order_csv_path("timeline_clear").with_extension("jsonl");
        let mut state = WebState::new(dishes.clone(), orders.clone());
        state.learning_events_path = Arc::new(path.clone());
        let event = build_learning_event(&dishes, &[], &orders[0]);
        rewrite_learning_events(std::slice::from_ref(&event), &path.to_string_lossy()).unwrap();
        state
            .learning_events
            .write()
            .expect("timeline lock")
            .push(event);
        let before = state.recommend(RecommendationRequest {
            selected_dish_ids: vec!["D01".to_string()],
            ..RecommendationRequest::default()
        });

        let result = state.clear_learning_timeline().unwrap();
        let after = state.recommend(RecommendationRequest {
            selected_dish_ids: vec!["D01".to_string()],
            ..RecommendationRequest::default()
        });

        assert_eq!(result.removed_event_count, 1);
        assert_eq!(state.learning_timeline().event_count, 0);
        assert_eq!(state.combined_orders().len(), 1);
        assert_eq!(state.combined_orders()[0].order_id, orders[0].order_id);
        assert_eq!(
            before.evidence_profile.total_order_count,
            after.evidence_profile.total_order_count
        );
        assert_eq!(
            before.evidence_profile.strongest_context_pair_count,
            after.evidence_profile.strongest_context_pair_count
        );
        assert!(
            crate::persistence::learning_events::load_learning_events(&path.to_string_lossy())
                .unwrap()
                .is_empty()
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn meal_sets_use_available_dishes_and_exact_budget_cents() {
        let dishes = vec![
            dish("D01", "Rice Bowl", "main", &["rice"], &["local"]),
            dish("D02", "Chicken Satay", "side", &["chicken"], &["grilled"]),
            dish("D03", "Banana", "dessert", &["banana"], &["sweet"]),
        ];
        let state = WebState::new(dishes, Vec::new());
        state.set_dish_availability("D02", false).unwrap();

        let sets = state
            .recommend_meal_sets(MealSetRequest {
                budget_cents: 5_000,
                party_size: 1,
                target_dish_count: Some(2),
                top_set_count: Some(1),
                liked_ingredients: vec![],
                disliked_ingredients: vec![],
                preferred_tags: vec![],
                required_categories: vec![],
                selected_dish_ids: vec![],
                time_context: None,
                diversity_mode: Some("balanced".to_string()),
            })
            .unwrap();

        assert!(sets[0].total_price_cents <= 5_000);
        assert!(sets[0].dishes.iter().all(|dish| dish.dish_id != "D02"));
    }
}
