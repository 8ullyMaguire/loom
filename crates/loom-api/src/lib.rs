//! Typed HTTP API for Loom.

pub mod error;
pub mod openapi;
pub mod routes;
pub mod state;

use axum::Router;
use state::AppState;

pub fn router(state: AppState) -> Router {
    routes::router().with_state(state)
}
