//! # Transitions System
//!
//! Handles visual transitions between scenes using SkSL shaders.
//!
//! ## Responsibilities
//! - **Transition Types**: Enum of supported transition effects.
//! - **Shader Definitions**: GLSL/SkSL code for each transition.
//! - **Shader Execution**: Composites two scene images with transition effect.
//!
//! ## Key Types
//! - `TransitionType`: Fade, Slide, Wipe, CircleOpen, Wave, Glitch, Iris, Spiral variants.
//! - `Transition`: Defines a transition between two timeline scenes.

use crate::animation::EasingType;
use skia_safe::{runtime_effect::ChildPtr, Data, RuntimeEffect};
use tracing::error;

/// The type of visual transition between scenes.
#[derive(Clone, Debug)]
pub enum TransitionType {
    Fade,
    SlideLeft,
    SlideRight,
    WipeLeft,
    WipeRight,
    CircleOpen,
    /// Wave distortion transition with configurable amplitude and frequency.
    Wave {
        amplitude: f32,
        frequency: f32,
    },
    /// Glitch/scanline distortion effect with configurable intensity.
    Glitch {
        intensity: f32,
    },
    /// Iris/circular wipe transition with configurable start and end radius.
    Iris {
        start_radius: f32,
        end_radius: f32,
    },
    /// Spiral transition with configurable number of rotations.
    Spiral {
        rotations: f32,
    },
}

/// A definition of a transition between two scenes.
#[derive(Clone)]
pub struct Transition {
    pub from_scene_idx: usize,
    pub to_scene_idx: usize,
    pub start_time: f64,
    pub duration: f64,
    pub kind: TransitionType,
    pub easing: EasingType,
}

/// Returns the SkSL shader source for the given transition type.
pub fn get_transition_shader(kind: &TransitionType) -> &'static str {
    match kind {
        TransitionType::Fade => {
            r#"
            uniform shader imageA;
            uniform shader imageB;
            uniform float progress;
            half4 main(float2 p) {
                half4 colorA = imageA.eval(p);
                half4 colorB = imageB.eval(p);
                return mix(colorA, colorB, progress);
            }
        "#
        }
        TransitionType::SlideLeft => {
            r#"
            uniform shader imageA;
            uniform shader imageB;
            uniform float progress;
            uniform float2 resolution;
            half4 main(float2 p) {
                float x_offset = resolution.x * progress;
                if (p.x < (resolution.x - x_offset)) {
                    return imageA.eval(float2(p.x + x_offset, p.y));
                } else {
                    return imageB.eval(float2(p.x - (resolution.x - x_offset), p.y));
                }
            }
        "#
        }
        TransitionType::SlideRight => {
            r#"
            uniform shader imageA;
            uniform shader imageB;
            uniform float progress;
            uniform float2 resolution;
            half4 main(float2 p) {
                float x_offset = resolution.x * progress;
                if (p.x > x_offset) {
                    return imageA.eval(float2(p.x - x_offset, p.y));
                } else {
                    return imageB.eval(float2(p.x - x_offset + resolution.x, p.y));
                }
            }
        "#
        }
        TransitionType::WipeLeft => {
            r#"
            uniform shader imageA;
            uniform shader imageB;
            uniform float progress;
            uniform float2 resolution;
            half4 main(float2 p) {
                 float boundary = resolution.x * (1.0 - progress);
                 if (p.x < boundary) {
                     return imageA.eval(p);
                 } else {
                     return imageB.eval(p);
                 }
            }
        "#
        }
        TransitionType::WipeRight => {
            r#"
            uniform shader imageA;
            uniform shader imageB;
            uniform float progress;
            uniform float2 resolution;
            half4 main(float2 p) {
                 float boundary = resolution.x * progress;
                 if (p.x > boundary) {
                     return imageA.eval(p);
                 } else {
                     return imageB.eval(p);
                 }
            }
        "#
        }
        TransitionType::CircleOpen => {
            r#"
            uniform shader imageA;
            uniform shader imageB;
            uniform float progress;
            uniform float2 resolution;
            half4 main(float2 p) {
                float2 center = resolution / 2.0;
                float max_radius = length(resolution);
                float current_radius = max_radius * progress;
                float dist = distance(p, center);
                if (dist < current_radius) {
                    return imageB.eval(p);
                } else {
                    return imageA.eval(p);
                }
            }
        "#
        }
        TransitionType::Wave {
            amplitude: _,
            frequency: _,
        } => {
            r#"
            uniform shader imageA;
            uniform shader imageB;
            uniform float progress;
            uniform float2 resolution;
            uniform float amplitude;
            uniform float frequency;
            half4 main(float2 p) {
                float2 center = resolution / 2.0;
                float dist = distance(p, center);
                float wave = sin(dist * frequency - progress * 10.0) * amplitude * (1.0 - progress);
                float2 distorted_p = p + normalize(p - center) * wave;
                half4 colorA = imageA.eval(p);
                half4 colorB = imageB.eval(distorted_p);
                return mix(colorA, colorB, progress);
            }
        "#
        }
        TransitionType::Glitch { intensity: _ } => {
            r#"
            uniform shader imageA;
            uniform shader imageB;
            uniform float progress;
            uniform float2 resolution;
            uniform float intensity;
            half4 main(float2 p) {
                float glitch_amount = intensity * progress * (1.0 - progress) * 4.0;
                float slice_height = resolution.y / 20.0;
                float slice_index = floor(p.y / slice_height);
                float offset = sin(slice_index * 12.9898) * glitch_amount * 50.0;
                float2 distorted_p = float2(p.x + offset, p.y);
                half4 colorA = imageA.eval(p);
                half4 colorB = imageB.eval(distorted_p);
                return mix(colorA, colorB, progress);
            }
        "#
        }
        TransitionType::Iris {
            start_radius: _,
            end_radius: _,
        } => {
            r#"
            uniform shader imageA;
            uniform shader imageB;
            uniform float progress;
            uniform float2 resolution;
            uniform float start_radius;
            uniform float end_radius;
            half4 main(float2 p) {
                float2 center = resolution / 2.0;
                float dist = distance(p, center);
                float current_radius = mix(start_radius, end_radius, progress);
                if (dist < current_radius) {
                    return imageB.eval(p);
                } else {
                    return imageA.eval(p);
                }
            }
        "#
        }
        TransitionType::Spiral { rotations: _ } => {
            r#"
            uniform shader imageA;
            uniform shader imageB;
            uniform float progress;
            uniform float2 resolution;
            uniform float rotations;
            half4 main(float2 p) {
                float2 center = resolution / 2.0;
                float2 uv = (p - center) / resolution;
                float angle = atan(uv.y, uv.x);
                float radius = length(uv);
                float spiral_angle = angle + rotations * 6.28318 * progress;
                float spiral_mask = smoothstep(0.0, 0.1, spiral_angle - progress * 6.28318 * rotations);
                half4 colorA = imageA.eval(p);
                half4 colorB = imageB.eval(p);
                return mix(colorA, colorB, spiral_mask);
            }
        "#
        }
    }
}

