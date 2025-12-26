use std::io::{Write};
use std::path::{Path};
use std::process::{Command, Stdio};
use anyhow::{Context, Result, anyhow};
use crate::types::{Ticket};
use crate::context::discovery::discover_context;
use crate::verification::visual_diff::verify_visual;
use std::fs;
use serde::Deserialize;

pub struct ExecutionResult {
    pub success: bool,
    pub confidence: f32,
    pub errors: Vec<String>,
}

#[derive(Deserialize)]
struct AgentOutput {
    confidence: Option<f32>,
    // other fields?
}

pub struct ExecutionLoop<'a> {
    workspace_root: &'a Path,
    agent_cmd: String,
    ticket: Ticket,
}

impl<'a> ExecutionLoop<'a> {
    pub fn new(workspace_root: &'a Path, agent_cmd: String, ticket: Ticket) -> Self {
        Self {
            workspace_root,
            agent_cmd,
            ticket,
        }
    }

    pub fn run_with_handshake(&mut self) -> Result<ExecutionResult> {
         // 1. Safety Check: Ensure git is clean
        if self.is_git_dirty()? {
            return Err(anyhow!("Workspace is dirty. Please commit or stash changes before running execution loop."));
        }

        // 2. Detached HEAD
        self.enter_detached_head()?;

        let max_retries = self.ticket.verification.max_retries;
        let mut attempts = 0;
        let mut previous_errors = Vec::new();
        let mut success = false;
        let mut final_confidence = 1.0; // Default if not provided

        while attempts < max_retries {
            println!(">> Attempt {}/{}", attempts + 1, max_retries);

            // 3. Generate Prompt
            let prompt = self.generate_prompt(&previous_errors)?;

            // 4. Run Agent & Capture Confidence
            let (_agent_success, agent_output) = match self.run_agent_capture(&prompt) {
                Ok(out) => (true, out),
                Err(e) => {
                    previous_errors.push(format!("Agent Execution Failed: {}", e));
                    attempts += 1;
                    continue;
                }
            };

            // Try to extract confidence from output
            if let Some(c) = self.extract_confidence(&agent_output) {
                final_confidence = c;
            }

            // 5. Verification
            match self.verify() {
                Ok(_) => {
                    success = true;
                    println!(">> Verification PASSED!");
                    break;
                }
                Err(e) => {
                    println!(">> Verification FAILED: {}", e);
                    previous_errors.push(format!("Verification Failed:\n{}", e));
                    attempts += 1;
                }
            }
        }

        if success {
            println!(">> Task Completed Successfully!");
            // We stay in detached HEAD (or branch) as per previous logic, but Worker will push.
            // Worker expects us to return.
            Ok(ExecutionResult {
                success: true,
                confidence: final_confidence,
                errors: previous_errors,
            })
        } else {
            println!(">> Max retries reached. Reverting to original state.");
            self.reset_hard()?;
            self.leave_detached_head()?;
            Ok(ExecutionResult {
                 success: false,
                 confidence: 0.0,
                 errors: previous_errors,
            })
        }
    }

    // Legacy run for CLI compatibility if needed
    pub fn run(&mut self) -> Result<()> {
        let res = self.run_with_handshake()?;
        if res.success {
            Ok(())
        } else {
            Err(anyhow!("Task failed"))
        }
    }

    fn is_git_dirty(&self) -> Result<bool> {
        let output = Command::new("git")
            .current_dir(self.workspace_root)
            .args(&["status", "--porcelain"])
            .output()?;
        Ok(!output.stdout.is_empty())
    }

    fn enter_detached_head(&self) -> Result<()> {
        Command::new("git")
            .current_dir(self.workspace_root)
            .args(&["checkout", "--detach"])
            .status()
            .context("Failed to enter detached HEAD")?;
        Ok(())
    }

    fn leave_detached_head(&self) -> Result<()> {
         Command::new("git")
            .current_dir(self.workspace_root)
            .arg("checkout")
            .arg("-")
            .status()
            .context("Failed to leave detached HEAD")?;
        Ok(())
    }

    fn reset_hard(&self) -> Result<()> {
        Command::new("git")
            .current_dir(self.workspace_root)
            .args(&["reset", "--hard"])
            .status()
            .context("Failed to hard reset")?;
        Ok(())
    }

