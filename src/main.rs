// Module declarations - these tell Rust where to find our code modules
mod config;
mod database;
mod models;
mod queries;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use serde_json::{Value, json};
use std::{net::SocketAddr, str::FromStr};
use tower_http::cors::CorsLayer;

// Import our modules
use crate::database::{DbPool, create_pool, health_check, run_migrations};
use crate::models::transaction_models;
use crate::models::user_models;
use crate::queries::user_queries;
use crate::{config::Config, queries::transaction_queries};

/// Application state shared across all req handlers
/// This allows handlers to access the database pool without global variables
#[derive(Clone)]
struct AppState {
    db: DbPool,
}

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
async fn db_health(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    match health_check(&state.db).await {
        Ok(_) => Ok(Json(json!({
            "status": "ok",
            "database": "connected"
        }))),
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

/// Example endpoint that queries the database
/// This demonstrates how to use the connection pool in a req handler
async fn example_query(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // Execute a simple query using SQLx
    // The `?` operator propagates errors - if the query fails, return 500
    let result: (i64,) = sqlx::query_as("SELECT $1 as value")
        .bind(42i64)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "message": "Database query successful",
        "result": result.0
    })))
}

/// Create a new user endpoint
/// Accepts a JSON body with email, name, and password
/// Returns the created user's name on success
async fn create_user_handler(
    State(state): State<AppState>,
    Json(req): Json<user_models::CreateUserRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Create a User instance from the req
    let user = user_models::UserCreate::new(req.email, req.name, req.password);

    // Insert the user into the database
    let name = user_queries::create_user(&state.db, &user)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "message": "User created successfully",
        "name": name
    })))
}

/// Get a user by name endpoint
/// Accepts name as a path parameter (URL-encoded if it contains spaces)
/// Returns user data if found, 404 if not found
async fn get_user_handler(
    State(state): State<AppState>,
    Path(email): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // Axum's Path extractor automatically URL-decodes the parameter
    // So "John%20Doe" becomes "John Doe"
    eprintln!("Looking for user with email: '{}'", email);

    let user = user_queries::get_user(&state.db, &email)
        .await
        .map_err(|e| {
            eprintln!("Error fetching user '{}': {}", email, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(json!({
        "message": "User retrieved successfully",
        "user": {
            "email": user.email,
            "name": user.name,
            "created_at": user.created_at.to_rfc3339(),
            "updated_at": user.updated_at.to_rfc3339()
        }
    })))
}

async fn get_users_handler(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // Axum's Path extractor automatically URL-decodes the parameter
    // So "John%20Doe" becomes "John Doe"
    eprintln!("Fetching all users");

    let users = user_queries::get_all_users(&state.db).await.map_err(|e| {
        eprintln!("Error fetching users: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(json!({
        "message": "Users retrieved successfully",
        "users": users
    })))
}

async fn create_transaction_handler(
    State(state): State<AppState>,
    Json(req): Json<transaction_models::CreateTransactionRequest>,
) -> Result<Json<Value>, StatusCode> {
    eprintln!("Received transaction request: {:?}", req);

    // Validate and convert transaction type
    let transaction_type = transaction_models::TransactionType::from_str(&req.transaction_type)
        .map_err(|e| {
            eprintln!("Invalid transaction type: {} - {}", req.transaction_type, e);
            StatusCode::BAD_REQUEST
        })?;

    // Validate and convert category (default to Other if not provided)
    let category = match req.category {
        Some(cat_str) => Some(
            transaction_models::TransactionCategory::from_str(&cat_str).map_err(|e| {
                eprintln!("Invalid category: {} - {}", cat_str, e);
                StatusCode::BAD_REQUEST
            })?,
        ),
        None => None,
    };

    eprintln!(
        "Parsed transaction_type: {:?}, category: {:?}",
        transaction_type, category
    );

    // Get user
    let user = user_queries::get_user(&state.db, &req.user_email)
        .await
        .map_err(|e| {
            eprintln!("Error fetching user '{}': {}", req.user_email, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Create transaction with validated enums
    let transaction = transaction_models::TransactionCreate::new(
        user.id,
        transaction_type,
        req.amount,
        category,
        req.description,
    );

    transaction_queries::create_transaction(&state.db, &transaction)
        .await
        .map_err(|e| {
            eprintln!("Error creating transaction: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(json!({
        "message": "Transaction created successfully"
    })))
}

async fn get_transactions_handler(
    State(state): State<AppState>,
    where_clause_params: Query<transaction_models::TransactionGetParameters>,
) -> Result<Json<Value>, StatusCode> {
    let transaction_get_params = where_clause_params.0;
    let user_id = transaction_get_params.user_id;
    let category = match transaction_get_params.category {
        Some(strr) => match transaction_models::TransactionCategory::from_str(&strr) {
            Ok(cat) => Some(cat),
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
        None => None,
    };
    let transaction_type = match transaction_get_params.transaction_type {
        Some(strr) => match transaction_models::TransactionType::from_str(&strr) {
            Ok(trans) => Some(trans),
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
        None => None,
    };
    let amount_min = transaction_get_params.amount_min;
    let amount_max = transaction_get_params.amount_max;

    let start_timestamp = transaction_get_params.start_timestamp;
    let end_timestamp = transaction_get_params.end_timestamp;

    let transactions = transaction_queries::get_transactions(
        &state.db,
        user_id,
        category,
        transaction_type,
        amount_min,
        amount_max,
        start_timestamp,
        end_timestamp,
    )
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    println!("{transactions:?}");
    return Ok(Json(json!({
        "message": "Transactions retrieved successfully",
        "users": transactions
    })));
}

async fn get_amount_handler(State(state): State<AppState>,
where_clause_params: Query<transaction_models::TransactionGetParameters>)-> Result<Json<Value>, StatusCode>{
    let transaction_get_params = where_clause_params.0;
    let user_id = transaction_get_params.user_id;
    if let None = user_id {
        return Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
    let category = match transaction_get_params.category {
        Some(strr) => match transaction_models::TransactionCategory::from_str(&strr) {
            Ok(cat) => Some(cat),
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
        None => None,
    };
    let transaction_type = match transaction_get_params.transaction_type {
        Some(strr) => match transaction_models::TransactionType::from_str(&strr) {
            Ok(trans) => Some(trans),
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
        None => None,
    };

    let start_timestamp = transaction_get_params.start_timestamp;
    let end_timestamp = transaction_get_params.end_timestamp;
    let money_sum = transaction_queries::get_user_transaction_sum(
        &state.db,
        user_id.unwrap(),
        category,
        transaction_type,
        start_timestamp,
        end_timestamp
    ).await.map_err(|e| {
        eprintln!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    return Ok(Json(json!({
        "message": "Transactions sum retrieved successfully",
        "users": money_sum
    })))

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
    let app_state = AppState { db: db_pool };

    // Build the Axum router
    // Routes define which handler functions respond to which URL paths
    let app = Router::new()
        // Health check endpoint - no database required
        .route("/health", get(health))
        // Database health check - tests database connectivity
        .route("/health/db", get(db_health))
        // Example endpoint demonstrating database queries
        .route("/api/example", get(example_query))
        // Create user endpoint
        .route("/api/users", post(create_user_handler))
        .route("/api/users/:email", get(get_user_handler))
        .route("/api/users", get(get_users_handler))
        .route("/api/transactions", post(create_transaction_handler))
        .route("/api/transactions", get(get_transactions_handler))
        .route("/api/transactions/amount", get(get_amount_handler))
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
