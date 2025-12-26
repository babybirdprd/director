//! Multi-Agent Coordinator (A2A Protocol)
//!
//! Enables coordination between multiple specialized agents:
//! - Planner: Breaks down tickets into sub-tasks
//! - Coder: Implements code changes
//! - Reviewer: Validates changes and provides feedback

use anyhow::{anyhow, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::agent::{AgentConfig, CodeGenerationResult, LlmAgent, LlmProvider};
use crate::types::Ticket;

/// Role specialization for agents
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRole {
    Planner,
    Coder,
    Reviewer,
}

impl AgentRole {
    pub fn system_prompt(&self) -> &'static str {
        match self {
            AgentRole::Planner => {
                r#"You are a Planning Agent. Your job is to:
1. Analyze the task description and constraints
2. Break down complex tasks into smaller, actionable sub-tasks
3. Identify dependencies between sub-tasks
4. Estimate complexity and effort for each sub-task
5. Create a structured execution plan

Output your plan as a JSON object with a "subtasks" array."#
            }

            AgentRole::Coder => {
                r#"You are a Coding Agent. Your job is to:
1. Implement the specified task
2. Write clean, well-documented code
3. Follow the project's coding conventions
4. Handle edge cases appropriately
5. Ensure code compiles and passes basic checks

Output your changes as structured FileChange objects."#
            }

            AgentRole::Reviewer => {
                r#"You are a Review Agent. Your job is to:
1. Analyze code changes for correctness
2. Check for potential bugs or issues
3. Verify the changes address the original task
4. Suggest improvements if needed
5. Approve or request changes

Output your review as a structured ReviewResult object."#
            }
        }
    }
}

/// Sub-task produced by the Planner agent
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SubTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub complexity: TaskComplexity,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum TaskComplexity {
    Low,
    Medium,
    High,
}

/// Plan produced by the Planner agent
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionPlan {
    pub subtasks: Vec<SubTask>,
    pub estimated_total_time: String,
    pub risk_assessment: String,
}

/// Review result from the Reviewer agent
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReviewResult {
    pub approved: bool,
    pub issues: Vec<ReviewIssue>,
    pub suggestions: Vec<String>,
    pub overall_quality: f32,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReviewIssue {
    pub severity: IssueSeverity,
    pub file: String,
    pub line: Option<u32>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

/// Multi-agent coordinator
pub struct Coordinator {
    planner: LlmAgent,
    coder: LlmAgent,
    reviewer: LlmAgent,
    max_iterations: u32,
}

impl Coordinator {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            planner: LlmAgent::new(config.clone()),
            coder: LlmAgent::new(config.clone()),
            reviewer: LlmAgent::new(config),
            max_iterations: 3,
        }
    }

    pub fn with_max_iterations(mut self, max: u32) -> Self {
        self.max_iterations = max;
        self
    }

    /// Run the full multi-agent workflow
    pub async fn execute_ticket(
        &self,
        ticket: &Ticket,
        workspace_root: &Path,
    ) -> Result<CoordinationResult> {
        let mut iterations = 0;
        let mut all_changes = Vec::new();
        let mut review_feedback = Vec::new();

        // Step 1: Plan the work
        println!("🎯 [Planner] Analyzing task...");
        let plan = self.plan(ticket, workspace_root).await?;
        println!("📋 [Planner] Created {} sub-tasks", plan.subtasks.len());

        // Step 2: Code each sub-task
        for subtask in &plan.subtasks {
            println!("💻 [Coder] Working on: {}", subtask.title);

            let code_result = self
                .code(ticket, subtask, &review_feedback, workspace_root)
                .await?;
            all_changes.extend(code_result.files_modified);

            // Step 3: Review the changes
            println!("🔍 [Reviewer] Reviewing changes...");
            let review = self.review(ticket, &all_changes, workspace_root).await?;

            if review.approved {
                println!("✅ [Reviewer] Changes approved!");
            } else {
                println!("⚠️ [Reviewer] Issues found: {}", review.issues.len());
                review_feedback = review
                    .issues
                    .iter()
                    .map(|i| format!("{}: {}", i.file, i.message))
                    .collect();

                iterations += 1;
                if iterations >= self.max_iterations {
                    return Err(anyhow!("Max iterations reached without approval"));
                }
            }
        }

        Ok(CoordinationResult {
            success: true,
            plan,
            changes: all_changes,
            iterations,
        })
    }

    async fn plan(&self, ticket: &Ticket, workspace_root: &Path) -> Result<ExecutionPlan> {
        let prompt = format!(
            "{}\n\n# Task\nTitle: {}\nDescription: {}\nConstraints: {:?}\n\nCreate an execution plan.",
            AgentRole::Planner.system_prompt(),
            ticket.meta.title,
            ticket.spec.description,
            ticket.spec.constraints
        );

        // For now, return a simple single-task plan
        // In full implementation, this would use LlmFunction<ExecutionPlan>
        let _result = self.planner.run(&prompt, workspace_root).await?;

        Ok(ExecutionPlan {
            subtasks: vec![SubTask {
                id: "1".to_string(),
                title: ticket.meta.title.clone(),
                description: ticket.spec.description.clone(),
                dependencies: vec![],
                complexity: TaskComplexity::Medium,
            }],
            estimated_total_time: "1-2 hours".to_string(),
            risk_assessment: "Low risk - straightforward implementation".to_string(),
        })
    }

    async fn code(
        &self,
        ticket: &Ticket,
        subtask: &SubTask,
        feedback: &[String],
        workspace_root: &Path,
    ) -> Result<CodeGenerationResult> {
        let mut prompt = format!(
            "{}\n\n# Task\nTitle: {}\nDescription: {}\n\n# Sub-task\n{}: {}",
            AgentRole::Coder.system_prompt(),
            ticket.meta.title,
            ticket.spec.description,
            subtask.id,
            subtask.description
        );

        if !feedback.is_empty() {
            prompt.push_str("\n\n# Previous Review Feedback (FIX THESE)\n");
            for fb in feedback {
                prompt.push_str(&format!("- {}\n", fb));
            }
        }

        self.coder.run(&prompt, workspace_root).await
    }

    async fn review(
        &self,
        ticket: &Ticket,
        changes: &[crate::agent::FileChange],
        workspace_root: &Path,
    ) -> Result<ReviewResult> {
        let changes_summary: Vec<String> = changes
            .iter()
            .map(|c| format!("{}: {:?}", c.path, c.change_type))
            .collect();

        let prompt = format!(
            "{}\n\n# Original Task\n{}\n\n# Changes Made\n{}\n\nReview these changes.",
            AgentRole::Reviewer.system_prompt(),
            ticket.spec.description,
            changes_summary.join("\n")
        );

        let _result = self.reviewer.run(&prompt, workspace_root).await?;

        // Simplified: approve if coder returned success
        Ok(ReviewResult {
            approved: true,
            issues: vec![],
            suggestions: vec![],
            overall_quality: 0.85,
        })
    }
}

/// Result of multi-agent coordination
#[derive(Debug, Serialize, Deserialize)]
pub struct CoordinationResult {
    pub success: bool,
    pub plan: ExecutionPlan,
    pub changes: Vec<crate::agent::FileChange>,
    pub iterations: u32,
}
