use clap::{Parser, Subcommand, ValueEnum};
use director_plan::{DirectorPlan, types::{Status, TicketSummary}};
use director_plan::context::discovery::discover_context;
use director_plan::execution_loop::ExecutionLoop;
use director_plan::worker::Worker;
use std::path::PathBuf;
use anyhow::{Result, Context};
use std::process::Command;
use colored::*;

use director_plan::server;

#[derive(Parser)]
#[command(name = "director-plan")]
#[command(about = "Headless Project Management for AI Agents", long_about = None)]
struct Cli {
    #[arg(long, default_value = "text")]
    log_format: LogFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, ValueEnum)]
enum LogFormat {
    Text,
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// List tickets
    List {
        #[arg(long, value_enum)]
        status: Option<StatusArg>,
        #[arg(long, value_enum, default_value_t = Format::Table)]
        format: Format,
    },
    /// Get context for a ticket
    Context {
        id: String,
    },
    /// Verify a ticket
    Verify {
        id: String,
    },
    /// Update a ticket
    Update {
        id: String,
        #[arg(long, value_enum)]
        status: Option<StatusArg>,
        #[arg(long)]
        owner: Option<String>,
        #[arg(long)]
        comment: Option<String>,
    },
    /// Execute a ticket using an agent
    Execute {
        id: String,
        #[arg(long)]
        agent: String,
    },
    /// Run the Radkit Worker
    Worker {
        #[arg(long, default_value_t = 1)]
        pool: usize,
    },
    /// Search documentation
    Docs {
        #[command(subcommand)]
        subcmd: DocsCommands,
    },
    /// Start the server
    Serve,
}

#[derive(Subcommand)]
enum DocsCommands {
    Search {
        query: String,
    },
}

#[derive(Clone, ValueEnum)]
#[value(rename_all = "snake_case")]
enum StatusArg {
    Todo,
    InProgress,
    Review,
    Done,
    Archived,
}

impl From<StatusArg> for Status {
    fn from(arg: StatusArg) -> Self {
        match arg {
            StatusArg::Todo => Status::Todo,
            StatusArg::InProgress => Status::InProgress,
            StatusArg::Review => Status::Review,
            StatusArg::Done => Status::Done,
            StatusArg::Archived => Status::Archived,
        }
    }
}

