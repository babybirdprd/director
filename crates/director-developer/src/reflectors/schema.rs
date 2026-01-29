//! # Reflector B: Data Contracts
//!
//! Generates JSON Schemas from director-schema types using schemars.

use anyhow::Result;
use schemars::schema_for;
use serde_json::json;

// Import all schema types
use director_schema::{
    Animation, AudioReactiveBinding, AudioTrack, EffectConfig, MovieRequest, Node, NodeKind,
    Scene, SpringAnimation, StyleMap, TextAnimator, TransformMap, TransitionConfig,
    TransitionType,
};

/// Generate JSON Schemas for all key types.
///
/// Returns a JSON object with type names as keys and their schemas as values.
pub fn reflect_all_schemas() -> Result<String> {
    let schemas = json!({
        "MovieRequest": schema_for!(MovieRequest),
        "Scene": schema_for!(Scene),
        "Node": schema_for!(Node),
        "NodeKind": schema_for!(NodeKind),
        "Animation": schema_for!(Animation),
        "SpringAnimation": schema_for!(SpringAnimation),
        "StyleMap": schema_for!(StyleMap),
        "TransformMap": schema_for!(TransformMap),
        "EffectConfig": schema_for!(EffectConfig),
        "AudioTrack": schema_for!(AudioTrack),
        "AudioReactiveBinding": schema_for!(AudioReactiveBinding),
        "TransitionConfig": schema_for!(TransitionConfig),
        "TransitionType": schema_for!(TransitionType),
        "TextAnimator": schema_for!(TextAnimator),
    });

    Ok(serde_json::to_string_pretty(&schemas)?)
}

/// Generate schema for a specific type by name.
pub fn schema_for_type(type_name: &str) -> Result<String> {
    let schema = match type_name {
        "MovieRequest" => schema_for!(MovieRequest),
        "Scene" => schema_for!(Scene),
        "Node" => schema_for!(Node),
        "NodeKind" => schema_for!(NodeKind),
        "Animation" => schema_for!(Animation),
        "SpringAnimation" => schema_for!(SpringAnimation),
        "EffectConfig" => schema_for!(EffectConfig),
        "StyleMap" => schema_for!(StyleMap),
        "TransformMap" => schema_for!(TransformMap),
        "AudioTrack" => schema_for!(AudioTrack),
        "AudioReactiveBinding" => schema_for!(AudioReactiveBinding),
        "TransitionConfig" => schema_for!(TransitionConfig),
        "TransitionType" => schema_for!(TransitionType),
        "TextAnimator" => schema_for!(TextAnimator),
        _ => return Err(anyhow::anyhow!("Unknown type: {}", type_name)),
    };

    Ok(serde_json::to_string_pretty(&schema)?)
}

/// Get a list of all available schema types.
pub fn list_types() -> Vec<&'static str> {
    vec![
        "MovieRequest",
        "Scene",
        "Node",
        "NodeKind",
        "Animation",
        "SpringAnimation",
        "StyleMap",
        "TransformMap",
        "EffectConfig",
        "AudioTrack",
        "AudioReactiveBinding",
        "TransitionConfig",
        "TransitionType",
        "TextAnimator",
    ]
}
