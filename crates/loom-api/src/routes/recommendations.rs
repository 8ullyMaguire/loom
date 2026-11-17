//! Recommendations: observable reasons, hidden ordering pull.
//!
//! No served field names the mechanism. `why` is built only from source,
//! language and the reader's own visible tagging history.

use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::config;
use crate::error::ApiError;
use crate::state::AppState;
use loom_core::gravity;

#[derive(Debug, Deserialize, ToSchema)]
pub struct RecommendationQuery {
    #[serde(default = "default_surface")]
    pub surface: String,
    #[serde(default = "default_reader")]
    pub reader: String,
    pub limit: Option<i64>,
}

fn default_surface() -> String {
    "api".into()
}
fn default_reader() -> String {
    "anonymous".into()
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RecommendationDto {
    pub uri: String,
    pub name: String,
    pub why: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RecommendationPageDto {
    pub surface: String,
    pub items: Vec<RecommendationDto>,
}

#[utoipa::path(get, path = "/api/v1/recommendations",
    responses((status = 200, body = RecommendationPageDto)))]
pub async fn recommendations(
    State(state): State<AppState>,
    Query(query): Query<RecommendationQuery>,
) -> Result<Json<RecommendationPageDto>, ApiError> {
    let cfg = config::load();
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let page = loom_store::search::search(
        state.db.pool(),
        &loom_store::search::SearchQuery {
            limit: Some(200),
            ..Default::default()
        },
    )
    .await?;

    let signals: (i64,) = sqlx::query_as(
        "select count(*) from tag_votes where contributor_key=$1 and confidence >= 60",
    )
    .bind(&query.reader)
    .fetch_one(state.db.pool())
    .await?;
    let affinity_rows: Vec<(String, f64)> = sqlx::query_as(
        "select feed_uri, count(*)::float8 / 20.0 from tag_votes
         where contributor_key=$1 and confidence >= 60 group by feed_uri",
    )
    .bind(&query.reader)
    .fetch_all(state.db.pool())
    .await?;
    let affinity: HashMap<String, f64> = affinity_rows.into_iter().collect();
    let g = gravity::ramp(
        signals.0 as u64,
        cfg.ceiling(&query.surface),
        cfg.gravity.u0,
    );

    let mut ranked: Vec<(f32, String, String, String)> = page
        .items
        .into_iter()
        .map(|hit| {
            let a = affinity.get(&hit.uri).copied().unwrap_or(0.0).min(1.0) as f32;
            let weight = gravity::score(1.0, a, g);
            let why = if a > 0.0 {
                format!("From {}; you have tagged this feed.", hit.source)
            } else {
                format!("Indexed from {}.", hit.source)
            };
            (weight, hit.uri, hit.name, why)
        })
        .collect();
    ranked.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    ranked.truncate(limit as usize);
    Ok(Json(RecommendationPageDto {
        surface: query.surface,
        items: ranked
            .into_iter()
            .map(|(_, uri, name, why)| RecommendationDto { uri, name, why })
            .collect(),
    }))
}
