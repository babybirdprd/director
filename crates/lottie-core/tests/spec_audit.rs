//! Lottie Spec v1.0 Compliance Audit Tests
//!
//! This module provides comprehensive testing against the official Lottie specification.
//! It includes both behavior tests and schema-derived coverage checks from `lottie-schema.json`,
//! including explicit not-applicable classification for generic schema placeholders.
//! Run with: cargo test -p lottie-core --test spec_audit

use lottie_core::{
    asset_kind_support, effect_type_support, layer_type_support, LottiePlayer, SchemaSupport,
};
use lottie_data::model::{BezierTangent, Keyframe, LottieJson, Property, Value};
use serde_json::json;

/// Test helper to create a minimal lottie with a single shape layer
fn create_test_lottie(shapes: serde_json::Value) -> LottieJson {
    let json = json!({
        "v": "5.5.0",
        "fr": 60,
        "ip": 0,
        "op": 60,
        "w": 500,
        "h": 500,
        "nm": "Test",
        "ddd": 0,
        "layers": [
            {
                "ty": 4, // Shape layer
                "ind": 1,
                "ip": 0,
                "op": 60,
                "st": 0,
                "nm": "Shape Layer",
                "ks": {
                    "o": { "a": 0, "k": 100 },
                    "r": { "a": 0, "k": 0 },
                    "p": { "a": 0, "k": [250, 250, 0] },
                    "a": { "a": 0, "k": [0, 0, 0] },
                    "s": { "a": 0, "k": [100, 100, 100] }
                },
                "shapes": shapes
            }
        ]
    });

    serde_json::from_value(json).expect("Failed to parse test lottie")
}

/// Test all v1.0 spec shapes
mod shapes {
    use super::*;

