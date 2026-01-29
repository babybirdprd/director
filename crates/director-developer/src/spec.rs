//! # Feature Spec Module
//!
//! The Unified Feature Spec in RON format for cross-cutting features.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A cross-cutting feature specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSpec {
    /// Feature title (e.g., "Glow Effect")
    pub title: String,

    /// User story describing the feature
    pub user_story: String,

    /// Priority level (1 = highest)
    #[serde(default = "default_priority")]
    pub priority: u8,

    /// Schema layer changes
    #[serde(default)]
    pub schema_changes: Vec<SchemaChange>,

    /// Scripting layer requirements
    #[serde(default)]
    pub scripting_requirements: Vec<ScriptingRequirement>,

    /// Pipeline layer requirements
    #[serde(default)]
    pub pipeline_requirements: Vec<PipelineRequirement>,

    /// Verification logic
    pub verification: VerificationSpec,

    /// Related types to include in context
    #[serde(default)]
    pub related_types: Vec<String>,

    /// Related Rhai functions to include in context
    #[serde(default)]
    pub related_functions: Vec<String>,
}

fn default_priority() -> u8 {
    2
}

/// A change to the schema layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChange {
    /// Target struct or enum name
    pub target: String,

    /// Description of change
    pub change: ChangeType,

    /// Optional default value
    #[serde(default)]
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    /// Add a new field
    AddField { name: String, field_type: String },
    /// Add a new enum variant
    AddVariant {
        name: String,
        fields: Vec<(String, String)>,
    },
    /// Modify existing field
    ModifyField { name: String, new_type: String },
}

/// A requirement for the scripting layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptingRequirement {
    /// Function name to expose
    pub function_name: String,

    /// Function signature (display format)
    pub signature: String,

    /// Optional doc comment
    #[serde(default)]
    pub doc_comment: Option<String>,
}

/// A requirement for the pipeline layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRequirement {
    /// Description of pipeline change
    pub description: String,

    /// Affected function or module
    #[serde(default)]
    pub affected_area: Option<String>,
}

/// Verification specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationSpec {
    /// Rhai script must compile
    #[serde(default = "default_true")]
    pub script_compiles: bool,

    /// JSON payload must validate against schema
    #[serde(default = "default_true")]
    pub schema_validates: bool,

    /// Custom verification scripts
    #[serde(default)]
    pub custom_scripts: Vec<String>,

    /// Expected test cases
    #[serde(default)]
    pub test_cases: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// Load a feature spec from a RON file.
pub fn load_spec(path: &Path) -> Result<FeatureSpec> {
    let content = std::fs::read_to_string(path)?;
    let spec: FeatureSpec = ron::from_str(&content)?;
    Ok(spec)
}

/// Validation result for a feature spec.
#[derive(Debug)]
pub struct ValidationResult {
    pub passed: bool,
    pub type_checks: Vec<TypeCheck>,
    pub function_checks: Vec<FunctionCheck>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Print a summary of the validation result.
    #[allow(dead_code)]
    pub fn print_summary(&self) {
        println!("\nValidation Result:");
        println!("==================");
        println!("Types checked: {}", self.type_checks.len());
        println!("Functions checked: {}", self.function_checks.len());
        println!("Warnings: {}", self.warnings.len());
        println!("Status: {}", if self.passed { "PASSED" } else { "FAILED" });
    }
}

#[derive(Debug)]
pub struct TypeCheck {
    pub type_name: String,
    pub exists: bool,
}

#[derive(Debug)]
pub struct FunctionCheck {
    pub pattern: String,
    pub found_count: usize,
}

/// Validate a feature spec and return structured results.
pub fn validate_spec(path: &Path) -> Result<ValidationResult> {
    let spec = load_spec(path)?;

    let mut type_checks = Vec::new();
    let mut function_checks = Vec::new();
    let mut warnings = Vec::new();
    let mut all_passed = true;

    // Validate related types exist
    for type_name in &spec.related_types {
        let exists = crate::reflectors::schema::schema_for_type(type_name).is_ok();
        type_checks.push(TypeCheck {
            type_name: type_name.clone(),
            exists,
        });
        if !exists {
            warnings.push(format!("Type '{}' not found in schema", type_name));
            all_passed = false;
        }
    }

    // Validate related functions exist
    for func_pattern in &spec.related_functions {
        let found_count = crate::reflectors::scripting::find_functions(func_pattern)
            .map(|f| f.len())
            .unwrap_or(0);
        function_checks.push(FunctionCheck {
            pattern: func_pattern.clone(),
            found_count,
        });
        if found_count == 0 {
            warnings.push(format!("No functions matching '{}'", func_pattern));
            all_passed = false;
        }
    }

    Ok(ValidationResult {
        passed: all_passed,
        type_checks,
        function_checks,
        warnings,
    })
}

/// Validate a feature spec against current system capabilities.
pub fn validate(path: &Path) -> Result<()> {
    let spec = load_spec(path)?;

    println!("Validating spec: {}", spec.title);
    println!("  Priority: {}", spec.priority);
    println!("  User Story: {}", spec.user_story);

    // Run structured validation
    let result = validate_spec(path)?;

    // Print type checks
    println!("\nChecking related types:");
    for check in &result.type_checks {
        let icon = if check.exists { "✓" } else { "✗" };
        println!("  {} Type '{}'", icon, check.type_name);
    }

    // Print function checks
    println!("\nChecking related functions:");
    for check in &result.function_checks {
        let icon = if check.found_count > 0 { "✓" } else { "✗" };
        println!(
            "  {} Pattern '{}' - {} match(es)",
            icon, check.pattern, check.found_count
        );
    }

    // Report schema changes
    if !spec.schema_changes.is_empty() {
        println!("\nProposed schema changes:");
        for change in &spec.schema_changes {
            println!("  → {}:{:?}", change.target, change.change);
        }
    }

    // Report scripting requirements
    if !spec.scripting_requirements.is_empty() {
        println!("\nScripting requirements:");
        for req in &spec.scripting_requirements {
            println!("  → {} : {}", req.function_name, req.signature);
        }
    }

    // Report pipeline requirements
    if !spec.pipeline_requirements.is_empty() {
        println!("\nPipeline requirements:");
        for req in &spec.pipeline_requirements {
            println!("  → {}", req.description);
        }
    }

    // Print warnings
    if !result.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &result.warnings {
            println!("  ⚠ {}", warning);
        }
    }

    // Final status
    if result.passed {
        println!(
            "\n✓ Validation complete: All checks passed for '{}'",
            spec.title
        );
    } else {
        println!(
            "\n✗ Validation complete: Some checks failed for '{}'",
            spec.title
        );
    }

    Ok(())
}
