use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ticket {
    pub meta: Meta,
    pub spec: Spec,
    pub verification: Verification,
    #[serde(default)]
    pub history: History,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Meta {
    pub id: String,
    pub title: String,
    pub status: Status,
    pub priority: Priority,
    #[serde(rename = "type")]
    pub ticket_type: Option<TicketType>,
    pub owner: Option<String>,
    #[serde(default = "default_created_at")]
    pub created_at: toml_datetime::Datetime,
}

fn default_created_at() -> toml_datetime::Datetime {
    let d = toml_datetime::Date { year: 2024, month: 1, day: 1 };
    let t = toml_datetime::Time { hour: 0, minute: 0, second: 0, nanosecond: 0 };
    toml_datetime::Datetime { date: Some(d), time: Some(t), offset: None }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Todo,
    InProgress,
    Review,
    Done,
    Archived,
}

impl ToString for Status {
    fn to_string(&self) -> String {
        match self {
            Status::Todo => "todo".to_string(),
            Status::InProgress => "in_progress".to_string(),
            Status::Review => "review".to_string(),
            Status::Done => "done".to_string(),
            Status::Archived => "archived".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TicketType {
    Feature,
    Bug,
    Chore,
    Spike,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Spec {
    pub description: String,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub relevant_files: Vec<String>,
    #[serde(default)]
    pub auto_context: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Verification {
    pub command: String,
    pub golden_image: Option<String>,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_confidence_threshold")]
    pub min_confidence: f32,
}

fn default_max_retries() -> u32 { 5 }
fn default_confidence_threshold() -> f32 { 0.8 }

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct History {
    #[serde(default)]
    pub log: Vec<String>,
}

// Frontend DTOs
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrontendTicket {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub owner: String,
    pub verification_status: String,
    pub metrics: Option<Metrics>,
    pub artifacts: Option<Artifacts>,
    pub logs: Option<Vec<String>>,
    pub specs: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metrics {
    pub render_time_ms: f64,
    pub render_time_diff: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artifacts {
    pub before_image: String,
    pub after_image: String,
    pub diff_image: Option<String>,
}

impl From<Ticket> for FrontendTicket {
    fn from(ticket: Ticket) -> Self {
        FrontendTicket {
            id: ticket.meta.id.clone(),
            title: ticket.meta.title.clone(),
            description: ticket.spec.description.clone(),
            status: ticket.meta.status.to_string(),
            priority: format!("{:?}", ticket.meta.priority).to_lowercase(),
            owner: ticket.meta.owner.clone().unwrap_or_else(|| "unassigned".to_string()),
            verification_status: "pending".to_string(), // Default as we don't track it yet
            metrics: None,
            artifacts: None,
            logs: if ticket.history.log.is_empty() { None } else { Some(ticket.history.log.clone()) },
            specs: Some(ticket.spec.description.clone()), // Mapping spec description to specs as well? Or raw TOML?
        }
    }
}

// For List output
#[derive(Debug, Serialize)]
pub struct TicketSummary {
    pub id: String,
    pub title: String,
    pub status: Status,
    pub priority: Priority,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_serialization() {
        let status = Status::InProgress;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"in_progress\"");

        let deserialized: Status = serde_json::from_str("\"in_progress\"").unwrap();
        assert_eq!(deserialized, Status::InProgress);
    }
}
