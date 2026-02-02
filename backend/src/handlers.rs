use crate::{AppState, api_key, auth::AuthUser, models::*};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;

// LLM Platform handlers
pub async fn create_llm_platform(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateLlmPlatformRequest>,
) -> Result<Json<LlmPlatform>, StatusCode> {
    let platform = state
        .repository
        .create_llm_platform(auth_user.user.id, req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(platform))
}

pub async fn get_llm_platforms(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<LlmPlatform>>, StatusCode> {
    let platforms = state
        .repository
        .get_llm_platforms(auth_user.user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(platforms))
}

pub async fn delete_llm_platform(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    let deleted = state
        .repository
        .delete_llm_platform(id, auth_user.user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Proxy API Key handlers
pub async fn create_proxy_api_key(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateProxyApiKeyRequest>,
) -> Result<Json<ProxyApiKeyResponse>, StatusCode> {
    // Verify the platform exists and belongs to the user
    let _platform = state
        .repository
        .get_llm_platform(req.llm_platform_id, auth_user.user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let api_key = api_key::generate_api_key();
    let key_hash = api_key::hash_api_key(&api_key);
    let key_prefix = api_key::get_key_prefix(&api_key);

    let proxy_key = state
        .repository
        .create_proxy_api_key(
            auth_user.user.id,
            &req.name,
            &key_hash,
            &key_prefix,
            req.llm_platform_id,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ProxyApiKeyResponse {
        id: proxy_key.id,
        key: api_key, // Only returned once
        key_prefix: proxy_key.key_prefix,
        name: proxy_key.name,
        llm_platform_id: proxy_key
            .llm_platform_id
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?,
        created_at: proxy_key.created_at,
    }))
}

pub async fn get_proxy_api_keys(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<ProxyApiKey>>, StatusCode> {
    let keys = state
        .repository
        .get_proxy_api_keys(auth_user.user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(keys))
}

pub async fn delete_proxy_api_key(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    let deleted = state
        .repository
        .delete_proxy_api_key(id, auth_user.user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Request Log handlers
#[derive(Deserialize)]
pub struct GetLogsQuery {
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    100
}

pub async fn get_request_logs(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<GetLogsQuery>,
) -> Result<Json<Vec<RequestLog>>, StatusCode> {
    let logs = state
        .repository
        .get_request_logs(auth_user.user.id, query.limit)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(logs))
}

pub async fn get_request_log(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<Json<RequestLog>, StatusCode> {
    let log = state
        .repository
        .get_request_log(id, auth_user.user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(log))
}

// User handler
pub async fn get_current_user(Extension(auth_user): Extension<AuthUser>) -> Json<User> {
    Json(auth_user.user)
}
