//! # Reflectors Module
//!
//! Implements the reflection strategies for system truth extraction.
//!
//! - **scripting**: Rhai API signatures via `gen_fn_metadata_to_json()`
//! - **schema**: JSON Schema generation via `schemars`
//! - **pipeline**: Live reflection of available render nodes
//! - **graph**: Workspace dependency analysis via `cargo metadata`

pub mod graph;
pub mod pipeline;
pub mod schema;
pub mod scripting;

use anyhow::Result;
use std::fs;
use std::path::Path;

/// Collected reflection data from all three layers.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReflectionData {
    pub scripting: Option<serde_json::Value>,
    pub schemas: Option<serde_json::Value>,
    pub pipeline: Option<serde_json::Value>,
}

/// Collect all reflection data.
pub fn collect_all() -> Result<ReflectionData> {
    let scripting_json = scripting::reflect_rhai_api()?;
    let schemas_json = schema::reflect_all_schemas()?;
    let pipeline_json = pipeline::reflect_capabilities()?;

    Ok(ReflectionData {
        scripting: Some(serde_json::from_str(&scripting_json)?),
        schemas: Some(serde_json::from_str(&schemas_json)?),
        pipeline: Some(serde_json::from_str(&pipeline_json)?),
    })
}

/// Dump all reflection data to the output directory.
pub fn dump_all(
    output: &Path,
    include_scripting: bool,
    include_schema: bool,
    include_pipeline: bool,
) -> Result<()> {
    fs::create_dir_all(output)?;

    if include_scripting {
        let api_json = scripting::reflect_rhai_api()?;
        fs::write(output.join("rhai_api.json"), &api_json)?;
        tracing::info!("Wrote rhai_api.json ({} bytes)", api_json.len());
    }

    if include_schema {
        let schemas = schema::reflect_all_schemas()?;
        fs::write(output.join("schemas.json"), &schemas)?;
        tracing::info!("Wrote schemas.json ({} bytes)", schemas.len());
    }

    if include_pipeline {
        let capabilities = pipeline::reflect_capabilities()?;
        fs::write(output.join("pipeline_capabilities.json"), &capabilities)?;
        tracing::info!(
            "Wrote pipeline_capabilities.json ({} bytes)",
            capabilities.len()
        );
    }

    tracing::info!("Dumped system truth to {}", output.display());
    Ok(())
}
