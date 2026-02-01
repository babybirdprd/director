// TODO: Implement missing AE features: Offset Path, ZigZag (smooth), PuckerBloat, and core expressions.
use glam::Vec2 as GlamVec2;
use kurbo::{BezPath, ParamCurve, ParamCurveArclen, ParamCurveDeriv, PathEl, Point, Vec2};

pub trait GeometryModifier {
    fn modify(&self, path: &mut BezPath);
}

// Helper functions for Vec2 operations (kurbo Vec2 doesn't have these methods)
fn vec2_length_squared(v: Vec2) -> f64 {
    v.x * v.x + v.y * v.y
}

fn vec2_normalize(v: Vec2) -> Vec2 {
    let len = (v.x * v.x + v.y * v.y).sqrt();
    if len > 1e-10 {
        Vec2::new(v.x / len, v.y / len)
    } else {
        Vec2::new(0.0, 0.0)
    }
}

fn vec2_lerp(a: Vec2, b: Vec2, t: f64) -> Vec2 {
    Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}

// ================================================================================================
// Zig Zag
// ================================================================================================

pub struct ZigZagModifier {
    pub ridges: f32,
    pub size: f32,
    pub smooth: bool,
}

impl GeometryModifier for ZigZagModifier {
    fn modify(&self, path: &mut BezPath) {
        if self.ridges <= 0.0 || self.size == 0.0 {
            return;
        }

        // 1. Calculate total length
        // We need to iterate elements. BezPath is a Vec<PathEl>.
        // But we need to handle MoveTo properly (multiple subpaths).
        // For simplicity, assume one continuous path or handle subpaths separately.
        // But usually ZigZag applies to the whole shape contour.

        // A robust implementation would handle multiple subpaths (MoveTo..ClosePath/MoveTo).
        // Let's iterate and collect subpaths.
        let mut subpaths = Vec::new();
        let mut current_subpath = BezPath::new();

        for el in path.elements() {
            match el {
                PathEl::MoveTo(p) => {
                    if !current_subpath.elements().is_empty() {
                        subpaths.push(current_subpath);
                    }
                    current_subpath = BezPath::new();
                    current_subpath.move_to(*p);
                }
                _ => {
                    current_subpath.push(*el);
                }
            }
        }
        if !current_subpath.elements().is_empty() {
            subpaths.push(current_subpath);
        }

        let mut new_path = BezPath::new();

        for sub in subpaths {
            let len = sub_path_length(&sub);
            if len == 0.0 {
                continue;
            }

            let step = len / (self.ridges as f64);
            let mut points = Vec::new();

            // Sample points along the path
            // This is a simplification. A real implementation needs to sample normals too.
            // We need: Position and Normal (or Tangent) at each step.

            // Flatten/Walk
            let mut walker = PathWalker::new(&sub);
            for i in 0..=(self.ridges as usize) {
                let t_dist = (i as f64 * step as f64).min(len);
                if let Some((pos, tangent)) = walker.sample(t_dist) {
                    // Normal is (-tangent.y, tangent.x)
                    let normal = Vec2::new(-tangent.y, tangent.x);

                    // Zig vs Zag
                    // i % 2.
                    // But usually Lottie ZigZag:
                    // If ridges is 3, we have start, peak, valley, peak, end?
                    // "Ridges" usually means number of peaks?
                    // If size > 0, peaks go out, valleys go in?
                    // Actually, Lottie ZigZag offsets *points*.
                    // Even indices: 0 offset? Or -size?
                    // Odd indices: +size?
                    // Standard: Start point is fixed?
                    // Let's assume alternating +size / -size.

                    let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
                    let offset = normal * (self.size as f64 * dir);
                    points.push(pos + offset);
                }
            }

            // Rebuild
            if points.is_empty() {
                continue;
            }
            new_path.move_to(points[0]);

            if self.smooth {
                // Smooth ZigZag: Use cubic bezier curves to create rounded waves
                // For each segment, we create a smooth curve through the zigzag points
                // This approximates a sine wave pattern
                if points.len() >= 2 {
                    new_path.move_to(points[0]);

                    for i in 1..points.len() {
                        let prev = points[i - 1];
                        let curr = points[i];

                        // Calculate the vector between points
                        let delta = curr - prev;
                        let dist = (delta.x * delta.x + delta.y * delta.y).sqrt();

                        if dist > 1e-10 {
                            // For smooth zigzag, we use quadratic bezier curves
                            // The control point is at the midpoint, but pushed perpendicular
                            // to create the wave amplitude
                            let mid = Point::new((prev.x + curr.x) * 0.5, (prev.y + curr.y) * 0.5);

                            // Calculate perpendicular direction
                            // The perpendicular to the segment direction
                            let dir = Vec2::new(delta.x / dist, delta.y / dist);
                            let perp = Vec2::new(-dir.y, dir.x);

                            // Determine amplitude direction based on whether this is a peak or valley
                            // Even segments (i % 2 == 1) go one way, odd go the other
                            let amplitude_dir = if i % 2 == 1 { 1.0 } else { -1.0 };
                            let amplitude = self.size as f64 * 0.5 * amplitude_dir;

                            // Control point offset perpendicular to the path
                            let ctrl =
                                Point::new(mid.x + perp.x * amplitude, mid.y + perp.y * amplitude);

                            // Use quadratic bezier: start -> ctrl -> end
                            new_path.quad_to(ctrl, curr);
                        } else {
                            new_path.line_to(curr);
                        }
                    }
                }
            } else {
                for p in points.iter().skip(1) {
                    new_path.line_to(*p);
                }
            }

            // If original was closed, we should close?
            // ZigZag usually breaks closure unless the number of ridges is even?
            // We'll leave it open unless we detect closure match.
        }

        *path = new_path;
    }
}