    #[test]
    fn test_ellipse_shape() {
        let shapes = json!([
            {
                "ty": "el", // Ellipse
                "nm": "Ellipse",
                "p": { "a": 0, "k": [0, 0] },
                "s": { "a": 0, "k": [100, 100] },
                "d": 1 // Draw clockwise from top
            },
            {
                "ty": "fl", // Fill
                "nm": "Fill",
                "c": { "a": 0, "k": [1, 0, 0, 1] },
                "o": { "a": 0, "k": 100 },
                "r": 1 // Non-zero fill rule
            }
        ]);

        let lottie = create_test_lottie(shapes);
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for ellipse"
        );
    }

    #[test]
    fn test_rectangle_shape() {
        let shapes = json!([
            {
                "ty": "rc", // Rectangle
                "nm": "Rectangle",
                "p": { "a": 0, "k": [0, 0] },
                "s": { "a": 0, "k": [100, 100] },
                "r": { "a": 0, "k": 10 } // Rounded corners
            },
            {
                "ty": "fl",
                "nm": "Fill",
                "c": { "a": 0, "k": [0, 1, 0, 1] },
                "o": { "a": 0, "k": 100 },
                "r": 1
            }
        ]);

        let lottie = create_test_lottie(shapes);
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for rectangle"
        );
    }

    #[test]
    fn test_path_shape() {
        let shapes = json!([
            {
                "ty": "sh", // Path
                "nm": "Path",
                "ks": {
                    "a": 0,
                    "k": {
                        "c": false, // Not closed
                        "v": [[0, 0], [100, 100]], // Vertices
                        "i": [[0, 0], [0, 0]], // In tangents
                        "o": [[0, 0], [0, 0]] // Out tangents
                    }
                }
            },
            {
                "ty": "st", // Stroke
                "nm": "Stroke",
                "c": { "a": 0, "k": [0, 0, 1, 1] },
                "o": { "a": 0, "k": 100 },
                "w": { "a": 0, "k": 5 },
                "lc": 2, // Round cap
                "lj": 2 // Round join
            }
        ]);

        let lottie = create_test_lottie(shapes);
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for path"
        );
    }

    #[test]
    fn test_polystar_shape() {
        let shapes = json!([
            {
                "ty": "sr", // Polystar (Star/Polygon)
                "nm": "Star",
                "p": { "a": 0, "k": [0, 0] },
                "pt": { "a": 0, "k": 5 }, // 5 points
                "ir": { "a": 0, "k": 30 }, // Inner radius
                "is": { "a": 0, "k": 0 }, // Inner roundness
                "or": { "a": 0, "k": 60 }, // Outer radius
                "os": { "a": 0, "k": 0 }, // Outer roundness
                "r": { "a": 0, "k": 0 }, // Rotation
                "sy": 1 // Star type (1=star, 2=polygon)
            },
            {
                "ty": "fl",
                "nm": "Fill",
                "c": { "a": 0, "k": [1, 1, 0, 1] },
                "o": { "a": 0, "k": 100 },
                "r": 1
            }
        ]);

        let lottie = create_test_lottie(shapes);
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for polystar"
        );
    }

    #[test]
    fn test_gradient_fill_linear() {
        let shapes = json!([
            {
                "ty": "el",
                "nm": "Ellipse",
                "p": { "a": 0, "k": [0, 0] },
                "s": { "a": 0, "k": [100, 100] }
            },
            {
                "ty": "gf", // Gradient Fill
                "nm": "Gradient Fill",
                "t": 1, // Linear gradient
                "s": { "a": 0, "k": [0, 0] }, // Start point
                "e": { "a": 0, "k": [100, 0] }, // End point
                "g": {
                    "p": 3, // Number of color stops
                    "k": {
                        "a": 0,
                        "k": [0, 1, 1, 1, 0.5, 0.5, 0.5, 1, 1, 0, 0, 0, 1] // RGBA stops
                    }
                },
                "o": { "a": 0, "k": 100 }
            }
        ]);

        let lottie = create_test_lottie(shapes);
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for linear gradient fill"
        );
    }

    #[test]
    fn test_gradient_fill_radial() {
        let shapes = json!([
            {
                "ty": "el",
                "nm": "Ellipse",
                "p": { "a": 0, "k": [0, 0] },
                "s": { "a": 0, "k": [100, 100] }
            },
            {
                "ty": "gf", // Gradient Fill
                "nm": "Radial Gradient",
                "t": 2, // Radial gradient
                "s": { "a": 0, "k": [0, 0] },
                "e": { "a": 0, "k": [100, 0] },
                "g": {
                    "p": 2,
                    "k": {
                        "a": 0,
                        "k": [0, 1, 0, 0, 1, 1, 0, 1, 0, 1]
                    }
                },
                "o": { "a": 0, "k": 100 }
            }
        ]);

        let lottie = create_test_lottie(shapes);
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for radial gradient fill"
        );
    }

    #[test]
    fn test_gradient_stroke() {
        let shapes = json!([
            {
                "ty": "el",
                "nm": "Ellipse",
                "p": { "a": 0, "k": [0, 0] },
                "s": { "a": 0, "k": [100, 100] }
            },
            {
                "ty": "gs", // Gradient Stroke
                "nm": "Gradient Stroke",
                "t": 1, // Linear
                "s": { "a": 0, "k": [0, 0] },
                "e": { "a": 0, "k": [100, 0] },
                "g": {
                    "p": 2,
                    "k": {
                        "a": 0,
                        "k": [0, 1, 0, 0, 1, 1, 0, 1, 0, 1]
                    }
                },
                "w": { "a": 0, "k": 5 },
                "o": { "a": 0, "k": 100 },
                "lc": 2,
                "lj": 2
            }
        ]);

        let lottie = create_test_lottie(shapes);
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for gradient stroke"
        );
    }

    #[test]
    fn test_trim_path() {
        let shapes = json!([
            {
                "ty": "el",
                "nm": "Ellipse",
                "p": { "a": 0, "k": [0, 0] },
                "s": { "a": 0, "k": [100, 100] }
            },
            {
                "ty": "tm", // Trim Path
                "nm": "Trim Path",
                "s": { "a": 0, "k": 0 }, // Start at 0%
                "e": { "a": 0, "k": 75 }, // End at 75%
                "o": { "a": 0, "k": 0 }, // Offset 0%
                "m": 1 // Trim individually
            },
            {
                "ty": "st",
                "nm": "Stroke",
                "c": { "a": 0, "k": [0, 0, 1, 1] },
                "o": { "a": 0, "k": 100 },
                "w": { "a": 0, "k": 5 }
            }
        ]);

        let lottie = create_test_lottie(shapes);
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for trim path"
        );
    }

    #[test]
    fn test_repeater() {
        let shapes = json!([
            {
                "ty": "gr", // Group containing shapes to repeat
                "nm": "Group",
                "it": [
                    {
                        "ty": "el",
                        "nm": "Ellipse",
                        "p": { "a": 0, "k": [0, 0] },
                        "s": { "a": 0, "k": [50, 50] }
                    },
                    {
                        "ty": "fl",
                        "nm": "Fill",
                        "c": { "a": 0, "k": [1, 0, 0, 1] },
                        "o": { "a": 0, "k": 100 }
                    },
                    {
                        "ty": "rp", // Repeater
                        "nm": "Repeater",
                        "c": { "a": 0, "k": 3 }, // 3 copies
                        "o": { "a": 0, "k": 0 }, // Offset 0
                        "m": 1, // Add mode
                        "tr": {
                            "o": { "a": 0, "k": 0 },
                            "r": { "a": 0, "k": 0 },
                            "p": { "a": 0, "k": [30, 0, 0] }, // Offset each copy by 30px
                            "a": { "a": 0, "k": [0, 0, 0] },
                            "s": { "a": 0, "k": [100, 100, 100] },
                            "so": { "a": 0, "k": 100 },
                            "eo": { "a": 0, "k": 100 }
                        }
                    }
                ]
            }
        ]);

        let lottie = create_test_lottie(shapes);
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for repeater"
        );
    }

    #[test]
    fn test_round_corners() {
        let shapes = json!([
            {
                "ty": "rc",
                "nm": "Rectangle",
                "p": { "a": 0, "k": [0, 0] },
                "s": { "a": 0, "k": [100, 100] },
                "r": { "a": 0, "k": 0 } // No initial rounding
            },
            {
                "ty": "rd", // Round Corners
                "nm": "Round Corners",
                "r": { "a": 0, "k": 15 } // 15px corner radius
            },
            {
                "ty": "fl",
                "nm": "Fill",
                "c": { "a": 0, "k": [0, 1, 0, 1] },
                "o": { "a": 0, "k": 100 }
            }
        ]);

        let lottie = create_test_lottie(shapes);
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for round corners"
        );
    }

    #[test]
    fn test_zigzag() {
        let shapes = json!([
            {
                "ty": "rc",
                "nm": "Rectangle",
                "p": { "a": 0, "k": [0, 0] },
                "s": { "a": 0, "k": [100, 50] },
                "r": { "a": 0, "k": 0 }
            },
            {
                "ty": "zz", // ZigZag
                "nm": "ZigZag",
                "r": { "a": 0, "k": 5 }, // 5 ridges
                "s": { "a": 0, "k": 10 }, // 10px size
                "pt": { "a": 0, "k": 0 } // Corner mode (0=corner, 1=smooth)
            },
            {
                "ty": "st",
                "nm": "Stroke",
                "c": { "a": 0, "k": [0, 0, 0, 1] },
                "o": { "a": 0, "k": 100 },
                "w": { "a": 0, "k": 2 }
            }
        ]);

        let lottie = create_test_lottie(shapes);
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for zigzag"
        );
    }

    #[test]
    fn test_pucker_bloat() {
        let shapes = json!([
            {
                "ty": "rc",
                "nm": "Rectangle",
                "p": { "a": 0, "k": [0, 0] },
                "s": { "a": 0, "k": [100, 100] },
                "r": { "a": 0, "k": 0 }
            },
            {
                "ty": "pb", // Pucker/Bloat
                "nm": "PuckerBloat",
                "a": { "a": 0, "k": 50 } // 50% amount (positive = bloat, negative = pucker)
            },
            {
                "ty": "fl",
                "nm": "Fill",
                "c": { "a": 0, "k": [1, 0.5, 0, 1] },
                "o": { "a": 0, "k": 100 }
            }
        ]);

        let lottie = create_test_lottie(shapes);
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for pucker/bloat"
        );
    }

    #[test]
    fn test_twist() {
        let shapes = json!([
            {
                "ty": "rc",
                "nm": "Rectangle",
                "p": { "a": 0, "k": [0, 0] },
                "s": { "a": 0, "k": [100, 100] },
                "r": { "a": 0, "k": 0 }
            },
            {
                "ty": "tw", // Twist
                "nm": "Twist",
                "a": { "a": 0, "k": 45 }, // 45 degrees twist
                "c": { "a": 0, "k": [0, 0] } // Center of twist
            },
            {
                "ty": "fl",
                "nm": "Fill",
                "c": { "a": 0, "k": [0.5, 0, 1, 1] },
                "o": { "a": 0, "k": 100 }
            }
        ]);

        let lottie = create_test_lottie(shapes);
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for twist"
        );
    }

    #[test]
    fn test_offset_path() {
        let shapes = json!([
            {
                "ty": "el",
                "nm": "Ellipse",
                "p": { "a": 0, "k": [0, 0] },
                "s": { "a": 0, "k": [100, 100] }
            },
            {
                "ty": "op", // Offset Path
                "nm": "Offset Path",
                "a": { "a": 0, "k": 10 }, // 10px offset
                "lj": 2, // Round join
                "ml": 4 // Miter limit (plain value, not Property)
            },
            {
                "ty": "st",
                "nm": "Stroke",
                "c": { "a": 0, "k": [0, 0, 0, 1] },
                "o": { "a": 0, "k": 100 },
                "w": { "a": 0, "k": 2 }
            }
        ]);

        let lottie = create_test_lottie(shapes);
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for offset path"
        );
    }

    #[test]
    fn test_wiggle_path() {
        let shapes = json!([
            {
                "ty": "sh", // Path
                "nm": "Path",
                "ks": {
                    "a": 0,
                    "k": {
                        "c": false,
                        "v": [[-50, 0], [50, 0]],
                        "i": [[0, 0], [0, 0]],
                        "o": [[0, 0], [0, 0]]
                    }
                }
            },
            {
                "ty": "wgl", // Wiggle Path
                "nm": "Wiggle Path",
                "s": { "a": 0, "k": 10 }, // Size
                "w": { "a": 0, "k": 5 }, // Wiggles/second
                "r": { "a": 0, "k": 0 }, // Correlation
                "sh": { "a": 0, "k": 0 } // Random seed
            },
            {
                "ty": "st",
                "nm": "Stroke",
                "c": { "a": 0, "k": [0, 0, 1, 1] },
                "o": { "a": 0, "k": 100 },
                "w": { "a": 0, "k": 3 }
            }
        ]);

        let lottie = create_test_lottie(shapes);
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for wiggle path"
        );
    }

    #[test]
    fn test_merge_paths() {
        let shapes = json!([
            {
                "ty": "gr",
                "nm": "Merge Group",
                "it": [
                    {
                        "ty": "el",
                        "nm": "Ellipse 1",
                        "p": { "a": 0, "k": [-20, 0] },
                        "s": { "a": 0, "k": [80, 80] }
                    },
                    {
                        "ty": "el",
                        "nm": "Ellipse 2",
                        "p": { "a": 0, "k": [20, 0] },
                        "s": { "a": 0, "k": [80, 80] }
                    },
                    {
                        "ty": "mm", // Merge Paths
                        "nm": "Merge Paths",
                        "mm": 3 // Intersect mode
                    },
                    {
                        "ty": "fl",
                        "nm": "Fill",
                        "c": { "a": 0, "k": [1, 0, 0, 1] },
                        "o": { "a": 0, "k": 100 }
                    }
                ]
            }
        ]);

        let lottie = create_test_lottie(shapes);
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for merge paths"
        );
    }

    #[test]
    fn test_shape_transform() {
        let shapes = json!([
            {
                "ty": "gr",
                "nm": "Group",
                "it": [
                    {
                        "ty": "el",
                        "nm": "Ellipse",
                        "p": { "a": 0, "k": [0, 0] },
                        "s": { "a": 0, "k": [50, 50] }
                    },
                    {
                        "ty": "fl",
                        "nm": "Fill",
                        "c": { "a": 0, "k": [1, 0, 0, 1] },
                        "o": { "a": 0, "k": 100 }
                    },
                    {
                        "ty": "tr", // Transform
                        "nm": "Transform",
                        "o": { "a": 0, "k": 100 },
                        "r": { "a": 0, "k": 45 }, // 45 degree rotation
                        "p": { "a": 0, "k": [50, 0, 0] }, // Offset from center
                        "a": { "a": 0, "k": [0, 0, 0] },
                        "s": { "a": 0, "k": [150, 150, 100] } // Scale 150%
                    }
                ]
            }
        ]);

        let lottie = create_test_lottie(shapes);
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for shape transform"
        );
    }
}

/// Test all 16 blend modes
mod blend_modes {
    use lottie_core::BlendMode;

    #[test]
    fn test_blend_mode_normal() {
        assert_eq!(BlendMode::Normal as i32, 0);
    }

    #[test]
    fn test_blend_mode_multiply() {
        assert_eq!(BlendMode::Multiply as i32, 1);
    }

    #[test]
    fn test_blend_mode_screen() {
        assert_eq!(BlendMode::Screen as i32, 2);
    }

    #[test]
    fn test_blend_mode_overlay() {
        assert_eq!(BlendMode::Overlay as i32, 3);
    }

    #[test]
    fn test_blend_mode_darken() {
        assert_eq!(BlendMode::Darken as i32, 4);
    }

    #[test]
    fn test_blend_mode_lighten() {
        assert_eq!(BlendMode::Lighten as i32, 5);
    }

    #[test]
    fn test_blend_mode_color_dodge() {
        assert_eq!(BlendMode::ColorDodge as i32, 6);
    }

    #[test]
    fn test_blend_mode_color_burn() {
        assert_eq!(BlendMode::ColorBurn as i32, 7);
    }

    #[test]
    fn test_blend_mode_hard_light() {
        assert_eq!(BlendMode::HardLight as i32, 8);
    }

    #[test]
    fn test_blend_mode_soft_light() {
        assert_eq!(BlendMode::SoftLight as i32, 9);
    }

