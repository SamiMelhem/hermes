mod db;
mod ai;
mod agent;
mod tools;
mod events;
mod context;

use std::sync::Arc;
use chrono::Utc;
use clap::{Parser, Subcommand};
use anyhow::Result;
use dotenvy::dotenv;
use tokio::sync::mpsc;
use agent::Agent;
use ai::Model;
use tools::Calculator;
use events::AgentEvent;

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

        // /// Optional model override (defaults to a fast model)
        // #[arg(short, long)]
        // model: Option<String>,

        /// Optional task ID to resume an existing conversation
        #[arg(short, long)]
        task_id: Option<i64>,
    },
    /// Check the status of the local runtime
    Status,
    /// View the history of AI tasks
    History,
}

const DATABASE_URL: &str = "sqlite://hermes.db?mode=rwc";

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let pool = db::init_db(DATABASE_URL).await?;
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { prompt, task_id } => {
            let model = Model::default(); // Using the default model configuration
            
            let active_task_id = if let Some(id) = task_id {
                println!("🔄 Resuming Task ID: {}", id);
                *id
            } else {
                let res = sqlx::query(
                    "INSERT INTO tasks (prompt, model, status, created_at) VALUES (?, ?, ?, ?)"
                )
                .bind(prompt)
                .bind(&model.name)
                .bind("Processing")
                .bind(Utc::now())
                .execute(&pool)
                .await?;
                res.last_insert_rowid()
            };

            let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY not found");
            let ai_client = ai::AiClient::new(api_key);
            
            let mut agent = Agent::new(ai_client, model.clone(), pool.clone(), active_task_id);
            
            // Register tools
            agent.add_tool(Arc::new(Calculator));
            agent.add_tool(Arc::new(tools::FileReader));
            agent.add_tool(Arc::new(tools::FileWriter));
            agent.add_tool(Arc::new(tools::FileLister));

            if task_id.is_some() {
                agent.load_history().await?;
            }

            println!("🚀 Hermes running on {}: \"{}\"", model.name, prompt);

            let (tx, mut rx) = mpsc::channel(100);

            // Run the agent in a spawned task so we can listen to events
            let prompt_clone = prompt.clone();
            let pool_clone = pool.clone();
            
            let handle = tokio::spawn(async move {
                agent.run_prompt(&prompt_clone, tx).await
            });

            let mut _final_response = String::new();

            // Listen to events
            while let Some(event) = rx.recv().await {
                match event {
                    AgentEvent::AgentStart => println!("-- Agent Started --"),
                    AgentEvent::AgentEnd => println!("-- Agent Finished --"),
                    AgentEvent::TurnStart => println!("🔄 Turn Started"),
                    AgentEvent::TurnEnd => println!("🔄 Turn Ended"),
                    AgentEvent::MsgStart => print!("🤖 Thinking..."),
                    AgentEvent::MsgUpdate(content) => {
                        // Clear the "Thinking..." or previous dots if we were streaming
                        // But for now let's just print the content on a new line or same line.
                        // Pi core often streams. For now let's just print it.
                        println!("\n\n🤖 Agent Response:\n{}\n", content);
                    },
                    AgentEvent::MsgEnd => (),
                    AgentEvent::ToolStart(name) => println!("🛠️  Tool Started: {}", name),
                    AgentEvent::ToolEnd(result) => {
                        println!("✅ Tool Ended.");
                        _final_response = result; // Hack: keep last tool result if no LLM response follows
                    },
                    AgentEvent::Error(err) => println!("❌ Error: {}", err),
                }
            }

            let _result = handle.await??;

            sqlx::query(
                "UPDATE tasks SET response = ?, status = ? WHERE id = ?"
            )
            .bind("Completed (Check history for full details)")
            .bind("Completed")
            .bind(active_task_id)
            .execute(&pool_clone)
            .await?;

            println!("\n✅ Task {} finalized.", active_task_id);
        }
        Commands::Status => {
            println!("📡 Hermes Runtime: ONLINE");
            println!("🔧 Core: Rust (Pi-style Event-Driven Loop)");
        }
        Commands::History => {
            use sqlx::Row;
            let rows = sqlx::query(
                "SELECT id, prompt, model, response, status, created_at FROM tasks ORDER BY created_at DESC LIMIT 10"
            )
            .fetch_all(&pool)
            .await?;
            
            println!("{:<5} | {:<20} | {:<10} | {:<10}", "ID", "Prompt", "Model", "Status");
            println!("{:-<50}", "");
            for row in rows {
                let id: i64 = row.get("id");
                let prompt: String = row.get("prompt");
                let model: String = row.get("model");
                let status: String = row.get("status");
                println!("{:<5} | {:<20} | {:<10} | {:<10}", id, &prompt[..prompt.len().min(20)], model, status);
            }
        }
    }

    Ok(())
}