fn sub_path_length(path: &BezPath) -> f64 {
    // Use PathWalker to calculate length
    let walker = PathWalker::new(path);
    walker.total_length
}

// ================================================================================================
// Pucker & Bloat
// ================================================================================================

pub struct PuckerBloatModifier {
    pub amount: f32, // Percentage
    pub center: GlamVec2,
}

impl GeometryModifier for PuckerBloatModifier {
    fn modify(&self, path: &mut BezPath) {
        if self.amount == 0.0 {
            return;
        }

        // Lottie Pucker/Bloat:
        // Modifies the length of tangents (control points) and moves vertices?
        // Actually, it pulls vertices towards/away from center,
        // AND adjusts tangents to maintain curvature or exaggerate it.
        //
        // Amount > 0: Bloat (Vertices move out? Tangents move in?)
        // Amount < 0: Pucker (Vertices move in? Tangents move out?)
        //
        // Specifically:
        // It interpolates the vertex position between the center and the original position.
        // It interpolates the tangent control points.

        let center = Point::new(self.center.x as f64, self.center.y as f64);
        let factor = self.amount / 100.0;

        // PuckerBloat is tricky on arbitrary paths.
        // On a Rect/Star it's clear. On a Path, it finds the "Center" of the shape?
        // Or uses the Transform center?
        // We have `center` passed in.

        let mut new_path = BezPath::new();
        // We need to iterate segments (Cubic).
        // If it's lines, it might turn them into curves?
        // Lottie PuckerBloat on a Rect turns lines into curves.

        // TODO: Implement full PuckerBloat logic.
        // For now, simple scaling of points relative to center?
        // No, that's just Scale.
        // Pucker/Bloat changes curvature.
        // If we have a line A-B. Midpoint M.
        // Pucker moves M towards center, A and B away?
        // Or moves A and B, and control points opposite?

        // Implementation:
        // Iterate elements.
        // Modify points: P = Center + (P - Center) * (1.0 + factor)?
        // Modify control points: C = Center + (C - Center) * (1.0 - factor)?
        // This creates the star/flower effect.

        let p_scale = 1.0 + factor as f64;
        let c_scale = 1.0 - factor as f64;

        // We need to track current point for MoveTo/LineTo conversion.
        for el in path.elements() {
            match el {
                PathEl::MoveTo(p) => {
                    let new_p = center + (*p - center) * p_scale;
                    new_path.move_to(new_p);
                }
                PathEl::LineTo(p) => {
                    let new_p = center + (*p - center) * p_scale;
                    new_path.line_to(new_p);
                }
                PathEl::CurveTo(p1, p2, p3) => {
                    let np1 = center + (*p1 - center) * c_scale;
                    let np2 = center + (*p2 - center) * c_scale;
                    let np3 = center + (*p3 - center) * p_scale;
                    new_path.curve_to(np1, np2, np3);
                }
                PathEl::QuadTo(p1, p2) => {
                    let np1 = center + (*p1 - center) * c_scale;
                    let np2 = center + (*p2 - center) * p_scale;
                    new_path.quad_to(np1, np2);
                }
                PathEl::ClosePath => {
                    new_path.close_path();
                }
            }
        }

        // To fix the "Previous Point" issue for LineTo, we need a better iterator.
        // But for this task, I will stick to modifying existing Curves and scaling Points.
        // It's a reasonable start.

        *path = new_path;
    }
}

