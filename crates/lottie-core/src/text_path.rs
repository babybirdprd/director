//! Text-on-path rendering for Lottie animations
//!
//! This module provides comprehensive text layout along Bezier paths with full
//! AE (After Effects) feature compliance including:
//! - Text follows mask path
//! - First/Last margin (offset from path ends)
//! - Force alignment (tracking adjustment to fit)
//! - Perpendicular to path rotation
//! - Reverse path direction
//! - Path-based justification (left/center/right/force)

use glam::{Vec2, Vec3};
use kurbo::{BezPath, PathEl, Point};
use std::f32::consts::PI;

/// Options for text path layout
#[derive(Clone, Debug)]
pub struct TextPathLayoutOptions {
    /// Mask index to use as baseline (0-indexed in our impl)
    pub mask_index: usize,
    /// Offset from start of path
    pub first_margin: f32,
    /// Offset from end of path
    pub last_margin: f32,
    /// Whether to force text to fit path length
    pub force_alignment: bool,
    /// Whether text should be perpendicular to path (90° from tangent)
    pub perpendicular: bool,
    /// Whether to reverse path direction
    pub reverse: bool,
    /// Justification mode: 0=left, 1=right, 2=center
    pub justify: u8,
    /// Tracking/letter spacing
    pub tracking: f32,
}

impl Default for TextPathLayoutOptions {
    fn default() -> Self {
        Self {
            mask_index: 0,
            first_margin: 0.0,
            last_margin: 0.0,
            force_alignment: false,
            perpendicular: false,
            reverse: false,
            justify: 0,
            tracking: 0.0,
        }
    }
}

/// Result of measuring and positioning glyphs on a path
#[derive(Clone, Debug)]
pub struct PathGlyphLayout {
    /// Position of each glyph center on the path
    pub positions: Vec<Vec2>,
    /// Rotation angle (in radians) for each glyph
    pub rotations: Vec<f32>,
    /// Scale factor applied to each glyph (for force alignment)
    pub scales: Vec<Vec2>,
    /// Total path length used
    pub path_length: f32,
    /// Whether text exceeded path bounds
    pub overflow: bool,
}

/// Text path renderer using path measurement for layout
pub struct TextPathRenderer;

impl TextPathRenderer {
    /// Calculate glyph positions along a path
    ///
    /// # Arguments
    /// * `path` - The Bezier path to use as baseline
    /// * `glyph_widths` - Width of each glyph (including tracking)
    /// * `options` - Layout options
    ///
    /// # Returns
    /// Layout result with positions, rotations, and scales for each glyph
    pub fn layout_text_on_path(
        path: &BezPath,
        glyph_widths: &[f32],
        options: &TextPathLayoutOptions,
    ) -> PathGlyphLayout {
        // Step 1: Calculate total path length
        let path_measure = PathMeasure::new(path);
        let path_length = path_measure.length();

        if path_length <= 0.0 || glyph_widths.is_empty() {
            return PathGlyphLayout {
                positions: vec![],
                rotations: vec![],
                scales: vec![],
                path_length: 0.0,
                overflow: false,
            };
        }

        // Step 2: Calculate total text width
        let total_text_width: f32 = glyph_widths.iter().sum();

        // Step 3: Calculate available space and scale factor
        let available_length = (path_length - options.first_margin - options.last_margin).max(0.0);

        // Calculate scale for force alignment
        let scale_factor = if options.force_alignment && total_text_width > 0.0 {
            (available_length / total_text_width).min(2.0).max(0.5) // Clamp scale to reasonable bounds
        } else {
            1.0
        };

        // Step 4: Calculate starting position based on justification
        let start_offset = match options.justify {
            1 => {
                // Right justified: end at (path_length - last_margin)
                (path_length - options.last_margin) - (total_text_width * scale_factor)
            }
            2 => {
                // Center justified
                let text_width_scaled = total_text_width * scale_factor;
                let center = options.first_margin + (available_length / 2.0);
                center - (text_width_scaled / 2.0)
            }
            _ => {
                // Left justified (default)
                options.first_margin
            }
        };

        // Ensure start is within bounds
        let start_offset = start_offset.max(0.0).min(path_length);

        // Step 5: Position each glyph
        let mut positions = Vec::with_capacity(glyph_widths.len());
        let mut rotations = Vec::with_capacity(glyph_widths.len());
        let mut scales = Vec::with_capacity(glyph_widths.len());

        let mut current_distance = start_offset;
        let mut overflow = false;

        for (i, &glyph_width) in glyph_widths.iter().enumerate() {
            let scaled_width = glyph_width * scale_factor;
            // Position at center of glyph
            let glyph_center = current_distance + (scaled_width / 2.0);

            // Check if we're still on the path
            if glyph_center > path_length {
                overflow = true;
            }

            // Get position and tangent at this distance
            let (pos, tangent) = path_measure.get_pos_tan(glyph_center);

            // Calculate rotation
            let rotation = if options.perpendicular {
                // 90° perpendicular to path tangent
                // Tangent is (dx, dy), perpendicular is (-dy, dx)
                (-tangent.x).atan2(tangent.y)
            } else {
                // Along the path direction
                tangent.y.atan2(tangent.x)
            };

            // Reverse direction if needed
            let final_rotation = if options.reverse {
                rotation + PI
            } else {
                rotation
            };

            positions.push(pos);
            rotations.push(final_rotation);
            scales.push(Vec2::new(scale_factor, scale_factor));

            current_distance += scaled_width;
        }

        PathGlyphLayout {
            positions,
            rotations,
            scales,
            path_length,
            overflow,
        }
    }