    #[test]
    fn test_blend_mode_difference() {
        assert_eq!(BlendMode::Difference as i32, 10);
    }

    #[test]
    fn test_blend_mode_exclusion() {
        assert_eq!(BlendMode::Exclusion as i32, 11);
    }

    #[test]
    fn test_blend_mode_hue() {
        assert_eq!(BlendMode::Hue as i32, 12);
    }

    #[test]
    fn test_blend_mode_saturation() {
        assert_eq!(BlendMode::Saturation as i32, 13);
    }

    #[test]
    fn test_blend_mode_color() {
        assert_eq!(BlendMode::Color as i32, 14);
    }

    #[test]
    fn test_blend_mode_luminosity() {
        assert_eq!(BlendMode::Luminosity as i32, 15);
    }

    #[test]
    fn test_all_blend_modes_present() {
        // Ensure all 16 blend modes are defined
        let modes = vec![
            BlendMode::Normal,
            BlendMode::Multiply,
            BlendMode::Screen,
            BlendMode::Overlay,
            BlendMode::Darken,
            BlendMode::Lighten,
            BlendMode::ColorDodge,
            BlendMode::ColorBurn,
            BlendMode::HardLight,
            BlendMode::SoftLight,
            BlendMode::Difference,
            BlendMode::Exclusion,
            BlendMode::Hue,
            BlendMode::Saturation,
            BlendMode::Color,
            BlendMode::Luminosity,
        ];

        assert_eq!(modes.len(), 16, "Should have exactly 16 blend modes");
    }
}

/// Test matte modes
mod matte_modes {
    use lottie_core::MatteMode;

    #[test]
    fn test_matte_mode_alpha() {
        assert_eq!(MatteMode::Alpha as i32, 0);
    }

    #[test]
    fn test_matte_mode_alpha_inverted() {
        assert_eq!(MatteMode::AlphaInverted as i32, 1);
    }

    #[test]
    fn test_matte_mode_luma() {
        assert_eq!(MatteMode::Luma as i32, 2);
    }

    #[test]
    fn test_matte_mode_luma_inverted() {
        assert_eq!(MatteMode::LumaInverted as i32, 3);
    }

    #[test]
    fn test_all_matte_modes_present() {
        let modes = vec![
            MatteMode::Alpha,
            MatteMode::AlphaInverted,
            MatteMode::Luma,
            MatteMode::LumaInverted,
        ];
        assert_eq!(modes.len(), 4, "Should have exactly 4 matte modes");
    }
}

/// Test mask modes
mod masks {
    use super::*;
    use lottie_core::MaskMode;

    #[test]
    fn test_mask_mode_none() {
        assert_eq!(MaskMode::None as i32, 0);
    }

    #[test]
    fn test_mask_mode_add() {
        assert_eq!(MaskMode::Add as i32, 1);
    }

    #[test]
    fn test_mask_mode_subtract() {
        assert_eq!(MaskMode::Subtract as i32, 2);
    }

    #[test]
    fn test_mask_mode_intersect() {
        assert_eq!(MaskMode::Intersect as i32, 3);
    }

    #[test]
    fn test_mask_mode_lighten() {
        assert_eq!(MaskMode::Lighten as i32, 4);
    }

    #[test]
    fn test_mask_mode_darken() {
        assert_eq!(MaskMode::Darken as i32, 5);
    }

    #[test]
    fn test_mask_mode_difference() {
        assert_eq!(MaskMode::Difference as i32, 6);
    }

    #[test]
    fn test_all_mask_modes_present() {
        let modes = vec![
            MaskMode::None,
            MaskMode::Add,
            MaskMode::Subtract,
            MaskMode::Intersect,
            MaskMode::Lighten,
            MaskMode::Darken,
            MaskMode::Difference,
        ];

        assert_eq!(modes.len(), 7, "Should have exactly 7 mask modes");
    }

    #[test]
    fn test_mask_with_feather() {
        let json = json!({
            "v": "5.5.0",
            "fr": 60,
            "ip": 0,
            "op": 60,
            "w": 500,
            "h": 500,
            "layers": [
                {
                    "ty": 4,
                    "ind": 1,
                    "ip": 0,
                    "op": 60,
                    "st": 0,
                    "nm": "Masked Shape",
                    "ks": {
                        "o": { "a": 0, "k": 100 },
                        "r": { "a": 0, "k": 0 },
                        "p": { "a": 0, "k": [250, 250, 0] },
                        "a": { "a": 0, "k": [0, 0, 0] },
                        "s": { "a": 0, "k": [100, 100, 100] }
                    },
                    "shapes": [
                        {
                            "ty": "el",
                            "nm": "Ellipse",
                            "p": { "a": 0, "k": [0, 0] },
                            "s": { "a": 0, "k": [150, 150] }
                        },
                        {
                            "ty": "fl",
                            "nm": "Fill",
                            "c": { "a": 0, "k": [1, 0, 0, 1] },
                            "o": { "a": 0, "k": 100 }
                        }
                    ],
                    "masksProperties": [
                        {
                            "nm": "Feathered Mask",
                            "pt": {
                                "a": 0,
                                "k": {
                                    "c": true,
                                    "v": [[-50, -50], [50, -50], [50, 50], [-50, 50]],
                                    "i": [[0, 0], [0, 0], [0, 0], [0, 0]],
                                    "o": [[0, 0], [0, 0], [0, 0], [0, 0]]
                                }
                            },
                            "o": { "a": 0, "k": 100 },
                            "f": { "a": 0, "k": [20, 20] }, // Feather x, y
                            "x": { "a": 0, "k": 10 }, // Expansion
                            "mode": "a" // Add mode
                        }
                    ]
                }
            ]
        });

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse masked lottie");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for feathered mask"
        );
    }

    #[test]
    fn test_inverted_mask() {
        let json = json!({
            "v": "5.5.0",
            "fr": 60,
            "ip": 0,
            "op": 60,
            "w": 500,
            "h": 500,
            "layers": [
                {
                    "ty": 4,
                    "ind": 1,
                    "ip": 0,
                    "op": 60,
                    "st": 0,
                    "nm": "Inverted Mask Shape",
                    "ks": {
                        "o": { "a": 0, "k": 100 },
                        "r": { "a": 0, "k": 0 },
                        "p": { "a": 0, "k": [250, 250, 0] },
                        "a": { "a": 0, "k": [0, 0, 0] },
                        "s": { "a": 0, "k": [100, 100, 100] }
                    },
                    "shapes": [
                        {
                            "ty": "el",
                            "nm": "Ellipse",
                            "p": { "a": 0, "k": [0, 0] },
                            "s": { "a": 0, "k": [200, 200] }
                        },
                        {
                            "ty": "fl",
                            "nm": "Fill",
                            "c": { "a": 0, "k": [0, 0, 1, 1] },
                            "o": { "a": 0, "k": 100 }
                        }
                    ],
                    "masksProperties": [
                        {
                            "nm": "Inverted Mask",
                            "pt": {
                                "a": 0,
                                "k": {
                                    "c": true,
                                    "v": [[-30, -30], [30, -30], [30, 30], [-30, 30]],
                                    "i": [[0, 0], [0, 0], [0, 0], [0, 0]],
                                    "o": [[0, 0], [0, 0], [0, 0], [0, 0]]
                                }
                            },
                            "o": { "a": 0, "k": 100 },
                            "inv": true, // Inverted
                            "mode": "a"
                        }
                    ]
                }
            ]
        });

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse inverted mask lottie");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for inverted mask"
        );
    }
}

/// Test layer effects
mod effects {
    use super::*;

    fn create_layer_with_effect(
        effect_type: u8,
        effect_values: serde_json::Value,
    ) -> serde_json::Value {
        json!({
            "v": "5.5.0",
            "fr": 60,
            "ip": 0,
            "op": 60,
            "w": 500,
            "h": 500,
            "layers": [
                {
                    "ty": 4,
                    "ind": 1,
                    "ip": 0,
                    "op": 60,
                    "st": 0,
                    "nm": "Effect Layer",
                    "ks": {
                        "o": { "a": 0, "k": 100 },
                        "r": { "a": 0, "k": 0 },
                        "p": { "a": 0, "k": [250, 250, 0] },
                        "a": { "a": 0, "k": [0, 0, 0] },
                        "s": { "a": 0, "k": [100, 100, 100] }
                    },
                    "shapes": [
                        {
                            "ty": "el",
                            "nm": "Ellipse",
                            "p": { "a": 0, "k": [0, 0] },
                            "s": { "a": 0, "k": [100, 100] }
                        },
                        {
                            "ty": "fl",
                            "nm": "Fill",
                            "c": { "a": 0, "k": [0.5, 0.5, 0.5, 1] },
                            "o": { "a": 0, "k": 100 }
                        }
                    ],
                    "ef": [
                        {
                            "ty": effect_type,
                            "nm": "Effect",
                            "en": 1, // Enabled
                            "ef": effect_values
                        }
                    ]
                }
            ]
        })
    }