// ================================================================================================
// Twist
// ================================================================================================

pub struct TwistModifier {
    pub angle: f32, // Degrees
    pub center: GlamVec2,
}

impl GeometryModifier for TwistModifier {
    fn modify(&self, path: &mut BezPath) {
        if self.angle == 0.0 {
            return;
        }

        let center = Point::new(self.center.x as f64, self.center.y as f64);
        let angle_rad = self.angle.to_radians() as f64;

        // Calculate adaptive radius based on path bounding box
        // This makes the twist effect scale-appropriate for different sized paths
        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;

        for el in path.elements() {
            let points: Vec<Point> = match el {
                PathEl::MoveTo(p) => vec![*p],
                PathEl::LineTo(p) => vec![*p],
                PathEl::CurveTo(p1, p2, p3) => vec![*p1, *p2, *p3],
                PathEl::QuadTo(p1, p2) => vec![*p1, *p2],
                PathEl::ClosePath => vec![],
            };
            for p in points {
                min_x = min_x.min(p.x);
                max_x = max_x.max(p.x);
                min_y = min_y.min(p.y);
                max_y = max_y.max(p.y);
            }
        }

        let width = max_x - min_x;
        let height = max_y - min_y;
        let max_dim = width.max(height);

        // Use 1/4 of the max dimension as the radius
        // This provides a good balance for the twist effect
        let radius = if max_dim > 0.0 { max_dim * 0.25 } else { 100.0 };

        let transform_point = |p: Point| -> Point {
            let vec = p - center;
            let dist = vec.hypot();
            if dist < 0.001 {
                return p;
            }

            let theta = angle_rad * (dist / radius);
            let (sin, cos) = theta.sin_cos();

            // Rotate vec
            let rx = vec.x * cos - vec.y * sin;
            let ry = vec.x * sin + vec.y * cos;

            center + Vec2::new(rx, ry)
        };

        let mut new_path = BezPath::new();
        for el in path.elements() {
            match el {
                PathEl::MoveTo(p) => new_path.move_to(transform_point(*p)),
                PathEl::LineTo(p) => new_path.line_to(transform_point(*p)),
                PathEl::CurveTo(p1, p2, p3) => new_path.curve_to(
                    transform_point(*p1),
                    transform_point(*p2),
                    transform_point(*p3),
                ),
                PathEl::QuadTo(p1, p2) => {
                    new_path.quad_to(transform_point(*p1), transform_point(*p2))
                }
                PathEl::ClosePath => new_path.close_path(),
            }
        }
        *path = new_path;
    }
}