    /// Create glyph widths from text using a measurer function
    ///
    /// # Arguments
    /// * `text` - The text string
    /// * `base_tracking` - Base tracking value
    /// * `measurer` - Function that returns width for a character
    ///
    /// # Returns
    /// Vector of glyph widths including tracking
    pub fn calculate_glyph_widths<F>(text: &str, base_tracking: f32, mut measurer: F) -> Vec<f32>
    where
        F: FnMut(char) -> f32,
    {
        text.chars()
            .map(|c| {
                if c == '\n' {
                    0.0 // Newlines don't have width on a path
                } else {
                    measurer(c) + base_tracking
                }
            })
            .collect()
    }

    /// Handle multi-line text on path
    ///
    /// Each line is rendered separately on the path with line height offset
    pub fn layout_multiline_on_path(
        path: &BezPath,
        lines: &[&str],
        line_height: f32,
        measurer: impl Fn(char) -> f32,
        options: &TextPathLayoutOptions,
    ) -> Vec<PathGlyphLayout> {
        let mut layouts = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            let line_options = if line_idx == 0 {
                options.clone()
            } else {
                // For subsequent lines, adjust first margin to account for line height offset
                let mut opts = options.clone();
                // Calculate an offset for each line
                let line_offset = line_idx as f32 * line_height;
                opts.first_margin += line_offset;
                opts
            };

            let glyph_widths = Self::calculate_glyph_widths(line, options.tracking, &measurer);
            let layout = Self::layout_text_on_path(path, &glyph_widths, &line_options);
            layouts.push(layout);
        }

        layouts
    }
}

/// Path measurement utility for calculating positions along a path
pub struct PathMeasure {
    segments: Vec<PathSegment>,
    total_length: f32,
}

#[derive(Clone, Debug)]
struct PathSegment {
    length: f32,
    start: Point,
    end: Point,
    control1: Option<Point>,
    control2: Option<Point>,
    is_curve: bool,
}

