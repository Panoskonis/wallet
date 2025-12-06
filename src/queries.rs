pub mod user_queries {
    use crate::database::DbPool;
    use crate::models::user_models as user;
    use anyhow::anyhow;
    use chrono::{DateTime, Utc};
    use sqlx::Row;
    use sqlx::postgres::PgRow;
    use uuid::Uuid;

    pub async fn create_user(pool: &DbPool, user: &user::UserCreate) -> anyhow::Result<String> {
        sqlx::query("INSERT INTO users (email, name, password) VALUES ($1, $2, $3)")
            .bind(&user.email)
            .bind(&user.name)
            .bind(&user.password)
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
    use sqlx::Row;
    use sqlx::postgres::PgRow;
    use sqlx::{QueryBuilder};
    use std::str::FromStr;
    use uuid::Uuid;

    pub async fn create_transaction(
        pool: &DbPool,
        transaction: &transaction::TransactionCreate,
    ) -> anyhow::Result<String> {
        let result = sqlx::query("INSERT INTO transactions (user_id,transaction_type,amount,category,description) VALUES ($1,$2::transaction_type,$3,$4,$5)")
            .bind(&transaction.user_id)
            .bind(&transaction.transaction_type.to_string())
            .bind(&transaction.amount)
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
                let transaction_type_string: &str = row.try_get("transaction_type")?;
                let transaction_type = TransactionType::from_str(transaction_type_string);
                let transaction_type = match transaction_type {
                    Ok(transaction) => transaction,
                    Err(e) => {
                        return Err(anyhow!(
                            "Could not convert {transaction_type_string} to TransactionType enum: {e}"
                        ));
                    }
                };
                let categore_string: &str = row.try_get("category")?;
                let category = TransactionCategory::from_str(categore_string);
                let category = match category {
                    Ok(cat) => cat,
                    Err(e) => {
                        return Err(anyhow!(
                            "Could not convert {categore_string} to TransactionCategory enum: {e}"
                        ));
                    }
                };
                let description: String = row.try_get("desription")?;
                let created_at: DateTime<Utc> = row.try_get("created_at")?;
                let updated_at: DateTime<Utc> = row.try_get("updated_at")?;
                let amount: f64 = row.try_get("amount")?;
                return Ok(transaction::TransactionQuery::new(
                    id,
                    user_id,
                    transaction_type,
                    amount,
                    category,
                    description,
                    created_at,
                    updated_at,
                ));
            }
            None => return Err(anyhow!("Provided row is None")),
        }
    }

    fn push_where_or_and <DB>(query: & mut QueryBuilder<DB>, where_is_inserted: & mut bool)-> ()
    where DB: sqlx::Database
    {
        if ! *where_is_inserted {
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
        period: Option<(DateTime<Utc>, DateTime<Utc>)>,
    ) -> anyhow::Result<Vec<transaction::TransactionQuery>> {
        let mut query = QueryBuilder::new("SELECT * FROM transactions");
        let mut where_is_inserted = false;
        if let Some(user_id) = user_id {
            push_where_or_and(&mut query, &mut where_is_inserted);
            query.push(" user_id = ?").push_bind(user_id);
            
        }
        if let Some(category) = category {
            push_where_or_and(&mut query, &mut where_is_inserted);
            query.push(" category = ?").push_bind(category.to_string());
        }
        if let Some(transaction_type) = transaction_type {
            push_where_or_and(&mut query, &mut where_is_inserted);
            query.push(" category = ?").push_bind(transaction_type.to_string());
        }
        if let Some(period) = period {
            push_where_or_and(&mut query, &mut where_is_inserted);
            query.push(" created_at >= ?").push_bind(period.0);
            query.push(" created_at <= ?").push_bind(period.1);
        }
    let query = query.build();
    let transactions = query.fetch_all(pool).await?;
    let trans = transactions.into_iter().map(|r| map_row_to_transaction(Some(r))).collect::<anyhow::Result<Vec<transaction::TransactionQuery>>>();
    return trans
    }
}
