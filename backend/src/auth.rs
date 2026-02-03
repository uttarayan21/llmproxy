use crate::{AppState, error::AppError, models::User, repository::Repository};
use axum::{
    async_trait,
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use axum_login::{AuthnBackend, UserId};
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
        if let Some(user) = user {
            if let Some(ref password_hash) = user.password_hash {
                // Verify password
                if verify_password(creds.password, password_hash).is_ok() {
                    return Ok(Some(user));
                }
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
    // Check if user is already authenticated via session (axum_login)
    if let Some(auth_session) = req.extensions().get::<axum_login::AuthSession<Backend>>() {
        if let Some(user) = auth_session.user.clone() {
            req.extensions_mut().insert(AuthUser { user });
            return Ok(next.run(req).await);
        }
    }

    // Fall back to Remote-User header (for reverse proxy auth)
    if let Some(header_value) = headers.get(REMOTE_USER_HEADER) {
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

        req.extensions_mut().insert(AuthUser { user });
        return Ok(next.run(req).await);
    }

    // For development: use environment variable or default
    // let dev_user = std::env::var("DEV_DEFAULT_USER").unwrap_or_else(|_| "dev-user".to_string());
    // let user = state
    //     .repository
    //     .get_or_create_user(&dev_user)
    //     .await
    //     .map_err(|e| {
    //         AppError::internal_server_error(
    //             "auth::auth_middleware",
    //             &format!("Failed to get or create user: {}", e),
    //         )
    //     })?;
    //
    // req.extensions_mut().insert(AuthUser { user });
    Ok(next.run(req).await)
}

// Password hashing utility
pub fn hash_password(password: &str) -> Result<String, AppError> {
    generate_hash(password);
    Ok(generate_hash(password))
}
