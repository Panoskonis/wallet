use crate::database::DbPool;
use crate::models::transaction_models;
use crate::models::user_models;
use crate::queries::transaction_queries;
use crate::queries::user_queries;
use serde_json::{Value, json};
use std::str::FromStr;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
/// Create a new user endpoint
/// Accepts a JSON body with email, name, and password
/// Returns the created user's name on success
#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
}
pub async fn create_user_handler(
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
pub async fn get_user_handler(
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

pub async fn get_users_handler(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
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

pub async fn create_transaction_handler(
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

pub async fn get_transactions_handler(
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

pub async fn get_amount_handler(
    State(state): State<AppState>,
    where_clause_params: Query<transaction_models::TransactionGetParameters>,
) -> Result<Json<Value>, StatusCode> {
    let transaction_get_params = where_clause_params.0;
    let user_id = transaction_get_params.user_id;
    if let None = user_id {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
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
        end_timestamp,
    )
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    return Ok(Json(json!({
        "message": "Transactions sum retrieved successfully",
        "amount": money_sum
    })));
}
