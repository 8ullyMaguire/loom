pub mod bundles;
pub mod capabilities;
pub mod recommendations;
pub mod search;
pub mod tags;

use axum::http::HeaderMap;
use axum::routing::{get, post};
use axum::Router;

use crate::error::ApiError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/bundles", post(bundles::create).get(bundles::list))
        .route(
            "/api/v1/bundles/{id}",
            get(bundles::get).patch(bundles::edit),
        )
        .route("/b/{id}", get(bundles::share))
        .route("/api/v1/tags", post(tags::vote).get(tags::aggregate))
        .route("/api/v1/search", get(search::search))
        .route("/api/v1/hidden-gems", get(search::hidden_gems))
        .route(
            "/api/v1/recommendations",
            get(recommendations::recommendations),
        )
        .route("/api/v1/capabilities", get(capabilities::capabilities))
}

pub(crate) async fn sunset_active(state: &AppState) -> Result<bool, ApiError> {
    let n: (i64,) = sqlx::query_as(
        "select count(*) from reports where created_at > now() - interval '30 days'",
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| ApiError::from(loom_store::Error::Database(e)))?;
    Ok(loom_core::sunset::should_close(
        n.0 as u64,
        state.sunset_threshold,
    ))
}

pub(crate) async fn allow_write(state: &AppState, headers: &HeaderMap) -> Result<(), ApiError> {
    if sunset_active(state).await? && headers.get(axum::http::header::AUTHORIZATION).is_none() {
        return Err(ApiError::auth("identity required after the sunset trigger"));
    }
    Ok(())
}

pub(crate) fn visitor_key(headers: &HeaderMap) -> String {
    headers
        .get("x-visitor-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous")
        .to_string()
}