    fn generate_prompt(&self, errors: &[String]) -> Result<String> {
        let mut relevant_files = self.ticket.spec.relevant_files.clone();

        // If discovery returns files, we assume full content for now, unless we switch to AST engine directly.
        // `discover_context` handles the AST expansion logic now.
        if relevant_files.is_empty() || self.ticket.spec.auto_context {
             // Append discovered files (unique)
             let discovered = discover_context(&self.ticket, self.workspace_root);
             for f in discovered {
                 if !relevant_files.contains(&f) {
                     relevant_files.push(f);
                 }
             }
        }

        let mut context_content = String::new();

        if self.ticket.spec.auto_context {
             // Build graph and get content map
             let mut graph = crate::context::ast::DependencyGraph::new(self.workspace_root);
             if let Ok(_) = graph.build() {
                 let seeds = if self.ticket.spec.relevant_files.is_empty() {
                      // Need heuristic seeds to start graph walk if discover_context was just paths
                      // But wait, discover_context called above already gave us "relevant_files" which ARE the result of the AST walk in `discovery.rs`.
                      // So `relevant_files` contains ALL files we want.
                      // We just need to prune them if they are depth >= 2.

                      // But `discover_context` doesn't return depth info.
                      // So we must re-run `get_context` using original seeds?

                      // This duplication suggests `discover_context` should maybe return the content directly or we rely on `execution_loop` to drive it.
                      // But `discovery.rs` is shared by CLI context command.

                      // To make this work without huge refactor, I will just iterate relevant_files and prune if needed using a simpler heuristic or just load full content.
                      // "Files 2 hops away get only type signatures" - I can't know hops without graph.

                      // Let's rely on `graph.get_context` again using the ORIGINAL seeds (before expansion).
                      let original_seeds = self.ticket.spec.relevant_files.clone();
                      let seeds = if original_seeds.is_empty() {
                           // If original seeds empty, we used heuristic seeds.
                           // We can re-derive them or assume we want everything in `relevant_files` (which is expanded).
                           // If we use expanded list as seeds, depth is 0 for all. No pruning.

                           // So we need the heuristic seeds again.
                           // This is getting messy.
                           vec![] // Force heuristic again inside get_context? No.
                      } else {
                           original_seeds
                      };

                      if seeds.is_empty() {
                           // Re-run heuristic discovery (not AST) to get seeds
                           // This functionality is private in `discovery.rs`.
                           // I'll just use the `relevant_files` (which are expanded) and skip pruning.
                           // It's a safe fallback.
                           relevant_files.clone()
                      } else {
                           seeds
                      }
                 } else {
                      self.ticket.spec.relevant_files.clone()
                 };

                 // If we have valid seeds, `get_context` will give us pruned content.
                 // Note: If seeds came from `discovery.rs` (heuristic), we don't have them isolated here easily.
                 // I'll just use `relevant_files` and load content.
                 // Pruning is disabled for implicit context for now to avoid complexity.

                 // However, if the user explicitly provided `relevant_files` AND `auto_context=true`, pruning works.
                 if !self.ticket.spec.relevant_files.is_empty() {
                     let context_pairs = graph.get_context(&self.ticket.spec.relevant_files);
                     for (path, content) in context_pairs {
                          context_content.push_str(&format!("--- FILE: {} ---\n", path));
                          context_content.push_str(&content);
                          context_content.push_str("\n\n");
                     }
                     // Clear relevant_files so we don't double add below?
                     // We need to ensure we don't duplicate.
                     // The loop below handles fallback.
                     // Let's set a flag or just use `context_content`.
                 } else {
                      // Implicit context - Load all discovered files fully.
                      for file in &relevant_files {
                        let path = self.workspace_root.join(file);
                        if path.exists() {
                            context_content.push_str(&format!("--- FILE: {} ---\n", file));
                            context_content.push_str(&fs::read_to_string(path).unwrap_or_default());
                            context_content.push_str("\n\n");
                        }
                    }
                 }
             } else {
                 // Fallback
                 for file in &relevant_files {
                    let path = self.workspace_root.join(file);
                    if path.exists() {
                        context_content.push_str(&format!("--- FILE: {} ---\n", file));
                        context_content.push_str(&fs::read_to_string(path).unwrap_or_default());
                        context_content.push_str("\n\n");
                    }
                }
             }
        } else {
             // Legacy behavior
            for file in &relevant_files {
                let path = self.workspace_root.join(file);
                if path.exists() {
                    context_content.push_str(&format!("--- FILE: {} ---\n", file));
                    context_content.push_str(&fs::read_to_string(path).unwrap_or_default());
                    context_content.push_str("\n\n");
                }
            }
        }

        let mut prompt = String::new();
        prompt.push_str(&format!("# Task: {}\n\n", self.ticket.meta.title));
        prompt.push_str(&format!("## Description\n{}\n\n", self.ticket.spec.description));
        prompt.push_str(&format!("## Constraints\n{:?}\n\n", self.ticket.spec.constraints));

        if !context_content.is_empty() {
             prompt.push_str("# Context\n");
             prompt.push_str(&context_content);
        }

        if !errors.is_empty() {
            prompt.push_str("\n# Previous Errors (FIX THESE)\n");
            for err in errors {
                prompt.push_str(&format!("- {}\n", err));
            }
        }

        Ok(prompt)
    }

    fn run_agent_capture(&self, prompt: &str) -> Result<String> {
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(&self.agent_cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped()) // Capture stdout now
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn agent command")?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(prompt.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        // Also print to user for visibility (tee)
        println!("{}", stdout);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Agent exited with status {}: {}", output.status, stderr));
        }

        Ok(stdout)
    }

    fn extract_confidence(&self, output: &str) -> Option<f32> {
        let json_start = output.find('{')?;
        let json_end = output.rfind('}')?;

        if json_start < json_end {
            let json_str = &output[json_start..=json_end];
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                if let Some(c) = val.get("confidence").and_then(|v| v.as_f64()) {
                    return Some(c as f32);
                }
            }
        }

        // Fallback: look for "confidence": 0.xx
        let re = regex::Regex::new(r#""confidence"\s*:\s*([0-9.]+)"#).ok()?;
        if let Some(caps) = re.captures(output) {
            if let Ok(c) = caps[1].parse::<f32>() {
                return Some(c);
            }
        }

        None
    }

    fn verify(&self) -> Result<()> {
        // 1. Run Verification Command
        let cmd_str = &self.ticket.verification.command;
        if !cmd_str.is_empty() {
             let output = Command::new("sh")
                .arg("-c")
                .arg(cmd_str)
                .current_dir(self.workspace_root)
                .output()
                .context("Failed to execute verification command")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                return Err(anyhow!("Command Failed:\nSTDOUT:\n{}\nSTDERR:\n{}", stdout, stderr));
            }
        }

        // 2. Visual Verification
        if let Some(golden_image) = &self.ticket.verification.golden_image {
             let report = verify_visual(self.workspace_root, golden_image)?;
             if report.diff_detected {
                 return Err(anyhow!("Visual Verification Failed: {}\nDiff Bounds: {:?}\nReason: {:?}",
                    report.mismatch_percentage, report.diff_bounds, report.reason));
             }
        }

        Ok(())
    }
}
