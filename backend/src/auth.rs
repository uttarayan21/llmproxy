use crate::{AppState, error::AppError, models::User, repository::Repository};
use axum::{
    async_trait,
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use axum_login::{AuthSession, AuthnBackend, UserId};
use password_auth::{generate_hash, verify_password};
use sqlx::SqlitePool;

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

/// Parse HTTP Basic Authentication header
fn parse_basic_auth(header_value: &str) -> Option<(String, String)> {
    // Basic auth format: "Basic base64(username:password)"
    let parts: Vec<&str> = header_value.split_whitespace().collect();
    if parts.len() != 2 || parts[0] != "Basic" {
        return None;
    }

    let decoded = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        parts[1],
    ).ok()?;
    
    let decoded_str = String::from_utf8(decoded).ok()?;
    let mut split = decoded_str.splitn(2, ':');
    
    let username = split.next()?.to_string();
    let password = split.next()?.to_string();
    
    Some((username, password))
}

// Middleware that supports multiple authentication methods based on config
pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_config = &state.config.auth;
    
    tracing::debug!("auth_middleware: Starting authentication check");

    // If all authentication is disabled, allow all requests
    if auth_config.disable_all {
        tracing::warn!("auth_middleware: Authentication is disabled, allowing request");
        // Create a dummy user for tracking purposes
        let user = state
            .repository
            .get_or_create_user("anonymous")
            .await
            .map_err(|e| {
                AppError::internal_server_error(
                    "auth::auth_middleware",
                    &format!("Failed to get or create anonymous user: {}", e),
                )
            })?;
        req.extensions_mut().insert(AuthUser { user });
        return Ok(next.run(req).await);
    }

    let mut authenticated_user: Option<User> = None;

    // Try session-based authentication if enabled
    if auth_config.enable_session && authenticated_user.is_none() {
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
    }

    // Try header-based authentication if enabled
    if auth_config.enable_header && authenticated_user.is_none() {
        if let Some(header_value) = headers.get(&auth_config.header_name) {
            tracing::debug!("auth_middleware: Found {} header", auth_config.header_name);
            let username = header_value
                .to_str()
                .map_err(|e| {
                    AppError::bad_request(
                        "auth::auth_middleware",
                        &format!("Invalid {} header: {}", auth_config.header_name, e),
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

            tracing::debug!(
                "auth_middleware: Authenticated via {}: {}",
                auth_config.header_name,
                username
            );
            authenticated_user = Some(user);
        } else {
            tracing::debug!("auth_middleware: No {} header found", auth_config.header_name);
        }
    }

    // Try HTTP Basic Authentication if enabled
    if auth_config.enable_basic && authenticated_user.is_none() {
        if let Some(auth_header) = headers.get("authorization") {
            tracing::debug!("auth_middleware: Found Authorization header");
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some((username, password)) = parse_basic_auth(auth_str) {
                    tracing::debug!("auth_middleware: Parsed basic auth for user: {}", username);
                    
                    // Verify credentials
                    let user = state
                        .repository
                        .get_user_by_username(&username)
                        .await
                        .map_err(|e| {
                            AppError::internal_server_error(
                                "auth::auth_middleware",
                                &format!("Failed to fetch user: {}", e),
                            )
                        })?;

                    if let Some(user) = user
                        && let Some(ref password_hash) = user.password_hash
                    {
                        if verify_password(password, password_hash).is_ok() {
                            tracing::debug!("auth_middleware: Basic auth successful for: {}", username);
                            authenticated_user = Some(user);
                        } else {
                            tracing::debug!("auth_middleware: Basic auth failed - invalid password");
                        }
                    } else {
                        tracing::debug!("auth_middleware: Basic auth failed - user not found or no password");
                    }
                } else {
                    tracing::debug!("auth_middleware: Failed to parse basic auth header");
                }
            }
        } else {
            tracing::debug!("auth_middleware: No Authorization header found");
        }
    }

    // If we have an authenticated user, insert AuthUser extension and continue
    if let Some(user) = authenticated_user {
        tracing::debug!(
            "auth_middleware: Authentication successful for user: {}",
            user.username
        );
        req.extensions_mut().insert(AuthUser { user });
        return Ok(next.run(req).await);
    }

    // No authentication method succeeded
    tracing::debug!("auth_middleware: No authentication method succeeded, returning 401");
    
    // Build helpful error message based on enabled methods
    let mut enabled_methods = Vec::new();
    if auth_config.enable_session {
        enabled_methods.push("session (login)".to_string());
    }
    if auth_config.enable_header {
        enabled_methods.push(format!("header ({})", auth_config.header_name));
    }
    if auth_config.enable_basic {
        enabled_methods.push("basic auth".to_string());
    }
    
    let method_str = if enabled_methods.is_empty() {
        "No authentication methods are enabled".to_string()
    } else {
        format!("Enabled methods: {}", enabled_methods.join(", "))
    };
    
    Err(AppError::unauthorized(
        "auth::auth_middleware",
        &format!("Authentication required. {}", method_str),
    ))
}

// Password hashing utility
pub fn hash_password(password: &str) -> Result<String, AppError> {
    Ok(generate_hash(password))
}
