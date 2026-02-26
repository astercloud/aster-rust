use crate::state::AppState;
use aster::session::{
    CommitOptions, CommitReport, MemoryCategory, MemoryHealth, MemorySearchResult, MemoryStats,
    SessionManager,
};
use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MemoryExtractRequest {
    pub session_id: String,
    #[serde(default)]
    pub force: bool,
    pub max_messages: Option<usize>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MemorySearchRequest {
    pub query: String,
    pub limit: Option<usize>,
    pub session_id: Option<String>,
    pub categories: Option<Vec<MemoryCategory>>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MemorySearchResponse {
    pub results: Vec<MemorySearchResult>,
}

#[utoipa::path(
    post,
    path = "/memory/extract",
    request_body = MemoryExtractRequest,
    responses(
        (status = 200, description = "Memory extracted from session", body = CommitReport),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Memory"
)]
async fn extract_memory(
    Json(request): Json<MemoryExtractRequest>,
) -> Result<Json<CommitReport>, StatusCode> {
    let report = SessionManager::commit_session(
        &request.session_id,
        CommitOptions {
            force: request.force,
            max_messages: request.max_messages,
        },
    )
    .await
    .map_err(|err| {
        if err.to_string().contains("Session not found") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(Json(report))
}

#[utoipa::path(
    post,
    path = "/memory/search",
    request_body = MemorySearchRequest,
    responses(
        (status = 200, description = "Memory search results", body = MemorySearchResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Memory"
)]
async fn search_memory(
    Json(request): Json<MemorySearchRequest>,
) -> Result<Json<MemorySearchResponse>, StatusCode> {
    let results = SessionManager::search_memories(
        &request.query,
        request.limit,
        request.session_id.as_deref(),
        request.categories,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(MemorySearchResponse { results }))
}

#[utoipa::path(
    get,
    path = "/memory/stats",
    responses(
        (status = 200, description = "Memory subsystem stats", body = MemoryStats),
        (status = 500, description = "Internal server error")
    ),
    tag = "Memory"
)]
async fn memory_stats() -> Result<Json<MemoryStats>, StatusCode> {
    let stats = SessionManager::memory_stats()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(stats))
}

#[utoipa::path(
    get,
    path = "/memory/health",
    responses(
        (status = 200, description = "Memory subsystem health", body = MemoryHealth),
        (status = 500, description = "Internal server error")
    ),
    tag = "Memory"
)]
async fn memory_health() -> Result<Json<MemoryHealth>, StatusCode> {
    let health = SessionManager::memory_health()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(health))
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/memory/extract", post(extract_memory))
        .route("/memory/search", post(search_memory))
        .route("/memory/stats", get(memory_stats))
        .route("/memory/health", get(memory_health))
        .with_state(state)
}
