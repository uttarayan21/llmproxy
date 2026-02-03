use crate::{AppState, api_key, auth::{AuthUser, Backend, Credentials, hash_password}, error::AppError, models::*};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use axum_login::AuthSession;
use serde::Deserialize;

// Authentication handlers
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<User>, AppError> {
    // Check if username already exists
    let existing = state
        .repository
        .get_user_by_username(&req.username)
        .await
        .map_err(|e| {
            AppError::internal_server_error(
                "handlers::register",
                &format!("Failed to check existing user: {}", e),
            )
        })?;

    if existing.is_some() {
        return Err(AppError::bad_request(
            "handlers::register",
            "Username already exists",
        ));
    }

    // Hash password
    let password_hash = hash_password(&req.password)?;

    // Create user
    let user = state
        .repository
        .create_user(&req.username, &password_hash)
        .await
        .map_err(|e| {
            AppError::internal_server_error(
                "handlers::register",
                &format!("Failed to create user: {}", e),
            )
        })?;

    Ok(Json(user))
}

pub async fn login(
    mut auth_session: AuthSession<Backend>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<User>, AppError> {
    let creds = Credentials {
        username: req.username,
        password: req.password,
    };

    let user = auth_session
        .authenticate(creds.clone())
        .await
        .map_err(|e| {
            AppError::internal_server_error(
                "handlers::login",
                &format!("Authentication error: {}", e),
            )
        })?
        .ok_or_else(|| {
            AppError::unauthorized("handlers::login", "Invalid username or password")
        })?;

    auth_session.login(&user).await.map_err(|e| {
        AppError::internal_server_error(
            "handlers::login",
            &format!("Failed to create session: {}", e),
        )
    })?;

    Ok(Json(user))
}

pub async fn logout(mut auth_session: AuthSession<Backend>) -> Result<StatusCode, AppError> {
    auth_session.logout().await.map_err(|e| {
        AppError::internal_server_error(
            "handlers::logout",
            &format!("Failed to destroy session: {}", e),
        )
    })?;

    Ok(StatusCode::OK)
}

pub async fn get_current_user(
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<User>, AppError> {
    Ok(Json(auth_user.user))
}


// LLM Platform handlers
pub async fn create_llm_platform(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateLlmPlatformRequest>,
) -> Result<Json<LlmPlatform>, AppError> {
    let platform = state
        .repository
        .create_llm_platform(auth_user.user.id, req)
        .await
        .map_err(|e| {
            AppError::internal_server_error(
                "handlers::create_llm_platform",
                &format!("Failed to create LLM platform: {}", e),
            )
        })?;

    Ok(Json(platform))
}

pub async fn get_llm_platforms(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<LlmPlatform>>, AppError> {
    let platforms = state
        .repository
        .get_llm_platforms(auth_user.user.id)
        .await
        .map_err(|e| {
            AppError::internal_server_error(
                "handlers::get_llm_platforms",
                &format!("Failed to fetch LLM platforms: {}", e),
            )
        })?;

    Ok(Json(platforms))
}

pub async fn update_llm_platform(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateLlmPlatformRequest>,
) -> Result<Json<LlmPlatform>, AppError> {
    let platform = state
        .repository
        .update_llm_platform(id, auth_user.user.id, req)
        .await
        .map_err(|e| {
            AppError::internal_server_error(
                "handlers::update_llm_platform",
                &format!("Failed to update LLM platform: {}", e),
            )
        })?
        .ok_or_else(|| AppError::not_found("handlers::update_llm_platform", "LLM platform"))?;

    Ok(Json(platform))
}

pub async fn delete_llm_platform(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let deleted = state
        .repository
        .delete_llm_platform(id, auth_user.user.id)
        .await
        .map_err(|e| {
            AppError::internal_server_error(
                "handlers::delete_llm_platform",
                &format!("Failed to delete LLM platform: {}", e),
            )
        })?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(
            "handlers::delete_llm_platform",
            "LLM platform",
        ))
    }
}

// Proxy API Key handlers
pub async fn create_proxy_api_key(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateProxyApiKeyRequest>,
) -> Result<Json<ProxyApiKeyResponse>, AppError> {
    // Verify the platform exists and belongs to the user
    let _platform = state
        .repository
        .get_llm_platform(req.llm_platform_id, auth_user.user.id)
        .await
        .map_err(|e| {
            AppError::internal_server_error(
                "handlers::create_proxy_api_key",
                &format!("Failed to verify LLM platform: {}", e),
            )
        })?
        .ok_or_else(|| AppError::not_found("handlers::create_proxy_api_key", "LLM platform"))?;

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
        .map_err(|e| {
            AppError::internal_server_error(
                "handlers::create_proxy_api_key",
                &format!("Failed to create proxy API key: {}", e),
            )
        })?;

    Ok(Json(ProxyApiKeyResponse {
        id: proxy_key.id,
        key: api_key, // Only returned once
        key_prefix: proxy_key.key_prefix,
        name: proxy_key.name,
        llm_platform_id: proxy_key.llm_platform_id.ok_or_else(|| {
            AppError::internal_server_error(
                "handlers::create_proxy_api_key",
                "API key created without platform_id",
            )
        })?,
        created_at: proxy_key.created_at,
    }))
}

pub async fn get_proxy_api_keys(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Vec<ProxyApiKey>>, AppError> {
    let keys = state
        .repository
        .get_proxy_api_keys(auth_user.user.id)
        .await
        .map_err(|e| {
            AppError::internal_server_error(
                "handlers::get_proxy_api_keys",
                &format!("Failed to fetch proxy API keys: {}", e),
            )
        })?;

    Ok(Json(keys))
}

pub async fn delete_proxy_api_key(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let deleted = state
        .repository
        .delete_proxy_api_key(id, auth_user.user.id)
        .await
        .map_err(|e| {
            AppError::internal_server_error(
                "handlers::delete_proxy_api_key",
                &format!("Failed to delete proxy API key: {}", e),
            )
        })?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(
            "handlers::delete_proxy_api_key",
            "Proxy API key",
        ))
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
) -> Result<Json<Vec<RequestLog>>, AppError> {
    let logs = state
        .repository
        .get_request_logs(auth_user.user.id, query.limit)
        .await
        .map_err(|e| {
            AppError::internal_server_error(
                "handlers::get_request_logs",
                &format!("Failed to fetch request logs: {}", e),
            )
        })?;

    Ok(Json(logs))
}

pub async fn get_request_log(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<Json<RequestLog>, AppError> {
    let log = state
        .repository
        .get_request_log(id, auth_user.user.id)
        .await
        .map_err(|e| {
            AppError::internal_server_error(
                "handlers::get_request_log",
                &format!("Failed to fetch request log: {}", e),
            )
        })?
        .ok_or_else(|| AppError::not_found("handlers::get_request_log", "Request log"))?;

    Ok(Json(log))
}
