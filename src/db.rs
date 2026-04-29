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

    // Tasks table for high-level overview
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

    // Messages table for full agentic history (Pi-style)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            role TEXT NOT NULL,
            content TEXT,
            tool_calls TEXT, -- JSON string
            tool_call_id TEXT,
            name TEXT,
            created_at DATETIME NOT NULL,
            FOREIGN KEY(task_id) REFERENCES tasks(id)
        )"
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}