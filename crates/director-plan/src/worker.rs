use std::path::{Path, PathBuf};
use std::time::Duration;
use std::process::Command;
use anyhow::{Result, anyhow, Context};
use crate::types::{Ticket, Status, Priority};
use crate::execution_loop::ExecutionLoop;
use reqwest::Client;
use serde_json::json;
use colored::*;

pub struct Worker {
    workspace_root: PathBuf,
    pool_size: usize,
    client: Client,
    server_url: String,
}

impl Worker {
    pub fn new(workspace_root: PathBuf, pool_size: usize) -> Self {
        Self {
            workspace_root,
            pool_size,
            client: Client::new(),
            server_url: "http://localhost:3000".to_string(), // Configurable?
        }
    }

    pub async fn run(&self) -> Result<()> {
        println!("{}", format!(">> Radkit Worker Started (Pool: {})", self.pool_size).green());
        println!(">> Polling {} for tickets...", self.server_url);

        loop {
            match self.poll_ticket().await {
                Ok(Some(ticket)) => {
                    println!("{}", format!(">> Found Ticket: {} - {}", ticket.meta.id, ticket.meta.title).cyan());
                    if let Err(e) = self.process_ticket(ticket).await {
                        eprintln!("{}", format!(">> Error processing ticket: {}", e).red());
                    }
                },
                Ok(None) => {
                    // No tickets, sleep
                    tokio::time::sleep(Duration::from_secs(5)).await;
                },
                Err(e) => {
                    eprintln!("{}", format!(">> Polling error: {}", e).red());
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            }
        }
    }

    async fn poll_ticket(&self) -> Result<Option<Ticket>> {
        // Fetch all tickets and filter locally for now (API might not support complex filter)
        let resp = self.client.get(format!("{}/api/tickets", self.server_url))
            .send()
            .await?;

        if !resp.status().is_success() {
             return Err(anyhow!("Server returned {}", resp.status()));
        }

        let tickets: Vec<crate::types::FrontendTicket> = resp.json().await?;

        // Find first TODO ticket assigned to 'radkit' (or unassigned?)
        // Prompt says: "marked status = 'todo' && assignee = 'radkit'"
        for ft in tickets {
            if ft.status == "todo" && ft.owner == "radkit" {
                // We need the full ticket TOML. The frontend ticket structure is flattened.
                // We assume we can read the file from disk using the ID since we are "Native".
                // Or we need an API to get the raw ticket.
                // Since `director-plan` server serves from the same FS, we can read FS.
                // ID is like "T-001". File is "plan/tickets/T-001.toml".

                let path = self.workspace_root.join("plan/tickets").join(format!("{}.toml", ft.id));
                if path.exists() {
                     let content = std::fs::read_to_string(&path)?;
                     let ticket: Ticket = toml_edit::de::from_str(&content)?;
                     return Ok(Some(ticket));
                }
            }
        }

        Ok(None)
    }

    async fn process_ticket(&self, mut ticket: Ticket) -> Result<()> {
        // 1. Claim Ticket (Set to InProgress)
        ticket.meta.status = Status::InProgress;
        self.save_ticket(&ticket)?;

        // 2. Create Branch
        let branch_name = format!("radkit/{}", ticket.meta.id.to_lowercase());
        self.create_branch(&branch_name)?;

        // 3. Execute Loop
        // We need to create ExecutionLoop.
        // What is the agent command?
        // We should probably read it from settings or config.
        // For now, let's assume a default or env var `RADKIT_AGENT_CMD`.
        let agent_cmd = std::env::var("RADKIT_AGENT_CMD").unwrap_or_else(|_| "cursor --prompt".to_string());

        // We need a way to pass the customized ExecutionLoop that captures output.
        // Since `ExecutionLoop` is in another module, we might need to modify it to return the result with confidence.
        // For now, let's instantiate it.

        // Wait, `ExecutionLoop::run` returns `Result<()>`. It doesn't return confidence.
        // I need to update `ExecutionLoop` to return `ExecutionResult`.

        // Let's assume I updated it (I will in next step).
        // For now, I'll call it and check side effects? No, I need the handshake.

        // I will update ExecutionLoop in the NEXT step.
        // So here I will write the code ASSUMING the new API exists, or I will use a placeholder.

        let mut loop_runner = ExecutionLoop::new(&self.workspace_root, agent_cmd, ticket.clone());

        // Assuming run_with_handshake is the new method
        let result = match loop_runner.run_with_handshake() {
             Ok(r) => r,
             Err(e) => {
                 // Execution failed (crashed or max retries)
                 ticket.meta.status = Status::Review; // Review because it failed
                 self.save_ticket(&ticket)?;
                 return Err(e);
             }
        };

        // 4. Check Confidence
        let min_confidence = ticket.verification.min_confidence;
        if result.confidence < min_confidence {
             println!(">> Confidence too low ({:.2} < {:.2}). Requesting feedback.", result.confidence, min_confidence);
             ticket.meta.status = Status::Review;
             // Append to log?
             ticket.history.log.push(format!("Radkit: Low confidence ({:.2}). Requesting human review.", result.confidence));
             self.save_ticket(&ticket)?;
             return Ok(());
        }

        // 5. Submit PR
        self.submit_pr(&branch_name, &ticket).await?;

        // 6. Mark Done (or Review?)
        // Usually PR implies "Review".
        ticket.meta.status = Status::Review;
        self.save_ticket(&ticket)?;

        // Checkout back to main/master?
        // Worker should reset for next ticket.
        self.reset_to_base()?;

        Ok(())
    }

    fn save_ticket(&self, ticket: &Ticket) -> Result<()> {
        let path = self.workspace_root.join("plan/tickets").join(format!("{}.toml", ticket.meta.id));
        let content = toml_edit::ser::to_string_pretty(ticket)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn create_branch(&self, branch: &str) -> Result<()> {
        // Ensure clean state
        Command::new("git").args(&["checkout", "main"]).current_dir(&self.workspace_root).output()?;
        Command::new("git").args(&["pull"]).current_dir(&self.workspace_root).output()?;

        // Create branch
        Command::new("git").args(&["checkout", "-b", branch]).current_dir(&self.workspace_root).status()?;
        Ok(())
    }

    fn reset_to_base(&self) -> Result<()> {
        Command::new("git").args(&["checkout", "main"]).current_dir(&self.workspace_root).status()?;
        Ok(())
    }

    async fn submit_pr(&self, branch: &str, ticket: &Ticket) -> Result<()> {
        println!(">> Pushing branch {}...", branch);
        let status = Command::new("git")
            .args(&["push", "-u", "origin", branch])
            .current_dir(&self.workspace_root)
            .status()?;

        if !status.success() {
             return Err(anyhow!("Failed to push branch"));
        }

        // Create PR via GitHub API
        println!(">> Creating PR...");
        let token = std::env::var("GITHUB_TOKEN").context("GITHUB_TOKEN not set")?;

        // Need to parse owner/repo from git remote?
        // Let's assume we can get it or user provided it.
        // Heuristic: git remote get-url origin
        let remote_out = Command::new("git").args(&["remote", "get-url", "origin"]).output()?;
        let remote_url = String::from_utf8_lossy(&remote_out.stdout).trim().to_string();
        // Extract owner/repo from "git@github.com:owner/repo.git" or "https://github.com/owner/repo"

        let (owner, repo) = parse_github_url(&remote_out.stdout)?;

        let url = format!("https://api.github.com/repos/{}/{}/pulls", owner, repo);

        let body = json!({
            "title": ticket.meta.title,
            "body": format!("{}\n\nCloses {}", ticket.spec.description, ticket.meta.id),
            "head": branch,
            "base": "main"
        });

        let resp = self.client.post(&url)
            .header("Authorization", format!("token {}", token))
            .header("User-Agent", "director-plan-radkit")
            .header("Accept", "application/vnd.github.v3+json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
             let err_text = resp.text().await?;
             return Err(anyhow!("Failed to create PR: {}", err_text));
        }

        println!(">> PR Created Successfully!");
        Ok(())
    }
}

fn parse_github_url(bytes: &[u8]) -> Result<(String, String)> {
    let s = String::from_utf8_lossy(bytes).trim().to_string();
    // Handle ssh: git@github.com:owner/repo.git
    // Handle https: https://github.com/owner/repo.git

    let path = if s.starts_with("git@") {
        s.split(':').nth(1).ok_or(anyhow!("Invalid git url"))?
    } else if s.starts_with("http") {
        s.split("github.com/").nth(1).ok_or(anyhow!("Invalid git url"))?
    } else {
        return Err(anyhow!("Unknown git url format"));
    };

    let path = path.strip_suffix(".git").unwrap_or(path);
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() < 2 {
        return Err(anyhow!("Could not parse owner/repo"));
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}
