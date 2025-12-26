//! LLM Agent integration using Radkit
//!
//! Provides structured LLM outputs for code generation tasks.

use anyhow::{anyhow, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Result of an LLM code generation request
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CodeGenerationResult {
    /// Files that were created or modified
    pub files_modified: Vec<FileChange>,
    /// Agent's confidence in the solution (0.0 - 1.0)
    pub confidence: f32,
    /// Brief explanation of changes made
    pub explanation: String,
    /// Whether the agent believes the task is complete
    pub task_complete: bool,
}

/// A file change made by the agent
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct FileChange {
    /// Path relative to workspace root
    pub path: String,
    /// Type of change
    pub change_type: ChangeType,
    /// New content for the file (for create/modify)
    pub content: Option<String>,
}

/// Type of file change
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum ChangeType {
    Create,
    Modify,
    Delete,
}

/// LLM Provider selection
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum LlmProvider {
    OpenAI,
    Anthropic,
    Gemini,
    /// Manual mode - use external CLI agent
    Manual,
}

impl Default for LlmProvider {
    fn default() -> Self {
        LlmProvider::OpenAI
    }
}

/// Configuration for LLM agent
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub provider: LlmProvider,
    pub model: Option<String>,
    pub manual_command: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::OpenAI,
            model: None,
            manual_command: None,
        }
    }
}

/// LLM Agent for code generation
pub struct LlmAgent {
    config: AgentConfig,
}

impl LlmAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }

    /// Run the agent with a prompt and get structured output
    pub async fn run(&self, prompt: &str, _workspace_root: &Path) -> Result<CodeGenerationResult> {
        match self.config.provider {
            LlmProvider::Manual => {
                // Fallback to shell command
                self.run_manual(prompt).await
            }
            LlmProvider::OpenAI => self.run_openai(prompt).await,
            LlmProvider::Anthropic => self.run_anthropic(prompt).await,
            LlmProvider::Gemini => self.run_gemini(prompt).await,
        }
    }

    async fn run_openai(&self, prompt: &str) -> Result<CodeGenerationResult> {
        use radkit::agent::LlmFunction;
        use radkit::models::providers::OpenAILlm;

        let model = self.config.model.as_deref().unwrap_or("gpt-4o");
        let llm = OpenAILlm::from_env(model).map_err(|e| {
            anyhow!(
                "Failed to initialize OpenAI: {}. Set OPENAI_API_KEY env var.",
                e
            )
        })?;

        let agent = LlmFunction::<CodeGenerationResult>::new(llm);
        let result = agent
            .run(prompt)
            .await
            .map_err(|e| anyhow!("OpenAI request failed: {}", e))?;

        Ok(result)
    }

    async fn run_anthropic(&self, prompt: &str) -> Result<CodeGenerationResult> {
        use radkit::agent::LlmFunction;
        use radkit::models::providers::AnthropicLlm;

        let model = self
            .config
            .model
            .as_deref()
            .unwrap_or("claude-sonnet-4-20250514");
        let llm = AnthropicLlm::from_env(model).map_err(|e| {
            anyhow!(
                "Failed to initialize Anthropic: {}. Set ANTHROPIC_API_KEY env var.",
                e
            )
        })?;

        let agent = LlmFunction::<CodeGenerationResult>::new(llm);
        let result = agent
            .run(prompt)
            .await
            .map_err(|e| anyhow!("Anthropic request failed: {}", e))?;

        Ok(result)
    }

    async fn run_gemini(&self, prompt: &str) -> Result<CodeGenerationResult> {
        use radkit::agent::LlmFunction;
        use radkit::models::providers::GeminiLlm;

        let model = self.config.model.as_deref().unwrap_or("gemini-1.5-pro");
        let llm = GeminiLlm::from_env(model).map_err(|e| {
            anyhow!(
                "Failed to initialize Gemini: {}. Set GEMINI_API_KEY env var.",
                e
            )
        })?;

        let agent = LlmFunction::<CodeGenerationResult>::new(llm);
        let result = agent
            .run(prompt)
            .await
            .map_err(|e| anyhow!("Gemini request failed: {}", e))?;

        Ok(result)
    }

    async fn run_manual(&self, prompt: &str) -> Result<CodeGenerationResult> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let cmd = self
            .config
            .manual_command
            .as_deref()
            .ok_or_else(|| anyhow!("Manual mode requires --agent-cmd flag"))?;

        let mut child = Command::new(if cfg!(windows) { "powershell" } else { "sh" })
            .args(if cfg!(windows) {
                vec!["-Command", cmd]
            } else {
                vec!["-c", cmd]
            })
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(prompt.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Try to parse structured output from stdout
        if let Ok(result) = serde_json::from_str::<CodeGenerationResult>(&stdout) {
            return Ok(result);
        }

        // Fallback: Create a basic result
        Ok(CodeGenerationResult {
            files_modified: vec![],
            confidence: if output.status.success() { 0.8 } else { 0.0 },
            explanation: stdout.to_string(),
            task_complete: output.status.success(),
        })
    }
}
