//! # Properties API
//!
//! Node property setters for Rhai scripts.
//!
//! ## Responsibilities
//! - **Content**: `set_content` for rich text updates
//! - **Style**: `set_style` for layout and visual style changes
//! - **Transform**: `set_pivot` for transform origin
//! - **Layering**: `set_z_index` for z-order
//! - **Masking**: `set_mask` for alpha masking
//! - **Blending**: `set_blend_mode` for compositing modes
//! - **Safety Guards**: stale-handle checks with script-level errors

use crate::node::{ImageNode, VideoNode};
use rhai::Engine;

use super::super::types::NodeHandle;
use super::super::utils::{
    parse_layout_style, parse_object_fit, parse_spans_from_dynamic, parse_text_style,
};

/// Register property-related Rhai functions.
pub fn register(engine: &mut Engine) {
    engine.register_fn(
        "set_content",
        |node: &mut NodeHandle, content: rhai::Dynamic| -> Result<(), Box<rhai::EvalAltResult>> {
            let spans = parse_spans_from_dynamic(content);
            let mut d = node.lock_director()?;
            node.ensure_alive(&d)?;
            let n = d
                .scene
                .get_node_mut(node.id)
                .ok_or_else(|| format!("Node {} not found", node.id))?;
            n.element.set_rich_text(spans);
            Ok(())
        },
    );

    engine.register_fn(
        "set_style",
        |node: &mut NodeHandle, style: rhai::Map| -> Result<(), Box<rhai::EvalAltResult>> {
            let mut d = node.lock_director()?;
            node.ensure_alive(&d)?;
            let n = d
                .scene
                .get_node_mut(node.id)
                .ok_or_else(|| format!("Node {} not found", node.id))?;

            let mut layout_style = n.element.layout_style();
            parse_layout_style(&style, &mut layout_style);
            n.element.set_layout_style(layout_style);
            n.dirty_style = true;

            // Handle Text
            n.element.modify_text_spans(&|spans| {
                for span in spans {
                    parse_text_style(&style, span);
                }
            });

            // Handle Media (Image/Video) ObjectFit
            if let Some(fit_str) = style
                .get("object_fit")
                .and_then(|v| v.clone().into_string().ok())
            {
                if let Some(fit) = parse_object_fit(&fit_str) {
                    if let Some(img_node) = n.element.as_any_mut().downcast_mut::<ImageNode>() {
                        img_node.object_fit = fit;
                    } else if let Some(vid_node) =
                        n.element.as_any_mut().downcast_mut::<VideoNode>()
                    {
                        vid_node.object_fit = fit;
                    }
                }
            }
            Ok(())
        },
    );

    engine.register_fn(
        "set_pivot",
        |node: &mut NodeHandle, x: f64, y: f64| -> Result<(), Box<rhai::EvalAltResult>> {
            let mut d = node.lock_director()?;
            node.ensure_alive(&d)?;
            let n = d
                .scene
                .get_node_mut(node.id)
                .ok_or_else(|| format!("Node {} not found", node.id))?;
            n.transform.pivot_x = x as f32;
            n.transform.pivot_y = y as f32;
            Ok(())
        },
    );

    engine.register_fn(
        "set_z_index",
        |node: &mut NodeHandle, z: i64| -> Result<(), Box<rhai::EvalAltResult>> {
            let mut d = node.lock_director()?;
            node.ensure_alive(&d)?;
            let n = d
                .scene
                .get_node_mut(node.id)
                .ok_or_else(|| format!("Node {} not found", node.id))?;
            n.z_index = z as i32;
            Ok(())
        },
    );

    engine.register_fn(
        "set_mask",
        |node: &mut NodeHandle, mask: NodeHandle| -> Result<(), Box<rhai::EvalAltResult>> {
            let mut d = node.lock_director()?;
            node.ensure_alive(&d)?;
            mask.ensure_alive(&d)?;
            if node.id == mask.id {
                return Err("A node cannot be used as its own mask".into());
            }

            // 1. Get the mask node's current parent
            let old_parent = d.scene.get_node(mask.id).and_then(|m_node| m_node.parent);

            // 2. Remove mask from old parent's children list
            if let Some(p_id) = old_parent {
                d.scene.remove_child(p_id, mask.id);
            }

            // 3. Set mask's parent to the new owner (node.id)
            if let Some(m_node) = d.scene.get_node_mut(mask.id) {
                m_node.parent = Some(node.id);
            }

            // 4. Assign mask_node to owner
            if let Some(n) = d.scene.get_node_mut(node.id) {
                n.mask_node = Some(mask.id);
            }
            Ok(())
        },
    );

    engine.register_fn(
        "set_blend_mode",
        |node: &mut NodeHandle, mode_str: &str| -> Result<(), Box<rhai::EvalAltResult>> {
            let mut d = node.lock_director()?;
            node.ensure_alive(&d)?;
            let mode = match mode_str {
                "clear" => skia_safe::BlendMode::Clear,
                "src" => skia_safe::BlendMode::Src,
                "dst" => skia_safe::BlendMode::Dst,
                "src_over" | "src-over" | "normal" => skia_safe::BlendMode::SrcOver,
                "dst_over" | "dst-over" => skia_safe::BlendMode::DstOver,
                "src_in" | "src-in" => skia_safe::BlendMode::SrcIn,
                "dst_in" | "dst-in" => skia_safe::BlendMode::DstIn,
                "src_out" | "src-out" => skia_safe::BlendMode::SrcOut,
                "dst_out" | "dst-out" => skia_safe::BlendMode::DstOut,
                "src_atop" | "src-atop" => skia_safe::BlendMode::SrcATop,
                "dst_atop" | "dst-atop" => skia_safe::BlendMode::DstATop,
                "xor" => skia_safe::BlendMode::Xor,
                "plus" | "add" => skia_safe::BlendMode::Plus,
                "modulate" => skia_safe::BlendMode::Modulate,
                "screen" => skia_safe::BlendMode::Screen,
                "overlay" => skia_safe::BlendMode::Overlay,
                "darken" => skia_safe::BlendMode::Darken,
                "lighten" => skia_safe::BlendMode::Lighten,
                "color_dodge" | "color-dodge" => skia_safe::BlendMode::ColorDodge,
                "color_burn" | "color-burn" => skia_safe::BlendMode::ColorBurn,
                "hard_light" | "hard-light" => skia_safe::BlendMode::HardLight,
                "soft_light" | "soft-light" => skia_safe::BlendMode::SoftLight,
                "difference" => skia_safe::BlendMode::Difference,
                "exclusion" => skia_safe::BlendMode::Exclusion,
                "multiply" => skia_safe::BlendMode::Multiply,
                "hue" => skia_safe::BlendMode::Hue,
                "saturation" => skia_safe::BlendMode::Saturation,
                "color" => skia_safe::BlendMode::Color,
                "luminosity" => skia_safe::BlendMode::Luminosity,
                _ => skia_safe::BlendMode::SrcOver,
            };
            if let Some(n) = d.scene.get_node_mut(node.id) {
                n.blend_mode = mode;
            }
            Ok(())
        },
    );
}
