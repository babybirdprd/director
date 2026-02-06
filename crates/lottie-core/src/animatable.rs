#[cfg(feature = "expressions")]
use crate::expressions::{
    CompObject, EffectObject, ExpressionEvaluator, LayerObject, PropertyValue,
};
#[cfg(feature = "expressions")]
use boa_engine::{js_string, JsValue};
use glam::{Vec2, Vec3, Vec4};
use lottie_data::model::{BezierPath, Property, TextDocument, Value};

/// Context for expression evaluation including layer and composition data
/// This enables expressions to access:
/// - Layer properties like `thisLayer.transform.position`
/// - Composition layers like `thisComp.layer(1)` or `thisComp.layer("name")`
/// - Expression controls like `effect("Slider Control")("Slider")`
#[cfg(feature = "expressions")]
pub struct ExpressionContext {
    pub layer: Option<LayerObject>,
    pub comp: Option<CompObject>,
    pub effects: Vec<EffectObject>,
}

#[cfg(not(feature = "expressions"))]
pub struct ExpressionContext;

impl ExpressionContext {
    /// Create a new expression context with layer data
    #[cfg(feature = "expressions")]
    pub fn with_layer(layer: LayerObject) -> Self {
        Self {
            layer: Some(layer),
            comp: None,
            effects: Vec::new(),
        }
    }

    /// Create a new expression context with layer, composition, and effects data
    #[cfg(feature = "expressions")]
    pub fn with_layer_comp_and_effects(
        layer: LayerObject,
        comp: CompObject,
        effects: Vec<EffectObject>,
    ) -> Self {
        Self {
            layer: Some(layer),
            comp: Some(comp),
            effects,
        }
    }

    /// Create a new expression context with layer and composition data
    #[cfg(feature = "expressions")]
    pub fn with_layer_and_comp(layer: LayerObject, comp: CompObject) -> Self {
        Self {
            layer: Some(layer),
            comp: Some(comp),
            effects: Vec::new(),
        }
    }

    /// Create a default expression context without layer or composition data
    pub fn default() -> Self {
        #[cfg(feature = "expressions")]
        {
            Self {
                layer: None,
                comp: None,
                effects: Vec::new(),
            }
        }
        #[cfg(not(feature = "expressions"))]
        {
            Self
        }
    }
}

pub trait Interpolatable: Sized + Clone {
    fn lerp(&self, other: &Self, t: f32) -> Self;

    fn lerp_spatial(
        &self,
        other: &Self,
        t: f32,
        _tan_in: Option<&Vec<f32>>,
        _tan_out: Option<&Vec<f32>>,
    ) -> Self {
        self.lerp(other, t)
    }
}

impl Interpolatable for TextDocument {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        if t < 1.0 {
            self.clone()
        } else {
            other.clone()
        }
    }
}

impl Interpolatable for BezierPath {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        // Handle hold keyframes
        if t <= 0.0 {
            return self.clone();
        }
        if t >= 1.0 {
            return other.clone();
        }

        // Match vertex counts by using the minimum
        // In a full implementation, we'd add vertices to match counts
        let min_verts = self.v.len().min(other.v.len());
        if min_verts == 0 {
            return self.clone();
        }

        let t_f64 = t as f64;
        let one_minus_t = 1.0 - t_f64;

        // Interpolate vertices
        let mut new_v = Vec::with_capacity(min_verts);
        let mut new_i = Vec::with_capacity(min_verts);
        let mut new_o = Vec::with_capacity(min_verts);

