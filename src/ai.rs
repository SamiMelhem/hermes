use anyhow::Result;
use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone)]
pub enum AgentMessage {
    User(String),
    Assistant(AssistantMessage),
    Tool(ToolResult),
    Artifact(ArtifactData),
    Notification(String),
}

#[derive(Debug, Clone)]
pub struct ModelCost {
    pub input_1m: f64,
    pub output_1m: f64,
    pub cache_read_1m: f64,
    pub cache_write_1m: f64,
}

#[derive(Debug, Clone)]
pub struct ModelCompat {
    pub parallel_tool_calling: bool,
    pub system_prompt: bool,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub context_window: usize,
    pub reasoning_level: u8,
    pub cost: ModelCost,
    pub compat: ModelCompat,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            id: "gpt-4o-mini".to_string(),
            name: "GPT-4o Mini".to_string(),
            provider: "OpenRouter".to_string(),
            context_window: 128000,
            reasoning_level: 1,
            cost: ModelCost {
                input_1m: 0.15,
                output_1m: 0.60,
                cache_read_1m: 0.0,
                cache_write_1m: 0.0,
            },
            compat: ModelCompat {
                parallel_tool_calling: true,
                system_prompt: true,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArtifactData {
    pub id: String,
    pub title: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolResult {
    pub call_id: String,
    pub name: String,
    pub result: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ToolFunctionCall,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolFunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LlmMessage {
    pub role: Role,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<LlmMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunctionDefinition,
}

#[derive(Serialize, Debug, Clone)]
pub struct ToolFunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: LlmMessage,
}

pub struct AiClient {
    client: Client,
    api_key: String,
}

impl AiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn completion(&self, model: &Model, messages: Vec<LlmMessage>, tools: Option<Vec<ToolDefinition>>) -> Result<LlmMessage> {
        let url = "https://openrouter.ai/api/v1/chat/completions";
        
        let req = ChatRequest {
            model: model.id.clone(),
            messages,
            tools,
        };

        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://hermes-ai.local")
            .header("X-Title", "Hermes AI Harness")
            .json(&req)
            .send()
            .await?
            .json::<ChatResponse>()
            .await?;

        Ok(response.choices[0].message.clone())
    }
}
