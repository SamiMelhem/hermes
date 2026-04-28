mod db;

use chrono::Utc;
use clap::{Parser, Subcommand};
use anyhow::Result;

/// Hermes: The High-Velocity AI Harness
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a prompt through the AI harness
    Run {
        /// The prompt to send to the model
        #[arg(short, long)]
        prompt: String,

        /// Optional model override (defaults to a fast model)
        #[arg(short, long)]
        model: Option<String>,
    },
    /// Check the status of the local runtime
    Status,
}

const DATABASE_URL: &str = "sqlite://hermes.db?mode=rwc";

#[tokio::main]
async fn main() -> Result<()> {
    // Init db
    let pool = db::init_db(DATABASE_URL).await?;

    // Parse CLI arguments
    let cli = Cli::parse();

    // Handle commands
    match &cli.command {
        Commands::Run { prompt, model } => {
            let model_name = model.as_deref().unwrap_or("gpt-4o-mini");
            println!("🚀 Running prompt on {}: \"{}\"", model_name, prompt);
            
            println!("✅ Task complete.");

            // save task to db
            sqlx::query(
                "INSERT INTO tasks (prompt, model, status, created_at)
                 VALUES (?,?,?,?)"
            )
            .bind(prompt)
            .bind(model_name)
            .bind("Completed") // Assume done (for now)
            .bind(Utc::now())
            .execute(&pool)
            .await?;

            println!("✅ Task saved to database.");

            // This is where we'll eventually call OpenRouter
        }
        Commands::Status => {
            println!("📡 Hermes Runtime: ONLINE");
            println!("📦 Database: SQLite (local)");
        }
    }

    Ok(())
}