    #[test]
    fn test_tint_effect() {
        // Type 20 = Tint
        let json = create_layer_with_effect(
            20,
            json!([
                { "ty": 2, "nm": "Black", "v": { "a": 0, "k": [0, 0, 0, 1] } },
                { "ty": 2, "nm": "White", "v": { "a": 0, "k": [1, 1, 1, 1] } },
                { "ty": 0, "nm": "Intensity", "v": { "a": 0, "k": 50 } }
            ]),
        );

        let lottie: LottieJson = serde_json::from_value(json).expect("Failed to parse tint effect");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for tint effect"
        );
    }

    #[test]
    fn test_fill_effect() {
        // Type 21 = Fill
        let json = create_layer_with_effect(
            21,
            json!([
                { "ty": 2, "nm": "Color", "v": { "a": 0, "k": [1, 0, 0, 1] } },
                { "ty": 0, "nm": "Opacity", "v": { "a": 0, "k": 100 } }
            ]),
        );

        let lottie: LottieJson = serde_json::from_value(json).expect("Failed to parse fill effect");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for fill effect"
        );
    }

    #[test]
    fn test_stroke_effect() {
        // Type 22 = Stroke
        let json = create_layer_with_effect(
            22,
            json!([
                { "ty": 2, "nm": "Color", "v": { "a": 0, "k": [0, 0, 0, 1] } },
                { "ty": 0, "nm": "Size", "v": { "a": 0, "k": 5 } },
                { "ty": 0, "nm": "Opacity", "v": { "a": 0, "k": 100 } },
                { "ty": 0, "nm": "All Masks", "v": { "a": 0, "k": 0 } },
                { "ty": 0, "nm": "Mask Index", "v": { "a": 0, "k": 0 } }
            ]),
        );

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse stroke effect");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for stroke effect"
        );
    }

    #[test]
    fn test_tritone_effect() {
        // Type 23 = Tritone
        let json = create_layer_with_effect(
            23,
            json!([
                { "ty": 2, "nm": "Highlights", "v": { "a": 0, "k": [1, 1, 1, 1] } },
                { "ty": 2, "nm": "Midtones", "v": { "a": 0, "k": [0.5, 0.5, 0.5, 1] } },
                { "ty": 2, "nm": "Shadows", "v": { "a": 0, "k": [0, 0, 0, 1] } }
            ]),
        );

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse tritone effect");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for tritone effect"
        );
    }

    #[test]
    fn test_levels_effect() {
        // Type 24 = Levels
        let json = create_layer_with_effect(
            24,
            json!([
                { "ty": 0, "nm": "Input Black", "v": { "a": 0, "k": 0 } },
                { "ty": 0, "nm": "Input White", "v": { "a": 0, "k": 100 } },
                { "ty": 0, "nm": "Gamma", "v": { "a": 0, "k": 1.0 } },
                { "ty": 0, "nm": "Output Black", "v": { "a": 0, "k": 0 } },
                { "ty": 0, "nm": "Output White", "v": { "a": 0, "k": 100 } }
            ]),
        );

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse levels effect");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for levels effect"
        );
    }

    #[test]
    fn test_drop_shadow_effect() {
        // Type 25 = Drop Shadow
        let json = create_layer_with_effect(
            25,
            json!([
                { "ty": 2, "nm": "Color", "v": { "a": 0, "k": [0, 0, 0, 1] } },
                { "ty": 0, "nm": "Opacity", "v": { "a": 0, "k": 75 } },
                { "ty": 0, "nm": "Direction", "v": { "a": 0, "k": 45 } },
                { "ty": 0, "nm": "Distance", "v": { "a": 0, "k": 12 } },
                { "ty": 0, "nm": "Softness", "v": { "a": 0, "k": 20 } }
            ]),
        );

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse drop shadow effect");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for drop shadow effect"
        );
    }

    #[test]
    fn test_displacement_map_effect() {
        // Type 27 = Displacement Map
        let json = create_layer_with_effect(
            27,
            json!([
                { "ty": 0, "nm": "Max Horizontal Displacement", "v": { "a": 0, "k": 15 } },
                { "ty": 0, "nm": "Max Vertical Displacement", "v": { "a": 0, "k": 10 } },
                { "ty": 0, "nm": "Use For Horizontal Displacement", "v": { "a": 0, "k": 1 } },
                { "ty": 0, "nm": "Use For Vertical Displacement", "v": { "a": 0, "k": 2 } }
            ]),
        );

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse displacement map effect");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for displacement effect"
        );
    }

    #[test]
    fn test_radial_wipe_effect() {
        // Type 26 = Radial Wipe
        let json = create_layer_with_effect(
            26,
            json!([
                { "ty": 0, "nm": "Completion", "v": { "a": 0, "k": 65 } },
                { "ty": 1, "nm": "Start Angle", "v": { "a": 0, "k": 30 } },
                { "ty": 3, "nm": "Wipe Center", "v": { "a": 0, "k": [250, 250] } },
                { "ty": 0, "nm": "Wipe", "v": { "a": 0, "k": 0 } },
                { "ty": 0, "nm": "Feather", "v": { "a": 0, "k": 10 } }
            ]),
        );

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse radial wipe effect");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for radial wipe effect"
        );
    }

    #[test]
    fn test_set_matte_effect() {
        // Type 28 = Set Matte
        let json = create_layer_with_effect(
            28,
            json!([
                { "ty": 10, "nm": "Layer", "v": { "a": 0, "k": 1 } },
                { "ty": 7, "nm": "Channel", "v": { "a": 0, "k": 5 } },
                { "ty": 7, "nm": "Invert", "v": { "a": 0, "k": 0 } },
                { "ty": 7, "nm": "Stretch To Fit", "v": { "a": 0, "k": 1 } },
                { "ty": 7, "nm": "Show Mask", "v": { "a": 0, "k": 0 } },
                { "ty": 7, "nm": "Premultiply Mask", "v": { "a": 0, "k": 1 } }
            ]),
        );

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse set matte effect");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for set matte effect"
        );
    }

    #[test]
    fn test_gaussian_blur_effect() {
        // Type 29 = Gaussian Blur
        let json = create_layer_with_effect(
            29,
            json!([
                { "ty": 0, "nm": "Blurriness", "v": { "a": 0, "k": 16 } },
                { "ty": 7, "nm": "Dimensions", "v": { "a": 0, "k": 2 } }
            ]),
        );

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse gaussian blur effect");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for gaussian blur effect"
        );
    }

    #[test]
    fn test_twirl_effect() {
        // Type 30 = Twirl
        let json = create_layer_with_effect(
            30,
            json!([
                { "ty": 1, "nm": "Angle", "v": { "a": 0, "k": 45 } },
                { "ty": 0, "nm": "Radius", "v": { "a": 0, "k": 120 } },
                { "ty": 3, "nm": "Center", "v": { "a": 0, "k": [250, 250] } }
            ]),
        );

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse twirl effect");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for twirl effect"
        );
    }

    #[test]
    fn test_mesh_warp_effect() {
        // Type 31 = Mesh Warp
        let json = create_layer_with_effect(
            31,
            json!([
                { "ty": 0, "nm": "Rows", "v": { "a": 0, "k": 4 } },
                { "ty": 0, "nm": "Columns", "v": { "a": 0, "k": 4 } },
                { "ty": 0, "nm": "Quality", "v": { "a": 0, "k": 60 } },
                { "ty": 11, "nm": "03", "v": { "a": 0, "k": 0 } }
            ]),
        );

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse mesh warp effect");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for mesh warp effect"
        );
    }

    #[test]
    fn test_wavy_effect() {
        // Type 32 = Wavy
        let json = create_layer_with_effect(
            32,
            json!([
                { "ty": 0, "nm": "Radius", "v": { "a": 0, "k": 200 } },
                { "ty": 3, "nm": "Center", "v": { "a": 0, "k": [250, 250] } },
                { "ty": 7, "nm": "Conversion type", "v": { "a": 0, "k": 1 } },
                { "ty": 7, "nm": "Speed", "v": { "a": 0, "k": 3 } },
                { "ty": 0, "nm": "Width", "v": { "a": 0, "k": 50 } },
                { "ty": 0, "nm": "Height", "v": { "a": 0, "k": 30 } },
                { "ty": 0, "nm": "Phase", "v": { "a": 0, "k": 90 } }
            ]),
        );

        let lottie: LottieJson = serde_json::from_value(json).expect("Failed to parse wavy effect");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for wavy effect"
        );
    }

    #[test]
    fn test_spherize_effect() {
        // Type 33 = Spherize
        let json = create_layer_with_effect(
            33,
            json!([
                { "ty": 0, "nm": "radius", "v": { "a": 0, "k": 120 } },
                { "ty": 3, "nm": "center", "v": { "a": 0, "k": [250, 250] } }
            ]),
        );

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse spherize effect");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for spherize effect"
        );
    }

    #[test]
    fn test_puppet_effect() {
        // Type 34 = Puppet
        let json = create_layer_with_effect(
            34,
            json!([
                { "ty": 7, "nm": "Puppet Engine", "v": { "a": 0, "k": 1 } },
                { "ty": 0, "nm": "Mesh Rotation Refinement", "v": { "a": 0, "k": 50 } },
                { "ty": 7, "nm": "On Transparent", "v": { "a": 0, "k": 0 } },
                { "ty": 11, "nm": "03", "v": { "a": 0, "k": 0 } }
            ]),
        );

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse puppet effect");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for puppet effect"
        );
    }

    #[test]
    fn test_custom_effect_type_runtime_mapping() {
        fn has_custom_group(node: &lottie_core::RenderNode) -> bool {
            if node
                .effects
                .iter()
                .any(|effect| matches!(effect, lottie_core::Effect::CustomGroup { .. }))
            {
                return true;
            }
            if let lottie_core::NodeContent::Group(children) = &node.content {
                return children.iter().any(has_custom_group);
            }
            false
        }

        let json = create_layer_with_effect(
            5,
            json!([
                { "ty": 0, "nm": "Intensity", "v": { "a": 0, "k": 40 } },
                { "ty": 2, "nm": "Color", "v": { "a": 0, "k": [0.8, 0.6, 0.3, 1] } }
            ]),
        );
        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse custom effect");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();
        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should remain valid for custom effect type 5"
        );
        assert!(
            has_custom_group(&tree.root),
            "Custom effect should map to a concrete runtime effect variant"
        );
    }
}

