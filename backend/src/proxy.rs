use crate::{AppState, api_key, error::AppError, models::CreateRequestLogRequest};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
};
use reqwest;
use std::time::Instant;

pub async fn proxy_handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
    method: Method,
    headers: HeaderMap,
    body: String,
) -> Result<Response, AppError> {
    let start = Instant::now();

    // Extract and validate proxy API key from Authorization header
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            AppError::unauthorized("proxy::proxy_handler", "Missing Authorization header")
        })?;

    let api_key = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        AppError::unauthorized(
            "proxy::proxy_handler",
            "Authorization header must be in format 'Bearer <token>'",
        )
    })?;

    let key_hash = api_key::hash_api_key(api_key);

    let proxy_key = state
        .repository
        .get_proxy_api_key_by_hash(&key_hash)
        .await
        .map_err(|e| {
            AppError::internal_server_error(
                "proxy::proxy_handler",
                &format!("Failed to lookup API key: {}", e),
            )
        })?
        .ok_or_else(|| AppError::unauthorized("proxy::proxy_handler", "Invalid API key"))?;

    // Get the platform ID from the API key
    let platform_id = proxy_key.llm_platform_id.ok_or_else(|| {
        AppError::internal_server_error(
            "proxy::proxy_handler",
            "API key is missing platform association",
        )
    })?;

    // Update last used timestamp
    let _ = state
        .repository
        .update_proxy_api_key_last_used(proxy_key.id)
        .await;

    // Get LLM platform configuration
    let platform = state
        .repository
        .get_llm_platform(platform_id, proxy_key.user_id)
        .await
        .map_err(|e| {
            AppError::internal_server_error(
                "proxy::proxy_handler",
                &format!("Failed to fetch LLM platform: {}", e),
            )
        })?
        .ok_or_else(|| AppError::not_found("proxy::proxy_handler", "LLM platform"))?;

    // Build target URL
    let target_url = format!("{}/{}", platform.base_url.trim_end_matches('/'), path);

    // Forward request to LLM platform
    let client = reqwest::Client::new();
    let mut req_builder = match method {
        Method::GET => client.get(&target_url),
        Method::POST => client.post(&target_url),
        Method::PUT => client.put(&target_url),
        Method::DELETE => client.delete(&target_url),
        Method::PATCH => client.patch(&target_url),
        _ => return Err(AppError::method_not_allowed("proxy::proxy_handler")),
    };

    // Build outgoing headers map for logging
    let mut outgoing_headers_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    // Add LLM platform API key
    req_builder = req_builder.header("Authorization", format!("Bearer {}", platform.api_key));
    outgoing_headers_map.insert(
        "Authorization".to_string(),
        format!("Bearer {}", "*".repeat(20)), // Redact actual key in logs
    );

    // Forward relevant headers (excluding our authorization)
    for (key, value) in headers.iter() {
        let key_str = key.as_str();
        if key_str != "authorization" && key_str != "host"
            && let Ok(value_str) = value.to_str()
        {
            req_builder = req_builder.header(key_str, value_str);
            outgoing_headers_map.insert(key_str.to_string(), value_str.to_string());
        }
    }

    // Add body if present
    let outgoing_body = if !body.is_empty() {
        req_builder = req_builder.body(body.clone());
        Some(body.clone())
    } else {
        None
    };

    // Serialize outgoing headers for logging
    let outgoing_headers = serde_json::to_string(&outgoing_headers_map).unwrap_or_default();

    // Execute request
    let result = req_builder.send().await;

    let duration_ms = start.elapsed().as_millis() as i64;

    // Log request/response
    let (response_status, response_headers, response_body, error) = match result {
        Ok(resp) => {
            let status = resp.status().as_u16() as i32;
            let headers_map: std::collections::HashMap<String, String> = resp
                .headers()
                .iter()
                .filter_map(|(k, v)| v.to_str().ok().map(|val| (k.to_string(), val.to_string())))
                .collect();
            let headers = serde_json::to_string(&headers_map).unwrap_or_default();
            let body = resp.text().await.unwrap_or_default();
            (Some(status), Some(headers), Some(body.clone()), None)
        }
        Err(e) => (None, None, None, Some(e.to_string())),
    };

    let request_headers_map: std::collections::HashMap<String, String> = headers
        .iter()
        .filter_map(|(k, v)| v.to_str().ok().map(|val| (k.to_string(), val.to_string())))
        .collect();
    let request_headers = serde_json::to_string(&request_headers_map).unwrap_or_default();

    let log_req = CreateRequestLogRequest {
        user_id: proxy_key.user_id,
        proxy_api_key_id: proxy_key.id,
        llm_platform_id: platform_id,
        method: method.to_string(),
        path: path.clone(),
        request_headers,
        request_body: if body.is_empty() { None } else { Some(body) },
        outgoing_url: Some(target_url),
        outgoing_headers: Some(outgoing_headers),
        outgoing_body,
        response_status,
        response_headers: response_headers.clone(),
        response_body: response_body.clone(),
        duration_ms: Some(duration_ms),
        error: error.clone(),
    };

    // Log to database (fire and forget)
    tokio::spawn(async move {
        let _ = state.repository.create_request_log(log_req).await;
    });

    // Return response
    if let Some(err) = error {
        return Err(AppError::bad_gateway("proxy::proxy_handler", &err));
    }

    let status_code = StatusCode::from_u16(response_status.unwrap() as u16)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    Ok((status_code, response_body.unwrap_or_default()).into_response())
}