        for i in 0..min_verts {
            // Interpolate vertex position
            let v1 = self.v[i];
            let v2 = other.v[i];
            let new_vert = [
                (v1[0] as f64 * one_minus_t + v2[0] as f64 * t_f64) as f32,
                (v1[1] as f64 * one_minus_t + v2[1] as f64 * t_f64) as f32,
            ];
            new_v.push(new_vert);

            // Interpolate in-tangent (if available)
            let i1 = self.i.get(i).copied().unwrap_or(v1);
            let i2 = other.i.get(i).copied().unwrap_or(v2);
            let new_in = [
                (i1[0] as f64 * one_minus_t + i2[0] as f64 * t_f64) as f32,
                (i1[1] as f64 * one_minus_t + i2[1] as f64 * t_f64) as f32,
            ];
            new_i.push(new_in);

            // Interpolate out-tangent (if available)
            let o1 = self.o.get(i).copied().unwrap_or(v1);
            let o2 = other.o.get(i).copied().unwrap_or(v2);
            let new_out = [
                (o1[0] as f64 * one_minus_t + o2[0] as f64 * t_f64) as f32,
                (o1[1] as f64 * one_minus_t + o2[1] as f64 * t_f64) as f32,
            ];
            new_o.push(new_out);
        }

        // Use closed flag from self (could also interpolate if they differ)
        let closed = self.c;

        BezierPath {
            c: closed,
            v: new_v,
            i: new_i,
            o: new_o,
        }
    }
}

impl Interpolatable for f32 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Interpolatable for Vec2 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Vec2::lerp(*self, *other, t)
    }

    fn lerp_spatial(
        &self,
        other: &Self,
        t: f32,
        tan_in: Option<&Vec<f32>>,
        tan_out: Option<&Vec<f32>>,
    ) -> Self {
        let p0 = *self;
        let p3 = *other;

        let t_out = if let Some(to) = tan_out {
            if to.len() >= 2 {
                Vec2::new(to[0], to[1])
            } else {
                Vec2::ZERO
            }
        } else {
            Vec2::ZERO
        };

        let t_in = if let Some(ti) = tan_in {
            if ti.len() >= 2 {
                Vec2::new(ti[0], ti[1])
            } else {
                Vec2::ZERO
            }
        } else {
            Vec2::ZERO
        };

        let p1 = p0 + t_out;
        let p2 = p3 + t_in;

        let one_minus_t = 1.0 - t;
        let one_minus_t_sq = one_minus_t * one_minus_t;
        let one_minus_t_cub = one_minus_t_sq * one_minus_t;

        let t_sq = t * t;
        let t_cub = t_sq * t;

        p0 * one_minus_t_cub
            + p1 * 3.0 * one_minus_t_sq * t
            + p2 * 3.0 * one_minus_t * t_sq
            + p3 * t_cub
    }
}

impl Interpolatable for Vec3 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Vec3::lerp(*self, *other, t)
    }

    fn lerp_spatial(
        &self,
        other: &Self,
        t: f32,
        tan_in: Option<&Vec<f32>>,
        tan_out: Option<&Vec<f32>>,
    ) -> Self {
        let p0 = *self;
        let p3 = *other;

        let t_out = if let Some(to) = tan_out {
            if to.len() >= 3 {
                Vec3::new(to[0], to[1], to[2])
            } else if to.len() >= 2 {
                Vec3::new(to[0], to[1], 0.0)
            } else {
                Vec3::ZERO
            }
        } else {
            Vec3::ZERO
        };

        let t_in = if let Some(ti) = tan_in {
            if ti.len() >= 3 {
                Vec3::new(ti[0], ti[1], ti[2])
            } else if ti.len() >= 2 {
                Vec3::new(ti[0], ti[1], 0.0)
            } else {
                Vec3::ZERO
            }
        } else {
            Vec3::ZERO
        };

        let p1 = p0 + t_out;
        let p2 = p3 + t_in;

        let one_minus_t = 1.0 - t;
        let one_minus_t_sq = one_minus_t * one_minus_t;
        let one_minus_t_cub = one_minus_t_sq * one_minus_t;

        let t_sq = t * t;
        let t_cub = t_sq * t;

        p0 * one_minus_t_cub
            + p1 * 3.0 * one_minus_t_sq * t
            + p2 * 3.0 * one_minus_t * t_sq
            + p3 * t_cub
    }
}

impl Interpolatable for Vec4 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Vec4::lerp(*self, *other, t)
    }
}

