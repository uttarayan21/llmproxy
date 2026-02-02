use crate::{AppState, api_key, error::AppError, models::CreateRequestLogRequest};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
};
use futures_util::StreamExt;
use reqwest;
use std::time::Instant;

// Context for logging requests
struct LogContext {
    state: AppState,
    user_id: i64,
    proxy_key_id: i64,
    platform_id: i64,
    method: Method,
    path: String,
    request_headers_map: std::collections::HashMap<String, String>,
    request_body: String,
    target_url: String,
    outgoing_headers: String,
    outgoing_body: Option<String>,
    start: Instant,
}

// Handle non-streaming responses
async fn handle_regular_response(
    resp: reqwest::Response,
    status: reqwest::StatusCode,
    headers: reqwest::header::HeaderMap,
    ctx: LogContext,
) -> Result<Response, AppError> {
    let duration_ms = ctx.start.elapsed().as_millis() as i64;

    // Extract response details
    let status_code = status.as_u16() as i32;
    let headers_map: std::collections::HashMap<String, String> = headers
        .iter()
        .filter_map(|(k, v)| v.to_str().ok().map(|val| (k.to_string(), val.to_string())))
        .collect();
    let response_headers_json = serde_json::to_string(&headers_map).unwrap_or_default();
    let response_body = resp.text().await.unwrap_or_default();

    // Log request/response
    let request_headers = serde_json::to_string(&ctx.request_headers_map).unwrap_or_default();
    let log_req = CreateRequestLogRequest {
        user_id: ctx.user_id,
        proxy_api_key_id: ctx.proxy_key_id,
        llm_platform_id: ctx.platform_id,
        method: ctx.method.to_string(),
        path: ctx.path.clone(),
        request_headers,
        request_body: if ctx.request_body.is_empty() {
            None
        } else {
            Some(ctx.request_body)
        },
        outgoing_url: Some(ctx.target_url),
        outgoing_headers: Some(ctx.outgoing_headers),
        outgoing_body: ctx.outgoing_body,
        response_status: Some(status_code),
        response_headers: Some(response_headers_json),
        response_body: Some(response_body.clone()),
        duration_ms: Some(duration_ms),
        error: None,
    };

    tokio::spawn(async move {
        let _ = ctx.state.repository.create_request_log(log_req).await;
    });

    // Return response
    let axum_status =
        StatusCode::from_u16(status_code as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    Ok((axum_status, response_body).into_response())
}

// Handle streaming responses (SSE)
async fn handle_streaming_response(
    resp: reqwest::Response,
    status: reqwest::StatusCode,
    headers: reqwest::header::HeaderMap,
    ctx: LogContext,
) -> Result<Response, AppError> {
    let status_code = status.as_u16() as i32;

    // Extract response headers
    let headers_map: std::collections::HashMap<String, String> = headers
        .iter()
        .filter_map(|(k, v)| v.to_str().ok().map(|val| (k.to_string(), val.to_string())))
        .collect();
    let response_headers_json = serde_json::to_string(&headers_map).unwrap_or_default();

    // Create a stream that collects chunks for logging
    let mut stream = resp.bytes_stream();
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<bytes::Bytes, std::io::Error>>(100);

    // Spawn a task to collect chunks and log after completion
    tokio::spawn(async move {
        let mut collected_chunks = Vec::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    collected_chunks.push(chunk.clone());
                    // Send chunk downstream (ignore errors if receiver is dropped)
                    let _ = tx.send(Ok(chunk)).await;
                }
                Err(e) => {
                    // Send error downstream
                    let _ = tx.send(Err(std::io::Error::other(e.to_string()))).await;
                    break;
                }
            }
        }

        // After stream completes, log the request
        let duration_ms = ctx.start.elapsed().as_millis() as i64;
        let response_body = collected_chunks
            .into_iter()
            .map(|c| String::from_utf8_lossy(&c).to_string())
            .collect::<Vec<_>>()
            .join("");

        let request_headers = serde_json::to_string(&ctx.request_headers_map).unwrap_or_default();
        let log_req = CreateRequestLogRequest {
            user_id: ctx.user_id,
            proxy_api_key_id: ctx.proxy_key_id,
            llm_platform_id: ctx.platform_id,
            method: ctx.method.to_string(),
            path: ctx.path,
            request_headers,
            request_body: if ctx.request_body.is_empty() {
                None
            } else {
                Some(ctx.request_body)
            },
            outgoing_url: Some(ctx.target_url),
            outgoing_headers: Some(ctx.outgoing_headers),
            outgoing_body: ctx.outgoing_body,
            response_status: Some(status_code),
            response_headers: Some(response_headers_json),
            response_body: Some(response_body),
            duration_ms: Some(duration_ms),
            error: None,
        };

        let _ = ctx.state.repository.create_request_log(log_req).await;
    });

    // Convert the receiver into a stream
    let body_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    let body = Body::from_stream(body_stream);

    // Build response with original headers
    let mut response = Response::new(body);
    *response.status_mut() =
        StatusCode::from_u16(status_code as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    // Forward response headers
    for (key, value) in headers.iter() {
        if let Ok(header_name) = axum::http::HeaderName::from_bytes(key.as_str().as_bytes())
            && let Ok(header_value) = axum::http::HeaderValue::from_bytes(value.as_bytes())
        {
            response.headers_mut().insert(header_name, header_value);
        }
    }

    Ok(response)
}

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
        if key_str != "authorization"
            && key_str != "host"
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

    // Build request headers map for logging
    let request_headers_map: std::collections::HashMap<String, String> = headers
        .iter()
        .filter_map(|(k, v)| v.to_str().ok().map(|val| (k.to_string(), val.to_string())))
        .collect();

    // Execute request
    let result = req_builder.send().await;

    match result {
        Ok(resp) => {
            let status = resp.status();
            let headers = resp.headers().clone();

            // Check if this is a streaming response (SSE)
            // let is_streaming = headers
            //     .get("content-type")
            //     .and_then(|v| v.to_str().ok())
            //     .map(|ct| ct.contains("text/event-stream") || ct.contains("stream"))
            //     .unwrap_or(false);
            let is_streaming = true;

            let ctx = LogContext {
                state: state.clone(),
                user_id: proxy_key.user_id,
                proxy_key_id: proxy_key.id,
                platform_id,
                method: method.clone(),
                path: path.clone(),
                request_headers_map: request_headers_map.clone(),
                request_body: body.clone(),
                target_url: target_url.clone(),
                outgoing_headers: outgoing_headers.clone(),
                outgoing_body: outgoing_body.clone(),
                start,
            };

            if is_streaming {
                // Handle streaming response
                handle_streaming_response(resp, status, headers, ctx).await
            } else {
                // Handle non-streaming response (original logic)
                handle_regular_response(resp, status, headers, ctx).await
            }
        }
        Err(e) => {
            // Log error
            let duration_ms = start.elapsed().as_millis() as i64;
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
                response_status: None,
                response_headers: None,
                response_body: None,
                duration_ms: Some(duration_ms),
                error: Some(e.to_string()),
            };

            tokio::spawn(async move {
                let _ = state.repository.create_request_log(log_req).await;
            });

            Err(AppError::bad_gateway(
                "proxy::proxy_handler",
                &e.to_string(),
            ))
        }
    }
}
