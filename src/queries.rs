pub mod user_queries {
    use crate::database::DbPool;
    use crate::models::user_models as user;
    use anyhow::anyhow;

    use argon2::{
        Argon2, PasswordHasher,
        password_hash::{SaltString, rand_core::OsRng},
    };

    use chrono::{DateTime, Utc};
    use sqlx::Row;
    use sqlx::postgres::PgRow;
    use uuid::Uuid;

    pub async fn create_user(pool: &DbPool, user: &user::UserCreate) -> anyhow::Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hashed_pwd = argon2
            .hash_password(user.password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("password hashing failed: {e}"))?
            .to_string();
        sqlx::query("INSERT INTO users (email, name, password) VALUES ($1, $2, $3)")
            .bind(&user.email)
            .bind(&user.name)
            .bind(&hashed_pwd)
            .execute(pool)
            .await?;
        Ok(user.name.clone())
    }

    fn map_row_to_user(row: Option<PgRow>) -> anyhow::Result<user::UserQuery> {
        match row {
            Some(row) => {
                let id: Uuid = row.try_get("id")?;
                let email: String = row.try_get("email")?;
                let name: String = row.try_get("name")?;
                let password: String = row.try_get("password")?;
                let created_at: DateTime<Utc> = row.try_get("created_at")?;
                let updated_at: DateTime<Utc> = row.try_get("updated_at")?;

                return Ok(user::UserQuery::new(
                    id, email, name, password, created_at, updated_at,
                ));
            }
            None => return Err(anyhow!("User could not be created from row")),
        }
    }

    pub async fn get_user(pool: &DbPool, email: &str) -> anyhow::Result<user::UserQuery> {
        let row = sqlx::query("SELECT id, email, name, password, created_at, updated_at FROM users WHERE email = $1 LIMIT 1")
        .bind(email)
        .fetch_optional(pool)
        .await?;

        return map_row_to_user(row);
    }

    pub async fn get_all_users(pool: &DbPool) -> anyhow::Result<Vec<user::UserQuery>> {
        let rows =
            sqlx::query("SELECT id, email, name, password, created_at, updated_at FROM users")
                .fetch_all(pool)
                .await?;

        return rows
            .into_iter()
            .map(|row| map_row_to_user(Some(row)))
            .collect::<anyhow::Result<Vec<user::UserQuery>>>();
    }
}

pub mod transaction_queries {
    use crate::database::DbPool;
    use crate::models::transaction_models::{
        self as transaction, TransactionCategory, TransactionType,
    };
    use anyhow::anyhow;
    use chrono::{DateTime, Utc};
    use rust_decimal::Decimal;
    use sqlx::QueryBuilder;
    use sqlx::postgres::PgRow;
    use sqlx::{Execute, Row};
    use std::str::FromStr;
    use uuid::Uuid;

    pub async fn create_transaction(
        pool: &DbPool,
        transaction: &transaction::TransactionCreate,
    ) -> anyhow::Result<String> {
        let amount = match transaction.transaction_type {
            TransactionType::Expense => (-1.0) * transaction.amount.abs(),
            TransactionType::Income => transaction.amount.abs(),
        };
        let result = sqlx::query("INSERT INTO transactions (user_id,transaction_type,amount,category,description) VALUES ($1,$2::transaction_type,$3,$4,$5)")
            .bind(&transaction.user_id)
            .bind(&transaction.transaction_type.to_string())
            .bind(amount)
            .bind(&transaction.category.to_string())
            .bind(&transaction.description)
            .execute(pool)
            .await?;

        println!(
            "Transaction inserted: {} rows affected",
            result.rows_affected()
        );
        Ok(transaction.user_id.to_string())
    }

