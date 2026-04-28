# Hermes: The High-Velocity AI Harness

Hermes is an AI orchestration layer designed for extreme efficiency, low-latency task routing, and intelligent memory management. It moves beyond simple wrappers by implementing a lean Rust-based runtime that manages prompt lifecycle, tool execution, and long-term memory with minimal overhead.

## 🚀 Vision
To create a "lean" agentic runtime that handles the heavy lifting of LLM orchestration—routing, memory, and tool usage—while maintaining sub-millisecond local latency and optimized token usage through automatic prompt condensing.

## 🏗️ Architecture

### 1. Low Latency Runtime (The "Pi" Philosophy)
- **Core**: Built in Rust for maximum performance and memory safety.
- **Execution**: Minimalist CLI and background daemon to handle persistent state.
- **Storage**: SQLite for local task logs, tool definitions, and short-term memory.

### 2. Intelligent Task Routing
- **Provider**: OpenRouter for unified access to the best-in-class models.
- **Router**: NVIDIA LLM Router to dynamically dispatch prompts based on complexity, cost, and latency requirements.

### 3. Efficient Tool Usage
- **Framework**: Integration with OpenAI Agents SDK.
- **Principles**: Implementing Anthropic's tool-use best practices (pre-computation, XML-tagging for clarity, and iterative refinement).

### 4. Memory & Context Optimization
- **Self-Condensing Prompts**: Utilizing Gemini's context window capabilities to condense long threads into "semantic checkpoints."
- **Memory Service**: A hybrid approach using `Claude-mem` patterns and local vector storage for long-term retrieval.

---

## 🛠️ Tech Stack

| Component | Technology |
| :--- | :--- |
| **Core Runtime** | Rust |
| **Database** | SQLite |
| **API Gateway** | OpenRouter |
| **Routing** | NVIDIA LLM Router |
| **Agent Logic** | OpenAI Agents SDK / Anthropic Principles |
| **Memory/CLI** | Gemini CLI + Pi + Claude-mem |
| **Experiments** | Python (Embeddings/Retrieval) |
| **UI (Optional)** | Tauri + TypeScript (Desktop App) |

---

## 📅 Implementation Plan

### Phase 1: Foundation (The Rust Core)
- [ ] Initialize Rust project structure.
- [ ] Set up SQLite schema for task tracking and tool registration.
- [ ] Implement basic OpenRouter integration for completions.

### Phase 2: Intelligent Routing
- [ ] Integrate NVIDIA LLM Router to categorize tasks (Simple vs. Complex).
- [ ] Implement fallback logic and cost-aware routing.

### Phase 3: Tool Execution Layer
- [ ] Define a standard `Tool` trait in Rust.
- [ ] Integrate OpenAI Agents SDK for standardized tool calls.
- [ ] Implement Anthropic-style prompt engineering for tool reliability.

### Phase 4: Memory & Prompt Compression
- [ ] Implement the "Self-Condensing" logic using Gemini.
- [ ] Integrate `Claude-mem` style long-term memory retrieval.
- [ ] Develop the "Pi" minimal runtime for background processing.

### Phase 5: Polish & UI
- [ ] (Optional) Develop a Tauri-based desktop dashboard.
- [ ] CLI enhancements (TUI, progress bars, logs).

