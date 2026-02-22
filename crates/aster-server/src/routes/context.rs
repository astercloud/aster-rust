use crate::routes::errors::ErrorResponse;
use aster::context::{ContextLayer, ContextService, ContextServiceStatus};
use axum::{extract::Query, http::StatusCode, routing::get, Json, Router};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ContextQuery {
    /// 上下文 URI，例如 aster://resources/docs/getting-started.md
    uri: String,
    /// 是否返回解析轨迹（用于调试）
    include_trace: Option<bool>,
}

#[derive(Debug, serde::Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContextReadResponse {
    uri: String,
    layer: String,
    content: String,
    source_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace: Option<Vec<ContextTraceStepResponse>>,
}

#[derive(Debug, serde::Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContextTraceStepResponse {
    stage: String,
    detail: String,
}

#[derive(Debug, serde::Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContextNamespaceStatusResponse {
    namespace: String,
    path: String,
    exists: bool,
    file_count: usize,
    dir_count: usize,
}

#[derive(Debug, serde::Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContextStatusResponse {
    root_dir: String,
    root_exists: bool,
    namespaces: Vec<ContextNamespaceStatusResponse>,
}

fn to_read_response(
    result: aster::context::ContextReadResult,
    include_trace: bool,
) -> ContextReadResponse {
    let value = result.document;
    let trace = if include_trace {
        Some(
            result
                .trace
                .into_iter()
                .map(|step| ContextTraceStepResponse {
                    stage: step.stage,
                    detail: step.detail,
                })
                .collect(),
        )
    } else {
        None
    };

    ContextReadResponse {
        uri: value.uri,
        layer: value.layer.as_str().to_string(),
        content: value.content,
        source_path: value.source_path.display().to_string(),
        trace,
    }
}

impl From<ContextServiceStatus> for ContextStatusResponse {
    fn from(value: ContextServiceStatus) -> Self {
        Self {
            root_dir: value.root_dir.display().to_string(),
            root_exists: value.root_exists,
            namespaces: value
                .namespaces
                .into_iter()
                .map(|namespace| ContextNamespaceStatusResponse {
                    namespace: namespace.namespace,
                    path: namespace.path.display().to_string(),
                    exists: namespace.exists,
                    file_count: namespace.file_count,
                    dir_count: namespace.dir_count,
                })
                .collect(),
        }
    }
}

fn map_context_error(layer: ContextLayer, err: anyhow::Error) -> ErrorResponse {
    let message = err.to_string();
    let not_found = message.contains("未找到") || message.contains("No such file or directory");
    let status = if not_found {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::BAD_REQUEST
    };
    ErrorResponse {
        message: format!("读取 {} 层失败: {}", layer.as_str(), message),
        status,
    }
}

#[utoipa::path(
    get,
    path = "/context/abstract",
    params(
        ("uri" = String, Query, description = "Context URI")
    ),
    responses(
        (status = 200, description = "Abstract content", body = ContextReadResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "Context not found", body = ErrorResponse)
    ),
    tag = "Context"
)]
pub async fn abstract_context(
    Query(query): Query<ContextQuery>,
) -> Result<Json<ContextReadResponse>, ErrorResponse> {
    let service = ContextService::default();
    let include_trace = query.include_trace.unwrap_or(false);
    service
        .abstract_content_with_trace(&query.uri)
        .map(|result| to_read_response(result, include_trace))
        .map(Json)
        .map_err(|err| map_context_error(ContextLayer::Abstract, err))
}

#[utoipa::path(
    get,
    path = "/context/overview",
    params(
        ("uri" = String, Query, description = "Context URI")
    ),
    responses(
        (status = 200, description = "Overview content", body = ContextReadResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "Context not found", body = ErrorResponse)
    ),
    tag = "Context"
)]
pub async fn overview_context(
    Query(query): Query<ContextQuery>,
) -> Result<Json<ContextReadResponse>, ErrorResponse> {
    let service = ContextService::default();
    let include_trace = query.include_trace.unwrap_or(false);
    service
        .overview_content_with_trace(&query.uri)
        .map(|result| to_read_response(result, include_trace))
        .map(Json)
        .map_err(|err| map_context_error(ContextLayer::Overview, err))
}

#[utoipa::path(
    get,
    path = "/context/read",
    params(
        ("uri" = String, Query, description = "Context URI")
    ),
    responses(
        (status = 200, description = "Detail content", body = ContextReadResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "Context not found", body = ErrorResponse)
    ),
    tag = "Context"
)]
pub async fn read_context(
    Query(query): Query<ContextQuery>,
) -> Result<Json<ContextReadResponse>, ErrorResponse> {
    let service = ContextService::default();
    let include_trace = query.include_trace.unwrap_or(false);
    service
        .detail_content_with_trace(&query.uri)
        .map(|result| to_read_response(result, include_trace))
        .map(Json)
        .map_err(|err| map_context_error(ContextLayer::Detail, err))
}

#[utoipa::path(
    get,
    path = "/context/status",
    responses(
        (status = 200, description = "Context pipeline status", body = ContextStatusResponse),
        (status = 500, description = "Failed to collect context status", body = ErrorResponse)
    ),
    tag = "Context"
)]
pub async fn context_status() -> Result<Json<ContextStatusResponse>, ErrorResponse> {
    let service = ContextService::default();
    service
        .status()
        .map(ContextStatusResponse::from)
        .map(Json)
        .map_err(|err| ErrorResponse {
            message: format!("读取 context 状态失败: {}", err),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        })
}

pub fn routes() -> Router {
    Router::new()
        .route("/context/abstract", get(abstract_context))
        .route("/context/overview", get(overview_context))
        .route("/context/read", get(read_context))
        .route("/context/status", get(context_status))
}