    fn map_row_to_transaction(row: Option<PgRow>) -> anyhow::Result<transaction::TransactionQuery> {
        match row {
            Some(row) => {
                let id: Uuid = row.try_get("id")?;
                let user_id: Uuid = row.try_get("user_id")?;
                let transaction_type = row.try_get("transaction_type")?;
                let category_string: &str = row.try_get("category")?;
                let category = TransactionCategory::from_str(category_string);
                let category = match category {
                    Ok(cat) => cat,
                    Err(e) => {
                        return Err(anyhow!(
                            "Could not convert {category_string} to TransactionCategory enum: {e}"
                        ));
                    }
                };
                let description: String = row.try_get("description")?;
                let created_at: DateTime<Utc> = row.try_get("created_at")?;
                let last_updated_at: DateTime<Utc> = row.try_get("last_updated_at")?;
                let amount: Decimal = row.try_get("amount")?;
                return Ok(transaction::TransactionQuery::new(
                    id,
                    user_id,
                    transaction_type,
                    amount,
                    category,
                    description,
                    created_at,
                    last_updated_at,
                ));
            }
            None => return Err(anyhow!("Provided row is None")),
        }
    }

    fn push_where_or_and<DB>(query: &mut QueryBuilder<DB>, where_is_inserted: &mut bool) -> ()
    where
        DB: sqlx::Database,
    {
        if !*where_is_inserted {
            query.push(" WHERE");
            *where_is_inserted = true;
        } else {
            query.push(" AND");
        }
    }

    pub async fn get_transactions(
        pool: &DbPool,
        user_id: Option<Uuid>,
        category: Option<TransactionCategory>,
        transaction_type: Option<TransactionType>,
        amount_min: Option<Decimal>,
        amount_max: Option<Decimal>,
        start_timestamp: Option<DateTime<Utc>>,
        end_timestamp: Option<DateTime<Utc>>,
    ) -> anyhow::Result<Vec<transaction::TransactionQuery>> {
        let mut query = QueryBuilder::new("SELECT * FROM transactions");
        let mut where_is_inserted = false;
        if let Some(user_id) = user_id {
            push_where_or_and(&mut query, &mut where_is_inserted);
            query.push(" user_id = ").push_bind(user_id);
        }
        if let Some(category) = category {
            push_where_or_and(&mut query, &mut where_is_inserted);
            query.push(" category = ").push_bind(category.to_string());
        }
        if let Some(transaction_type) = transaction_type {
            push_where_or_and(&mut query, &mut where_is_inserted);
            query
                .push(" transaction_type = ")
                .push_bind(transaction_type);
        }
        if let Some(start_timestamp) = start_timestamp {
            push_where_or_and(&mut query, &mut where_is_inserted);
            query.push(" created_at >= ").push_bind(start_timestamp);
        }

        if let Some(end_timestamp) = end_timestamp {
            push_where_or_and(&mut query, &mut where_is_inserted);
            query.push(" created_at <= ").push_bind(end_timestamp);
        }
        if let Some(amount_min) = amount_min {
            push_where_or_and(&mut query, &mut where_is_inserted);
            query.push(" amount >= ").push_bind(amount_min);
        }
        if let Some(amount_max) = amount_max {
            push_where_or_and(&mut query, &mut where_is_inserted);
            query.push(" amount <= ").push_bind(amount_max);
        }
        let query = query.build();
        println!("transaction query build {}", query.sql());
        let transactions = query.fetch_all(pool).await?;
        let trans = transactions
            .into_iter()
            .map(|r| map_row_to_transaction(Some(r)))
            .collect::<anyhow::Result<Vec<transaction::TransactionQuery>>>();
        return trans;
    }

    pub async fn get_user_transaction_sum(
        pool: &DbPool,
        user_id: Uuid,
        category: Option<TransactionCategory>,
        transaction_type: Option<TransactionType>,
        start_timestamp: Option<DateTime<Utc>>,
        end_timestamp: Option<DateTime<Utc>>,
    ) -> anyhow::Result<Decimal> {
        let mut total_sum = Decimal::from(0);
        let transactions = get_transactions(
            pool,
            Some(user_id),
            category,
            transaction_type,
            None,
            None,
            start_timestamp,
            end_timestamp,
        )
        .await?;

        for tr in transactions.iter() {
            total_sum += tr.amount;
        }

        return Ok(total_sum);
    }
}
