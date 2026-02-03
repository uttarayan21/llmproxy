# Agent Guidelines for LLMPROXY

This document provides guidelines for AI coding agents working on the LLMPROXY codebase.

## Project Overview

LLMPROXY is a Rust-based observability platform for LLM APIs. It consists of:
- **Backend**: Axum web framework with SQLite database (Rust)
- **Frontend**: Yew WebAssembly framework (Rust)
- **Database**: SQLite with migrations in `backend/migrations/`
- **Build Tool**: Uses `just` command runner for task automation

## Build, Lint, and Test Commands

### Using Just (Recommended)

```bash
# Build
just build              # Production build
just build-dev          # Development build
just check              # Type check without building

# Run
just run                # Run backend (production)
just dev                # Run with auto-reload (requires cargo-watch)

# Test
just test               # Run all tests
just test-verbose       # Run tests with output
cd backend && cargo test test_name           # Run single test
cd backend && cargo test test_name -- --show-output  # With output

# Code Quality
just fmt                # Format all code
just lint               # Run clippy
just fix                # Auto-fix clippy warnings

# Database
just db-setup           # Initialize database
just db-backup          # Backup database
just db-schema          # View schema
```

### Manual Commands

```bash
# Backend
cd backend && cargo build --release
cd backend && cargo test
cd backend && cargo clippy -- -D warnings

# Frontend
cd frontend && trunk build --release
cd frontend && cargo test

# Single Test (Examples)
cd backend && cargo test test_generate_api_key -- --nocapture
cd backend && cargo test test_hash_api_key_consistency -- --show-output
cd backend && cargo test error::tests:: -- --nocapture  # Run all tests in error module
```

## Code Style Guidelines

### Import Organization

Follow this three-tier order:
1. Crate-internal imports (`use crate::...`)
2. External crate imports (alphabetically)
3. Standard library imports (`use std::...`)

```rust
use crate::{AppState, models::*};
use axum::{extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
```

### Naming Conventions

- **Functions**: `snake_case` with verb-object pattern (`create_user`, `get_platforms`)
- **Types**: `PascalCase` (`User`, `LlmPlatform`, `ProxyApiKey`)
- **Request/Response DTOs**: `CreateXRequest`, `XResponse` pattern
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Module files**: `snake_case` (e.g., `api_key.rs`, `request_logs.rs`)

### Type Patterns

- Use `Option<T>` for nullable fields (especially database columns)
- Repository layer: Return `anyhow::Result<T>`
- Handler layer: Return `Result<Json<T>, AppError>` or `Result<StatusCode, AppError>`
- Always derive traits in this order: `#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]`

### Error Handling

The codebase uses a structured error type `AppError` that implements `axum::response::IntoResponse`. This provides rich error information including location, error type, message, and HTTP status code.

**Repository Pattern** (return anyhow::Result):
```rust
pub async fn create_user(&self, username: &str) -> Result<User> {
    let result = sqlx::query("INSERT INTO users (username) VALUES (?)")
        .bind(username)
        .execute(&self.pool)
        .await?;
    // ... return user
}
```

**Handler Pattern** (use AppError for structured error responses):
```rust
use crate::error::AppError;

pub async fn create_platform(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateRequest>,
) -> Result<Json<Platform>, AppError> {
    let platform = state
        .repository
        .create_platform(auth_user.user.id, req)
        .await
        .map_err(|e| {
            AppError::internal_server_error(
                "handlers::create_platform",
                &format!("Failed to create platform: {}", e),
            )
        })?;
    Ok(Json(platform))
}
```

**AppError Convenience Constructors**:
```rust
// 500 Internal Server Error
AppError::internal_server_error("location", "error message")

// 404 Not Found
AppError::not_found("location", "Resource name")

// 401 Unauthorized
AppError::unauthorized("location", "auth failed message")

// 400 Bad Request
AppError::bad_request("location", "validation error")

// 502 Bad Gateway
AppError::bad_gateway("location", "upstream error")

// 405 Method Not Allowed
AppError::method_not_allowed("location")
```

**Error Response Format** (JSON):
```json
{
  "location": "handlers::create_platform",
  "error": "Internal Server Error",
  "message": "Failed to create platform: database connection error"
}
```

**Fire-and-forget** (explicitly ignore non-critical errors):
```rust
let _ = state.repository.log_request(data).await;
```

### Database Patterns