// For gradient colors (Vec<f32>)
impl Interpolatable for Vec<f32> {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self.iter()
            .zip(other.iter())
            .map(|(a, b)| a + (b - a) * t)
            .collect()
    }
}

// Helper to convert Interpolatable to JS Value
#[cfg(feature = "expressions")]
pub trait ToJsValue {
    fn to_js_value(&self, context: &mut boa_engine::Context) -> JsValue;
    fn from_js_value(v: &JsValue, context: &mut boa_engine::Context) -> Option<Self>
    where
        Self: Sized;
}

#[cfg(not(feature = "expressions"))]
pub trait ToJsValue {}

#[cfg(not(feature = "expressions"))]
impl<T> ToJsValue for T {}

#[cfg(feature = "expressions")]
impl ToJsValue for f32 {
    fn to_js_value(&self, _context: &mut boa_engine::Context) -> JsValue {
        JsValue::new(*self)
    }
    fn from_js_value(v: &JsValue, context: &mut boa_engine::Context) -> Option<Self> {
        v.to_number(context).ok().map(|n| n as f32)
    }
}

#[cfg(feature = "expressions")]
impl ToJsValue for Vec<f32> {
    fn to_js_value(&self, context: &mut boa_engine::Context) -> JsValue {
        let vals: Vec<JsValue> = self.iter().map(|f| JsValue::new(*f)).collect();
        boa_engine::object::builtins::JsArray::from_iter(vals, context).into()
    }
    fn from_js_value(v: &JsValue, context: &mut boa_engine::Context) -> Option<Self> {
        if let Some(obj) = v.as_object() {
            if obj.is_array() {
                if let Ok(len_val) = obj.get(js_string!("length"), context) {
                    if let Ok(len) = len_val.to_number(context) {
                        let len_u64 = len as u64;
                        let mut vec = Vec::with_capacity(len_u64 as usize);
                        for i in 0..len_u64 {
                            if let Ok(val) = obj.get(i, context) {
                                if let Ok(n) = val.to_number(context) {
                                    vec.push(n as f32);
                                }
                            }
                        }
                        return Some(vec);
                    }
                }
            }
        }
        None
    }
}

#[cfg(feature = "expressions")]
impl ToJsValue for Vec2 {
    fn to_js_value(&self, context: &mut boa_engine::Context) -> JsValue {
        let vals = vec![JsValue::new(self.x), JsValue::new(self.y)];
        boa_engine::object::builtins::JsArray::from_iter(vals, context).into()
    }
    fn from_js_value(v: &JsValue, context: &mut boa_engine::Context) -> Option<Self> {
        if let Some(obj) = v.as_object() {
            if obj.is_array() {
                let x = obj.get(0, context).ok()?.to_number(context).ok()? as f32;
                let y = obj.get(1, context).ok()?.to_number(context).ok()? as f32;
                return Some(Vec2::new(x, y));
            }
        }
        None
    }
}

#[cfg(feature = "expressions")]
impl ToJsValue for Vec3 {
    fn to_js_value(&self, context: &mut boa_engine::Context) -> JsValue {
        let vals = vec![
            JsValue::new(self.x),
            JsValue::new(self.y),
            JsValue::new(self.z),
        ];
        boa_engine::object::builtins::JsArray::from_iter(vals, context).into()
    }
    fn from_js_value(v: &JsValue, context: &mut boa_engine::Context) -> Option<Self> {
        if let Some(obj) = v.as_object() {
            if obj.is_array() {
                let x = obj.get(0, context).ok()?.to_number(context).ok()? as f32;
                let y = obj.get(1, context).ok()?.to_number(context).ok()? as f32;
                let z = obj.get(2, context).ok()?.to_number(context).ok()? as f32;
                return Some(Vec3::new(x, y, z));
            }
        }
        None
    }
}

