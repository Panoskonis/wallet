pub mod user_models {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UserCreate {
        pub email: String,
        pub name: String,
        pub password: String,
    }

    impl UserCreate {
        pub fn new(email: String, name: String, password: String) -> Self {
            Self {
                email,
                name,
                password,
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UserQuery {
        pub id: Uuid,
        pub email: String,
        pub name: String,
        pub password: String,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }
    impl UserQuery {
        pub fn new(
            id: Uuid,
            email: String,
            name: String,
            password: String,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        ) -> Self {
            Self {
                id,
                email,
                name,
                password,
                created_at,
                updated_at,
            }
        }
    }
    #[derive(serde::Deserialize)]
    pub struct CreateUserRequest {
        pub email: String,
        pub name: String,
        pub password: String,
    }
}

pub mod transaction_models {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;
    use uuid::Uuid;

    // Simple enums for internal type safety
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum TransactionType {
        Expense,
        Income,
    }

    impl ToString for TransactionType {
        fn to_string(&self) -> String {
            match self {
                TransactionType::Expense => "Expense".to_string(),
                TransactionType::Income => "Income".to_string(),
            }
        }
    }

    impl FromStr for TransactionType {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "Expense" => Ok(TransactionType::Expense),
                "Income" => Ok(TransactionType::Income),
                _ => Err(format!("Invalid transaction type: {}", s)),
            }
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum TransactionCategory {
        Groceries,
        Restaurant,
        Housing,
        Holidays,
        Shopping,
        Entertainment,
        Other,
    }

    impl ToString for TransactionCategory {
        fn to_string(&self) -> String {
            match self {
                TransactionCategory::Groceries => "Groceries".to_string(),
                TransactionCategory::Restaurant => "Restaurant".to_string(),
                TransactionCategory::Housing => "Housing".to_string(),
                TransactionCategory::Holidays => "Holidays".to_string(),
                TransactionCategory::Shopping => "Shopping".to_string(),
                TransactionCategory::Entertainment => "Entertainment".to_string(),
                TransactionCategory::Other => "Other".to_string(),
            }
        }
    }

    impl FromStr for TransactionCategory {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "Groceries" => Ok(TransactionCategory::Groceries),
                "Restaurant" => Ok(TransactionCategory::Restaurant),
                "Housing" => Ok(TransactionCategory::Housing),
                "Holidays" => Ok(TransactionCategory::Holidays),
                "Shopping" => Ok(TransactionCategory::Shopping),
                "Entertainment" => Ok(TransactionCategory::Entertainment),
                "Other" => Ok(TransactionCategory::Other),
                _ => Err(format!("Invalid transaction category: {}", s)),
            }
        }
    }

    // Internal struct with type-safe enums
    #[derive(Debug, Clone)]
    pub struct TransactionCreate {
        pub user_id: Uuid,
        pub transaction_type: TransactionType,
        pub amount: f64,
        pub category: TransactionCategory,
        pub description: String,
    }

    impl TransactionCreate {
        pub fn new(
            user_id: Uuid,
            transaction_type: TransactionType,
            amount: f64,
            category: Option<TransactionCategory>,
            description: Option<String>,
        ) -> Self {
            Self {
                user_id,
                transaction_type,
                amount,
                category: category.unwrap_or(TransactionCategory::Other),
                description: description.unwrap_or_default(),
            }
        }
    }

    // API request struct - accepts simple strings
    #[derive(Deserialize, Debug)]
    pub struct CreateTransactionRequest {
        pub user_email: String,
        pub transaction_type: String,
        pub amount: f64,
        pub category: Option<String>,
        pub description: Option<String>,
    }

    #[derive(Deserialize, Debug, Serialize)]
    pub struct TransactionQuery {
        id: Uuid,
        user_id: Uuid,
        transaction_type: TransactionType,
        amount: f64,
        category: TransactionCategory,
        description: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }
    impl TransactionQuery {
        pub fn new(
            id: Uuid,
            user_id: Uuid,
            transaction_type: TransactionType,
            amount: f64,
            category: TransactionCategory,
            description: String,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        ) -> Self {
            Self {
                id,
                user_id,
                transaction_type,
                amount,
                category,
                description,
                created_at,
                updated_at,
            }
        }
    }
}
