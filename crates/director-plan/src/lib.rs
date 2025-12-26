pub mod agent;
pub mod context;
pub mod coordinator;
pub mod execution_loop;
pub mod server;
pub mod tools;
pub mod types;
pub mod verification;
pub mod worker;

use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::PathBuf;
use types::{Status, Ticket};
use walkdir::WalkDir;

pub struct DirectorPlan {
    workspace_root: PathBuf,
}

impl DirectorPlan {
    pub fn new(root: PathBuf) -> Self {
        Self {
            workspace_root: root,
        }
    }

    pub fn get_tickets_dir(&self) -> PathBuf {
        self.workspace_root.join("plan/tickets")
    }

    pub fn list_tickets(&self, status_filter: Option<Status>) -> Result<Vec<Ticket>> {
        let tickets_dir = self.get_tickets_dir();
        if !tickets_dir.exists() {
            return Ok(vec![]);
        }

        let mut tickets = Vec::new();
        for entry in WalkDir::new(tickets_dir)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().map_or(false, |ext| ext == "toml") {
                let content = fs::read_to_string(entry.path())
                    .with_context(|| format!("Failed to read ticket file: {:?}", entry.path()))?;
                let ticket: Ticket = toml_edit::de::from_str(&content)
                    .with_context(|| format!("Failed to parse ticket file: {:?}", entry.path()))?;

                if let Some(filter) = &status_filter {
                    if &ticket.meta.status == filter {
                        tickets.push(ticket);
                    }
                } else {
                    tickets.push(ticket);
                }
            }
        }

        // Sort by ID
        tickets.sort_by(|a, b| a.meta.id.cmp(&b.meta.id));

        Ok(tickets)
    }

    pub fn get_ticket(&self, id: &str) -> Result<Ticket> {
        let ticket_path = self.get_tickets_dir().join(format!("{}.toml", id));
        if !ticket_path.exists() {
            return Err(anyhow!("Ticket {} not found", id));
        }

        let content = fs::read_to_string(&ticket_path).context("Failed to read ticket file")?;
        let ticket: Ticket =
            toml_edit::de::from_str(&content).context("Failed to parse ticket file")?;

        Ok(ticket)
    }
}