#[derive(Clone, ValueEnum)]
enum Format {
    Json,
    Table,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let builder = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env());

    match cli.log_format {
        LogFormat::Json => builder.json().init(),
        LogFormat::Text => builder.init(),
    }

    let root = std::env::current_dir()?;
    let plan = DirectorPlan::new(root.clone());

    match cli.command {
        Commands::Serve => {
             server::start_server(root).await?;
        }
        Commands::Worker { pool } => {
            let worker = Worker::new(root, pool);
            worker.run().await?;
        }
        Commands::List { status, format } => {
            let filter = status.map(Status::from);
            let tickets = plan.list_tickets(filter)?;

            match format {
                Format::Json => {
                    let summaries: Vec<TicketSummary> = tickets.into_iter().map(|t| TicketSummary {
                        id: t.meta.id,
                        title: t.meta.title,
                        status: t.meta.status,
                        priority: t.meta.priority,
                    }).collect();
                    println!("{}", serde_json::to_string_pretty(&summaries)?);
                }
                Format::Table => {
                    for t in tickets {
                        println!("{} [{}] {} ({:?})",
                            t.meta.id.bold(),
                            t.meta.status.to_string().cyan(),
                            t.meta.title,
                            t.meta.priority
                        );
                    }
                }
            }
        }
        Commands::Context { id } => {
            let ticket = plan.get_ticket(&id)?;
            println!("# TASK: {} {}", ticket.meta.id, ticket.meta.title);
            println!("## Description");
            println!("{}", ticket.spec.description);
            println!("\n## Constraints");
            for c in &ticket.spec.constraints {
                println!("- {}", c);
            }

            let mut relevant_files = ticket.spec.relevant_files.clone();

            // Auto-Context
            if relevant_files.is_empty() {
                // If implicit or explicit auto_context is desired.
                // PR says: "When director-plan context <T-ID> is called, if relevant_files is empty in the TOML, the engine now dynamically populates context."
                println!("\n>> Auto-Context Discovery Triggered...");
                relevant_files = discover_context(&ticket, &root);
            }

            for file_path in relevant_files {
                let p = root.join(&file_path);
                if p.exists() {
                    println!("\n## Context File: {}", file_path);
                    match std::fs::read_to_string(&p) {
                        Ok(content) => println!("```\n{}\n```", content),
                        Err(e) => println!("Error reading file: {}", e),
                    }
                } else {
                    println!("\n## Context File: {} (NOT FOUND)", file_path);
                }
            }
        }
        Commands::Verify { id } => {
            // Git safety check
            let git_status = Command::new("git")
                .arg("status")
                .arg("--porcelain")
                .output()
                .context("Failed to run git status")?;

            if !git_status.stdout.is_empty() {
                anyhow::bail!("Git tree is not clean. Commit or stash changes before verifying.");
            }

            let ticket = plan.get_ticket(&id)?;
            println!("Running verification for {}: {}", id, ticket.verification.command);

            // Basic splitting by whitespace - improving this would require shell-parsing logic
            let parts: Vec<&str> = ticket.verification.command.split_whitespace().collect();
            if parts.is_empty() {
                anyhow::bail!("Verification command is empty");
            }

            let status = Command::new(parts[0])
                .args(&parts[1..])
                .status()
                .context("Failed to execute verification command")?;

            if status.success() {
                println!("{}", "PASS".green().bold());
            } else {
                println!("{}", "FAIL".red().bold());
                std::process::exit(1);
            }
        }
        Commands::Update { id, status, owner, comment } => {
             update_ticket(&plan, &id, status.map(Status::from), owner, comment)?;
        }
        Commands::Execute { id, agent } => {
            let ticket = plan.get_ticket(&id)?;
            let mut loop_runner = ExecutionLoop::new(&root, agent, ticket);
            loop_runner.run()?;
        }
        Commands::Docs { subcmd } => {
            match subcmd {
                DocsCommands::Search { query } => {
                    search_docs(&root, &query)?;
                }
            }
        }
    }

    Ok(())
}

fn update_ticket(plan: &DirectorPlan, id: &str, status: Option<Status>, owner: Option<String>, comment: Option<String>) -> Result<()> {
    let ticket_path = plan.get_tickets_dir().join(format!("{}.toml", id));
    if !ticket_path.exists() {
         anyhow::bail!("Ticket {} not found", id);
    }

    let content = std::fs::read_to_string(&ticket_path)?;
    let mut doc = content.parse::<toml_edit::DocumentMut>()?;

    if let Some(s) = status {
        doc["meta"]["status"] = toml_edit::value(s.to_string());
    }

    if let Some(o) = owner {
        doc["meta"]["owner"] = toml_edit::value(o);
    }

    if let Some(c) = comment {
        let entry = format!("[{}] {}", chrono::Utc::now().to_rfc3339(), c);

        // Ensure history table exists
        if doc.get("history").is_none() {
             doc["history"] = toml_edit::Item::Table(toml_edit::Table::new());
        }

        let history = doc["history"].as_table_mut().unwrap();

        // Ensure log array exists
        if history.get("log").is_none() {
            history.insert("log", toml_edit::Item::Value(toml_edit::Value::Array(toml_edit::Array::new())));
        }

        if let Some(log) = history.get_mut("log") {
            if let Some(arr) = log.as_array_mut() {
                 arr.push(entry);
            }
        }
    }

    std::fs::write(ticket_path, doc.to_string())?;
    println!("Ticket {} updated.", id);

    Ok(())
}

fn search_docs(root: &PathBuf, query: &str) -> Result<()> {
    let docs_dir = root.join("docs");
    if !docs_dir.exists() {
        println!("No docs directory found.");
        return Ok(());
    }

    let query_lower = query.to_lowercase();

    for entry in walkdir::WalkDir::new(docs_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if content.to_lowercase().contains(&query_lower) {
                    println!("Found in: {:?}", entry.path());
                    // print snippets?
                    for line in content.lines() {
                        if line.to_lowercase().contains(&query_lower) {
                             println!("  {}", line.trim());
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
