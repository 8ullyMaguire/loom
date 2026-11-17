//! User confidence tags. The number is a judgement, never a fact about a feed.

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct TagVoteRequest {
    pub feed: String,
    pub tag: String,
    pub confidence: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TagAggregateResponse {
    pub status: String,
    pub n: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_low: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_high: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct TagQuery {
    pub feed: String,
    pub tag: String,
}

#[utoipa::path(post, path = "/api/v1/tags", request_body = TagVoteRequest,
    responses((status = 201, body = TagAggregateResponse)))]
pub async fn vote(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<TagVoteRequest>,
) -> Result<Response, ApiError> {
    super::allow_write(&state, &headers).await?;
    if !(0..=100).contains(&body.confidence) {
        return Err(ApiError::invalid("confidence must be 0 to 100"));
    }
    if body.tag.is_empty() || body.tag.len() > 40 {
        return Err(ApiError::invalid("tag must be 1 to 40 characters"));
    }
    loom_store::tags::vote(
        state.db.pool(),
        &super::visitor_key(&headers),
        &body.feed,
        &body.tag,
        body.confidence,
    )
    .await?;
    let a = loom_store::tags::aggregate(state.db.pool(), &body.feed, &body.tag).await?;
    Ok((
        StatusCode::CREATED,
        Json(TagAggregateResponse {
            status: a.status.into(),
            n: a.n,
            mean: a.mean,
            ci_low: a.ci_low,
            ci_high: a.ci_high,
        }),
    )
        .into_response())
}

#[utoipa::path(get, path = "/api/v1/tags", responses((status = 200, body = TagAggregateResponse)))]
pub async fn aggregate(
    State(state): State<AppState>,
    Query(query): Query<TagQuery>,
) -> Result<Json<TagAggregateResponse>, ApiError> {
    let a = loom_store::tags::aggregate(state.db.pool(), &query.feed, &query.tag).await?;
    Ok(Json(TagAggregateResponse {
        status: a.status.into(),
        n: a.n,
        mean: a.mean,
        ci_low: a.ci_low,
        ci_high: a.ci_high,
    }))
}
