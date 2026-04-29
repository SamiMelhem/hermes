mod db;
mod ai;

use chrono::Utc;
use clap::{Parser, Subcommand};
use anyhow::Result;
use dotenvy::dotenv;

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
    /// View the history of AI tasks
    History,
}

const DATABASE_URL: &str = "sqlite://hermes.db?mode=rwc";

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env
    dotenv().ok();

    // Init db
    let pool = db::init_db(DATABASE_URL).await?;

    // Parse CLI arguments
    let cli = Cli::parse();

    // Handle commands
    match &cli.command {
        Commands::Run { prompt, model } => {
            let model_name = model.as_deref().unwrap_or("gpt-4o-mini");
            println!("🚀 Running prompt on {}: \"{}\"", model_name, prompt);

            // Save initial task to db with "Processing" status
            let result = sqlx::query(
                "INSERT INTO tasks (prompt, model, status, created_at)
                 VALUES (?,?,?,?)"
            )
            .bind(prompt)
            .bind(model_name)
            .bind("Processing")
            .bind(Utc::now())
            .execute(&pool)
            .await?;

            let task_id = result.last_insert_rowid();
            println!("✅ Task created (ID: {}). Status: Processing", task_id);

            let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY not found");
            let ai_client = ai::AiClient::new(api_key);
            
            println!("📡 Contacting model...");
            let response_text = ai_client.completion(model_name, prompt).await?;

            println!("\n🤖 Response:\n{}", response_text);

            // save task to db
            sqlx::query(
                "INSERT INTO tasks (prompt, model, response, status, created_at)
                 VALUES (?,?,?,?,?)"
            )
            .bind(prompt)
            .bind(model_name)
            .bind(&response_text)
            .bind("Completed") // Assume done (for now)
            .bind(Utc::now())
            .execute(&pool)
            .await?;

            println!("\n✅ Task saved to database.");
        }
        Commands::Status => {
            println!("📡 Hermes Runtime: ONLINE");
            println!("📦 Database: SQLite (local)");
        }
        Commands::History => {
            // Add logic for retrieving previous conversatio history
        }
    }

    Ok(())
}