// ================================================================================================
// Wiggle Paths
// ================================================================================================

pub struct WiggleModifier {
    pub seed: f32,
    pub time: f32,
    pub speed: f32,  // wiggles/sec
    pub amount: f32, // size
    pub correlation: f32,
}

impl GeometryModifier for WiggleModifier {
    fn modify(&self, path: &mut BezPath) {
        if self.amount == 0.0 {
            return;
        }

        // Noise function
        // time * speed
        let t = self.time * self.speed;

        // For each vertex, apply displacement.
        // Deterministic: use seed + vertex_index.

        let mut new_path = BezPath::new();
        let mut idx = 0;

        let noise = |idx: usize, offset: f32| -> Vec2 {
            // Simple noise: hash(idx, seed, t)
            // We want smooth noise over t.
            // Lerp(Hash(floor(t)), Hash(ceil(t)), fract(t))

            let input = t + offset; // Offset by vertex/correlation
            let t_i = input.floor();
            let t_f = input - t_i;

            // Hash function
            let h = |k: f32| -> f32 { ((k * 12.9898 + self.seed).sin() * 43758.5453).fract() };

            let n1 = h(t_i);
            let n2 = h(t_i + 1.0);
            let _val = n1 + (n2 - n1) * t_f; // Linear. Cubic is better but Linear ok for now.

            // Map 0..1 to -1..1
            // let v = (val - 0.5) * 2.0; // Unused

            // We need 2D displacement.
            // Use different seeds for X and Y.
            let hx = |k: f32| -> f32 {
                ((k * 12.9898 + self.seed + (idx as f32) * 1.1).sin() * 43758.5453).fract()
            };
            let hy = |k: f32| -> f32 {
                ((k * 78.233 + self.seed + (idx as f32) * 1.7).sin() * 43758.5453).fract()
            };

            let rx = hx(t_i) + (hx(t_i + 1.0) - hx(t_i)) * t_f;
            let ry = hy(t_i) + (hy(t_i + 1.0) - hy(t_i)) * t_f;

            Vec2::new((rx as f64 - 0.5) * 2.0, (ry as f64 - 0.5) * 2.0)
        };

        // Logic for correlation?
        // If correlation is 100% (1.0), all vertices move same.
        // If 0%, independent.
        // We can simulate this by adding `idx * (1.0 - correlation)` to the time input?
        // Or to the hash seed?
        // If we add to time `t`: `t_eff = t + idx * factor`.
        // This creates a "wave" effect.
        // If we add to seed/hash, it's spatially random.
        // Wiggle usually implies independent or wavy.
        // Let's use `offset` parameter in noise logic.

        // Iterate
        for el in path.elements() {
            match el {
                PathEl::MoveTo(p) => {
                    let d = noise(idx, 0.0) * self.amount as f64;
                    new_path.move_to(*p + d);
                    idx += 1;
                }
                PathEl::LineTo(p) => {
                    let d = noise(idx, 0.0) * self.amount as f64;
                    new_path.line_to(*p + d);
                    idx += 1;
                }
                PathEl::CurveTo(p1, p2, p3) => {
                    let d1 = noise(idx, 0.1) * self.amount as f64;
                    let d2 = noise(idx + 1, 0.2) * self.amount as f64; // Control points wiggle too?
                    let d3 = noise(idx + 2, 0.0) * self.amount as f64;
                    new_path.curve_to(*p1 + d1, *p2 + d2, *p3 + d3);
                    idx += 3;
                }
                PathEl::QuadTo(p1, p2) => {
                    let d1 = noise(idx, 0.1) * self.amount as f64;
                    let d2 = noise(idx + 1, 0.0) * self.amount as f64;
                    new_path.quad_to(*p1 + d1, *p2 + d2);
                    idx += 2;
                }
                PathEl::ClosePath => {
                    new_path.close_path();
                }
            }
        }
        *path = new_path;
    }
}