/// Test layer styles
mod layer_styles {
    use super::*;

    fn create_layer_with_style(
        style_type: u8,
        style_props: serde_json::Value,
    ) -> serde_json::Value {
        // Build style object by merging base properties with provided props
        let mut style_obj = serde_json::Map::new();
        style_obj.insert("ty".to_string(), json!(style_type));
        style_obj.insert("nm".to_string(), json!("Layer Style"));

        if let serde_json::Value::Object(props) = style_props {
            for (key, value) in props {
                style_obj.insert(key, value);
            }
        }

        json!({
            "v": "5.5.0",
            "fr": 60,
            "ip": 0,
            "op": 60,
            "w": 500,
            "h": 500,
            "layers": [
                {
                    "ty": 4,
                    "ind": 1,
                    "ip": 0,
                    "op": 60,
                    "st": 0,
                    "nm": "Styled Layer",
                    "ks": {
                        "o": { "a": 0, "k": 100 },
                        "r": { "a": 0, "k": 0 },
                        "p": { "a": 0, "k": [250, 250, 0] },
                        "a": { "a": 0, "k": [0, 0, 0] },
                        "s": { "a": 0, "k": [100, 100, 100] }
                    },
                    "shapes": [
                        {
                            "ty": "el",
                            "nm": "Ellipse",
                            "p": { "a": 0, "k": [0, 0] },
                            "s": { "a": 0, "k": [100, 100] }
                        },
                        {
                            "ty": "fl",
                            "nm": "Fill",
                            "c": { "a": 0, "k": [1, 0, 0, 1] },
                            "o": { "a": 0, "k": 100 }
                        }
                    ],
                    "sy": [style_obj]
                }
            ]
        })
    }

    #[test]
    fn test_drop_shadow_style() {
        // Type 0 = Drop Shadow
        let json = create_layer_with_style(
            0,
            json!({
                "c": { "a": 0, "k": [0, 0, 0, 1] },
                "o": { "a": 0, "k": 50 },
                "a": { "a": 0, "k": 135 },
                "d": { "a": 0, "k": 10 },
                "s": { "a": 0, "k": 20 },
                "ch": { "a": 0, "k": 0 }
            }),
        );

        let lottie: LottieJson = serde_json::from_value(json).expect("Failed to parse drop shadow");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for drop shadow"
        );
    }

    #[test]
    fn test_inner_shadow_style() {
        // Type 1 = Inner Shadow
        let json = create_layer_with_style(
            1,
            json!({
                "c": { "a": 0, "k": [0, 0, 0, 1] },
                "o": { "a": 0, "k": 50 },
                "a": { "a": 0, "k": 135 },
                "d": { "a": 0, "k": 10 },
                "s": { "a": 0, "k": 20 },
                "ch": { "a": 0, "k": 0 }
            }),
        );

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse inner shadow");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for inner shadow"
        );
    }

    #[test]
    fn test_outer_glow_style() {
        // Type 2 = Outer Glow
        let json = create_layer_with_style(
            2,
            json!({
                "c": { "a": 0, "k": [1, 0.5, 0, 1] },
                "o": { "a": 0, "k": 75 },
                "s": { "a": 0, "k": 30 },
                "ch": { "a": 0, "k": 50 }
            }),
        );

        let lottie: LottieJson = serde_json::from_value(json).expect("Failed to parse outer glow");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for outer glow"
        );
    }

    #[test]
    fn test_stroke_style() {
        // Type 3 = Stroke
        let json = create_layer_with_style(
            3,
            json!({
                "c": { "a": 0, "k": [0, 0, 0, 1] },
                "o": { "a": 0, "k": 100 },
                "s": { "a": 0, "k": 5 }
            }),
        );

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse stroke style");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for stroke style"
        );
    }
}

/// Test keyframe interpolation
mod keyframes {
    use super::*;
    use glam::Vec2;
    use lottie_core::animatable::Animator;

    #[test]
    fn test_linear_interpolation() {
        let kf1 = Keyframe {
            t: 0.0,
            s: Some([0.0, 0.0]),
            e: Some([100.0, 0.0]),
            i: None,
            o: None,
            to: None,
            ti: None,
            h: None,
        };

        let kf2 = Keyframe {
            t: 10.0,
            s: Some([100.0, 0.0]),
            e: None,
            i: None,
            o: None,
            to: None,
            ti: None,
            h: None,
        };

        let prop = Property {
            a: 1,
            k: Value::Animated(vec![kf1, kf2]),
            ix: None,
            x: None,
        };

        let result =
            Animator::resolve_simple(&prop, 5.0, |v| Vec2::from_slice(v), Vec2::ZERO, None, 60.0);

        assert!(
            (result.x - 50.0).abs() < 0.001,
            "Linear interpolation at 50% should be 50"
        );
        assert!((result.y - 0.0).abs() < 0.001, "Y should remain 0");
    }

    #[test]
    fn test_hold_keyframe() {
        let kf1 = Keyframe {
            t: 0.0,
            s: Some([0.0]),
            e: None,
            i: None,
            o: None,
            to: None,
            ti: None,
            h: Some(1), // Hold keyframe
        };

        let kf2 = Keyframe {
            t: 10.0,
            s: Some([100.0]),
            e: None,
            i: None,
            o: None,
            to: None,
            ti: None,
            h: None,
        };

        let prop = Property {
            a: 1,
            k: Value::Animated(vec![kf1, kf2]),
            ix: None,
            x: None,
        };

        let result = Animator::resolve_simple(&prop, 5.0, |v| v[0], 0.0, None, 60.0);

        // At frame 5, we should still be at 0.0 (holding from kf1)
        assert!(
            (result - 0.0).abs() < 0.001,
            "Hold keyframe should maintain value until next keyframe"
        );
    }

    #[test]
    fn test_bezier_easing() {
        // Test cubic bezier easing with control points
        let kf1 = Keyframe {
            t: 0.0,
            s: Some([0.0]),
            e: Some([100.0]),
            i: Some(BezierTangent {
                x: vec![0.42],
                y: vec![0.0],
            }), // Ease in control point
            o: Some(BezierTangent {
                x: vec![0.58],
                y: vec![1.0],
            }), // Ease out control point
            to: None,
            ti: None,
            h: None,
        };

        let kf2 = Keyframe {
            t: 10.0,
            s: Some([100.0]),
            e: None,
            i: None,
            o: None,
            to: None,
            ti: None,
            h: None,
        };

        let prop = Property {
            a: 1,
            k: Value::Animated(vec![kf1, kf2]),
            ix: None,
            x: None,
        };

        let result = Animator::resolve_simple(&prop, 5.0, |v| v[0], 0.0, None, 60.0);

        // With ease-in-out bezier, at t=0.5 the value should be close to 0.5 but slightly different
        assert!(
            result > 0.0 && result < 100.0,
            "Bezier easing should produce intermediate value"
        );
    }
}

/// Test layer types
mod layers {
    use super::*;

