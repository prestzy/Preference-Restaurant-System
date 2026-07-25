//! Web layer for the QR-code restaurant ordering prototype.
//!
//! The web layer is deliberately thin. It owns HTTP routes, shared web state,
//! and server-rendered HTML, while CSV loading and recommendation scoring stay
//! in their existing modules.

pub mod handlers;
pub mod routes;
mod session;
pub mod state;
mod templates;
mod validation;