#[cfg(feature = "expressions")]
impl ToJsValue for Vec4 {
    fn to_js_value(&self, context: &mut boa_engine::Context) -> JsValue {
        let vals = vec![
            JsValue::new(self.x),
            JsValue::new(self.y),
            JsValue::new(self.z),
            JsValue::new(self.w),
        ];
        boa_engine::object::builtins::JsArray::from_iter(vals, context).into()
    }
    fn from_js_value(v: &JsValue, context: &mut boa_engine::Context) -> Option<Self> {
        if let Some(obj) = v.as_object() {
            if obj.is_array() {
                let x = obj.get(0, context).ok()?.to_number(context).ok()? as f32;
                let y = obj.get(1, context).ok()?.to_number(context).ok()? as f32;
                let z = obj.get(2, context).ok()?.to_number(context).ok()? as f32;
                let w = obj.get(3, context).ok()?.to_number(context).ok()? as f32;
                return Some(Vec4::new(x, y, z, w));
            }
        }
        None
    }
}

#[cfg(feature = "expressions")]
impl ToJsValue for BezierPath {
    fn to_js_value(&self, _context: &mut boa_engine::Context) -> JsValue {
        JsValue::Undefined
    }
    fn from_js_value(_v: &JsValue, _context: &mut boa_engine::Context) -> Option<Self> {
        None
    }
}

#[cfg(feature = "expressions")]
impl ToJsValue for TextDocument {
    fn to_js_value(&self, _context: &mut boa_engine::Context) -> JsValue {
        JsValue::Undefined
    }
    fn from_js_value(_v: &JsValue, _context: &mut boa_engine::Context) -> Option<Self> {
        None
    }
}

// Cubic Bezier Easing
pub fn solve_cubic_bezier(p1: Vec2, p2: Vec2, x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }

    // Newton-Raphson
    let mut t = x;
    for _ in 0..8 {
        let one_minus_t = 1.0 - t;
        let x_est = 3.0 * one_minus_t * one_minus_t * t * p1.x
            + 3.0 * one_minus_t * t * t * p2.x
            + t * t * t;

        let err = x_est - x;
        if err.abs() < 1e-4 {
            break;
        }

        let dx_dt = 3.0 * one_minus_t * one_minus_t * p1.x
            + 6.0 * one_minus_t * t * (p2.x - p1.x)
            + 3.0 * t * t * (1.0 - p2.x);

        if dx_dt.abs() < 1e-6 {
            break;
        }
        t -= err / dx_dt;
    }

    let one_minus_t = 1.0 - t;
    3.0 * one_minus_t * one_minus_t * t * p1.y + 3.0 * one_minus_t * t * t * p2.y + t * t * t
}

pub struct Animator;

/// Trait for converting types to/from PropertyValue for expression evaluation
#[cfg(feature = "expressions")]
pub trait ToPropertyValue {
    fn to_property_value(&self) -> PropertyValue;
    fn from_property_value(value: &PropertyValue) -> Option<Self>
    where
        Self: Sized;
}

#[cfg(not(feature = "expressions"))]
pub trait ToPropertyValue {}

#[cfg(not(feature = "expressions"))]
impl<T> ToPropertyValue for T {}

#[cfg(feature = "expressions")]
impl ToPropertyValue for f32 {
    fn to_property_value(&self) -> PropertyValue {
        PropertyValue::Scalar(*self as f64)
    }
    fn from_property_value(value: &PropertyValue) -> Option<Self> {
        Some(value.as_scalar() as f32)
    }
}

#[cfg(feature = "expressions")]
impl ToPropertyValue for Vec2 {
    fn to_property_value(&self) -> PropertyValue {
        PropertyValue::Vector(vec![self.x as f64, self.y as f64])
    }
    fn from_property_value(value: &PropertyValue) -> Option<Self> {
        let vec = value.as_vector();
        if vec.len() >= 2 {
            Some(Vec2::new(vec[0] as f32, vec[1] as f32))
        } else {
            None
        }
    }
}

#[cfg(feature = "expressions")]
impl ToPropertyValue for Vec3 {
    fn to_property_value(&self) -> PropertyValue {
        PropertyValue::Vector(vec![self.x as f64, self.y as f64, self.z as f64])
    }
    fn from_property_value(value: &PropertyValue) -> Option<Self> {
        let vec = value.as_vector();
        if vec.len() >= 3 {
            Some(Vec3::new(vec[0] as f32, vec[1] as f32, vec[2] as f32))
        } else {
            None
        }
    }
}

