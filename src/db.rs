use sqlx::{sqlite::SqlitePool, FromRow};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use anyhow::Result;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub prompt: String,
    pub model: String,
    pub response: Option<String>,
    pub status: String, // "Pending", "Processing", "Completed", "Error"
    pub created_at: DateTime<Utc>,
    // files referenced: String (filepath),
    // scripts referenced: String (filepath),
}

pub async fn init_db(database_url: &str) -> Result<SqlitePool> {
    let pool = SqlitePool::connect(database_url).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            prompt TEXT NOT NULL,
            model TEXT NOT NULL,
            response TEXT,
            status TEXT NOT NULL,
            created_at DATETIME NOT NULL
        )"
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}