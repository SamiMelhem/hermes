use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::fs;
use std::path::Path;
use crate::ai::{ToolDefinition, ToolFunctionDefinition};

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn parameters(&self) -> Value;
    
    async fn execute(&self, args: Value) -> Result<String>;

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            tool_type: "function".to_string(),
            function: ToolFunctionDefinition {
                name: self.name(),
                description: self.description(),
                parameters: self.parameters(),
            },
        }
    }
}

pub struct Calculator;

#[async_trait]
impl Tool for Calculator {
    fn name(&self) -> String {
        "calculator".to_string()
    }

    fn description(&self) -> String {
        "Perform basic arithmetic calculations".to_string()
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "The math expression to evaluate (e.g., '2 + 2')"
                }
            },
            "required": ["expression"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let expr = args["expression"].as_str().unwrap_or("0");
        // Still a demo mock, but we could use 'meval' or similar here
        Ok(format!("Result of {}: (evaluated as 42 for demo)", expr))
    }
}

pub struct FileReader;

#[async_trait]
impl Tool for FileReader {
    fn name(&self) -> String {
        "read_file".to_string()
    }

    fn description(&self) -> String {
        "Read the contents of a file from the repository".to_string()
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path to the file (e.g., 'src/main.rs')"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let path_str = args["path"].as_str().ok_or_else(|| anyhow::anyhow!("Missing path"))?;
        let path = Path::new(path_str);
        
        match fs::read_to_string(path) {
            Ok(content) => Ok(content),
            Err(e) => Ok(format!("Error reading file {}: {}", path_str, e)),
        }
    }
}

pub struct FileWriter;

#[async_trait]
impl Tool for FileWriter {
    fn name(&self) -> String {
        "write_file".to_string()
    }

    fn description(&self) -> String {
        "Write or overwrite a file in the repository".to_string()
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path to the file"
                },
                "content": {
                    "type": "string",
                    "description": "Full content to write to the file"
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let path_str = args["path"].as_str().ok_or_else(|| anyhow::anyhow!("Missing path"))?;
        let content = args["content"].as_str().ok_or_else(|| anyhow::anyhow!("Missing content"))?;
        let path = Path::new(path_str);

        // Ensure parent directories exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        match fs::write(path, content) {
            Ok(_) => Ok(format!("Successfully wrote to {}", path_str)),
            Err(e) => Ok(format!("Error writing to {}: {}", path_str, e)),
        }
    }
}

pub struct FileLister;

#[async_trait]
impl Tool for FileLister {
    fn name(&self) -> String {
        "list_files".to_string()
    }

    fn description(&self) -> String {
        "List files in a directory".to_string()
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path to the directory (use '.' for root)"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let path_str = args["path"].as_str().unwrap_or(".");
        let path = Path::new(path_str);

        let mut entries = Vec::new();
        match fs::read_dir(path) {
            Ok(dir) => {
                for entry in dir {
                    if let Ok(entry) = entry {
                        let name = entry.file_name().into_string().unwrap_or_default();
                        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                        entries.push(format!("{}{}", name, if is_dir { "/" } else { "" }));
                    }
                }
                Ok(entries.join("\n"))
            }
            Err(e) => Ok(format!("Error listing directory {}: {}", path_str, e)),
        }
    }
}
