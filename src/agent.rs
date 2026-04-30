use anyhow::Result;
use std::sync::Arc;
use sqlx::{SqlitePool, Row};
use chrono::Utc;
use tokio::sync::mpsc;
use crate::ai::{AiClient, Model, AgentMessage, AssistantMessage, ToolResult, ToolCall};
use crate::tools::Tool;
use crate::context::ContextManager;
use crate::events::AgentEvent;

pub struct Agent {
    client: AiClient,
    model: Model,
    tools: Vec<Arc<dyn Tool>>,
    context: ContextManager,
    pool: SqlitePool,
    task_id: i64,
}

impl Agent {
    pub fn new(client: AiClient, model: Model, pool: SqlitePool, task_id: i64) -> Self {
        Self {
            client,
            model,
            tools: Vec::new(),
            context: ContextManager::new(),
            pool,
            task_id,
        }
    }

    pub fn add_tool(&mut self, tool: Arc<dyn Tool>) {
        self.tools.push(tool);
    }

    /// Load existing history for this task from the database
    pub async fn load_history(&mut self) -> Result<()> {
        let rows = sqlx::query(
            "SELECT role, content, tool_calls, tool_call_id, name FROM messages WHERE task_id = ? ORDER BY id ASC"
        )
        .bind(self.task_id)
        .fetch_all(&self.pool)
        .await?;

        for row in rows {
            let role_str: String = row.get("role");
            let content: Option<String> = row.get("content");

            let msg = match role_str.as_str() {
                "user" => AgentMessage::User(content.unwrap_or_default()),
                "assistant" => {
                    let tool_calls_str: Option<String> = row.get("tool_calls");
                    let tool_calls: Option<Vec<ToolCall>> = tool_calls_str
                        .and_then(|tc| serde_json::from_str(&tc).ok());
                    AgentMessage::Assistant(AssistantMessage {
                        content,
                        tool_calls,
                    })
                },
                "tool" => {
                    AgentMessage::Tool(ToolResult {
                        call_id: row.get::<Option<String>, _>("tool_call_id").unwrap_or_default(),
                        name: row.get::<Option<String>, _>("name").unwrap_or_default(),
                        result: content.unwrap_or_default(),
                    })
                },
                _ => AgentMessage::User(content.unwrap_or_default()), // Fallback
            };
            self.context.add_message(msg);
        }
        Ok(())
    }

    async fn on_message_start(&self, turn_count: usize) -> Result<()> {
        println!("🚀 [Turn {}] Starting message completion...", turn_count);
        Ok(())
    }

    async fn on_message_end(&self, turn_count: usize) -> Result<()> {
        println!("✅ [Turn {}] Message completion finished.", turn_count);
        Ok(())
    }

    async fn before_tool_call(&self, call: &ToolCall) -> Result<()> {
        println!("🛠️  [Hook] About to execute tool: {}", call.function.name);
        Ok(())
    }

    async fn after_tool_call(&self, call: &ToolCall, result: &str) -> Result<()> {
        println!("✅ [Hook] Tool {} finished with {} chars of output.", call.function.name, result.len());
        Ok(())
    }