// ================================================================================================
// Offset Path
// ================================================================================================

pub struct OffsetPathModifier {
    pub amount: f32,
    pub line_join: u8,
    pub miter_limit: f32,
}

impl GeometryModifier for OffsetPathModifier {
    fn modify(&self, path: &mut BezPath) {
        if self.amount == 0.0 {
            return;
        }

        let mut new_path = BezPath::new();
        let mut current_subpath: Vec<(Point, Vec2)> = Vec::new(); // (position, normal)
        let mut last_point = Point::ZERO;
        let mut start_point = Point::ZERO;

        // Line join types: 1 = miter, 2 = round, 3 = bevel
        let line_join = self.line_join;
        let miter_limit = self.miter_limit as f64;

        // Process each element
        for el in path.elements() {
            match el {
                PathEl::MoveTo(p) => {
                    // Flush previous subpath if exists
                    if !current_subpath.is_empty() {
                        build_offset_subpath(
                            &current_subpath,
                            &mut new_path,
                            line_join,
                            miter_limit,
                            false,
                            self.amount,
                        );
                        current_subpath.clear();
                    }
                    last_point = *p;
                    start_point = *p;
                    // Don't add to current_subpath yet, wait for first segment
                }
                PathEl::LineTo(p) => {
                    let tangent = (*p - last_point);
                    if vec2_length_squared(tangent) > 1e-10 {
                        let norm_tangent = vec2_normalize(tangent);
                        let normal = Vec2::new(-norm_tangent.y, norm_tangent.x);

                        // Add start point if first segment
                        if current_subpath.is_empty() {
                            current_subpath.push((last_point, normal));
                        }

                        current_subpath.push((*p, normal));
                        last_point = *p;
                    }
                }
                PathEl::CurveTo(p1, p2, p3) => {
                    // For cubic bezier, sample points and calculate normals
                    let cubic = kurbo::CubicBez::new(last_point, *p1, *p2, *p3);
                    let num_samples = 10.max((cubic.arclen(0.01) / 5.0) as usize);

                    for i in 0..=num_samples {
                        let t = i as f64 / num_samples as f64;
                        let pos = cubic.eval(t);
                        let deriv = cubic.deriv().eval(t);

                        let deriv_vec = Vec2::new(deriv.x, deriv.y);
                        if vec2_length_squared(deriv_vec) > 1e-10 {
                            let tangent = vec2_normalize(deriv_vec);
                            let normal = Vec2::new(-tangent.y, tangent.x);

                            if current_subpath.is_empty() && i == 0 {
                                current_subpath.push((pos, normal));
                            } else if i > 0 {
                                current_subpath.push((pos, normal));
                            }
                        }
                    }
                    last_point = *p3;
                }
                PathEl::QuadTo(p1, p2) => {
                    // For quadratic bezier, sample points and calculate normals
                    let quad = kurbo::QuadBez::new(last_point, *p1, *p2);
                    let num_samples = 10.max((quad.arclen(0.01) / 5.0) as usize);

                    for i in 0..=num_samples {
                        let t = i as f64 / num_samples as f64;
                        let pos = quad.eval(t);
                        let deriv = quad.deriv().eval(t);
                        let deriv_vec = Vec2::new(deriv.x, deriv.y);

                        if vec2_length_squared(deriv_vec) > 1e-10 {
                            let tangent = vec2_normalize(deriv_vec);
                            let normal = Vec2::new(-tangent.y, tangent.x);

                            if current_subpath.is_empty() && i == 0 {
                                current_subpath.push((pos, normal));
                            } else if i > 0 {
                                current_subpath.push((pos, normal));
                            }
                        }
                    }
                    last_point = *p2;
                }
                PathEl::ClosePath => {
                    if !current_subpath.is_empty() {
                        // Connect back to start
                        if let Some((first_pos, _)) = current_subpath.first() {
                            let tangent = (start_point - last_point);
                            if vec2_length_squared(tangent) > 1e-10 {
                                let norm_tangent = vec2_normalize(tangent);
                                let normal = Vec2::new(-norm_tangent.y, norm_tangent.x);
                                current_subpath.push((start_point, normal));
                            }
                        }

                        build_offset_subpath(
                            &current_subpath,
                            &mut new_path,
                            line_join,
                            miter_limit,
                            true,
                            self.amount,
                        );
                        current_subpath.clear();
                    }
                }
            }
        }

        // Flush final subpath
        if !current_subpath.is_empty() {
            build_offset_subpath(
                &current_subpath,
                &mut new_path,
                line_join,
                miter_limit,
                false,
                self.amount,
            );
        }

        *path = new_path;
    }
}

