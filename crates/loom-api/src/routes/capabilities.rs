//! What is open right now. The document follows the declared sunset rule.

use axum::extract::State;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct CapabilitiesDto {
    pub bundle_create_anonymous: bool,
    pub tags: bool,
    pub search: bool,
    pub recommendations: bool,
    pub hidden_gems: bool,
    /// False until the CLI/TUI row ships; the document never claims unbuilt features.
    pub tui: bool,
    pub reports_30d: i64,
    pub threshold: u64,
}

#[utoipa::path(get, path = "/api/v1/capabilities",
    responses((status = 200, body = CapabilitiesDto)))]
pub async fn capabilities(
    State(state): State<AppState>,
) -> Result<Json<CapabilitiesDto>, crate::error::ApiError> {
    let (reports_30d,): (i64,) = sqlx::query_as(
        "select count(*) from reports where created_at > now() - interval '30 days'",
    )
    .fetch_one(state.db.pool())
    .await?;
    Ok(Json(CapabilitiesDto {
        bundle_create_anonymous: !loom_core::sunset::should_close(
            reports_30d as u64,
            state.sunset_threshold,
        ),
        tags: true,
        search: true,
        recommendations: true,
        hidden_gems: true,
        tui: false,
        reports_30d,
        threshold: state.sunset_threshold,
    }))
}
