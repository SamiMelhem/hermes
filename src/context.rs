use crate::ai::{AgentMessage, LlmMessage, Role, Model};

pub struct ContextManager {
    messages: Vec<AgentMessage>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn add_message(&mut self, msg: AgentMessage) {
        self.messages.push(msg);
    }

    pub fn get_messages(&self) -> &Vec<AgentMessage> {
        &self.messages
    }

    pub fn to_llm_payload(&self, _model: &Model) -> Vec<LlmMessage> {
        let mut llm_messages = Vec::new();

        for msg in &self.messages {
            match msg {
                AgentMessage::User(content) => {
                    llm_messages.push(LlmMessage {
                        role: Role::User,
                        content: Some(content.clone()),
                        tool_calls: None,
                        tool_call_id: None,
                        name: None,
                    });
                }
                AgentMessage::Assistant(am) => {
                    llm_messages.push(LlmMessage {
                        role: Role::Assistant,
                        content: am.content.clone(),
                        tool_calls: am.tool_calls.clone(),
                        tool_call_id: None,
                        name: None,
                    });
                }
                AgentMessage::Tool(tool_result) => {
                    llm_messages.push(LlmMessage {
                        role: Role::Tool,
                        content: Some(tool_result.result.clone()),
                        tool_calls: None,
                        tool_call_id: Some(tool_result.call_id.clone()),
                        name: Some(tool_result.name.clone()),
                    });
                }
                // Artifacts and Notifications might just be system or user messages, or ignored by LLM depending on design.
                // For now, let's include them as Assistant messages if they are Artifacts, and System if Notifications.
                AgentMessage::Artifact(data) => {
                    llm_messages.push(LlmMessage {
                        role: Role::Assistant,
                        content: Some(format!("Generated Artifact: {}\n\n{}", data.title, data.content)),
                        tool_calls: None,
                        tool_call_id: None,
                        name: None,
                    });
                }
                AgentMessage::Notification(_text) => {
                    // Notifications might not be sent to LLM, but if we do, maybe System?
                    // Let's skip them for LLM payload for now, as they are for the User/TUI.
                }
            }
        }

        llm_messages
    }
}
