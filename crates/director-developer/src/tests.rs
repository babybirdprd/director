//! # Tests for director-developer
//!
//! Unit and integration tests for the reflection and synthesis functionality.

#[cfg(test)]
mod tests {
    use crate::reflectors::{pipeline, schema, scripting};
    use crate::spec;
    use std::path::Path;

    // ============ Scripting Reflector Tests ============

    #[test]
    fn test_reflect_rhai_api_returns_valid_json() {
        let result = scripting::reflect_rhai_api();
        assert!(result.is_ok(), "Failed to reflect Rhai API");

        let json = result.unwrap();
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
        assert!(parsed.is_ok(), "Rhai API JSON is not valid");

        let value = parsed.unwrap();
        assert!(
            value.get("functions").is_some(),
            "JSON should have 'functions' key"
        );
    }

    #[test]
    fn test_find_functions_with_pattern() {
        let result = scripting::find_functions("add_");
        assert!(result.is_ok(), "Failed to find functions");

        let functions = result.unwrap();
        assert!(
            !functions.is_empty(),
            "Should find at least one function matching 'add_'"
        );
    }

    #[test]
    fn test_find_functions_with_no_match() {
        let result = scripting::find_functions("zzz_nonexistent_zzz");
        assert!(result.is_ok(), "Should succeed even with no matches");

        let functions = result.unwrap();
        assert!(functions.is_empty(), "Should find no functions");
    }

    #[test]
    fn test_get_api_summary() {
        let result = scripting::get_api_summary();
        assert!(result.is_ok(), "Failed to get API summary");

        let summary = result.unwrap();
        assert!(
            summary.function_count > 0,
            "Should have at least some functions"
        );
    }

    // ============ Schema Reflector Tests ============

    #[test]
    fn test_reflect_all_schemas() {
        let result = schema::reflect_all_schemas();
        assert!(result.is_ok(), "Failed to reflect schemas");

        let json = result.unwrap();
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
        assert!(parsed.is_ok(), "Schemas JSON is not valid");
    }

    #[test]
    fn test_schema_for_known_type() {
        let result = schema::schema_for_type("MovieRequest");
        assert!(result.is_ok(), "Failed to get schema for MovieRequest");

        let schema_json = result.unwrap();
        assert!(
            schema_json.contains("MovieRequest"),
            "Schema should mention type name"
        );
    }

    #[test]
    fn test_schema_for_unknown_type() {
        let result = schema::schema_for_type("NonExistentType");
        assert!(result.is_err(), "Should fail for unknown type");
    }

    #[test]
    fn test_list_types() {
        let types = schema::list_types();
        assert!(types.len() >= 10, "Should have at least 10 types");
        assert!(
            types.contains(&"MovieRequest"),
            "Should include MovieRequest"
        );
        assert!(types.contains(&"Node"), "Should include Node");
        assert!(types.contains(&"Scene"), "Should include Scene");
    }

    // ============ Pipeline Reflector Tests ============

    #[test]
    fn test_reflect_capabilities() {
        let result = pipeline::reflect_capabilities();
        assert!(result.is_ok(), "Failed to reflect capabilities");

        let json = result.unwrap();
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
        assert!(parsed.is_ok(), "Capabilities JSON is not valid");

        let value = parsed.unwrap();
        assert!(value.get("node_types").is_some(), "Should have node_types");
        assert!(value.get("effects").is_some(), "Should have effects");
        assert!(
            value.get("transitions").is_some(),
            "Should have transitions"
        );
    }

    #[test]
    fn test_supports_animation() {
        // Universal properties
        assert!(pipeline::supports_animation("Box", "opacity"));
        assert!(pipeline::supports_animation("Text", "x"));
        assert!(pipeline::supports_animation("Image", "scale"));

        // Type-specific properties
        assert!(pipeline::supports_animation("Box", "border_radius"));
        assert!(pipeline::supports_animation("Text", "font_size"));
        assert!(pipeline::supports_animation("Lottie", "speed"));

        // Invalid properties
        assert!(!pipeline::supports_animation("Image", "border_radius"));
        assert!(!pipeline::supports_animation("Box", "src"));
    }

    #[test]
    fn test_get_animatable_properties() {
        let box_props = pipeline::get_animatable_properties("Box");
        assert!(box_props.contains(&"opacity"));
        assert!(box_props.contains(&"border_radius"));

        let text_props = pipeline::get_animatable_properties("Text");
        assert!(text_props.contains(&"font_size"));

        let lottie_props = pipeline::get_animatable_properties("Lottie");
        assert!(lottie_props.contains(&"speed"));
    }

    // ============ Spec Module Tests ============

    #[test]
    fn test_load_example_spec() {
        let spec_path = Path::new("examples/glow_effect.ron");
        if spec_path.exists() {
            let result = spec::load_spec(spec_path);
            assert!(result.is_ok(), "Failed to load glow_effect.ron");

            let spec = result.unwrap();
            assert_eq!(spec.title, "Glow Effect");
            assert_eq!(spec.priority, 2);
            assert!(!spec.related_types.is_empty());
        }
    }

    #[test]
    fn test_validate_spec_structured() {
        let spec_path = Path::new("examples/glow_effect.ron");
        if spec_path.exists() {
            let result = spec::validate_spec(spec_path);
            assert!(result.is_ok(), "Failed to validate spec");

            let validation = result.unwrap();
            // The example spec references existing types, so should pass
            assert!(!validation.type_checks.is_empty());
            assert!(!validation.function_checks.is_empty());
        }
    }
}
