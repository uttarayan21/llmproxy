use axum::{
    Router, middleware,
    routing::{delete, get, post},
};
use backend::{AppState, auth, db, handlers, proxy, repository::Repository};
use std::env;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get configuration from environment
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:llmproxy.db".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    // Setup database
    let pool = db::create_pool(&database_url).await?;
    db::run_migrations(&pool).await?;

    let repository = Repository::new(pool);
    let app_state = AppState { repository };

    // Build API routes (protected by auth middleware)
    let api_routes = Router::new()
        .route("/api/user", get(handlers::get_current_user))
        .route("/api/platforms", post(handlers::create_llm_platform))
        .route("/api/platforms", get(handlers::get_llm_platforms))
        .route("/api/platforms/:id", delete(handlers::delete_llm_platform))
        .route("/api/keys", post(handlers::create_proxy_api_key))
        .route("/api/keys", get(handlers::get_proxy_api_keys))
        .route("/api/keys/:id", delete(handlers::delete_proxy_api_key))
        .route("/api/logs", get(handlers::get_request_logs))
        .route("/api/logs/:id", get(handlers::get_request_log))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth::auth_middleware,
        ));

    // Proxy routes (uses API key authentication, not reverse proxy auth)
    let proxy_routes = Router::new().route(
        "/proxy/:platform_id/*path",
        post(proxy::proxy_handler)
            .get(proxy::proxy_handler)
            .put(proxy::proxy_handler)
            .delete(proxy::proxy_handler)
            .patch(proxy::proxy_handler),
    );

    // Build application
    let app = Router::new()
        .merge(api_routes)
        .merge(proxy_routes)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Start server
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
