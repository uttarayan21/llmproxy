use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LlmPlatform {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub base_url: String,
    #[serde(skip_serializing)]
    pub api_key: String,
    pub platform_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLlmPlatformRequest {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub platform_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProxyApiKey {
    pub id: i64,
    pub user_id: i64,
    #[serde(skip_serializing)]
    pub key_hash: String,
    pub key_prefix: String,
    pub name: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProxyApiKeyRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyApiKeyResponse {
    pub id: i64,
    pub key: String, // Only returned once during creation
    pub key_prefix: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RequestLog {
    pub id: i64,
    pub user_id: i64,
    pub proxy_api_key_id: i64,
    pub llm_platform_id: i64,
    pub method: String,
    pub path: String,
    pub request_headers: String,
    pub request_body: Option<String>,
    pub response_status: Option<i32>,
    pub response_headers: Option<String>,
    pub response_body: Option<String>,
    pub duration_ms: Option<i64>,
    pub error: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRequestLogRequest {
    pub user_id: i64,
    pub proxy_api_key_id: i64,
    pub llm_platform_id: i64,
    pub method: String,
    pub path: String,
    pub request_headers: String,
    pub request_body: Option<String>,
    pub response_status: Option<i32>,
    pub response_headers: Option<String>,
    pub response_body: Option<String>,
    pub duration_ms: Option<i64>,
    pub error: Option<String>,
}