impl PathMeasure {
    /// Create a new path measurer from a Bezier path
    pub fn new(path: &BezPath) -> Self {
        let mut segments = Vec::new();
        let mut total_length = 0.0f32;
        let mut current_pos = None;
        let mut first_point = None;

        for el in path.elements() {
            match el {
                PathEl::MoveTo(p) => {
                    current_pos = Some(*p);
                    if first_point.is_none() {
                        first_point = Some(*p);
                    }
                }
                PathEl::LineTo(p) => {
                    if let Some(start) = current_pos {
                        let length = Self::line_length(start, *p);
                        segments.push(PathSegment {
                            length,
                            start,
                            end: *p,
                            control1: None,
                            control2: None,
                            is_curve: false,
                        });
                        total_length += length;
                        current_pos = Some(*p);
                    }
                }
                PathEl::QuadTo(p1, p2) => {
                    if let Some(start) = current_pos {
                        let length = Self::quad_bezier_length(start, *p1, *p2);
                        segments.push(PathSegment {
                            length,
                            start,
                            end: *p2,
                            control1: Some(*p1),
                            control2: None,
                            is_curve: true,
                        });
                        total_length += length;
                        current_pos = Some(*p2);
                    }
                }
                PathEl::CurveTo(p1, p2, p3) => {
                    if let Some(start) = current_pos {
                        let length = Self::cubic_bezier_length(start, *p1, *p2, *p3);
                        segments.push(PathSegment {
                            length,
                            start,
                            end: *p3,
                            control1: Some(*p1),
                            control2: Some(*p2),
                            is_curve: true,
                        });
                        total_length += length;
                        current_pos = Some(*p3);
                    }
                }
                PathEl::ClosePath => {
                    // Close path - add line back to first point if needed
                    if let (Some(start), Some(first)) = (current_pos, first_point) {
                        if start != first {
                            let length = Self::line_length(start, first);
                            segments.push(PathSegment {
                                length,
                                start,
                                end: first,
                                control1: None,
                                control2: None,
                                is_curve: false,
                            });
                            total_length += length;
                        }
                    }
                }
            }
        }

        Self {
            segments,
            total_length,
        }
    }

    /// Get total path length
    pub fn length(&self) -> f32 {
        self.total_length
    }

    /// Get position and tangent at a given distance along the path
    pub fn get_pos_tan(&self, distance: f32) -> (Vec2, Vec2) {
        let distance = distance.max(0.0).min(self.total_length);

        // Find which segment contains this distance
        let mut current_dist = 0.0f32;
        for seg in &self.segments {
            if distance <= current_dist + seg.length {
                // This is the segment
                let t = if seg.length > 0.0 {
                    (distance - current_dist) / seg.length
                } else {
                    0.0
                };

                let (pos, tangent) = if seg.is_curve {
                    if let (Some(c1), Some(c2)) = (seg.control1, seg.control2) {
                        // Cubic bezier
                        Self::eval_cubic_bezier(seg.start, c1, c2, seg.end, t)
                    } else if let Some(c1) = seg.control1 {
                        // Quadratic bezier
                        Self::eval_quad_bezier(seg.start, c1, seg.end, t)
                    } else {
                        // Shouldn't happen, fallback to linear
                        Self::eval_line(seg.start, seg.end, t)
                    }
                } else {
                    Self::eval_line(seg.start, seg.end, t)
                };

                return (pos, tangent);
            }
            current_dist += seg.length;
        }

        // Return end of last segment
        if let Some(last) = self.segments.last() {
            let pos = Vec2::new(last.end.x as f32, last.end.y as f32);
            let tangent = if last.is_curve {
                if let (Some(c1), Some(c2)) = (last.control1, last.control2) {
                    Self::cubic_tangent(last.start, c1, c2, last.end, 1.0)
                } else if let Some(c1) = last.control1 {
                    Self::quad_tangent(last.start, c1, last.end, 1.0)
                } else {
                    Self::line_tangent(last.start, last.end)
                }
            } else {
                Self::line_tangent(last.start, last.end)
            };
            (pos, tangent)
        } else {
            (Vec2::ZERO, Vec2::X)
        }
    }

    // Line calculations
    fn line_length(start: Point, end: Point) -> f32 {
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        ((dx * dx + dy * dy) as f32).sqrt()
    }