- Use raw SQL with `sqlx::query` or `sqlx::query_as`
- Always use parameter binding with `?` placeholders
- Fetch methods: `fetch_one()`, `fetch_optional()`, `fetch_all()`
- Multi-line queries use raw string literals (`r#"..."`#)

```rust
let users = sqlx::query_as::<_, User>(
    "SELECT * FROM users WHERE username = ? ORDER BY created_at DESC",
)
.bind(username)
.fetch_all(&self.pool)
.await?;
```

### Async/Await

- Always use explicit `async fn` (never use `async { }` blocks in signatures)
- Each `.await` on its own line for readability
- Use `tokio::spawn` for background tasks (backend)
- Use `wasm_bindgen_futures::spawn_local` for async in frontend

### Handler Patterns

**Standard signature** (extractors in this order):
```rust
pub async fn handler_name(
    State(state): State<AppState>,           // 1. State
    Extension(auth_user): Extension<AuthUser>, // 2. Extensions
    Path(id): Path<i64>,                      // 3. Path params
    Query(params): Query<QueryParams>,        // 4. Query params
    Json(req): Json<RequestType>,             // 5. Body
) -> Result<Json<ResponseType>, StatusCode>
```

**Boolean to status code pattern**:
```rust
if deleted {
    Ok(StatusCode::NO_CONTENT)
} else {
    Err(StatusCode::NOT_FOUND)
}
```

### Comments

- Use `//` for inline comments (not `///` doc comments)
- Keep comments brief and focused on "why" not "what"
- Use section comments to organize large files: `// LLM Platform handlers`
- No excessive documentation - let code be self-documenting

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_behavior() {
        let result = function_under_test();
        assert_eq!(result, expected_value);
    }
}
```

## Architecture Patterns

### Project Structure

```
backend/src/
├── main.rs          # Application entry, router setup
├── lib.rs           # Module exports, AppState
├── handlers.rs      # HTTP handlers (thin layer)
├── proxy.rs         # Proxy request handler
├── repository.rs    # Database operations (thick layer)
├── models.rs        # Data structures
├── auth.rs          # Authentication middleware
├── api_key.rs       # API key utilities
├── db.rs            # Database connection setup
└── embedded.rs      # Frontend asset embedding

frontend/src/
├── lib.rs           # App component, routing
└── components/      # Yew components
```

### Key Principles

1. **Thin handlers, thick repository**: Business logic in repository layer
2. **Explicit error mapping**: Convert all repository errors to HTTP status codes in handlers
3. **Type safety**: Use strong types, avoid stringly-typed APIs
4. **Security**: API keys are hashed (SHA256), platform keys stored plain-text
5. **Per-platform API keys**: Each API key is linked to exactly one LLM platform

## Important Notes

- Frontend builds are embedded into the backend binary via `rust-embed`
- Database migrations are in `backend/migrations/` (run sequentially)
- API keys format: `llmp_` + 32-char UUID (without hyphens)
- Proxy endpoint: `/proxy/*path` (platform determined by API key)
- Auth: UI uses `Remote-User` header, proxy uses `Authorization: Bearer` tokens
- The backend binary is located in `$CARGO_TARGET_DIR` not the usual `target` folder
- Always update the openapi.json file whenever you modify or update any of the routes.
- **Streaming**: Proxy automatically detects and forwards streaming responses (SSE) from LLM platforms
  - Detection: Checks `Content-Type` header for `text/event-stream` or `stream`
  - Implementation: Uses `reqwest::bytes_stream()` and Axum `Body::from_stream()`
  - Logging: Collects chunks during streaming, logs complete response after stream ends
  - Context struct: `LogContext` groups parameters to avoid too_many_arguments clippy warning

## Common Tasks

**Add new API endpoint**:
1. Add handler in `backend/src/handlers.rs`
2. Add repository method in `backend/src/repository.rs`
3. Register route in `backend/src/main.rs`
4. Update models in `backend/src/models.rs` if needed

**Add database column**:
1. Create migration in `backend/migrations/XXX_name.sql`
2. Update model structs in `backend/src/models.rs`
3. Update repository queries in `backend/src/repository.rs`
4. Run `just db-setup` or apply migration manually

**Add frontend component**:
1. Create component in `frontend/src/components/`
2. Export from `frontend/src/components/mod.rs`
3. Use in `frontend/src/lib.rs` (App component)
4. Frontend auto-rebuilds when backend compiles
