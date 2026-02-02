use crate::{AppState, models::User};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

pub const REMOTE_USER_HEADER: &str = "Remote-User";

#[derive(Clone)]
pub struct AuthUser {
    pub user: User,
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Try to get username from Remote-User header, or use dev default
    let username = if let Some(header_value) = headers.get(REMOTE_USER_HEADER) {
        header_value.to_str().unwrap_or("dev-user").to_string()
    } else {
        // For development: use environment variable or default
        std::env::var("DEV_DEFAULT_USER").unwrap_or_else(|_| "dev-user".to_string())
    };

    // Get or create user
    let user = state
        .repository
        .get_or_create_user(&username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Insert user into request extensions
    req.extensions_mut().insert(AuthUser { user });

    Ok(next.run(req).await)
}
