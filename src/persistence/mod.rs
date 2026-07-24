//! Persistence helpers for operational prototype data.
//!
//! Recommendation history remains in `data/orders.csv`. Customer contact
//! details are stored separately so the collaborative-filtering dataset is not
//! polluted with operational customer fields.

pub mod order_details;