/// Draws a transition effect between two images onto the canvas.
pub fn draw_transition(
    canvas: &skia_safe::Canvas,
    img_a: &skia_safe::Image,
    img_b: &skia_safe::Image,
    progress: f32,
    kind: &TransitionType,
    width: i32,
    height: i32,
) {
    let sksl = get_transition_shader(kind);
    let result = RuntimeEffect::make_for_shader(sksl, None);
    if let Ok(effect) = result {
        let mut uniform_bytes = Vec::new();
        uniform_bytes.extend_from_slice(&progress.to_le_bytes());

        match kind {
            TransitionType::Fade => {}
            _ => {
                uniform_bytes.extend_from_slice(&(width as f32).to_le_bytes());
                uniform_bytes.extend_from_slice(&(height as f32).to_le_bytes());
            }
        }

        // Add transition-specific uniforms
        match kind {
            TransitionType::Wave {
                amplitude,
                frequency,
            } => {
                uniform_bytes.extend_from_slice(&amplitude.to_le_bytes());
                uniform_bytes.extend_from_slice(&frequency.to_le_bytes());
            }
            TransitionType::Glitch { intensity } => {
                uniform_bytes.extend_from_slice(&intensity.to_le_bytes());
            }
            TransitionType::Iris {
                start_radius,
                end_radius,
            } => {
                uniform_bytes.extend_from_slice(&start_radius.to_le_bytes());
                uniform_bytes.extend_from_slice(&end_radius.to_le_bytes());
            }
            TransitionType::Spiral { rotations } => {
                uniform_bytes.extend_from_slice(&rotations.to_le_bytes());
            }
            _ => {}
        }

        let uniforms_data = Data::new_copy(&uniform_bytes);

        let shader_a = img_a
            .to_shader(None, skia_safe::SamplingOptions::default(), None)
            .unwrap();
        let shader_b = img_b
            .to_shader(None, skia_safe::SamplingOptions::default(), None)
            .unwrap();

        let children = [ChildPtr::Shader(shader_a), ChildPtr::Shader(shader_b)];

        if let Some(shader) = effect.make_shader(uniforms_data, &children, None) {
            let mut paint = skia_safe::Paint::default();
            paint.set_shader(Some(shader));
            canvas.draw_rect(
                skia_safe::Rect::from_wh(width as f32, height as f32),
                &paint,
            );
        } else {
            error!("Failed to make shader");
        }
    } else {
        error!("Shader compilation error: {:?}", result.err());
    }
}
