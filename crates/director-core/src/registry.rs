//! # Capability Registry
//!
//! Runtime registry for node types, effects, and other capabilities.
//! This enables live reflection by `director-developer` without hardcoded manifests.

use serde::Serialize;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Metadata about a renderable node type.
#[derive(Debug, Clone, Serialize)]
pub struct NodeTypeInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub supports_children: bool,
    pub animatable_properties: Vec<&'static str>,
    pub features: Vec<&'static str>,
}

/// Metadata about a visual effect.
#[derive(Debug, Clone, Serialize)]
pub struct EffectInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub params: Vec<(&'static str, &'static str)>, // (name, type)
}

/// Metadata about a scene transition.
#[derive(Debug, Clone, Serialize)]
pub struct TransitionInfo {
    pub name: &'static str,
    pub description: &'static str,
}

/// Metadata about an easing function.
#[derive(Debug, Clone, Serialize)]
pub struct EasingInfo {
    pub name: &'static str,
    pub description: &'static str,
}

/// Global capability registry - lazily initialized.
static REGISTRY: OnceLock<CapabilityRegistry> = OnceLock::new();

/// The capability registry containing all Director engine capabilities.
pub struct CapabilityRegistry {
    pub node_types: HashMap<&'static str, NodeTypeInfo>,
    pub effects: HashMap<&'static str, EffectInfo>,
    pub transitions: HashMap<&'static str, TransitionInfo>,
    pub easings: HashMap<&'static str, EasingInfo>,
}

impl CapabilityRegistry {
    /// Get the global registry instance.
    pub fn global() -> &'static Self {
        REGISTRY.get_or_init(Self::build)
    }

    fn build() -> Self {
        let mut reg = Self {
            node_types: HashMap::new(),
            effects: HashMap::new(),
            transitions: HashMap::new(),
            easings: HashMap::new(),
        };

        reg.register_builtin_nodes();
        reg.register_builtin_effects();
        reg.register_builtin_transitions();
        reg.register_builtin_easings();

        reg
    }

    fn register_builtin_nodes(&mut self) {
        let nodes = [
            NodeTypeInfo {
                name: "Box",
                description: "Rectangular container with optional border radius and shadow",
                supports_children: true,
                animatable_properties: vec![
                    "opacity",
                    "x",
                    "y",
                    "scale",
                    "rotation",
                    "bg_color",
                    "border_radius",
                ],
                features: vec!["flexbox_layout", "grid_layout"],
            },
            NodeTypeInfo {
                name: "Text",
                description: "Rich text with font styling and per-glyph animation",
                supports_children: false,
                animatable_properties: vec![
                    "opacity",
                    "x",
                    "y",
                    "scale",
                    "rotation",
                    "font_size",
                    "color",
                ],
                features: vec!["per_glyph_animation", "text_animators", "typewriter"],
            },
            NodeTypeInfo {
                name: "Image",
                description: "Static image with object-fit options",
                supports_children: false,
                animatable_properties: vec!["opacity", "x", "y", "scale", "rotation"],
                features: vec!["object_fit", "lazy_loading"],
            },
            NodeTypeInfo {
                name: "Video",
                description: "Video playback with frame stepping",
                supports_children: false,
                animatable_properties: vec!["opacity", "x", "y", "scale", "rotation", "volume"],
                features: vec!["frame_accurate_seeking", "audio_extraction", "loop"],
            },
            NodeTypeInfo {
                name: "Lottie",
                description: "Lottie JSON animation playback",
                supports_children: false,
                animatable_properties: vec!["opacity", "x", "y", "scale", "rotation", "speed"],
                features: vec!["gpu_accelerated", "dynamic_speed", "frame_control"],
            },
            NodeTypeInfo {
                name: "Vector",
                description: "SVG vector graphics",
                supports_children: false,
                animatable_properties: vec!["opacity", "x", "y", "scale", "rotation"],
                features: vec!["path_animation", "dynamic_colors"],
            },
            NodeTypeInfo {
                name: "Effect",
                description: "Visual effect wrapper (blur, shadows, color matrix)",
                supports_children: true,
                animatable_properties: vec!["sigma", "blur", "intensity"],
                features: vec!["effect_chaining"],
            },
            NodeTypeInfo {
                name: "Composition",
                description: "Nested timeline (pre-comp)",
                supports_children: true,
                animatable_properties: vec!["opacity", "x", "y", "scale", "rotation"],
                features: vec!["time_remapping", "nested_scenes", "independent_timeline"],
            },
        ];

        for node in nodes {
            self.node_types.insert(node.name, node);
        }
    }

    fn register_builtin_effects(&mut self) {
        let effects = [
            EffectInfo {
                name: "Blur",
                description: "Gaussian blur",
                params: vec![("sigma", "f32")],
            },
            EffectInfo {
                name: "DropShadow",
                description: "Drop shadow with offset and color",
                params: vec![
                    ("blur", "f32"),
                    ("offset_x", "f32"),
                    ("offset_y", "f32"),
                    ("color", "Color"),
                ],
            },
            EffectInfo {
                name: "ColorMatrix",
                description: "4x5 color transformation matrix",
                params: vec![("matrix", "[f32; 20]")],
            },
            EffectInfo {
                name: "Grayscale",
                description: "Desaturate to grayscale",
                params: vec![],
            },
            EffectInfo {
                name: "Sepia",
                description: "Sepia tone preset",
                params: vec![],
            },
            EffectInfo {
                name: "DirectionalBlur",
                description: "Motion blur in a direction",
                params: vec![("strength", "f32"), ("angle", "f32"), ("samples", "u32")],
            },
            EffectInfo {
                name: "FilmGrain",
                description: "Film grain overlay",
                params: vec![("intensity", "f32"), ("size", "f32")],
            },
        ];

        for effect in effects {
            self.effects.insert(effect.name, effect);
        }
    }

    fn register_builtin_transitions(&mut self) {
        let transitions = [
            TransitionInfo {
                name: "Fade",
                description: "Cross-dissolve between scenes",
            },
            TransitionInfo {
                name: "SlideLeft",
                description: "Slide outgoing scene left",
            },
            TransitionInfo {
                name: "SlideRight",
                description: "Slide outgoing scene right",
            },
            TransitionInfo {
                name: "WipeLeft",
                description: "Wipe revealing new scene from right",
            },
            TransitionInfo {
                name: "WipeRight",
                description: "Wipe revealing new scene from left",
            },
            TransitionInfo {
                name: "CircleOpen",
                description: "Circular reveal from center",
            },
        ];

        for transition in transitions {
            self.transitions.insert(transition.name, transition);
        }
    }

    fn register_builtin_easings(&mut self) {
        let easings = [
            ("linear", "Constant speed, no acceleration"),
            ("ease_in", "Start slow, accelerate"),
            ("ease_out", "Start fast, decelerate"),
            ("ease_in_out", "Slow start and end, fast middle"),
            ("bounce_out", "Bounce at the end"),
            ("bounce_in", "Bounce at the start"),
            ("bounce_in_out", "Bounce at both ends"),
            ("elastic_out", "Elastic overshoot at end"),
            ("elastic_in", "Elastic anticipation at start"),
            ("elastic_in_out", "Elastic at both ends"),
            ("back_out", "Slight overshoot"),
            ("back_in", "Slight anticipation"),
            ("back_in_out", "Anticipation and overshoot"),
        ];

        for (name, description) in easings {
            self.easings.insert(name, EasingInfo { name, description });
        }
    }
}