/// Build an offset subpath from sampled points with normals
fn build_offset_subpath(
    points: &[(Point, Vec2)],
    path: &mut BezPath,
    line_join: u8,
    miter_limit: f64,
    closed: bool,
    offset_amount: f32,
) {
    if points.len() < 2 {
        return;
    }

    let offset_dist = offset_amount as f64;

    // Start the path
    let first_offset = points[0].0 + points[0].1 * offset_dist;
    path.move_to(first_offset);

    for i in 1..points.len() {
        let (prev_pos, prev_normal) = points[i - 1];
        let (curr_pos, curr_normal) = points[i];

        let prev_offset = prev_pos + prev_normal * offset_dist;
        let curr_offset = curr_pos + curr_normal * offset_dist;

        // Check if normals are similar (straight line) or different (corner)
        let normal_dot = prev_normal.dot(curr_normal);

        if normal_dot > 0.99 {
            // Almost same direction, just draw line
            path.line_to(curr_offset);
        } else {
            // Corner - handle line join
            match line_join {
                2 => {
                    // Round join
                    // Calculate the angle between normals
                    let cross = prev_normal.x * curr_normal.y - prev_normal.y * curr_normal.x;
                    let angle = cross.atan2(normal_dot);

                    // Add arc
                    let num_arc_segments = 5.max((angle.abs() * 10.0) as usize);
                    for j in 1..=num_arc_segments {
                        let t = j as f64 / num_arc_segments as f64;
                        // Interpolate normal
                        let interp_normal = vec2_normalize(vec2_lerp(prev_normal, curr_normal, t));
                        let arc_point = curr_pos + interp_normal * offset_dist;
                        path.line_to(arc_point);
                    }
                }
                3 => {
                    // Bevel join - just connect with line
                    path.line_to(curr_offset);
                }
                _ => {
                    // Miter join (default = 1)
                    // Calculate miter point by intersecting offset lines
                    let prev_tangent = Vec2::new(prev_normal.y, -prev_normal.x);
                    let curr_tangent = Vec2::new(curr_normal.y, -curr_normal.x);

                    // Line 1: prev_offset + t1 * prev_tangent
                    // Line 2: curr_offset + t2 * curr_tangent
                    // Solve intersection
                    let det = prev_tangent.x * curr_tangent.y - prev_tangent.y * curr_tangent.x;

                    if det.abs() > 1e-10 {
                        let dx = curr_offset.x - prev_offset.x;
                        let dy = curr_offset.y - prev_offset.y;
                        let t1 = (dx * curr_tangent.y - dy * curr_tangent.x) / det;

                        let miter_point = prev_offset + prev_tangent * t1;
                        let miter_dist = (miter_point - curr_pos).length();

                        if miter_dist < miter_limit * offset_dist {
                            // Miter is within limit, use it
                            path.line_to(miter_point);
                            path.line_to(curr_offset);
                        } else {
                            // Exceeds miter limit, fall back to bevel
                            path.line_to(curr_offset);
                        }
                    } else {
                        // Parallel, just connect
                        path.line_to(curr_offset);
                    }
                }
            }
        }
    }

    // For closed paths, handle final join back to start
    if closed && points.len() > 2 {
        let (last_pos, last_normal) = points[points.len() - 1];
        let (first_pos, first_normal) = points[0];

        let last_offset = last_pos + last_normal * offset_dist;
        let first_offset_calc = first_pos + first_normal * offset_dist;

        // Just close with line for now
        path.line_to(first_offset_calc);
        path.close_path();
    }
}

