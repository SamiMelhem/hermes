#[derive(Debug, Clone)]
pub enum AgentEvent {
    AgentStart,
    AgentEnd,
    TurnStart,
    TurnEnd,
    MsgStart,
    MsgUpdate(String),
    MsgEnd,
    ToolStart(String), // tool name
    ToolEnd(String),   // tool result or status
    Error(String),
}
