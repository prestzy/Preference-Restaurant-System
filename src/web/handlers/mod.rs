//! Focused HTTP handlers for the web prototype.
//!
//! Each submodule owns one user-facing area. This keeps menu rendering, cart
//! checkout, admin tools, and recommendation APIs cohesive without coupling
//! them to route declaration or `main.rs`.

pub mod admin;
pub mod assistant;
pub mod cart;
pub mod customer;
pub mod menu;
pub mod orders;
pub mod recommendations;
pub mod search;