#[cfg(feature = "expressions")]
impl ToPropertyValue for Vec4 {
    fn to_property_value(&self) -> PropertyValue {
        PropertyValue::Vector(vec![
            self.x as f64,
            self.y as f64,
            self.z as f64,
            self.w as f64,
        ])
    }
    fn from_property_value(value: &PropertyValue) -> Option<Self> {
        let vec = value.as_vector();
        if vec.len() >= 4 {
            Some(Vec4::new(
                vec[0] as f32,
                vec[1] as f32,
                vec[2] as f32,
                vec[3] as f32,
            ))
        } else {
            None
        }
    }
}

/// Simple animated property wrapper for expression evaluation
/// Provides basic value_at_time and velocity_at_time support
#[cfg(feature = "expressions")]
struct SimpleAnimatedProperty {
    current_value: PropertyValue,
}

#[cfg(feature = "expressions")]
impl crate::expressions::AnimatedProperty for SimpleAnimatedProperty {
    fn value_at_time(&self, _time: f64) -> PropertyValue {
        self.current_value.clone()
    }

    fn velocity_at_time(&self, _time: f64) -> PropertyValue {
        // Return zero velocity as we don't have keyframe data here
        match &self.current_value {
            PropertyValue::Scalar(_) => PropertyValue::Scalar(0.0),
            PropertyValue::Vector(v) => PropertyValue::Vector(vec![0.0; v.len()]),
        }
    }

    fn speed_at_time(&self, _time: f64) -> f64 {
        0.0
    }
}

impl Animator {
    /// Resolve a property value, optionally with expression support
    ///
    /// When `ctx` contains a LayerObject with transform data, expressions can access
    /// layer properties like `thisLayer.transform.position`.
    pub fn resolve<T, U>(
        prop: &Property<T>,
        frame: f32,
        converter: impl Fn(&T) -> U,
        default: U,
        #[cfg(feature = "expressions")] evaluator: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] _evaluator: Option<&mut ()>, // Dummy type
        _frame_rate: f32,
        #[cfg(feature = "expressions")] ctx: &ExpressionContext,
        #[cfg(not(feature = "expressions"))] _ctx: &ExpressionContext,
    ) -> U
    where
        U: Interpolatable + 'static + ToJsValue + ToPropertyValue,
    {
        // 1. Calculate Base Value (Keyframes)
        let base_value = Self::resolve_keyframes(prop, frame, &converter, default.clone());

        // 2. Expression Check
        #[cfg(feature = "expressions")]
        if let Some(expr) = &prop.x {
            if let Some(eval) = evaluator {
                let time = frame / _frame_rate; // Seconds

                // Calculate Loop Value (pre-calc logic for loopOut("cycle"))
                let loop_value = if let Value::Animated(keyframes) = &prop.k {
                    if !keyframes.is_empty() {
                        let first_t = keyframes[0].t;
                        let last_t = keyframes[keyframes.len() - 1].t;
                        let duration = last_t - first_t;

                        if duration > 0.0 && frame > last_t {
                            let t_since_end = frame - last_t;
                            let cycle_offset = t_since_end % duration;
                            let cycle_frame = first_t + cycle_offset;
                            Self::resolve_keyframes(prop, cycle_frame, &converter, default.clone())
                        } else {
                            base_value.clone()
                        }
                    } else {
                        base_value.clone()
                    }
                } else {
                    base_value.clone()
                };

                // Use the new evaluate_on_property if we have layer context
                // Otherwise fall back to legacy evaluate
                let layer_obj = ctx.layer.clone().unwrap_or_else(|| LayerObject::new(0, ""));

                // Convert values to PropertyValue for the new API
                let base_prop_value = base_value.to_property_value();
                let loop_prop_value = loop_value.to_property_value();

                // Create a simple animated property wrapper for value_at_time support
                let prop_wrapper = SimpleAnimatedProperty {
                    current_value: base_prop_value.clone(),
                };

                match eval.evaluate_on_property(
                    expr,
                    &prop_wrapper,
                    &base_prop_value,
                    time as f64,
                    &layer_obj,
                    ctx.comp.as_ref(),
                    Some(&ctx.effects),
                    Some(&loop_prop_value),
                    None,
                    &crate::expressions::DirectorVariableContext::default(),
                ) {
                    Ok(result) => {
                        if let Some(val) = U::from_property_value(&result) {
                            return val;
                        }
                    }
                    Err(_e) => {
                        // Fall back to legacy evaluate on failure
                        let (js_val, js_loop_val) = {
                            let ctx = eval.context();
                            (base_value.to_js_value(ctx), loop_value.to_js_value(ctx))
                        };

                        match eval.evaluate(expr, &js_val, &js_loop_val, time, _frame_rate) {
                            Ok(res) => {
                                let context = eval.context();
                                if let Some(val) = U::from_js_value(&res, context) {
                                    return val;
                                }
                            }
                            Err(_) => {}
                        }
                    }
                }
            }
        }

        base_value
    }

