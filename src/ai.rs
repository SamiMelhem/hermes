use anyhow::Result;
use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
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

    pub async fn completion(&self, model: &str, prompt: &str) -> Result<String> {
        let url = "https://openrouter.ai/api/v1/chat/completions";
        
        let req = ChatRequest {
            model: model.to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://hermes-ai.local") // OpenRouter requirement
            .header("X-Title", "Hermes AI Harness")           // OpenRouter requirement
            .json(&req)
            .send()
            .await?
            .json::<ChatResponse>()
            .await?;

        Ok(response.choices[0].message.content.clone())
    }
}
