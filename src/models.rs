use serde::Deserialize;

/// Raw dish record exactly as it appears in `data/dishes.csv`.
///
/// The `ingredients` and `tags` fields are still comma-separated strings here
/// because CSV loading should not hide the original file format. The loader
/// converts this row into the cleaner `Dish` model used by the prototype.
#[derive(Debug, Clone, Deserialize)]
pub struct DishRow {
    pub dish_id: String,
    pub name: String,
    pub ingredients: String,
    pub category: String,
    pub tags: String,
}

/// Clean dish model used by the recommendation algorithms and GUI.
///
/// Ingredients and tags are stored as vectors of lowercase strings so the
/// ingredient-based filtering step can compare user preferences consistently.
#[derive(Debug, Clone)]
pub struct Dish {
    pub dish_id: String,
    pub name: String,
    pub ingredients: Vec<String>,
    pub category: String,
    pub tags: Vec<String>,
}

/// Raw order record exactly as it appears in `data/orders.csv`.
///
/// `ordered_dishes` remains a comma-separated string until the loader converts
/// it into an `Order` with a vector of uppercase dish IDs.
#[derive(Debug, Clone, Deserialize)]
pub struct OrderRow {
    pub order_id: String,
    pub session_user_id: String,
    pub ordered_dishes: String,
    pub timestamp: String,
}

/// Clean order model used to build the collaborative filtering matrix.
///
/// Each order represents a single restaurant ordering session. Dishes that
/// appear in the same order are treated as co-ordered items.
#[derive(Debug, Clone)]
pub struct Order {
    pub order_id: String,
    pub session_user_id: String,
    pub ordered_dishes: Vec<String>,
    pub timestamp: String,
}

/// User preference values entered in the GUI.
///
/// The fields are intentionally simple vectors because the FYP prototype is
/// designed to demonstrate explainable filtering rather than complex user
/// profiling. Ingredients and tags are lowercase; dish IDs are uppercase.
#[derive(Debug, Clone, Default)]
pub struct UserPreference {
    pub liked_ingredients: Vec<String>,
    pub disliked_ingredients: Vec<String>,
    pub preferred_tags: Vec<String>,
    pub selected_dish_ids: Vec<String>,
}

impl UserPreference {
    /// Returns true when the user entered content-based preferences.
    ///
    /// The hybrid recommender uses this to decide whether it should give more
    /// weight to ingredient filtering or to collaborative co-ordering signals.
    pub fn has_content_preferences(&self) -> bool {
        !self.liked_ingredients.is_empty()
            || !self.disliked_ingredients.is_empty()
            || !self.preferred_tags.is_empty()
    }

    /// Returns true when the user entered already-selected/current dish IDs.
    ///
    /// Collaborative filtering needs at least one selected dish before it can
    /// look for other dishes that are often ordered together with it.
    pub fn has_selected_dishes(&self) -> bool {
        !self.selected_dish_ids.is_empty()
    }
}

/// A single recommendation row shown in the GUI.
///
/// Keeping the individual ingredient, co-order, and hybrid scores visible makes
/// the prototype suitable for an FYP presentation because every recommendation
/// can be explained in plain language.
#[derive(Debug, Clone)]
pub struct RecommendationResult {
    pub dish: Dish,
    pub ingredient_score: f32,
    pub co_order_score: f32,
    pub final_score: f32,
    pub matched_liked_ingredients: Vec<String>,
    pub matched_preferred_tags: Vec<String>,
    pub matched_disliked_ingredients: Vec<String>,
    pub related_selected_dish_ids: Vec<String>,
    pub explanation: String,
}
