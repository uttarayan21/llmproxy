use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    #[serde(skip)]
    pub status: StatusCode,
    pub location: String,
    pub error: String,
    pub message: String,
}

impl AppError {
    pub fn new(status: StatusCode, location: &str, error: &str, message: &str) -> Self {
        Self {
            status,
            location: location.to_string(),
            error: error.to_string(),
            message: message.to_string(),
        }
    }

    // Convenience constructors for common errors
    pub fn internal_server_error(location: &str, message: &str) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            location,
            "Internal Server Error",
            message,
        )
    }

    pub fn not_found(location: &str, resource: &str) -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            location,
            "Not Found",
            &format!("{} not found", resource),
        )
    }

    pub fn unauthorized(location: &str, message: &str) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, location, "Unauthorized", message)
    }

    pub fn bad_request(location: &str, message: &str) -> Self {
        Self::new(StatusCode::BAD_REQUEST, location, "Bad Request", message)
    }

    pub fn bad_gateway(location: &str, message: &str) -> Self {
        Self::new(StatusCode::BAD_GATEWAY, location, "Bad Gateway", message)
    }

    pub fn method_not_allowed(location: &str) -> Self {
        Self::new(
            StatusCode::METHOD_NOT_ALLOWED,
            location,
            "Method Not Allowed",
            "The HTTP method is not supported for this endpoint",
        )
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} at {}: {}",
            self.status.as_u16(),
            self.error,
            self.location,
            self.message
        )
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status;
        (status, Json(self)).into_response()
    }
}

// Allow converting anyhow::Error to AppError
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::internal_server_error("unknown", &err.to_string())
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub location: String,
}

impl From<AppError> for ErrorResponse {
    fn from(err: AppError) -> Self {
        Self {
            error: err.error,
            message: err.message,
            location: err.location,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_creation() {
        let error = AppError::internal_server_error("test_location", "test message");
        assert_eq!(error.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.location, "test_location");
        assert_eq!(error.error, "Internal Server Error");
        assert_eq!(error.message, "test message");
    }

    #[test]
    fn test_not_found_error() {
        let error = AppError::not_found("handlers::get_user", "User");
        assert_eq!(error.status, StatusCode::NOT_FOUND);
        assert_eq!(error.location, "handlers::get_user");
        assert_eq!(error.error, "Not Found");
        assert_eq!(error.message, "User not found");
    }

    #[test]
    fn test_unauthorized_error() {
        let error = AppError::unauthorized("proxy::handler", "Invalid token");
        assert_eq!(error.status, StatusCode::UNAUTHORIZED);
        assert_eq!(error.location, "proxy::handler");
        assert_eq!(error.error, "Unauthorized");
        assert_eq!(error.message, "Invalid token");
    }

    #[test]
    fn test_display_format() {
        let error = AppError::bad_request("api::validate", "Missing field");
        let display = format!("{}", error);
        assert!(display.contains("400"));
        assert!(display.contains("Bad Request"));
        assert!(display.contains("api::validate"));
        assert!(display.contains("Missing field"));
    }

    #[test]
    fn test_error_serialization() {
        let error = AppError::bad_gateway("proxy::forward", "Connection refused");
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("Bad Gateway"));
        assert!(json.contains("proxy::forward"));
        assert!(json.contains("Connection refused"));
        // Status should not be serialized
        assert!(!json.contains("502"));
    }
}
