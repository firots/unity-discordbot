use std::env;
use sqlx::SqlitePool;
use crate::Error;
use crate::constans::SQLITE_DATABASE_PATH;

pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub async fn new() -> Self {
        let database_path = env::var(SQLITE_DATABASE_PATH).unwrap();
        let url = format!("sqlite://{}", database_path);
        Self {
            pool: SqlitePool::connect(&url).await.unwrap(),
        }
    }

    pub async fn create_tables_if_needed(&self) -> Result<(), Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS user_gift_codes (
                user_id INTEGER NOT NULL,
                gift_code_key TEXT NOT NULL,
                PRIMARY KEY(user_id, gift_code_key)
            )"
        ).execute(&self.pool).await?;
    
        Ok(())
    }
    
    pub async fn is_user_redeemed_gift_code_in_db(&self, gift_code_key: &String, user_id: u64) -> Result<bool, Error> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM user_gift_codes WHERE user_id = ? AND gift_code_key = ?"
        )
        .bind(user_id as i64)
        .bind(gift_code_key)
        .fetch_one(&self.pool).await?;
    
        Ok(row.0 > 0)
    }
    
    pub async fn redeem_gift_code_in_db(&self, gift_code_key: &String, user_id: u64) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO user_gift_codes (user_id, gift_code_key) VALUES (?, ?)"
        )
        .bind(user_id.to_string())
        .bind(gift_code_key)
        .execute(&self.pool).await?;
    
        Ok(())
    }
}

