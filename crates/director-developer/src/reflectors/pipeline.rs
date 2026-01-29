//! # Reflector C: Pipeline
//!
//! Live reflection of director-core to list available render nodes and capabilities.
//!
//! This reflector queries the runtime registry instead of using a static manifest.

use anyhow::Result;
use director_core::registry;
use serde_json::json;

/// Reflect pipeline capabilities using the live registry.
///
/// Returns JSON describing available node types, effects, transitions,
/// and system constraints.
pub fn reflect_capabilities() -> Result<String> {
    // Get live data from the registry
    let node_types: Vec<_> = registry::list_node_types()
        .iter()
        .map(|n| {
            json!({
                "name": n.name,
                "description": n.description,
                "supports_children": n.supports_children,
                "animatable_properties": n.animatable_properties,
                "features": n.features,
            })
        })
        .collect();

    let effects: Vec<_> = registry::list_effects()
        .iter()
        .map(|e| {
            json!({
                "name": e.name,
                "description": e.description,
                "params": e.params.iter().map(|(n, t)| format!("{}: {}", n, t)).collect::<Vec<_>>(),
            })
        })
        .collect();

    let transitions: Vec<_> = registry::list_transitions()
        .iter()
        .map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
            })
        })
        .collect();

    let easings: Vec<_> = registry::list_easings().iter().map(|e| e.name).collect();

    let capabilities = json!({
        "node_types": node_types,
        "effects": effects,
        "transitions": transitions,
        "easings": easings,
        "layout_modes": ["Flexbox", "Grid"],
        "constraints": {
            "max_nested_compositions": 10,
            "max_audio_tracks": 16,
            "audio_formats": ["mp3", "wav", "aac"],
            "video_formats": ["mp4", "webm"],
            "image_formats": ["png", "jpg", "jpeg", "gif", "webp"],
            "max_resolution": { "width": 7680, "height": 4320 },
            "fps_range": { "min": 1, "max": 120 }
        }
    });

    Ok(serde_json::to_string_pretty(&capabilities)?)
}

/// Check if a specific node type supports a property animation.
/// This now uses the live registry.
pub fn supports_animation(node_type: &str, property: &str) -> bool {
    registry::supports_animation(node_type, property)
}

/// Get all animatable properties for a node type.
/// This now uses the live registry.
pub fn get_animatable_properties(node_type: &str) -> Vec<&'static str> {
    registry::get_animatable_properties(node_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflect_capabilities_is_valid_json() {
        let json_str = reflect_capabilities().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert!(parsed.get("node_types").is_some());
        assert!(parsed.get("effects").is_some());
        assert!(parsed.get("transitions").is_some());
        assert!(parsed.get("easings").is_some());
    }

    #[test]
    fn test_live_registry_data() {
        let json_str = reflect_capabilities().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        let node_types = parsed["node_types"].as_array().unwrap();
        assert!(node_types.len() >= 8);

        let effects = parsed["effects"].as_array().unwrap();
        assert!(effects.len() >= 7);
    }
}