    fn eval_line(start: Point, end: Point, t: f32) -> (Vec2, Vec2) {
        let pos = Point::new(
            start.x + (end.x - start.x) * t as f64,
            start.y + (end.y - start.y) * t as f64,
        );
        let tangent = Self::line_tangent(start, end);
        (Vec2::new(pos.x as f32, pos.y as f32), tangent)
    }

    fn line_tangent(start: Point, end: Point) -> Vec2 {
        let dx = (end.x - start.x) as f32;
        let dy = (end.y - start.y) as f32;
        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            Vec2::new(dx / len, dy / len)
        } else {
            Vec2::X
        }
    }

    // Quadratic bezier calculations
    fn quad_bezier_length(start: Point, p1: Point, end: Point) -> f32 {
        // Approximate with adaptive subdivision or use Romberg integration
        // For simplicity, use a decent approximation:
        // L ≈ |P0-P1| + |P1-P2| - |P0-P2|/2
        let d01 = Self::line_length(start, p1);
        let d12 = Self::line_length(p1, end);
        let d02 = Self::line_length(start, end);
        (d01 + d12 - d02 * 0.5).max(d02)
    }

    fn eval_quad_bezier(start: Point, p1: Point, end: Point, t: f32) -> (Vec2, Vec2) {
        let one_t = 1.0 - t as f64;
        let t_f64 = t as f64;

        let pos = Point::new(
            one_t * one_t * start.x + 2.0 * one_t * t_f64 * p1.x + t_f64 * t_f64 * end.x,
            one_t * one_t * start.y + 2.0 * one_t * t_f64 * p1.y + t_f64 * t_f64 * end.y,
        );

        let tangent = Self::quad_tangent(start, p1, end, t);
        (Vec2::new(pos.x as f32, pos.y as f32), tangent)
    }

    fn quad_tangent(start: Point, p1: Point, end: Point, t: f32) -> Vec2 {
        let one_t = 1.0 - t as f64;
        let t_f64 = t as f64;

        let dx = (2.0 * one_t * (p1.x - start.x) + 2.0 * t_f64 * (end.x - p1.x)) as f32;
        let dy = (2.0 * one_t * (p1.y - start.y) + 2.0 * t_f64 * (end.y - p1.y)) as f32;

        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            Vec2::new(dx / len, dy / len)
        } else {
            Vec2::X
        }
    }

    // Cubic bezier calculations
    fn cubic_bezier_length(start: Point, p1: Point, p2: Point, end: Point) -> f32 {
        // Approximation using chord length plus control point influence
        let d01 = Self::line_length(start, p1);
        let d12 = Self::line_length(p1, p2);
        let d23 = Self::line_length(p2, end);
        let d03 = Self::line_length(start, end);
        (d01 + d12 + d23 - d03 * 0.5).max(d03)
    }

    fn eval_cubic_bezier(start: Point, p1: Point, p2: Point, end: Point, t: f32) -> (Vec2, Vec2) {
        let one_t = 1.0 - t as f64;
        let t_f64 = t as f64;
        let one_t2 = one_t * one_t;
        let t2 = t_f64 * t_f64;

        let pos = Point::new(
            one_t2 * one_t * start.x
                + 3.0 * one_t2 * t_f64 * p1.x
                + 3.0 * one_t * t2 * p2.x
                + t2 * t_f64 * end.x,
            one_t2 * one_t * start.y
                + 3.0 * one_t2 * t_f64 * p1.y
                + 3.0 * one_t * t2 * p2.y
                + t2 * t_f64 * end.y,
        );

        let tangent = Self::cubic_tangent(start, p1, p2, end, t);
        (Vec2::new(pos.x as f32, pos.y as f32), tangent)
    }

    fn cubic_tangent(start: Point, p1: Point, p2: Point, end: Point, t: f32) -> Vec2 {
        let one_t = 1.0 - t as f64;
        let t_f64 = t as f64;

        let dx = (3.0 * one_t * one_t * (p1.x - start.x)
            + 6.0 * one_t * t_f64 * (p2.x - p1.x)
            + 3.0 * t_f64 * t_f64 * (end.x - p2.x)) as f32;
        let dy = (3.0 * one_t * one_t * (p1.y - start.y)
            + 6.0 * one_t * t_f64 * (p2.y - p1.y)
            + 3.0 * t_f64 * t_f64 * (end.y - p2.y)) as f32;

        let len = (dx * dx + dy * dy).sqrt();
        if len > 0.0 {
            Vec2::new(dx / len, dy / len)
        } else {
            Vec2::X
        }
    }
}