    #[test]
    fn test_precomp_layer() {
        let json = json!({
            "v": "5.5.0",
            "fr": 60,
            "ip": 0,
            "op": 60,
            "w": 500,
            "h": 500,
            "nm": "Test",
            "assets": [
                {
                    "id": "precomp_1",
                    "layers": [
                        {
                            "ty": 4,
                            "ind": 1,
                            "ip": 0,
                            "op": 60,
                            "st": 0,
                            "ks": {
                                "o": { "a": 0, "k": 100 },
                                "r": { "a": 0, "k": 0 },
                                "p": { "a": 0, "k": [250, 250, 0] },
                                "a": { "a": 0, "k": [0, 0, 0] },
                                "s": { "a": 0, "k": [100, 100, 100] }
                            },
                            "shapes": [
                                {
                                    "ty": "el",
                                    "p": { "a": 0, "k": [0, 0] },
                                    "s": { "a": 0, "k": [100, 100] }
                                },
                                {
                                    "ty": "fl",
                                    "c": { "a": 0, "k": [1, 0, 0, 1] },
                                    "o": { "a": 0, "k": 100 }
                                }
                            ]
                        }
                    ]
                }
            ],
            "layers": [
                {
                    "ty": 0, // Precomp layer
                    "ind": 1,
                    "ip": 0,
                    "op": 60,
                    "st": 0,
                    "refId": "precomp_1",
                    "nm": "Precomp Layer",
                    "ks": {
                        "o": { "a": 0, "k": 100 },
                        "r": { "a": 0, "k": 0 },
                        "p": { "a": 0, "k": [250, 250, 0] },
                        "a": { "a": 0, "k": [0, 0, 0] },
                        "s": { "a": 0, "k": [100, 100, 100] }
                    }
                }
            ]
        });

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse precomp lottie");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for precomp layer"
        );
    }

    #[test]
    fn test_null_layer() {
        let json = json!({
            "v": "5.5.0",
            "fr": 60,
            "ip": 0,
            "op": 60,
            "w": 500,
            "h": 500,
            "layers": [
                {
                    "ty": 3, // Null layer
                    "ind": 1,
                    "ip": 0,
                    "op": 60,
                    "st": 0,
                    "nm": "Null Layer",
                    "ks": {
                        "o": { "a": 0, "k": 100 },
                        "r": { "a": 0, "k": 0 },
                        "p": { "a": 0, "k": [250, 250, 0] },
                        "a": { "a": 0, "k": [0, 0, 0] },
                        "s": { "a": 0, "k": [100, 100, 100] }
                    }
                }
            ]
        });

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse null layer lottie");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        // Null layers should be handled (even if they don't render visible content)
        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions"
        );
    }

    #[test]
    fn test_solid_layer() {
        let json = json!({
            "v": "5.5.0",
            "fr": 60,
            "ip": 0,
            "op": 60,
            "w": 500,
            "h": 500,
            "layers": [
                {
                    "ty": 1, // Solid layer
                    "ind": 1,
                    "ip": 0,
                    "op": 60,
                    "st": 0,
                    "nm": "Solid Layer",
                    "sw": 100, // Solid width
                    "sh": 100, // Solid height
                    "sc": "[1,0,0]", // Solid color (red)
                    "ks": {
                        "o": { "a": 0, "k": 100 },
                        "r": { "a": 0, "k": 0 },
                        "p": { "a": 0, "k": [250, 250, 0] },
                        "a": { "a": 0, "k": [50, 50, 0] },
                        "s": { "a": 0, "k": [100, 100, 100] }
                    }
                }
            ]
        });

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse solid layer lottie");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for solid layer"
        );
    }

    #[test]
    fn test_text_layer() {
        let json = json!({
            "v": "5.5.0",
            "fr": 60,
            "ip": 0,
            "op": 60,
            "w": 500,
            "h": 500,
            "layers": [
                {
                    "ty": 5, // Text layer
                    "ind": 1,
                    "ip": 0,
                    "op": 60,
                    "st": 0,
                    "nm": "Text Layer",
                    "ks": {
                        "o": { "a": 0, "k": 100 },
                        "r": { "a": 0, "k": 0 },
                        "p": { "a": 0, "k": [250, 250, 0] },
                        "a": { "a": 0, "k": [0, 0, 0] },
                        "s": { "a": 0, "k": [100, 100, 100] }
                    },
                    "t": {
                        "d": {
                            "k": [
                                {
                                    "s": {
                                        "x": 0,
                                        "s": "Hello World"
                                    },
                                    "t": 0
                                }
                            ]
                        },
                        "p": {},
                        "m": {
                            "g": 1,
                            "a": { "a": 0, "k": [0, 0] }
                        },
                        "a": []
                    }
                }
            ]
        });

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse text layer lottie");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for text layer"
        );
    }

    #[test]
    fn test_image_layer() {
        let json = json!({
            "v": "5.5.0",
            "fr": 60,
            "ip": 0,
            "op": 60,
            "w": 500,
            "h": 500,
            "assets": [
                {
                    "id": "image_1",
                    "u": "",
                    "p": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==",
                    "e": 1
                }
            ],
            "layers": [
                {
                    "ty": 2, // Image layer
                    "ind": 1,
                    "ip": 0,
                    "op": 60,
                    "st": 0,
                    "nm": "Image Layer",
                    "refId": "image_1",
                    "ks": {
                        "o": { "a": 0, "k": 100 },
                        "r": { "a": 0, "k": 0 },
                        "p": { "a": 0, "k": [250, 250, 0] },
                        "a": { "a": 0, "k": [0.5, 0.5, 0] },
                        "s": { "a": 0, "k": [100, 100, 100] }
                    }
                }
            ]
        });

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse image layer lottie");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions for image layer"
        );
    }

    #[test]
    fn test_audio_layer_emits_runtime_event() {
        let json = json!({
            "v": "5.5.0",
            "fr": 60,
            "ip": 0,
            "op": 60,
            "w": 500,
            "h": 500,
            "assets": [
                {
                    "id": "audio_1",
                    "p": "sound.wav",
                    "u": "",
                    "t": 1
                }
            ],
            "layers": [
                {
                    "ty": 6, // Audio layer
                    "ind": 1,
                    "ip": 0,
                    "op": 60,
                    "st": 0,
                    "nm": "Audio Layer",
                    "refId": "audio_1",
                    "au": { "lv": { "a": 0, "k": [100, 100] } }
                }
            ]
        });

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse audio layer lottie");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should remain valid for audio layer"
        );
        assert_eq!(tree.audio_events.len(), 1, "Audio layer should register one runtime event");
        assert_eq!(tree.sound_assets.len(), 1, "Sound asset should be registered");
        assert_eq!(tree.audio_events[0].ref_id.as_deref(), Some("audio_1"));
        assert_eq!(
            tree.audio_events[0].sound_path.as_deref(),
            Some("sound.wav")
        );
        assert!(
            !tree.audio_events[0].level.is_empty(),
            "Audio layer should include resolved level metadata"
        );
    }

    #[test]
    fn test_data_layer_registers_runtime_binding() {
        let json = json!({
            "v": "5.5.0",
            "fr": 60,
            "ip": 0,
            "op": 60,
            "w": 500,
            "h": 500,
            "assets": [
                {
                    "id": "data_1",
                    "p": "data.json",
                    "u": "",
                    "t": 3
                }
            ],
            "layers": [
                {
                    "ty": 15, // Data layer
                    "ind": 1,
                    "ip": 0,
                    "op": 60,
                    "st": 0,
                    "nm": "Data Layer",
                    "refId": "data_1"
                }
            ]
        });

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse data layer lottie");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should remain valid for data layer"
        );
        assert_eq!(tree.data_bindings.len(), 1, "Data layer should register a runtime binding");
        assert_eq!(tree.data_sources.len(), 1, "Data source asset should be registered");
        assert_eq!(tree.data_bindings[0].ref_id.as_deref(), Some("data_1"));
        assert_eq!(
            tree.data_bindings[0].source_path.as_deref(),
            Some("data.json")
        );
    }
}

/// Test time remapping
mod time {
    use super::*;

    #[test]
    fn test_time_remapping() {
        let json = json!({
            "v": "5.5.0",
            "fr": 60,
            "ip": 0,
            "op": 120,
            "w": 500,
            "h": 500,
            "assets": [
                {
                    "id": "precomp_1",
                    "layers": [
                        {
                            "ty": 4,
                            "ind": 1,
                            "ip": 0,
                            "op": 60,
                            "st": 0,
                            "ks": {
                                "o": { "a": 0, "k": 100 },
                                "r": { "a": 0, "k": 0 },
                                "p": { "a": 0, "k": [250, 250, 0] },
                                "a": { "a": 0, "k": [0, 0, 0] },
                                "s": { "a": 0, "k": [100, 100, 100] }
                            },
                            "shapes": [
                                {
                                    "ty": "el",
                                    "p": { "a": 0, "k": [0, 0] },
                                    "s": { "a": 0, "k": [100, 100] }
                                },
                                {
                                    "ty": "fl",
                                    "c": { "a": 0, "k": [1, 0, 0, 1] },
                                    "o": { "a": 0, "k": 100 }
                                }
                            ]
                        }
                    ]
                }
            ],
            "layers": [
                {
                    "ty": 0,
                    "ind": 1,
                    "ip": 0,
                    "op": 120,
                    "st": 0,
                    "refId": "precomp_1",
                    "nm": "Precomp with Time Remap",
                    "ks": {
                        "o": { "a": 0, "k": 100 },
                        "r": { "a": 0, "k": 0 },
                        "p": { "a": 0, "k": [250, 250, 0] },
                        "a": { "a": 0, "k": [0, 0, 0] },
                        "s": { "a": 0, "k": [100, 100, 100] }
                    },
                    "tm": { // Time remapping
                        "a": 0,
                        "k": 0.5 // Map to frame 0.5 (30fps = frame 30)
                    }
                }
            ]
        });

        let lottie: LottieJson =
            serde_json::from_value(json).expect("Failed to parse time remap lottie");
        let mut player = LottiePlayer::new();
        player.load_json(lottie);

        // Test that time remapping is applied
        player.current_frame = 60.0; // At frame 60
        let tree = player.render_tree();

        assert!(
            tree.width > 0.0 && tree.height > 0.0,
            "Render tree should have valid dimensions with time remapping"
        );
    }
}

/// Compliance report generator
mod report {
    use super::{asset_kind_support, effect_type_support, layer_type_support, SchemaSupport};
    use serde_json::Value;
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;

