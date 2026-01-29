//! # Reflector A: Scripting
//!
//! Extracts Rhai API signatures from director-core using the `metadata` feature.

use anyhow::Result;
use director_core::{scripting::register_rhai_api, DefaultAssetLoader};
use rhai::Engine;
use std::sync::Arc;

/// Reflect all registered Rhai functions and types.
///
/// Returns JSON containing function signatures, parameters, and documentation.
pub fn reflect_rhai_api() -> Result<String> {
    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);

    // Register the full Director API
    register_rhai_api(&mut engine, loader);

    // Use Rhai's metadata feature to dump all functions
    // This requires `rhai = { features = ["metadata"] }`
    let metadata = engine.gen_fn_metadata_to_json(false)?;

    Ok(metadata)
}

/// Function signature extracted from Rhai metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionSignature {
    pub name: String,
    pub namespace: Option<String>,
    pub params: Vec<ParamInfo>,
    pub return_type: Option<String>,
    pub doc_comments: Option<String>,
}

/// Parameter info for a function.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParamInfo {
    pub name: Option<String>,
    pub typ: Option<String>,
}

/// Get function signatures matching a pattern.
pub fn find_functions(pattern: &str) -> Result<Vec<FunctionSignature>> {
    let json = reflect_rhai_api()?;
    let metadata: serde_json::Value = serde_json::from_str(&json)?;

    let mut results = Vec::new();

    if let Some(functions) = metadata.get("functions").and_then(|f| f.as_array()) {
        for func in functions {
            if let Some(name) = func.get("name").and_then(|n| n.as_str()) {
                if name.contains(pattern) {
                    results.push(FunctionSignature {
                        name: name.to_string(),
                        namespace: func
                            .get("namespace")
                            .and_then(|n| n.as_str())
                            .map(|s| s.to_string()),
                        params: extract_params(func),
                        return_type: func
                            .get("returnType")
                            .and_then(|r| r.as_str())
                            .map(|s| s.to_string()),
                        doc_comments: func.get("docComments").and_then(|d| {
                            if d.is_array() {
                                d.as_array().map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str())
                                        .collect::<Vec<_>>()
                                        .join("\n")
                                })
                            } else {
                                d.as_str().map(|s| s.to_string())
                            }
                        }),
                    });
                }
            }
        }
    }

    Ok(results)
}

fn extract_params(func: &serde_json::Value) -> Vec<ParamInfo> {
    let mut params = Vec::new();

    if let Some(param_names) = func.get("params").and_then(|p| p.as_array()) {
        for param in param_names {
            if let Some(param_obj) = param.as_object() {
                params.push(ParamInfo {
                    name: param_obj
                        .get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string()),
                    typ: param_obj
                        .get("type")
                        .and_then(|t| t.as_str())
                        .map(|s| s.to_string()),
                });
            } else if let Some(name) = param.as_str() {
                params.push(ParamInfo {
                    name: Some(name.to_string()),
                    typ: None,
                });
            }
        }
    }

    params
}

/// Get a summary of the Rhai API for quick reference.
pub fn get_api_summary() -> Result<ApiSummary> {
    let json = reflect_rhai_api()?;
    let metadata: serde_json::Value = serde_json::from_str(&json)?;

    let function_count = metadata
        .get("functions")
        .and_then(|f| f.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    let module_count = metadata
        .get("modules")
        .and_then(|m| m.as_object())
        .map(|o| o.len())
        .unwrap_or(0);

    Ok(ApiSummary {
        function_count,
        module_count,
    })
}

/// Summary of the Rhai API.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ApiSummary {
    pub function_count: usize,
    pub module_count: usize,
}