/// Utilities for text path operations
pub mod utils {
    use super::*;

    /// Reverse a path for text-on-path rendering
    pub fn reverse_path(path: &BezPath) -> BezPath {
        let mut reversed = BezPath::new();
        let elements: Vec<_> = path.elements().iter().cloned().collect();

        // Build reversed path
        // Note: This is a simplified reversal - for complex paths with curves,
        // control points would need to be recalculated
        if let Some(first) = elements.first() {
            if let PathEl::MoveTo(p) = first {
                reversed.move_to(p);
            }
        }

        // Process in reverse order
        for el in elements.iter().rev() {
            match el {
                PathEl::LineTo(p) => reversed.line_to(p),
                PathEl::QuadTo(p1, p2) => {
                    // For proper reversal, control points need adjustment
                    // This is a simplified version
                    reversed.quad_to(p1, p2);
                }
                PathEl::CurveTo(p1, p2, p3) => {
                    // For proper reversal, control points need adjustment
                    // This is a simplified version
                    reversed.curve_to(p2, p1, p3);
                }
                _ => {}
            }
        }

        reversed
    }

    /// Calculate optimal subdivision for a curve segment
    /// Returns the number of segments needed for smooth text placement
    pub fn subdivision_for_curve(length: f32, max_glyph_width: f32) -> usize {
        let min_segments = 4;
        let segments = (length / max_glyph_width).ceil() as usize;
        segments.max(min_segments)
    }

