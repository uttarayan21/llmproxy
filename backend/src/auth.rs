use crate::{AppState, error::AppError, models::User, repository::Repository};
use axum::{
    async_trait,
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use axum_login::{AuthnBackend, AuthSession, UserId};
use password_auth::{generate_hash, verify_password};
use sqlx::SqlitePool;

pub const REMOTE_USER_HEADER: &str = "Remote-User";

#[derive(Clone)]
pub struct AuthUser {
    pub user: User,
}

// Authentication backend for axum_login
#[derive(Debug, Clone)]
pub struct Backend {
    pub repository: Repository,
}

impl Backend {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            repository: Repository::new(pool),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = AppError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user = self
            .repository
            .get_user_by_username(&creds.username)
            .await
            .map_err(|e| {
                AppError::internal_server_error(
                    "auth::authenticate",
                    &format!("Failed to fetch user: {}", e),
                )
            })?;

        // Check if user exists and has a password
        if let Some(user) = user
            && let Some(ref password_hash) = user.password_hash
        {
            // Verify password
            if verify_password(creds.password, password_hash).is_ok() {
                return Ok(Some(user));
            }
        }

        Ok(None)
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        self.repository.get_user_by_id(*user_id).await.map_err(|e| {
            AppError::internal_server_error(
                "auth::get_user",
                &format!("Failed to fetch user: {}", e),
            )
        })
    }
}

// Middleware that supports both Remote-User header and session-based auth
pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    tracing::debug!("auth_middleware: Starting authentication check");
    
    let mut authenticated_user: Option<User> = None;

    // First, try to get user from axum_login session
    // The AuthSession is stored in extensions by the auth_layer
    if let Some(session) = req.extensions().get::<AuthSession<Backend>>() {
        tracing::debug!("auth_middleware: Found AuthSession in extensions");
        if let Some(user) = &session.user {
            tracing::debug!("auth_middleware: Session has user: {}", user.username);
            authenticated_user = Some(user.clone());
        } else {
            tracing::debug!("auth_middleware: Session exists but no user logged in");
        }
    } else {
        tracing::debug!("auth_middleware: No AuthSession found in extensions");
    }

    // If not authenticated via session, try Remote-User header (for reverse proxy)
    if authenticated_user.is_none() {
        if let Some(header_value) = headers.get(REMOTE_USER_HEADER) {
            tracing::debug!("auth_middleware: Found Remote-User header");
            let username = header_value
                .to_str()
                .map_err(|e| {
                    AppError::bad_request(
                        "auth::auth_middleware",
                        &format!("Invalid Remote-User header: {}", e),
                    )
                })?
                .to_string();

            let user = state
                .repository
                .get_or_create_user(&username)
                .await
                .map_err(|e| {
                    AppError::internal_server_error(
                        "auth::auth_middleware",
                        &format!("Failed to get or create user: {}", e),
                    )
                })?;

            tracing::debug!("auth_middleware: Authenticated via Remote-User: {}", username);
            authenticated_user = Some(user);
        } else {
            tracing::debug!("auth_middleware: No Remote-User header found");
        }
    }

    // If we have an authenticated user, insert AuthUser extension and continue
    if let Some(user) = authenticated_user {
        tracing::debug!("auth_middleware: Authentication successful for user: {}", user.username);
        req.extensions_mut().insert(AuthUser { user });
        return Ok(next.run(req).await);
    }

    // No authentication method succeeded
    tracing::debug!("auth_middleware: No authentication method succeeded, returning 401");
    Err(AppError::unauthorized(
        "auth::auth_middleware",
        "Authentication required. Please login or provide Remote-User header.",
    ))
}

// Password hashing utility
pub fn hash_password(password: &str) -> Result<String, AppError> {
    generate_hash(password);
    Ok(generate_hash(password))
}