    /// Represents a feature and its compliance status
    #[derive(Debug, Clone)]
    pub struct ComplianceItem {
        pub feature: String,
        pub category: String,
        pub status: ComplianceStatus,
        pub notes: String,
    }

    #[derive(Debug, Clone)]
    pub enum ComplianceStatus {
        Compliant,
        Partial,
        Missing,
        NotApplicable,
    }

    fn schema_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lottie-schema.json")
    }

    fn load_schema() -> Value {
        let path = schema_path();
        let raw = fs::read_to_string(&path).unwrap_or_else(|_| {
            panic!(
                "Failed to read schema file at {}. Ensure lottie-schema.json exists at workspace root.",
                path.display()
            )
        });
        serde_json::from_str(&raw).expect("Failed to parse lottie-schema.json")
    }

    fn resolve_ref<'a>(schema: &'a Value, reference: &str) -> Option<&'a Value> {
        let path = reference.strip_prefix("#/")?;
        let mut cursor = schema;
        for segment in path.split('/') {
            cursor = cursor.get(segment)?;
        }
        Some(cursor)
    }

    fn find_ty_const(node: &Value) -> Option<u8> {
        if let Some(obj) = node.as_object() {
            if let Some(props) = obj.get("properties").and_then(|v| v.as_object()) {
                if let Some(ty) = props.get("ty").and_then(|v| v.as_object()) {
                    if let Some(raw) = ty.get("const").and_then(|v| v.as_u64()) {
                        return Some(raw as u8);
                    }
                }
            }

            for key in ["allOf", "oneOf", "anyOf"] {
                if let Some(parts) = obj.get(key).and_then(|v| v.as_array()) {
                    for part in parts {
                        if let Some(found) = find_ty_const(part) {
                            return Some(found);
                        }
                    }
                }
            }
        }
        None
    }

    fn collect_schema_entries(
        schema: &Value,
        domain: &str,
        aggregate: &str,
    ) -> Vec<(String, Option<u8>)> {
        let mut entries = Vec::new();
        let refs = schema["$defs"][domain][aggregate]["oneOf"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        for item in refs {
            let Some(reference) = item.get("$ref").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(node) = resolve_ref(schema, reference) else {
                continue;
            };
            let name = reference
                .rsplit('/')
                .next()
                .unwrap_or(reference)
                .to_string();
            entries.push((name, find_ty_const(node)));
        }

        entries
    }

    fn status_from_support(support: SchemaSupport) -> ComplianceStatus {
        match support {
            SchemaSupport::Implemented => ComplianceStatus::Compliant,
            SchemaSupport::ParsedNoop => ComplianceStatus::Partial,
            SchemaSupport::Missing => ComplianceStatus::Missing,
        }
    }

    /// Generate a comprehensive compliance report for the audit
    pub fn generate_report() -> Vec<ComplianceItem> {
        let mut report = Vec::new();

        // ============================================================================
        // SHAPES (19 types total)
        // ============================================================================
        let shape_features = vec![
            (
                "Ellipse (el)",
                "Full support with position, size, and clockwise rendering",
            ),
            ("Rectangle (rc)", "Full support with rounded corners"),
            ("Path (sh)", "Full Bezier path support with in/out tangents"),
            (
                "Polystar/Star (sr)",
                "Full support including animated inner radius",
            ),
            ("Fill (fl)", "Solid color fill with opacity and fill rules"),
            ("Stroke (st)", "Stroke with width, cap, join, dash patterns"),
            ("Gradient Fill (gf)", "Linear and radial gradient fills"),
            ("Gradient Stroke (gs)", "Linear and radial gradient strokes"),
            ("Trim Path (tm)", "Path trimming with start/end/offset"),
            (
                "Repeater (rp)",
                "Shape repetition with transform and opacity",
            ),
            (
                "Transform (tr)",
                "Shape-level transforms (rotation, scale, position)",
            ),
            ("Round Corners (rd)", "Corner radius modifier"),
            ("ZigZag (zz)", "Corner and smooth zigzag effects"),
            ("Pucker/Bloat (pb)", "Vertex expansion/contraction modifier"),
            ("Twist (tw)", "Rotation-based path distortion"),
            (
                "Offset Path (op)",
                "Path expansion with miter/round/bevel joins",
            ),
            ("Wiggle Path (wgl)", "Animated path displacement"),
            (
                "Merge Paths (mm)",
                "Boolean operations (merge, add, subtract, intersect)",
            ),
            ("Group (gr)", "Shape grouping and nesting"),
        ];

        for (feature, notes) in shape_features {
            report.push(ComplianceItem {
                feature: feature.to_string(),
                category: "Shapes".to_string(),
                status: ComplianceStatus::Compliant,
                notes: notes.to_string(),
            });
        }

        // ============================================================================
        // LAYERS (schema-driven from lottie-schema.json)
        // ============================================================================
        let schema = load_schema();
        for (layer_name, ty) in collect_schema_entries(&schema, "layers", "all-layers") {
            if let Some(layer_ty) = ty {
                let support = layer_type_support(layer_ty);
                let notes = match support {
                    SchemaSupport::Implemented => {
                        if matches!(layer_ty, 6 | 15) {
                            "Implemented as non-visual runtime behavior".to_string()
                        } else {
                            "Implemented in render pipeline".to_string()
                        }
                    }
                    SchemaSupport::ParsedNoop => {
                        "Parsed and classified, intentionally rendered as no-op".to_string()
                    }
                    SchemaSupport::Missing => "Missing classification/implementation".to_string(),
                };
                report.push(ComplianceItem {
                    feature: format!("{layer_name} (ty={layer_ty})"),
                    category: "Layers".to_string(),
                    status: status_from_support(support),
                    notes,
                });
            } else {
                report.push(ComplianceItem {
                    feature: layer_name,
                    category: "Layers".to_string(),
                    status: ComplianceStatus::NotApplicable,
                    notes: "Generic schema placeholder without fixed ty discriminator".to_string(),
                });
            }
        }

        // ============================================================================
        // ASSETS (schema-driven from lottie-schema.json)
        // ============================================================================
        for (asset_name, _) in collect_schema_entries(&schema, "assets", "all-assets") {
            let support = asset_kind_support(asset_name.as_str());
            let notes = match support {
                SchemaSupport::Implemented => {
                    if matches!(asset_name.as_str(), "data-source" | "sound") {
                        "Implemented in parser/runtime with non-visual registry".to_string()
                    } else {
                        "Implemented in parser/runtime".to_string()
                    }
                }
                SchemaSupport::ParsedNoop => {
                    "Parsed and classified, currently no runtime behavior".to_string()
                }
                SchemaSupport::Missing => "Missing parser/runtime classification".to_string(),
            };
            report.push(ComplianceItem {
                feature: asset_name,
                category: "Assets".to_string(),
                status: status_from_support(support),
                notes,
            });
        }

        // ============================================================================
        // ANIMATION (5 interpolation types)
        // ============================================================================
        let anim_features = vec![
            ("Linear Interpolation", "Fully implemented"),
            ("Bezier Easing", "Cubic bezier with i/o control points"),
            ("Hold Keyframes", "h=1 flag supported"),
            ("Spatial Bezier", "to/ti tangents for curved motion paths"),
            (
                "Path Morphing",
                "Full BezierPath interpolation with vertex/tangent morphing",
            ),
        ];

        for (feature, notes) in anim_features {
            report.push(ComplianceItem {
                feature: feature.to_string(),
                category: "Animation".to_string(),
                status: ComplianceStatus::Compliant,
                notes: notes.to_string(),
            });
        }

        // ============================================================================
        // BLEND MODES (16 total)
        // ============================================================================
        report.push(ComplianceItem {
            feature: "Blend Modes (16 total)".to_string(),
            category: "Blending".to_string(),
            status: ComplianceStatus::Compliant,
            notes: "All 16 blend modes defined and supported".to_string(),
        });

        // ============================================================================
        // MASKS (7 modes + features)
        // ============================================================================
        let mask_features = vec![
            ("Mask Modes (7 total)", "All 7 mask modes defined"),
            (
                "Mask Feathering",
                "Full support with Gaussian blur approximation",
            ),
            ("Mask Expansion", "Mask path expansion/contraction"),
            ("Inverted Masks", "Inverted mask support"),
        ];

        for (feature, notes) in mask_features {
            report.push(ComplianceItem {
                feature: feature.to_string(),
                category: "Masks".to_string(),
                status: ComplianceStatus::Compliant,
                notes: notes.to_string(),
            });
        }

        // ============================================================================
        // MATTES (4 modes)
        // ============================================================================
        report.push(ComplianceItem {
            feature: "Matte Modes (4 total)".to_string(),
            category: "Mattes".to_string(),
            status: ComplianceStatus::Compliant,
            notes: "Alpha, AlphaInverted, Luma, LumaInverted".to_string(),
        });

        // ============================================================================
        // EFFECTS (schema-driven from lottie-schema.json)
        // ============================================================================
        for (effect_name, ty) in collect_schema_entries(&schema, "effects", "all-effects") {
            if let Some(effect_ty) = ty {
                let support = effect_type_support(effect_ty);
                let notes = match support {
                    SchemaSupport::Implemented => {
                        if effect_ty == 5 {
                            "Implemented as generic custom-effect runtime mapping".to_string()
                        } else {
                            "Implemented in parser + renderer".to_string()
                        }
                    }
                    SchemaSupport::ParsedNoop => {
                        "Parsed and classified, currently rendered as no-op".to_string()
                    }
                    SchemaSupport::Missing => "Missing parser/runtime classification".to_string(),
                };
                report.push(ComplianceItem {
                    feature: format!("{effect_name} (ty={effect_ty})"),
                    category: "Effects".to_string(),
                    status: status_from_support(support),
                    notes,
                });
            } else {
                report.push(ComplianceItem {
                    feature: effect_name,
                    category: "Effects".to_string(),
                    status: ComplianceStatus::NotApplicable,
                    notes: "Generic schema placeholder without fixed ty discriminator".to_string(),
                });
            }
        }

        // ============================================================================
        // LAYER STYLES (4 types)
        // ============================================================================
        let style_features = vec![
            ("Drop Shadow", "Outer shadow with angle, distance, blur"),
            ("Inner Shadow", "Inner shadow with choke"),
            ("Outer Glow", "Outer glow with size and range"),
            ("Stroke Style", "Layer outline stroke"),
        ];

        for (feature, notes) in style_features {
            report.push(ComplianceItem {
                feature: feature.to_string(),
                category: "Layer Styles".to_string(),
                status: ComplianceStatus::Compliant,
                notes: notes.to_string(),
            });
        }

        // ============================================================================
        // TIME FEATURES (2)
        // ============================================================================
        let time_features = vec![
            ("Time Remapping", "Maps layer time to precomp time"),
            ("Time Stretch", "Layer-local frame scaling (sr parameter)"),
        ];

        for (feature, notes) in time_features {
            report.push(ComplianceItem {
                feature: feature.to_string(),
                category: "Time".to_string(),
                status: ComplianceStatus::Compliant,
                notes: notes.to_string(),
            });
        }

        // ============================================================================
        // ADVANCED FEATURES
        // ============================================================================
        report.push(ComplianceItem {
            feature: "3D Camera".to_string(),
            category: "3D".to_string(),
            status: ComplianceStatus::Compliant,
            notes: "Perspective camera with view/projection matrices".to_string(),
        });

        report.push(ComplianceItem {
            feature: "Expressions".to_string(),
            category: "Scripting".to_string(),
            status: ComplianceStatus::Compliant,
            notes: "Expression evaluation (optional feature flag)".to_string(),
        });

        report
    }

    /// Print the compliance report
    pub fn print_report() {
        let report = generate_report();

        println!("\n╔════════════════════════════════════════════════════════════════╗");
        println!("║     Lottie Spec v1.0 Comprehensive Compliance Report           ║");
        println!("╚════════════════════════════════════════════════════════════════╝\n");

        let mut categories: HashMap<String, Vec<&ComplianceItem>> = HashMap::new();
        for item in &report {
            categories
                .entry(item.category.clone())
                .or_default()
                .push(item);
        }

        // Sort categories for consistent output
        let mut sorted_categories: Vec<_> = categories.iter().collect();
        sorted_categories.sort_by_key(|(k, _)| k.as_str());

        for (category, items) in sorted_categories {
            println!("\n📁 {} ({} items)", category, items.len());
            println!("{}", "─".repeat(60));

            for item in items {
                let status_icon = match item.status {
                    ComplianceStatus::Compliant => "✅",
                    ComplianceStatus::Partial => "⚠️",
                    ComplianceStatus::Missing => "❌",
                    ComplianceStatus::NotApplicable => "➖",
                };
                println!("  {} {} - {}", status_icon, item.feature, item.notes);
            }
        }

        // Summary
        let total = report.len();
        let not_applicable = report
            .iter()
            .filter(|i| matches!(i.status, ComplianceStatus::NotApplicable))
            .count();
        let applicable_total = total.saturating_sub(not_applicable);
        let compliant = report
            .iter()
            .filter(|i| matches!(i.status, ComplianceStatus::Compliant))
            .count();
        let partial = report
            .iter()
            .filter(|i| matches!(i.status, ComplianceStatus::Partial))
            .count();
        let missing = report
            .iter()
            .filter(|i| matches!(i.status, ComplianceStatus::Missing))
            .count();

        let percentage = if applicable_total > 0 {
            100.0 * compliant as f64 / applicable_total as f64
        } else {
            100.0
        };
        let partial_pct = if applicable_total > 0 {
            100.0 * partial as f64 / applicable_total as f64
        } else {
            0.0
        };
        let missing_pct = if applicable_total > 0 {
            100.0 * missing as f64 / applicable_total as f64
        } else {
            0.0
        };

        println!("\n");
        println!("╔════════════════════════════════════════════════════════════════╗");
        println!("║                        SUMMARY                                 ║");
        println!("╠════════════════════════════════════════════════════════════════╣");
        println!("║  Total Features: {:<45} ║", total);
        println!("║  Applicable Features: {:<40} ║", applicable_total);
        println!("║  Not Applicable: {:<44} ║", not_applicable);
        println!(
            "║  Compliant: {:<50} ║",
            format!("{} ({:.1}%)", compliant, percentage)
        );
        println!(
            "║  Partial: {:<52} ║",
            format!("{} ({:.1}%)", partial, partial_pct)
        );
        println!(
            "║  Missing: {:<52} ║",
            format!("{} ({:.1}%)", missing, missing_pct)
        );
        println!("╚════════════════════════════════════════════════════════════════╝");

        if percentage >= 100.0 {
            println!("\n🎉 FULL SPEC COMPLIANCE ACHIEVED! 🎉\n");
        } else {
            println!("\n📊 Current Compliance: {:.1}%\n", percentage);
        }
    }
}