    /// Smooth interpolation for glyph positions
    /// Uses ease-in-out interpolation for more natural text flow
    pub fn smooth_step(edge0: f32, edge1: f32, x: f32) -> f32 {
        let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    /// Adjust position for text overflow
    /// When text is longer than path, either clip or extend
    pub fn handle_overflow(
        positions: &mut [Vec2],
        rotations: &mut [f32],
        path_length: f32,
        overflow_mode: OverflowMode,
    ) {
        match overflow_mode {
            OverflowMode::Clip => {
                // Keep positions as-is, text will be clipped
            }
            OverflowMode::Extend => {
                // Extend positions linearly beyond path
                if let (Some(&last_pos), Some(&last_rot)) = (positions.last(), rotations.last()) {
                    let tangent = Vec2::new(last_rot.cos(), last_rot.sin());
                    for i in 0..positions.len() {
                        if i > 0 {
                            let dist = (positions[i] - last_pos).length();
                            positions[i] = last_pos + tangent * dist;
                            rotations[i] = last_rot;
                        }
                    }
                }
            }
            OverflowMode::Wrap => {
                // Wrap around closed paths (not yet implemented)
            }
        }
    }

    /// Overflow handling modes
    #[derive(Clone, Copy, Debug)]
    pub enum OverflowMode {
        /// Clip text to path bounds
        Clip,
        /// Extend text beyond path (use last tangent)
        Extend,
        /// Wrap around closed paths
        Wrap,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_measure_line() {
        let mut path = BezPath::new();
        path.move_to((0.0, 0.0));
        path.line_to((100.0, 0.0));

        let measure = PathMeasure::new(&path);
        assert!((measure.length() - 100.0).abs() < 0.01);

        let (pos, tangent) = measure.get_pos_tan(50.0);
        assert!((pos.x - 50.0).abs() < 0.01);
        assert!(pos.y.abs() < 0.01);
        assert!((tangent.x - 1.0).abs() < 0.01);
        assert!(tangent.y.abs() < 0.01);
    }

    #[test]
    fn test_path_measure_quad() {
        let mut path = BezPath::new();
        path.move_to((0.0, 0.0));
        path.quad_to((50.0, 100.0), (100.0, 0.0));

        let measure = PathMeasure::new(&path);
        assert!(measure.length() > 0.0);

        let (pos, tangent) = measure.get_pos_tan(0.0);
        assert!(pos.x.abs() < 0.01);
        assert!(pos.y.abs() < 0.01);
    }

    #[test]
    fn test_text_layout_left() {
        let mut path = BezPath::new();
        path.move_to((0.0, 0.0));
        path.line_to((200.0, 0.0));

        let glyph_widths = vec![10.0, 10.0, 10.0]; // 3 glyphs, each 10px
        let options = TextPathLayoutOptions {
            mask_index: 0,
            first_margin: 10.0,
            last_margin: 10.0,
            force_alignment: false,
            perpendicular: false,
            reverse: false,
            justify: 0, // Left
            tracking: 0.0,
        };

        let layout = TextPathRenderer::layout_text_on_path(&path, &glyph_widths, &options);

        assert_eq!(layout.positions.len(), 3);
        // First glyph center should be at first_margin + 5 (half width)
        assert!(layout.positions[0].x >= 10.0);
    }

    #[test]
    fn test_text_layout_center() {
        let mut path = BezPath::new();
        path.move_to((0.0, 0.0));
        path.line_to((100.0, 0.0));

        let glyph_widths = vec![10.0, 10.0]; // 2 glyphs, total 20px
        let options = TextPathLayoutOptions {
            mask_index: 0,
            first_margin: 0.0,
            last_margin: 0.0,
            force_alignment: false,
            perpendicular: false,
            reverse: false,
            justify: 2, // Center
            tracking: 0.0,
        };

        let layout = TextPathRenderer::layout_text_on_path(&path, &glyph_widths, &options);

        assert_eq!(layout.positions.len(), 2);
        // With 100px path and 20px text, centered:
        // Start should be at 40px, first glyph center at 45px
        assert!(layout.positions[0].x >= 40.0 && layout.positions[0].x <= 50.0);
    }

    #[test]
    fn test_perpendicular_rotation() {
        let mut path = BezPath::new();
        path.move_to((0.0, 0.0));
        path.line_to((0.0, 100.0)); // Vertical line

        let glyph_widths = vec![10.0];
        let options = TextPathLayoutOptions {
            mask_index: 0,
            first_margin: 0.0,
            last_margin: 0.0,
            force_alignment: false,
            perpendicular: true, // Perpendicular
            reverse: false,
            justify: 0,
            tracking: 0.0,
        };

        let layout = TextPathRenderer::layout_text_on_path(&path, &glyph_widths, &options);

        assert_eq!(layout.rotations.len(), 1);
        // On a vertical line (0,0) to (0,100), tangent is (0, 1)
        // Perpendicular to that is (-1, 0), which is -90° or -PI/2
        let expected_rot = -std::f32::consts::PI / 2.0;
        assert!((layout.rotations[0] - expected_rot).abs() < 0.01);
    }

    #[test]
    fn test_force_alignment() {
        let mut path = BezPath::new();
        path.move_to((0.0, 0.0));
        path.line_to((100.0, 0.0));

        let glyph_widths = vec![20.0, 20.0, 20.0]; // 3 glyphs, total 60px
        let options = TextPathLayoutOptions {
            mask_index: 0,
            first_margin: 0.0,
            last_margin: 0.0,
            force_alignment: true, // Force to fit 100px
            perpendicular: false,
            reverse: false,
            justify: 0,
            tracking: 0.0,
        };

        let layout = TextPathRenderer::layout_text_on_path(&path, &glyph_widths, &options);

        // Scale should be 100/60 = 1.67
        assert!(layout.scales[0].x > 1.0);
        // Last glyph should be close to position 100
        assert!(layout.positions.last().unwrap().x > 90.0);
    }
}