// ============ Public API ============

/// List all registered node types.
pub fn list_node_types() -> Vec<&'static NodeTypeInfo> {
    CapabilityRegistry::global().node_types.values().collect()
}

/// Get info for a specific node type.
pub fn get_node_type(name: &str) -> Option<&'static NodeTypeInfo> {
    CapabilityRegistry::global().node_types.get(name)
}

/// List all registered effects.
pub fn list_effects() -> Vec<&'static EffectInfo> {
    CapabilityRegistry::global().effects.values().collect()
}

/// Get info for a specific effect.
pub fn get_effect(name: &str) -> Option<&'static EffectInfo> {
    CapabilityRegistry::global().effects.get(name)
}

/// List all registered transitions.
pub fn list_transitions() -> Vec<&'static TransitionInfo> {
    CapabilityRegistry::global().transitions.values().collect()
}

/// List all registered easings.
pub fn list_easings() -> Vec<&'static EasingInfo> {
    CapabilityRegistry::global().easings.values().collect()
}

/// Check if a node type supports animation of a specific property.
pub fn supports_animation(node_type: &str, property: &str) -> bool {
    if let Some(info) = get_node_type(node_type) {
        info.animatable_properties.contains(&property)
    } else {
        false
    }
}

/// Get all animatable properties for a node type.
pub fn get_animatable_properties(node_type: &str) -> Vec<&'static str> {
    if let Some(info) = get_node_type(node_type) {
        info.animatable_properties.clone()
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_initializes() {
        let registry = CapabilityRegistry::global();
        assert!(!registry.node_types.is_empty());
        assert!(!registry.effects.is_empty());
        assert!(!registry.transitions.is_empty());
        assert!(!registry.easings.is_empty());
    }

    #[test]
    fn test_list_node_types() {
        let nodes = list_node_types();
        assert!(nodes.len() >= 8);

        let names: Vec<_> = nodes.iter().map(|n| n.name).collect();
        assert!(names.contains(&"Box"));
        assert!(names.contains(&"Text"));
        assert!(names.contains(&"Lottie"));
    }

    #[test]
    fn test_supports_animation() {
        assert!(supports_animation("Box", "opacity"));
        assert!(supports_animation("Box", "border_radius"));
        assert!(supports_animation("Text", "font_size"));
        assert!(supports_animation("Lottie", "speed"));

        assert!(!supports_animation("Image", "border_radius"));
        assert!(!supports_animation("Unknown", "opacity"));
    }

    #[test]
    fn test_get_animatable_properties() {
        let box_props = get_animatable_properties("Box");
        assert!(box_props.contains(&"opacity"));
        assert!(box_props.contains(&"border_radius"));

        let unknown_props = get_animatable_properties("NonExistent");
        assert!(unknown_props.is_empty());
    }
}
