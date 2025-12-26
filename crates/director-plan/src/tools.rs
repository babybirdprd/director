//! Tools for LLM agents
//!
//! These tools can be called by agents to interact with the filesystem and run tests.

use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// Tool for reading a file from the workspace
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadFileTool {
    /// Path relative to workspace root
    pub path: String,
}

impl ReadFileTool {
    pub fn execute(&self, workspace_root: &Path) -> Result<String> {
        let full_path = workspace_root.join(&self.path);
        std::fs::read_to_string(&full_path)
            .with_context(|| format!("Failed to read file: {}", self.path))
    }
}

/// Tool for writing a file to the workspace
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WriteFileTool {
    /// Path relative to workspace root
    pub path: String,
    /// Content to write
    pub content: String,
}

impl WriteFileTool {
    pub fn execute(&self, workspace_root: &Path) -> Result<()> {
        let full_path = workspace_root.join(&self.path);

        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&full_path, &self.content)
            .with_context(|| format!("Failed to write file: {}", self.path))
    }
}

/// Tool for running tests
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RunTestsTool {
    /// Optional path to specific test file or directory
    pub path: Option<String>,
    /// Optional test filter pattern
    pub filter: Option<String>,
}

impl RunTestsTool {
    pub fn execute(&self, workspace_root: &Path) -> Result<TestResult> {
        let mut cmd = Command::new("cargo");
        cmd.arg("test");
        cmd.current_dir(workspace_root);

        if let Some(path) = &self.path {
            cmd.arg("--test").arg(path);
        }

        if let Some(filter) = &self.filter {
            cmd.arg("--").arg(filter);
        }

        let output = cmd.output().context("Failed to run cargo test")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(TestResult {
            success: output.status.success(),
            stdout,
            stderr,
            exit_code: output.status.code().unwrap_or(-1),
        })
    }
}

/// Result from running tests
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TestResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Tool for running arbitrary shell commands
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ShellTool {
    /// Command to run
    pub command: String,
}

impl ShellTool {
    pub fn execute(&self, workspace_root: &Path) -> Result<TestResult> {
        let output = if cfg!(windows) {
            Command::new("powershell")
                .args(["-Command", &self.command])
                .current_dir(workspace_root)
                .output()?
        } else {
            Command::new("sh")
                .args(["-c", &self.command])
                .current_dir(workspace_root)
                .output()?
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(TestResult {
            success: output.status.success(),
            stdout,
            stderr,
            exit_code: output.status.code().unwrap_or(-1),
        })
    }
}

/// Tool for listing directory contents
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListDirTool {
    /// Path relative to workspace root
    pub path: String,
}

impl ListDirTool {
    pub fn execute(&self, workspace_root: &Path) -> Result<Vec<String>> {
        let full_path = workspace_root.join(&self.path);
        let mut entries = Vec::new();

        for entry in std::fs::read_dir(&full_path)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.file_type()?.is_dir();
            entries.push(if is_dir { format!("{}/", name) } else { name });
        }

        entries.sort();
        Ok(entries)
    }
}

/// Tool for searching code with grep
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GrepTool {
    /// Pattern to search for
    pub pattern: String,
    /// Path to search in (relative to workspace)
    pub path: Option<String>,
}

impl GrepTool {
    pub fn execute(&self, workspace_root: &Path) -> Result<Vec<GrepMatch>> {
        let search_path = self.path.as_deref().unwrap_or(".");

        let output = Command::new("rg")
            .args(["--json", "--no-heading", &self.pattern, search_path])
            .current_dir(workspace_root)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut matches = Vec::new();

        for line in stdout.lines() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                if val["type"] == "match" {
                    if let (Some(path), Some(line_num), Some(text)) = (
                        val["data"]["path"]["text"].as_str(),
                        val["data"]["line_number"].as_u64(),
                        val["data"]["lines"]["text"].as_str(),
                    ) {
                        matches.push(GrepMatch {
                            path: path.to_string(),
                            line_number: line_num as u32,
                            line_content: text.trim().to_string(),
                        });
                    }
                }
            }
        }

        Ok(matches)
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GrepMatch {
    pub path: String,
    pub line_number: u32,
    pub line_content: String,
}

/// Enum of all available tools
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "tool", content = "args")]
pub enum ToolCall {
    ReadFile(ReadFileTool),
    WriteFile(WriteFileTool),
    RunTests(RunTestsTool),
    Shell(ShellTool),
    ListDir(ListDirTool),
    Grep(GrepTool),
}

impl ToolCall {
    pub fn execute(&self, workspace_root: &Path) -> Result<serde_json::Value> {
        match self {
            ToolCall::ReadFile(t) => {
                let content = t.execute(workspace_root)?;
                Ok(serde_json::json!({"content": content}))
            }
            ToolCall::WriteFile(t) => {
                t.execute(workspace_root)?;
                Ok(serde_json::json!({"success": true}))
            }
            ToolCall::RunTests(t) => {
                let result = t.execute(workspace_root)?;
                Ok(serde_json::to_value(result)?)
            }
            ToolCall::Shell(t) => {
                let result = t.execute(workspace_root)?;
                Ok(serde_json::to_value(result)?)
            }
            ToolCall::ListDir(t) => {
                let entries = t.execute(workspace_root)?;
                Ok(serde_json::json!({"entries": entries}))
            }
            ToolCall::Grep(t) => {
                let matches = t.execute(workspace_root)?;
                Ok(serde_json::to_value(matches)?)
            }
        }
    }
}
