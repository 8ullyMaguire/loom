use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchQueryDto {
    pub q: Option<String>,
    pub tag: Option<String>,
    pub lang: Option<String>,
    pub source: Option<String>,
    pub mode: Option<String>,
    pub min_votes: Option<i64>,
    pub cursor: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchHitDto {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub source: String,
    pub ranking_mode: String,
    pub score: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchPageDto {
    pub items: Vec<SearchHitDto>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HiddenGemsDto {
    pub impression_share: f64,
    pub items: Vec<SearchHitDto>,
}

fn hit(h: loom_store::search::SearchHit) -> SearchHitDto {
    SearchHitDto {
        uri: h.uri,
        name: h.name,
        description: h.description,
        source: h.source,
        ranking_mode: h.ranking_mode,
        score: h.score,
    }
}

#[utoipa::path(get, path = "/api/v1/search", responses((status = 200, body = SearchPageDto)))]
pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQueryDto>,
) -> Result<Json<SearchPageDto>, ApiError> {
    let q = loom_store::search::SearchQuery {
        q: query.q,
        tag: query.tag,
        lang: query.lang,
        source: query.source,
        mode: query.mode,
        min_votes: query.min_votes,
        cursor: query.cursor,
        limit: query.limit,
    };
    let page = loom_store::search::search(state.db.pool(), &q).await?;
    Ok(Json(SearchPageDto {
        items: page.items.into_iter().map(hit).collect(),
        next_cursor: page.next_cursor,
    }))
}

#[utoipa::path(get, path = "/api/v1/hidden-gems", responses((status = 200, body = HiddenGemsDto)))]
pub async fn hidden_gems(State(state): State<AppState>) -> Result<Json<HiddenGemsDto>, ApiError> {
    let gems = loom_store::search::hidden_gems(state.db.pool(), 20).await?;
    Ok(Json(HiddenGemsDto {
        impression_share: gems.impression_share,
        items: gems.items.into_iter().map(hit).collect(),
    }))
}
