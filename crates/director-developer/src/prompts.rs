//! # Prompt Templates
//!
//! Pre-built AI prompt templates for common development tasks.

use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Available prompt templates.
const TEMPLATES: &[(&str, &str, &str)] = &[
    (
        "implement_effect",
        "Implement a new visual effect",
        include_str!("../prompts/implement_effect.prompt"),
    ),
    (
        "implement_node",
        "Implement a new node type",
        include_str!("../prompts/implement_node.prompt"),
    ),
    (
        "add_rhai_function",
        "Add a new Rhai scripting function",
        include_str!("../prompts/add_rhai_function.prompt"),
    ),
    (
        "debug_animation",
        "Debug animation issues",
        include_str!("../prompts/debug_animation.prompt"),
    ),
    (
        "extend_schema",
        "Add or modify a schema type",
        include_str!("../prompts/extend_schema.prompt"),
    ),
];

/// Template metadata.
#[derive(Debug, Clone)]
pub struct TemplateInfo {
    pub name: &'static str,
    pub description: &'static str,
}

/// List available templates.
pub fn list_templates() -> Vec<TemplateInfo> {
    TEMPLATES
        .iter()
        .map(|(name, desc, _)| TemplateInfo {
            name,
            description: desc,
        })
        .collect()
}

/// Get a template by name.
pub fn get_template(name: &str) -> Option<&'static str> {
    TEMPLATES
        .iter()
        .find(|(n, _, _)| *n == name)
        .map(|(_, _, content)| *content)
}

/// Generate a prompt from a template with variable substitution.
///
/// Variables use the format `{{VARIABLE_NAME}}` and `{{variable_name|lower}}`.
pub fn generate_prompt(template_name: &str, vars: &HashMap<String, String>) -> Result<String> {
    let template = get_template(template_name).ok_or_else(|| {
        anyhow!(
            "Unknown template: {}. Use 'director-dev prompt --list' to see available templates.",
            template_name
        )
    })?;

    let mut result = template.to_string();

    for (key, value) in vars {
        // Replace uppercase version (e.g., {{EFFECT_NAME}})
        result = result.replace(&format!("{{{{{}}}}}", key.to_uppercase()), value);

        // Replace lowercase version (e.g., {{effect_name}})
        result = result.replace(&format!("{{{{{}}}}}", key.to_lowercase()), value);

        // Replace _lower variant (e.g., {{effect_name_lower}})
        let lower_key = format!("{}_lower", key.to_lowercase());
        result = result.replace(
            &format!("{{{{{}}}}}", lower_key),
            &value.to_lowercase().replace(' ', "_"),
        );
    }

    Ok(result)
}

/// Parse variable arguments in KEY=VALUE format.
pub fn parse_vars(var_args: &[String]) -> HashMap<String, String> {
    var_args
        .iter()
        .filter_map(|s| {
            let parts: Vec<&str> = s.splitn(2, '=').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_templates() {
        let templates = list_templates();
        assert_eq!(templates.len(), 5);

        let names: Vec<_> = templates.iter().map(|t| t.name).collect();
        assert!(names.contains(&"implement_effect"));
        assert!(names.contains(&"implement_node"));
        assert!(names.contains(&"add_rhai_function"));
    }

    #[test]
    fn test_get_template() {
        let template = get_template("implement_effect");
        assert!(template.is_some());
        assert!(template.unwrap().contains("{{EFFECT_NAME}}"));

        let missing = get_template("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_generate_prompt() {
        let mut vars = HashMap::new();
        vars.insert("EFFECT_NAME".to_string(), "Glow".to_string());
        vars.insert("EFFECT_DESCRIPTION".to_string(), "Glow effect".to_string());

        let result = generate_prompt("implement_effect", &vars).unwrap();

        assert!(result.contains("Glow"));
        assert!(result.contains("glow")); // lowercase version
        assert!(!result.contains("{{EFFECT_NAME}}")); // Should be replaced
    }

    #[test]
    fn test_parse_vars() {
        let args = vec![
            "EFFECT_NAME=Glow".to_string(),
            "DESCRIPTION=A glow effect".to_string(),
            "invalid".to_string(), // Should be ignored
        ];

        let vars = parse_vars(&args);
        assert_eq!(vars.len(), 2);
        assert_eq!(vars.get("EFFECT_NAME"), Some(&"Glow".to_string()));
        assert_eq!(vars.get("DESCRIPTION"), Some(&"A glow effect".to_string()));
    }
}
