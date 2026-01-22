// Module declarations - these tell Rust where to find our code modules
mod config;
mod database;
mod handlers;
mod models;
mod queries;

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use serde_json::{Value, json};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
// Import our modules
use crate::config::Config;
use crate::database::{create_pool, health_check, run_migrations};
/// Application state shared across all req handlers
/// This allows handlers to access the database pool without global variables

/// Health check endpoint - returns 200 OK if the server is running
/// This is useful for load balancers and monitoring systems
async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "message": "Wallet API is running"
    }))
}

/// Database health check endpoint - verifies database connectivity
/// Returns 200 OK if database is accessible, 503 Service Unavailable otherwise
async fn db_health(State(state): State<handlers::AppState>) -> Result<Json<Value>, StatusCode> {
    match health_check(&state.db).await {
        Ok(_) => Ok(Json(json!({
            "status": "ok",
            "database": "connected"
        }))),
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

/// Main entry point of the application
/// Sets up the Axum web server, routes, middleware, and starts listening
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from environment variables
    let config = Config::from_env()?;

    // Initialize logging based on RUST_LOG environment variable
    // This allows controlling log verbosity (debug, info, warn, error)
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&config.rust_log))
        .init();

    println!("🚀 Starting Wallet API server...");
    println!("📊 Connecting to database...");

    // Create database connection pool
    // The pool manages multiple connections efficiently
    let db_pool = create_pool(&config.database_url).await?;
    println!("✅ Database connection established");

    // Run database migrations
    // Migrations create and update database schema (tables, indexes, etc.)
    println!("📦 Running database migrations...");
    run_migrations(&db_pool).await?;

    // Create application state with the database pool
    // This state will be shared across all req handlers
    let app_state = handlers::AppState { db: db_pool };

    // Build the Axum router
    // Routes define which handler functions respond to which URL paths
    let app = Router::new()
        // Health check endpoint - no database required
        .route("/health", get(health))
        // Database health check - tests database connectivity
        .route("/health/db", get(db_health))
        // Create user endpoint
        .route("/api/users", post(handlers::create_user_handler))
        .route("/api/users/:email", get(handlers::get_user_handler))
        .route("/api/users", get(handlers::get_users_handler))
        .route(
            "/api/transactions",
            post(handlers::create_transaction_handler),
        )
        .route("/api/transactions", get(handlers::get_transactions_handler))
        .route(
            "/api/transactions/amount",
            get(handlers::get_amount_handler),
        )
        // Add CORS middleware to allow cross-origin requests
        // This is important for web applications making API calls
        .layer(CorsLayer::permissive())
        // Attach application state to the router
        // This makes the database pool available to all handlers
        .with_state(app_state);

    // Create socket address from host and port
    // Parse the host string (e.g., "0.0.0.0") into an IP address
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid address {}:{} - {}", config.host, config.port, e))?;
    println!("🌐 Server listening on http://{}", addr);

    // Create a listener for graceful shutdown
    // This allows the server to finish handling requests before shutting down
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Start the server with graceful shutdown support
    // The server will run until it receives a shutdown signal (Ctrl+C)
    axum::serve(listener, app).await?;

    Ok(())
}
