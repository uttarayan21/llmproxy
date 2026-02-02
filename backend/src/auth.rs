use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use crate::{repository::Repository, models::User, AppState};

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
    let username = headers
        .get(REMOTE_USER_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Get or create user
    let user = state
        .repository
        .get_or_create_user(username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Insert user into request extensions
    req.extensions_mut().insert(AuthUser { user });

    Ok(next.run(req).await)
}