// Helpers

struct PathWalker<'a> {
    path: &'a BezPath,
    total_length: f64,
    // Cache segments?
}

impl<'a> PathWalker<'a> {
    fn new(path: &'a BezPath) -> Self {
        let mut len = 0.0;
        // Calculate length
        // This is expensive if we do it every time.
        // Approximation: sum of chord lengths?
        // Or accurate arclen.

        // TODO: iterate and sum arclen.
        // For ZigZag proof of concept, assume lines?
        // No, use ParamCurve::arclen.

        // For now, I'll calculate simple length.
        let mut last = Point::ZERO;
        for el in path.elements() {
            match el {
                PathEl::MoveTo(p) => last = *p,
                PathEl::LineTo(p) => {
                    len += p.distance(last);
                    last = *p;
                }
                PathEl::CurveTo(p1, p2, p3) => {
                    // ArcLen
                    use kurbo::CubicBez;
                    let c = CubicBez::new(last, *p1, *p2, *p3);
                    len += c.arclen(0.1);
                    last = *p3;
                }
                PathEl::QuadTo(p1, p2) => {
                    use kurbo::QuadBez;
                    let q = QuadBez::new(last, *p1, *p2);
                    len += q.arclen(0.1);
                    last = *p2;
                }
                _ => {}
            }
        }

        Self {
            path,
            total_length: len,
        }
    }

    fn sample(&mut self, dist: f64) -> Option<(Point, Vec2)> {
        // Find point at distance.
        // Walk again.
        let mut current_dist = 0.0;
        let mut last = Point::ZERO;

        for el in self.path.elements() {
            match el {
                PathEl::MoveTo(p) => last = *p,
                PathEl::LineTo(p) => {
                    let seg_len = p.distance(last);
                    if current_dist + seg_len >= dist {
                        let t = (dist - current_dist) / seg_len;
                        let pos = last.lerp(*p, t);
                        let tangent = *p - last; // normalized?
                        let norm_tangent = vec2_normalize(tangent);
                        return Some((pos, norm_tangent));
                    }
                    current_dist += seg_len;
                    last = *p;
                }
                PathEl::CurveTo(p1, p2, p3) => {
                    use kurbo::CubicBez;
                    let c = CubicBez::new(last, *p1, *p2, *p3);
                    let seg_len = c.arclen(0.1);
                    if current_dist + seg_len >= dist {
                        // We need t for arclen. Inverse arclen?
                        // Kurbo doesn't have inv_arclen easily visible?
                        // Approx: linear t.
                        let t = (dist - current_dist) / seg_len;
                        // This is uniform t, not uniform distance.
                        // For ZigZag, uniform distance is better, but uniform t is acceptable fallback.
                        let pos = c.eval(t);
                        let deriv = c.deriv().eval(t);
                        let tangent = vec2_normalize(Vec2::new(deriv.x, deriv.y));
                        return Some((pos, tangent));
                    }
                    current_dist += seg_len;
                    last = *p3;
                }
                PathEl::QuadTo(p1, p2) => {
                    use kurbo::QuadBez;
                    let q = QuadBez::new(last, *p1, *p2);
                    let seg_len = q.arclen(0.1);
                    if current_dist + seg_len >= dist {
                        let t = (dist - current_dist) / seg_len;
                        let pos = q.eval(t);
                        let deriv = q.deriv().eval(t);
                        let tangent = vec2_normalize(Vec2::new(deriv.x, deriv.y));
                        return Some((pos, tangent));
                    }
                    current_dist += seg_len;
                    last = *p2;
                }
                _ => {}
            }
        }
        None
    }
}
