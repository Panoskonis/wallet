pub mod user_queries {
    use crate::database::DbPool;
    use crate::models::user_models as user;
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

    fn map_row_to_user(row: Option<PgRow>) -> anyhow::Result<Option<user::UserQuery>> {
        match row {
            Some(row) => {
                let id: Uuid = row.try_get("id")?;
                let email: String = row.try_get("email")?;
                let name: String = row.try_get("name")?;
                let password: String = row.try_get("password")?;
                let created_at: DateTime<Utc> = row.try_get("created_at")?;
                let updated_at: DateTime<Utc> = row.try_get("updated_at")?;

                return Ok(Some(user::UserQuery::new(
                    id, email, name, password, created_at, updated_at,
                )));
            }
            None => return Ok(None),
        }
    }

    pub async fn get_user(pool: &DbPool, email: &str) -> anyhow::Result<Option<user::UserQuery>> {
        let row = sqlx::query("SELECT id, email, name, password, created_at, updated_at FROM users WHERE email = $1 LIMIT 1")
        .bind(email)
        .fetch_optional(pool)
        .await?;

        return map_row_to_user(row);
    }

    pub async fn get_all_users(pool: &DbPool) -> anyhow::Result<Vec<Option<user::UserQuery>>> {
        let rows =
            sqlx::query("SELECT id, email, name, password, created_at, updated_at FROM users")
                .fetch_all(pool)
                .await?;

        return rows
            .into_iter()
            .map(|row| map_row_to_user(Some(row)))
            .collect::<anyhow::Result<Vec<Option<user::UserQuery>>>>();
    }
}

pub mod transaction_queries {
    use crate::database::DbPool;
    use crate::models::transaction_models as transaction;
    use chrono::{DateTime, Utc};
    use sqlx::Row;
    use sqlx::postgres::PgRow;
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

        eprintln!(
            "Transaction inserted: {} rows affected",
            result.rows_affected()
        );
        Ok(transaction.user_id.to_string())
    }
}