    async fn save_message(&self, msg: &AgentMessage) -> Result<()> {
        let (role_str, content, tool_calls_json, tool_call_id, name) = match msg {
            AgentMessage::User(c) => ("user", Some(c.clone()), None, None, None),
            AgentMessage::Assistant(am) => ("assistant", am.content.clone(), am.tool_calls.as_ref().and_then(|t| serde_json::to_string(t).ok()), None, None),
            AgentMessage::Tool(t) => ("tool", Some(t.result.clone()), None, Some(t.call_id.clone()), Some(t.name.clone())),
            AgentMessage::Artifact(a) => ("assistant", Some(a.content.clone()), None, None, None),
            AgentMessage::Notification(n) => ("system", Some(n.clone()), None, None, None),
        };

        sqlx::query(
            "INSERT INTO messages (task_id, role, content, tool_calls, tool_call_id, name, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(self.task_id)
        .bind(role_str)
        .bind(&content)
        .bind::<Option<String>>(tool_calls_json)
        .bind(&tool_call_id)
        .bind(&name)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn prompt(&mut self, text: &str, tx: mpsc::Sender<AgentEvent>) -> Result<()> {
        let _ = tx.send(AgentEvent::AgentStart).await;

        let msg = AgentMessage::User(text.to_string());
        self.save_message(&msg).await?;
        self.context.add_message(msg);

        self.run_loop(tx.clone()).await?;

        let _ = tx.send(AgentEvent::AgentEnd).await;
        Ok(())
    }

    async fn run_loop(&mut self, tx: mpsc::Sender<AgentEvent>) -> Result<()> {
        let mut turn_count = 0;
        let max_turns = 10;

        loop {
            if turn_count >= max_turns {
                let _ = tx.send(AgentEvent::Error("Max turns reached".to_string())).await;
                return Ok(());
            }
            turn_count += 1;
            let _ = tx.send(AgentEvent::TurnStart).await;

            let tool_definitions = if self.tools.is_empty() {
                None
            } else {
                Some(self.tools.iter().map(|t| t.definition()).collect())
            };

            let _ = tx.send(AgentEvent::MsgStart).await;
            
            let payload = self.context.to_llm_payload(&self.model);
            
            // "on_message_start" hook
            self.on_message_start(turn_count).await?;

            let response = self.client.completion(&self.model, payload, tool_definitions).await?;
            
            // Tells user that the model is thinking
            let _ = tx.send(AgentEvent::MsgUpdate("Thinking...".to_string())).await;

            // Handle the response
            // Save and add assistant response to history (always, to keep track of tool calls)
            let msg = AgentMessage::Assistant(AssistantMessage
            {
                content: response.content.clone(),
                tool_calls: response.tool_calls.clone(),
            });

            self.save_message(&msg).await?;
            self.context.add_message(msg);

            if let Some(content) = &response.content {
                let _ = tx.send(AgentEvent::MsgUpdate(content.clone())).await;
                let _ = tx.send(AgentEvent::MsgEnd).await;
            }

            // This is where "on_message_end" hook would go
            self.on_message_end(turn_count).await?;

            if let Some(tool_calls) = &response.tool_calls {
                // Hack to add the tool call to history so next completion knows it happened
                // We'll need a better way to represent LLM's tool calls in AgentMessage
                // For now, let's just proceed to execute them.
                
                for call in tool_calls {
                    let _ = tx.send(AgentEvent::ToolStart(call.function.name.clone())).await;
                    
                    // "before_tool_call" hook
                    self.before_tool_call(call).await?;
                    
                    let tool_result = self.execute_tool_call(call).await?;
                    
                    // "after_tool_call" hook
                    self.after_tool_call(call, &tool_result).await?;

                    let _ = tx.send(AgentEvent::ToolEnd(tool_result.clone())).await;

                    let tool_msg = AgentMessage::Tool(ToolResult {
                        call_id: call.id.clone(),
                        name: call.function.name.clone(),
                        result: tool_result,
                    });

                    self.save_message(&tool_msg).await?;
                    self.context.add_message(tool_msg);
                }
                let _ = tx.send(AgentEvent::TurnEnd).await;
                continue;
            } else {
                let _ = tx.send(AgentEvent::TurnEnd).await;
                return Ok(());
            }
        }
    }

    async fn execute_tool_call(&self, call: &ToolCall) -> Result<String> {
        let tool = self.tools.iter().find(|t| t.name() == call.function.name);
        
        match tool {
            Some(t) => {
                let args: serde_json::Value = serde_json::from_str(&call.function.arguments)?;
                t.execute(args).await
            }
            None => Ok(format!("Error: Tool {} not found", call.function.name)),
        }
    }
}