    /// Legacy resolve without expression context (for backward compatibility)
    pub fn resolve_simple<T, U>(
        prop: &Property<T>,
        frame: f32,
        converter: impl Fn(&T) -> U,
        default: U,
        #[cfg(feature = "expressions")] evaluator: Option<&mut ExpressionEvaluator>,
        #[cfg(not(feature = "expressions"))] _evaluator: Option<&mut ()>,
        _frame_rate: f32,
    ) -> U
    where
        U: Interpolatable + 'static + ToJsValue,
    {
        // Simple resolve without expression context
        let base_value = Self::resolve_keyframes(prop, frame, &converter, default.clone());

        #[cfg(feature = "expressions")]
        if let Some(expr) = &prop.x {
            if let Some(eval) = evaluator {
                let time = frame / _frame_rate;

                let loop_value = if let Value::Animated(keyframes) = &prop.k {
                    if !keyframes.is_empty() {
                        let first_t = keyframes[0].t;
                        let last_t = keyframes[keyframes.len() - 1].t;
                        let duration = last_t - first_t;

                        if duration > 0.0 && frame > last_t {
                            let t_since_end = frame - last_t;
                            let cycle_offset = t_since_end % duration;
                            let cycle_frame = first_t + cycle_offset;
                            Self::resolve_keyframes(prop, cycle_frame, &converter, default.clone())
                        } else {
                            base_value.clone()
                        }
                    } else {
                        base_value.clone()
                    }
                } else {
                    base_value.clone()
                };

                let (js_val, js_loop_val) = {
                    let ctx = eval.context();
                    (base_value.to_js_value(ctx), loop_value.to_js_value(ctx))
                };

                match eval.evaluate(expr, &js_val, &js_loop_val, time, _frame_rate) {
                    Ok(res) => {
                        let context = eval.context();
                        if let Some(val) = U::from_js_value(&res, context) {
                            return val;
                        }
                    }
                    Err(_) => {}
                }
            }
        }

        base_value
    }

