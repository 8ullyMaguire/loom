//! Bundle HTTP surface. All responses are typed; no Json<Value> anywhere.

use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateBundleRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub feeds: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateBundleResponse {
    pub id: Uuid,
    pub edit_token: String,
    pub url: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BundleDto {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub feeds: Vec<String>,
    pub version: i32,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub cursor: Option<Uuid>,
    pub limit: Option<i64>,
}

fn visitor_key(headers: &HeaderMap) -> String {
    headers
        .get("x-visitor-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous")
        .to_string()
}

async fn sunset_active(state: &AppState) -> Result<bool, ApiError> {
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

async fn allow_write(state: &AppState, headers: &HeaderMap) -> Result<(), ApiError> {
    if sunset_active(state).await? && headers.get(header::AUTHORIZATION).is_none() {
        return Err(ApiError::auth("identity required after the sunset trigger"));
    }
    Ok(())
}

#[utoipa::path(
    post,
    path = "/api/v1/bundles",
    request_body = CreateBundleRequest,
    responses((status = 201, body = CreateBundleResponse))
)]
pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateBundleRequest>,
) -> Result<Response, ApiError> {
    allow_write(&state, &headers).await?;
    let created = loom_store::bundles::create(
        state.db.pool(),
        &visitor_key(&headers),
        &body.name,
        &body.description,
        body.feeds,
        None,
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(CreateBundleResponse {
            id: created.view.id,
            edit_token: created.edit_token,
            url: format!("/b/{}", created.view.id),
        }),
    )
        .into_response())
}

#[utoipa::path(get, path = "/api/v1/bundles", responses((status = 200, body = [BundleDto])))]
pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<BundleDto>>, ApiError> {
    let rows = loom_store::bundles::list(
        state.db.pool(),
        None,
        query.cursor,
        query.limit.unwrap_or(50),
    )
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|v| BundleDto {
                id: v.id,
                name: v.name,
                description: v.description,
                feeds: v.uris,
                version: v.version,
            })
            .collect(),
    ))
}

#[utoipa::path(get, path = "/api/v1/bundles/{id}", responses((status = 200, body = BundleDto)))]
pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BundleDto>, ApiError> {
    let view = loom_store::bundles::get(state.db.pool(), id)
        .await?
        .ok_or_else(|| ApiError::not_found("bundle not found"))?;
    Ok(Json(BundleDto {
        id: view.id,
        name: view.name,
        description: view.description,
        feeds: view.uris,
        version: view.version,
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EditBundleRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub feeds: Vec<String>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/bundles/{id}",
    request_body = EditBundleRequest,
    responses((status = 200, body = BundleDto))
)]
pub async fn edit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(body): Json<EditBundleRequest>,
) -> Result<Json<BundleDto>, ApiError> {
    let token = headers
        .get("x-edit-token")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::auth("X-Edit-Token required"))?;
    let view = loom_store::bundles::edit(
        state.db.pool(),
        id,
        token,
        &body.name,
        &body.description,
        body.feeds,
    )
    .await?;
    Ok(Json(BundleDto {
        id: view.id,
        name: view.name,
        description: view.description,
        feeds: view.uris,
        version: view.version,
    }))
}

fn escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// One public link opens every feed in the bundle in Bluesky.
pub async fn share(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Html<String>, ApiError> {
    let view = loom_store::bundles::get(state.db.pool(), id)
        .await?
        .ok_or_else(|| ApiError::not_found("bundle not found"))?;
    let rows: Vec<(String, String)> =
        sqlx::query_as("select did, rkey from feeds where uri = any($1)")
            .bind(&view.uris)
            .fetch_all(state.db.pool())
            .await
            .map_err(|e| ApiError::from(loom_store::Error::Database(e)))?;
    let mut links = String::new();
    for uri in &view.uris {
        if let Some((did, rkey)) = rows.iter().find(|(d, _)| uri.contains(d)) {
            links.push_str(&format!(
                "<li><a href=\"https://bsky.app/profile/{}/feed/{}\">{}</a></li>\n",
                escape(did),
                escape(rkey),
                escape(rkey)
            ));
        }
    }
    let html = format!(
        "<!doctype html><html lang=\"en\" data-loom-ssr=\"1\"><head><meta charset=\"utf-8\">\
        <title>{}</title></head><body><h1>{}</h1><p>{}</p><ul>{}</ul></body></html>",
        escape(&view.name),
        escape(&view.name),
        escape(&view.description),
        links
    );
    Ok(Html(html))
}
