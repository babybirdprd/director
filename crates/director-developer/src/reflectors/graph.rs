//! # Reflector D: Dependency Graph
//!
//! Workspace crate dependency analysis using `cargo metadata`.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::process::Command;

/// Workspace crate dependency information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateInfo {
    pub name: String,
    pub version: String,
    pub path: String,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
}

/// Get the workspace dependency graph.
pub fn get_dependency_graph() -> Result<HashMap<String, CrateInfo>> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version=1", "--no-deps"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let packages = metadata["packages"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No packages found"))?;

    let mut crates: HashMap<String, CrateInfo> = HashMap::new();

    for pkg in packages {
        let name = pkg["name"].as_str().unwrap_or_default().to_string();
        let version = pkg["version"].as_str().unwrap_or_default().to_string();
        let manifest_path = pkg["manifest_path"].as_str().unwrap_or_default();
        let path = std::path::Path::new(manifest_path)
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_default();

        // Get workspace-local dependencies only
        let deps: Vec<String> = pkg["dependencies"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter(|d| d["path"].is_string()) // Only workspace deps
                    .filter_map(|d| d["name"].as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        crates.insert(
            name.clone(),
            CrateInfo {
                name,
                version,
                path,
                dependencies: deps,
                dependents: vec![],
            },
        );
    }

    // Calculate reverse dependencies
    let names: Vec<String> = crates.keys().cloned().collect();
    for name in &names {
        if let Some(info) = crates.get(name) {
            let deps = info.dependencies.clone();
            for dep in deps {
                if let Some(dep_info) = crates.get_mut(&dep) {
                    dep_info.dependents.push(name.clone());
                }
            }
        }
    }

    Ok(crates)
}

/// Generate a Mermaid diagram of the dependency graph.
pub fn generate_mermaid_diagram() -> Result<String> {
    let graph = get_dependency_graph()?;

    let mut mermaid = String::from("graph TD\n");

    // Sort for consistent output
    let mut edges: Vec<(String, String)> = vec![];
    for (name, info) in &graph {
        for dep in &info.dependencies {
            edges.push((name.clone(), dep.clone()));
        }
    }
    edges.sort();

    for (from, to) in edges {
        mermaid.push_str(&format!(
            "    {} --> {}\n",
            from.replace('-', "_"),
            to.replace('-', "_")
        ));
    }

    Ok(mermaid)
}

/// Generate JSON representation of the graph.
pub fn generate_json() -> Result<String> {
    let graph = get_dependency_graph()?;
    Ok(serde_json::to_string_pretty(&graph)?)
}

/// Get impact analysis: what crates are affected if this one changes.
pub fn get_impact(crate_name: &str) -> Result<Vec<String>> {
    let graph = get_dependency_graph()?;

    // Find all transitive dependents using BFS
    let mut affected: HashSet<String> = HashSet::new();
    let mut queue: Vec<&str> = vec![crate_name];

    while let Some(current) = queue.pop() {
        if let Some(info) = graph.get(current) {
            for dependent in &info.dependents {
                if !affected.contains(dependent) {
                    affected.insert(dependent.clone());
                    queue.push(dependent);
                }
            }
        }
    }

    let mut result: Vec<String> = affected.into_iter().collect();
    result.sort();
    Ok(result)
}

/// Get dependency chain: what crates does this one depend on.
pub fn get_dependencies(crate_name: &str) -> Result<Vec<String>> {
    let graph = get_dependency_graph()?;

    // Find all transitive dependencies using BFS
    let mut deps: HashSet<String> = HashSet::new();
    let mut queue: Vec<&str> = vec![crate_name];

    while let Some(current) = queue.pop() {
        if let Some(info) = graph.get(current) {
            for dep in &info.dependencies {
                if !deps.contains(dep) {
                    deps.insert(dep.clone());
                    queue.push(dep);
                }
            }
        }
    }

    let mut result: Vec<String> = deps.into_iter().collect();
    result.sort();
    Ok(result)
}

/// List all workspace crates with their direct dependency count.
pub fn list_crates() -> Result<Vec<(String, usize, usize)>> {
    let graph = get_dependency_graph()?;
    let mut result: Vec<(String, usize, usize)> = graph
        .iter()
        .map(|(name, info)| (name.clone(), info.dependencies.len(), info.dependents.len()))
        .collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_dependency_graph() {
        let graph = get_dependency_graph().unwrap();
        assert!(!graph.is_empty());

        // Should have director-core at minimum
        assert!(graph.contains_key("director-core"));
    }

    #[test]
    fn test_mermaid_generation() {
        let mermaid = generate_mermaid_diagram().unwrap();
        assert!(mermaid.starts_with("graph TD"));
        assert!(mermaid.contains("-->"));
    }

    #[test]
    fn test_impact_analysis() {
        let impact = get_impact("director-core").unwrap();
        // director-core should have dependents
        assert!(!impact.is_empty());
    }

    #[test]
    fn test_list_crates() {
        let crates = list_crates().unwrap();
        assert!(!crates.is_empty());

        let names: Vec<_> = crates.iter().map(|(n, _, _)| n.as_str()).collect();
        assert!(names.contains(&"director-core"));
        assert!(names.contains(&"director-developer"));
    }
}