    fn resolve_keyframes<T, U>(
        prop: &Property<T>,
        frame: f32,
        converter: &impl Fn(&T) -> U,
        default: U,
    ) -> U
    where
        U: Interpolatable,
    {
        match &prop.k {
            Value::Default => default,
            Value::Static(v) => converter(v),
            Value::Animated(keyframes) => {
                if keyframes.is_empty() {
                    return default;
                }

                // Optimization: Binary Search
                // Find the first keyframe where kf.t > frame
                // 'idx' will be the index of that keyframe.
                // The current segment is between idx-1 and idx.
                let idx = keyframes.partition_point(|kf| kf.t <= frame);

                // If idx == 0, then all keyframes have t > frame. frame is before start.
                if idx == 0 {
                    if let Some(s) = &keyframes[0].s {
                        return converter(s);
                    }
                    return default;
                }

                let len = keyframes.len();
                // If idx == len, then all keyframes have t <= frame. frame is after end (or exactly at end).
                if idx >= len {
                    let last = &keyframes[len - 1];
                    // Use end value if present, else start value
                    if let Some(e) = &last.e {
                        return converter(e);
                    }
                    if let Some(s) = &last.s {
                        return converter(s);
                    }
                    return default;
                }

                // Segment is [idx-1, idx]
                let kf_start = &keyframes[idx - 1];
                let kf_end = &keyframes[idx];

                let start_val = kf_start
                    .s
                    .as_ref()
                    .map(|v| converter(v))
                    .unwrap_or(default.clone());

                // End value logic
                let end_val = kf_start
                    .e
                    .as_ref()
                    .map(|v| converter(v))
                    .or_else(|| kf_end.s.as_ref().map(|v| converter(v)))
                    .unwrap_or(start_val.clone());

                let duration = kf_end.t - kf_start.t;
                if duration <= 0.0 {
                    return start_val;
                }

                let mut local_t = (frame - kf_start.t) / duration;

                // Easing - extract from BezierTangent structure
                let p1 = if let Some(o) = &kf_start.o {
                    let x = o.x.first().copied().unwrap_or(0.0);
                    let y = o.y.first().copied().unwrap_or(0.0);
                    Vec2::new(x, y)
                } else {
                    Vec2::new(0.0, 0.0)
                };
                let p2 = if let Some(i) = &kf_end.i {
                    let x = i.x.first().copied().unwrap_or(1.0);
                    let y = i.y.first().copied().unwrap_or(1.0);
                    Vec2::new(x, y)
                } else {
                    Vec2::new(1.0, 1.0)
                };

                // If Hold keyframe
                if let Some(h) = kf_start.h {
                    if h == 1 {
                        return start_val;
                    }
                }

                local_t = solve_cubic_bezier(p1, p2, local_t);

                start_val.lerp_spatial(&end_val, local_t, kf_end.ti.as_ref(), kf_start.to.as_ref())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lottie_data::model::{Keyframe, Value};

    #[test]
    fn test_animator_resolve_binary_search() {
        // Setup a property with keyframes at 0, 10, 20
        let keyframes = vec![
            Keyframe {
                t: 0.0,
                s: Some(0.0),
                e: Some(10.0),
                i: None,
                o: None,
                to: None,
                ti: None,
                h: None,
            },
            Keyframe {
                t: 10.0,
                s: Some(10.0),
                e: Some(20.0),
                i: None,
                o: None,
                to: None,
                ti: None,
                h: None,
            },
            Keyframe {
                t: 20.0,
                s: Some(20.0),
                e: Some(30.0),
                i: None,
                o: None,
                to: None,
                ti: None,
                h: None,
            },
        ];

        let prop = Property {
            a: 1,
            k: Value::Animated(keyframes),
            ix: None,
            x: None,
        };

        let conv = |v: &f32| *v;

        // 1. Exact match start
        assert_eq!(
            Animator::resolve_simple(&prop, 0.0, conv, -1.0, None, 60.0),
            0.0
        );

        // 2. Exact match middle
        assert_eq!(
            Animator::resolve_simple(&prop, 10.0, conv, -1.0, None, 60.0),
            10.0
        );

        // 3. Exact match end
        assert_eq!(
            Animator::resolve_simple(&prop, 20.0, conv, -1.0, None, 60.0),
            30.0
        );

        // 4. Before first
        assert_eq!(
            Animator::resolve_simple(&prop, -5.0, conv, -1.0, None, 60.0),
            0.0
        );

        // 5. After last
        assert_eq!(
            Animator::resolve_simple(&prop, 25.0, conv, -1.0, None, 60.0),
            30.0
        );

        // 6. Mid-segment
        assert_eq!(
            Animator::resolve_simple(&prop, 5.0, conv, -1.0, None, 60.0),
            5.0
        );

        // 7. Mid-segment 2
        assert_eq!(
            Animator::resolve_simple(&prop, 15.0, conv, -1.0, None, 60.0),
            15.0
        );
    }
}