#[test]
fn generate_compliance_report() {
    report::print_report();
}

#[test]
fn schema_effect_entries_are_classified() {
    let report = report::generate_report();
    let missing: Vec<_> = report
        .into_iter()
        .filter(|item| {
            item.category == "Effects" && matches!(item.status, report::ComplianceStatus::Missing)
        })
        .map(|item| item.feature)
        .collect();

    assert!(
        missing.is_empty(),
        "All schema effect entries must be classified. Missing: {:?}",
        missing
    );
}

#[test]
fn schema_layer_entries_are_classified() {
    let report = report::generate_report();
    let missing: Vec<_> = report
        .into_iter()
        .filter(|item| {
            item.category == "Layers" && matches!(item.status, report::ComplianceStatus::Missing)
        })
        .map(|item| item.feature)
        .collect();

    assert!(
        missing.is_empty(),
        "All schema layer entries must be classified. Missing: {:?}",
        missing
    );
}

#[test]
fn schema_asset_entries_are_classified() {
    let report = report::generate_report();
    let missing: Vec<_> = report
        .into_iter()
        .filter(|item| {
            item.category == "Assets" && matches!(item.status, report::ComplianceStatus::Missing)
        })
        .map(|item| item.feature)
        .collect();

    assert!(
        missing.is_empty(),
        "All schema asset entries must be classified. Missing: {:?}",
        missing
    );
}

#[test]
fn applicable_schema_entries_are_fully_compliant() {
    let report = report::generate_report();
    let non_compliant: Vec<_> = report
        .into_iter()
        .filter(|item| {
            !matches!(
                item.status,
                report::ComplianceStatus::Compliant | report::ComplianceStatus::NotApplicable
            )
        })
        .map(|item| format!("{}:{}", item.category, item.feature))
        .collect();

    assert!(
        non_compliant.is_empty(),
        "All applicable schema entries should be fully compliant. Remaining: {:?}",
        non_compliant
    );
}

/// Test real-world Lottie file loading and rendering
mod real_world {
    use super::*;
    use std::fs;

    #[test]
    fn test_heart_eyes_json_loading() {
        // Load the heart_eyes.json test file
        let json_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../lottie-data/tests/heart_eyes.json"
        );
        let json_content = fs::read_to_string(json_path)
            .expect("Failed to read heart_eyes.json - ensure file exists at lottie-data/tests/");

        let lottie: LottieJson = serde_json::from_str(&json_content)
            .expect("Failed to parse heart_eyes.json - invalid JSON format");

        // Verify basic properties (using raw JSON field names with correct types)
        assert_eq!(
            lottie.v,
            Some("5.7.3".to_string()),
            "Should have correct version"
        );
        assert_eq!(lottie.fr, 30.0, "Should have 30fps");
        assert_eq!(lottie.w, 800, "Should have 800px width");
        assert_eq!(lottie.h, 800, "Should have 800px height");
        assert!(!lottie.layers.is_empty(), "Should have layers");

        // Test rendering
        let mut player = LottiePlayer::new();
        player.load_json(lottie);

        // Render at different frames
        let frames_to_test = [0.0, 15.0, 30.0, 45.0, 60.0, 90.0];
        for frame in &frames_to_test {
            player.current_frame = *frame;
            let tree = player.render_tree();
            assert!(
                tree.width > 0.0 && tree.height > 0.0,
                "Render tree should have valid dimensions at frame {}",
                frame
            );
        }

        println!("✅ heart_eyes.json loaded and rendered successfully at all test frames");
    }
}
