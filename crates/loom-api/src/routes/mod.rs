pub mod bundles;

use axum::routing::{get, post};
use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/bundles", post(bundles::create).get(bundles::list))
        .route(
            "/api/v1/bundles/{id}",
            get(bundles::get).patch(bundles::edit),
        )
        .route("/b/{id}", get(bundles::share))
}
