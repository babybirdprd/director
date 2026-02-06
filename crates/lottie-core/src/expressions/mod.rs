//! AE-Exact Expression System for Lottie
//!
//! Implements After Effects expression evaluation with proper Property objects
//! supporting methods like wiggle(), valueAtTime() and attributes like value, velocity.

pub mod property;
pub mod wiggle;

#[cfg(feature = "expressions")]
pub use property::TransformSampler;
pub use property::{
    AnimatedProperty, CompObject, ControlValue, EffectObject, LayerObject, PropertyObject,
    PropertyValue, TransformObject,
};
pub use wiggle::{wiggle_property_value, wiggle_property_value_with_amps, WiggleState};

#[cfg(feature = "expressions")]
use boa_engine::{
    context::ContextBuilder, js_string, native_function::NativeFunction, object::builtins::JsArray,
    object::ObjectInitializer, property::Attribute, Context, JsArgs, JsResult, JsValue, Source,
};
#[cfg(feature = "expressions")]
use boa_gc::{Finalize, Trace};

/// Loop type for expressions
#[derive(Clone, Debug, PartialEq)]
#[cfg(feature = "expressions")]
#[derive(boa_gc::Trace, boa_gc::Finalize)]
pub enum LoopType {
    Cycle,
    PingPong,
    Continue,
    Offset,
}

impl LoopType {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pingpong" => LoopType::PingPong,
            "continue" => LoopType::Continue,
            "offset" => LoopType::Offset,
            _ => LoopType::Cycle,
        }
    }
}

#[cfg(feature = "expressions")]
pub struct ExpressionEvaluator {
    context: Context,
    loop_out_type: LoopType,
    loop_in_type: LoopType,
}

#[cfg(feature = "expressions")]
impl ExpressionEvaluator {
    pub fn new() -> Self {
        let mut context = ContextBuilder::default().build().unwrap();

        // Register Global Helpers (AE-style)
        Self::register_math_helpers(&mut context);

        Self {
            context,
            loop_out_type: LoopType::Cycle,
            loop_in_type: LoopType::Cycle,
        }
    }

    fn register_math_helpers(context: &mut Context) {
        // add(a, b)
        context
            .register_global_callable(
                js_string!("add"),
                1,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    let a = args.get_or_undefined(0);
                    let b = args.get_or_undefined(1);
                    helper_add(a, b, context)
                }),
            )
            .unwrap();

        // mul(a, b)
        context
            .register_global_callable(
                js_string!("mul"),
                1,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    let a = args.get_or_undefined(0);
                    let b = args.get_or_undefined(1);
                    helper_mul(a, b, context)
                }),
            )
            .unwrap();

        // sub(a, b)
        context
            .register_global_callable(
                js_string!("sub"),
                1,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    let a = args.get_or_undefined(0);
                    let b = args.get_or_undefined(1);
                    helper_sub(a, b, context)
                }),
            )
            .unwrap();

        // div(a, b)
        context
            .register_global_callable(
                js_string!("div"),
                1,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    let a = args.get_or_undefined(0);
                    let b = args.get_or_undefined(1);
                    helper_div(a, b, context)
                }),
            )
            .unwrap();

        // Math functions
        // length(vec) or length(a, b)
        context
            .register_global_callable(
                js_string!("length"),
                1,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    if args.len() >= 2 {
                        // length(a, b) - distance between two points
                        let a = args.get_or_undefined(0);
                        let b = args.get_or_undefined(1);

                        if let (Some(a_arr), Some(b_arr)) = (a.as_object(), b.as_object()) {
                            if a_arr.is_array() && b_arr.is_array() {
                                let len = a_arr
                                    .get(js_string!("length"), context)
                                    .ok()
                                    .and_then(|v| v.to_number(context).ok())
                                    .unwrap_or(0.0)
                                    as usize;

                                let mut sum_sq = 0.0f64;
                                for i in 0..len {
                                    if let (Ok(va), Ok(vb)) =
                                        (a_arr.get(i, context), b_arr.get(i, context))
                                    {
                                        if let (Ok(na), Ok(nb)) =
                                            (va.to_number(context), vb.to_number(context))
                                        {
                                            let diff = na - nb;
                                            sum_sq += diff * diff;
                                        }
                                    }
                                }
                                return Ok(JsValue::new(sum_sq.sqrt()));
                            }
                        }
                        Ok(JsValue::nan())
                    } else {
                        // length(vec) - magnitude
                        let vec = args.get_or_undefined(0);
                        if let Some(arr) = vec.as_object() {
                            if arr.is_array() {
                                let len = arr
                                    .get(js_string!("length"), context)
                                    .ok()
                                    .and_then(|v| v.to_number(context).ok())
                                    .unwrap_or(0.0)
                                    as usize;

                                let mut sum_sq = 0.0f64;
                                for i in 0..len {
                                    if let Ok(v) = arr.get(i, context) {
                                        if let Ok(n) = v.to_number(context) {
                                            sum_sq += n * n;
                                        }
                                    }
                                }
                                return Ok(JsValue::new(sum_sq.sqrt()));
                            }
                        }
                        Ok(JsValue::nan())
                    }
                }),
            )
            .unwrap();

        // normalize(vec)
        context
            .register_global_callable(
                js_string!("normalize"),
                1,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    let vec = args.get_or_undefined(0);
                    if let Some(arr) = vec.as_object() {
                        if arr.is_array() {
                            let len = arr
                                .get(js_string!("length"), context)
                                .ok()
                                .and_then(|v| v.to_number(context).ok())
                                .unwrap_or(0.0) as usize;

                            let mut components = Vec::new();
                            let mut sum_sq = 0.0f64;

                            for i in 0..len {
                                if let Ok(v) = arr.get(i, context) {
                                    if let Ok(n) = v.to_number(context) {
                                        components.push(n);
                                        sum_sq += n * n;
                                    }
                                }
                            }

                            let mag = sum_sq.sqrt();
                            if mag > 0.0 {
                                let normalized: Vec<JsValue> =
                                    components.iter().map(|n| JsValue::new(n / mag)).collect();
                                return Ok(JsArray::from_iter(normalized, context).into());
                            }
                        }
                    }
                    Ok(vec.clone())
                }),
            )
            .unwrap();

        // dot(a, b)
        context
            .register_global_callable(
                js_string!("dot"),
                2,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    let a = args.get_or_undefined(0);
                    let b = args.get_or_undefined(1);

                    if let (Some(a_arr), Some(b_arr)) = (a.as_object(), b.as_object()) {
                        if a_arr.is_array() && b_arr.is_array() {
                            let len_a = a_arr
                                .get(js_string!("length"), context)
                                .ok()
                                .and_then(|v| v.to_number(context).ok())
                                .unwrap_or(0.0) as usize;
                            let len_b = b_arr
                                .get(js_string!("length"), context)
                                .ok()
                                .and_then(|v| v.to_number(context).ok())
                                .unwrap_or(0.0) as usize;
                            let len = len_a.min(len_b);

                            let mut dot_product = 0.0f64;
                            for i in 0..len {
                                if let (Ok(va), Ok(vb)) =
                                    (a_arr.get(i, context), b_arr.get(i, context))
                                {
                                    if let (Ok(na), Ok(nb)) =
                                        (va.to_number(context), vb.to_number(context))
                                    {
                                        dot_product += na * nb;
                                    }
                                }
                            }
                            return Ok(JsValue::new(dot_product));
                        }
                    }
                    Ok(JsValue::new(0.0))
                }),
            )
            .unwrap();

        // cross(a, b) - 3D only
        context
            .register_global_callable(
                js_string!("cross"),
                2,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    let a = args.get_or_undefined(0);
                    let b = args.get_or_undefined(1);

                    if let (Some(a_arr), Some(b_arr)) = (a.as_object(), b.as_object()) {
                        if a_arr.is_array() && b_arr.is_array() {
                            let mut get_component =
                                |arr: &boa_engine::object::JsObject, idx: usize| -> f64 {
                                    arr.get(idx, context)
                                        .ok()
                                        .and_then(|v| v.to_number(context).ok())
                                        .unwrap_or(0.0)
                                };

                            let ax = get_component(a_arr, 0);
                            let ay = get_component(a_arr, 1);
                            let az = get_component(a_arr, 2);
                            let bx = get_component(b_arr, 0);
                            let by = get_component(b_arr, 1);
                            let bz = get_component(b_arr, 2);

                            let cx = ay * bz - az * by;
                            let cy = az * bx - ax * bz;
                            let cz = ax * by - ay * bx;

                            let result = vec![JsValue::new(cx), JsValue::new(cy), JsValue::new(cz)];
                            return Ok(JsArray::from_iter(result, context).into());
                        }
                    }
                    Ok(JsValue::undefined())
                }),
            )
            .unwrap();

        // lookAt(from, to) - returns rotation in degrees
        context
            .register_global_callable(
                js_string!("lookAt"),
                2,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    let from = args.get_or_undefined(0);
                    let to = args.get_or_undefined(1);

                    if let (Some(from_arr), Some(to_arr)) = (from.as_object(), to.as_object()) {
                        if from_arr.is_array() && to_arr.is_array() {
                            let mut get_component =
                                |arr: &boa_engine::object::JsObject, idx: usize| -> f64 {
                                    arr.get(idx, context)
                                        .ok()
                                        .and_then(|v| v.to_number(context).ok())
                                        .unwrap_or(0.0)
                                };

                            let fx = get_component(from_arr, 0);
                            let fy = get_component(from_arr, 1);
                            let fz = get_component(from_arr, 2);
                            let tx = get_component(to_arr, 0);
                            let ty = get_component(to_arr, 1);
                            let tz = get_component(to_arr, 2);

                            let dx = tx - fx;
                            let dy = ty - fy;

                            // 2D rotation (ignores z for now)
                            let angle_rad = dy.atan2(dx);
                            let angle_deg = angle_rad.to_degrees();

                            return Ok(JsValue::new(angle_deg));
                        }
                    }
                    Ok(JsValue::new(0.0))
                }),
            )
            .unwrap();

        // seedRandom(seed, timeless = false)
        context
            .register_global_callable(
                js_string!("seedRandom"),
                1,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    // For now, this is a no-op but sets up the infrastructure
                    // In a full implementation, this would seed the random number generator
                    let _seed = args.get_or_undefined(0).to_number(context).unwrap_or(0.0);
                    let _timeless = args.get_or_undefined(1).to_boolean();
                    Ok(JsValue::undefined())
                }),
            )
            .unwrap();

        // Interpolation functions: linear, ease, easeIn, easeOut
        // linear(t, tMin, tMax, value1, value2)
        context
            .register_global_callable(
                js_string!("linear"),
                1,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    helper_interpolate(args, context, InterpolationType::Linear)
                }),
            )
            .unwrap();

        // ease(t, tMin, tMax, value1, value2) - uses easeInOut interpolation
        context
            .register_global_callable(
                js_string!("ease"),
                1,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    helper_interpolate(args, context, InterpolationType::Ease)
                }),
            )
            .unwrap();

        // easeIn(t, tMin, tMax, value1, value2)
        context
            .register_global_callable(
                js_string!("easeIn"),
                1,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    helper_interpolate(args, context, InterpolationType::EaseIn)
                }),
            )
            .unwrap();

        // easeOut(t, tMin, tMax, value1, value2)
        context
            .register_global_callable(
                js_string!("easeOut"),
                1,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    helper_interpolate(args, context, InterpolationType::EaseOut)
                }),
            )
            .unwrap();
    }

    pub fn set_loop_out_type(&mut self, loop_type: LoopType) {
        self.loop_out_type = loop_type;
    }

    pub fn set_loop_in_type(&mut self, loop_type: LoopType) {
        self.loop_in_type = loop_type;
    }

    pub fn evaluate(
        &mut self,
        script: &str,
        current_value: &JsValue,
        loop_value: &JsValue,
        time: f32,
        _frame_rate: f32,
    ) -> Result<JsValue, String> {
        // Set Globals
        self.context
            .register_global_property(js_string!("value"), current_value.clone(), Attribute::all())
            .map_err(|e| format!("Failed to register value: {}", e))?;

        self.context
            .register_global_property(
                js_string!("__loop_value"),
                loop_value.clone(),
                Attribute::all(),
            )
            .map_err(|e| format!("Failed to register loop value: {}", e))?;

        self.context
            .register_global_property(js_string!("time"), JsValue::new(time), Attribute::all())
            .map_err(|e| format!("Failed to register time: {}", e))?;

        // LoopOut - simpler implementation without complex captures
        self.context
            .register_global_callable(
                js_string!("loopOut"),
                0,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    // The loop type is handled in pre-calculation phase
                    // Just return the pre-calculated loop value
                    let val = context
                        .global_object()
                        .get(js_string!("__loop_value"), context)
                        .unwrap_or_default();
                    Ok(val)
                }),
            )
            .map_err(|e| format!("Failed to register loopOut: {}", e))?;

        // LoopIn - simpler implementation
        self.context
            .register_global_callable(
                js_string!("loopIn"),
                0,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    // Return the pre-calculated loopIn value
                    let val = context
                        .global_object()
                        .get(js_string!("__loop_in_value"), context)
                        .unwrap_or_default();
                    Ok(val)
                }),
            )
            .map_err(|e| format!("Failed to register loopIn: {}", e))?;

        match self.context.eval(Source::from_bytes(script)) {
            Ok(res) => Ok(res),
            Err(e) => Err(format!("Eval error: {}", e)),
        }
    }

    pub fn context(&mut self) -> &mut Context {
        &mut self.context
    }

    /// Evaluate expression with PropertyObject system for AE-exact behavior
    ///
    /// The `layer` parameter should include transform data for accessing properties
    /// like `thisLayer.transform.position` in expressions.
    /// The `comp` parameter enables access to other layers via `thisComp.layer(index)` or `thisComp.layer(name)`.
    /// The `effects` parameter enables access to expression controls via `effect("Name")("Control")`.
    pub fn evaluate_on_property(
        &mut self,
        script: &str,
        property: &dyn AnimatedProperty,
        current_value: &PropertyValue,
        time: f64,
        layer: &LayerObject,
        comp: Option<&CompObject>,
        effects: Option<&[EffectObject]>,
        loop_out_value: Option<&PropertyValue>,
        loop_in_value: Option<&PropertyValue>,
        director_vars: &DirectorVariableContext,
    ) -> Result<PropertyValue, String> {
        let velocity = property.velocity_at_time(time);
        let speed = property.speed_at_time(time);

        let prop_obj = PropertyObject::new(current_value.clone(), time, velocity, speed, None);

        let prop_js = create_property_object(&prop_obj, property, &mut self.context)?;
        let layer_js = create_layer_object(layer, &mut self.context, time)?;

        self.context
            .register_global_property(
                js_string!("thisProperty"),
                prop_js.clone(),
                Attribute::all(),
            )
            .map_err(|e| format!("Failed to register thisProperty: {}", e))?;

        self.context
            .register_global_property(js_string!("thisLayer"), layer_js.clone(), Attribute::all())
            .map_err(|e| format!("Failed to register thisLayer: {}", e))?;

        self.context
            .register_global_property(
                js_string!("index"),
                JsValue::new(layer.index() as f64),
                Attribute::all(),
            )
            .map_err(|e| format!("Failed to register index: {}", e))?;

        self.context
            .register_global_property(js_string!("time"), JsValue::new(time), Attribute::all())
            .map_err(|e| format!("Failed to register time: {}", e))?;

        let value_js = current_value.to_js_value(&mut self.context);
        self.context
            .register_global_property(js_string!("value"), value_js, Attribute::all())
            .map_err(|e| format!("Failed to register value: {}", e))?;

        // Register loop values
        if let Some(loop_val) = loop_out_value {
            let loop_js = loop_val.to_js_value(&mut self.context);
            self.context
                .register_global_property(js_string!("__loop_value"), loop_js, Attribute::all())
                .map_err(|e| format!("Failed to register loop value: {}", e))?;
        }

        if let Some(loop_val) = loop_in_value {
            let loop_js = loop_val.to_js_value(&mut self.context);
            self.context
                .register_global_property(js_string!("__loop_in_value"), loop_js, Attribute::all())
                .map_err(|e| format!("Failed to register loopIn value: {}", e))?;
        }

        // Register Director variable context
        let vars_js = create_director_variables_object(director_vars, &mut self.context)?;
        self.context
            .register_global_property(js_string!("vars"), vars_js, Attribute::all())
            .map_err(|e| format!("Failed to register vars: {}", e))?;

        // Register thisDirector and thisScene objects
        let director_js = create_director_context_object(director_vars, &mut self.context)?;
        self.context
            .register_global_property(js_string!("thisDirector"), director_js, Attribute::all())
            .map_err(|e| format!("Failed to register thisDirector: {}", e))?;

        // Register thisComp object for cross-layer references
        if let Some(comp_obj) = comp {
            let comp_js = create_comp_object(comp_obj, &mut self.context, time)?;
            self.context
                .register_global_property(js_string!("thisComp"), comp_js, Attribute::all())
                .map_err(|e| format!("Failed to register thisComp: {}", e))?;
        }

        // Register effect() function for expression controls
        if let Some(effects_list) = effects {
            register_effect_function(effects_list, &mut self.context)?;
        }

        let result = self
            .context
            .eval(Source::from_bytes(script))
            .map_err(|e| format!("Expression evaluation error: {}", e))?;

        PropertyValue::from_js_value(&result, &mut self.context)
            .ok_or_else(|| "Failed to convert expression result to PropertyValue".to_string())
    }
}

/// Context for Director variables accessible in expressions
#[derive(Clone, Debug, Default)]
pub struct DirectorVariableContext {
    /// Local variables (set on specific Lottie node)
    pub local: std::collections::HashMap<String, PropertyValue>,
    /// Scene-level variables
    pub scene: std::collections::HashMap<String, PropertyValue>,
    /// Global variables (all Lottie nodes)
    pub global: std::collections::HashMap<String, PropertyValue>,
    /// Director timeline time
    pub director_time: f64,
    /// Scene duration
    pub scene_duration: f64,
    /// Scene name
    pub scene_name: String,
}

impl DirectorVariableContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a variable with priority: local > scene > global
    pub fn get(&self, name: &str) -> Option<&PropertyValue> {
        self.local
            .get(name)
            .or_else(|| self.scene.get(name))
            .or_else(|| self.global.get(name))
    }

    pub fn set_local(&mut self, name: impl Into<String>, value: PropertyValue) {
        self.local.insert(name.into(), value);
    }

    pub fn set_scene(&mut self, name: impl Into<String>, value: PropertyValue) {
        self.scene.insert(name.into(), value);
    }

    pub fn set_global(&mut self, name: impl Into<String>, value: PropertyValue) {
        self.global.insert(name.into(), value);
    }
}

#[cfg(feature = "expressions")]
fn create_director_variables_object(
    ctx: &DirectorVariableContext,
    context: &mut Context,
) -> Result<JsValue, String> {
    // Add all variables with proper priority (local overrides scene overrides global)
    let mut all_vars: std::collections::HashMap<String, &PropertyValue> =
        std::collections::HashMap::new();

    // Start with globals
    for (k, v) in &ctx.global {
        all_vars.insert(k.clone(), v);
    }

    // Override with scene vars
    for (k, v) in &ctx.scene {
        all_vars.insert(k.clone(), v);
    }

    // Override with local vars (highest priority)
    for (k, v) in &ctx.local {
        all_vars.insert(k.clone(), v);
    }

    // Add all properties to JS object
    // Collect JS values first to avoid double mutable borrow of context
    // Must collect values BEFORE creating ObjectInitializer since it holds context ref
    let js_values: Vec<_> = all_vars
        .iter()
        .map(|(k, v)| (k.clone(), v.to_js_value(context)))
        .collect();

    // Now create ObjectInitializer after all context borrows are done
    let mut obj_init = ObjectInitializer::new(context);
    for (key, js_val) in js_values {
        obj_init.property(js_string!(key.as_str()), js_val, Attribute::all());
    }

    let obj = obj_init.build();
    Ok(JsValue::from(obj))
}

#[cfg(feature = "expressions")]
fn create_director_context_object(
    ctx: &DirectorVariableContext,
    context: &mut Context,
) -> Result<JsValue, String> {
    // Build nested scene object first to avoid overlapping mutable borrows
    let scene_obj = {
        let mut scene_init = ObjectInitializer::new(context);
        scene_init
            .property(
                js_string!("name"),
                JsValue::from(js_string!(ctx.scene_name.as_str())),
                Attribute::all(),
            )
            .property(
                js_string!("duration"),
                JsValue::new(ctx.scene_duration),
                Attribute::all(),
            )
            .build()
    };

    // Now create parent object after scene_init borrow is released
    let mut obj_init = ObjectInitializer::new(context);

    obj_init.property(
        js_string!("time"),
        JsValue::new(ctx.director_time),
        Attribute::all(),
    );

    obj_init.property(
        js_string!("duration"),
        JsValue::new(ctx.scene_duration),
        Attribute::all(),
    );

    obj_init.property(
        js_string!("scene"),
        JsValue::from(scene_obj),
        Attribute::all(),
    );

    let obj = obj_init.build();
    Ok(JsValue::from(obj))
}

#[cfg(feature = "expressions")]
#[derive(Clone, Copy, Finalize)]
struct PropertyCaptures {
    /// Data pointer to the trait object (thin pointer)
    data_ptr: *const (),
    /// Vtable pointer for the trait object
    vtable_ptr: *const (),
    current_time: f64,
}

#[cfg(feature = "expressions")]
unsafe impl Trace for PropertyCaptures {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // Raw pointers and f64 don't contain GC references, nothing to trace
    }

    unsafe fn trace_non_roots(&self) {
        // No GC roots in this type
    }

    fn run_finalizer(&self) {
        // No finalization needed for raw pointers
    }
}

#[cfg(feature = "expressions")]
impl PropertyCaptures {
    /// Create from a trait object reference
    fn new(property_ref: &dyn AnimatedProperty, current_time: f64) -> Self {
        let fat_ptr = property_ref as *const dyn AnimatedProperty;
        // Decompose fat pointer into components
        // SAFETY: This is implementation-defined but stable for trait objects
        let (data_ptr, vtable_ptr): (*const (), *const ()) =
            unsafe { std::mem::transmute(fat_ptr) };
        Self {
            data_ptr,
            vtable_ptr,
            current_time,
        }
    }

    /// Reconstruct the trait object reference
    unsafe fn as_property_ref(&self) -> &dyn AnimatedProperty {
        let fat_ptr: *const dyn AnimatedProperty =
            std::mem::transmute((self.data_ptr, self.vtable_ptr));
        &*fat_ptr
    }
}

#[cfg(feature = "expressions")]
fn create_property_object(
    prop: &PropertyObject,
    property_ref: &dyn AnimatedProperty,
    context: &mut Context,
) -> Result<JsValue, String> {
    // Convert values to JsValue before creating ObjectInitializer to avoid borrow issues
    let current_value = prop.value().clone();
    let current_time = prop.time();
    let value_js = current_value.to_js_value(context);
    let velocity_js = prop.velocity().to_js_value(context);
    let speed_js = JsValue::new(prop.speed());

    // Create captures struct with decomposed trait object pointer
    let captures = PropertyCaptures::new(property_ref, current_time);

    // Build functions first (they don't need context borrow)
    let wiggle_fn = NativeFunction::from_copy_closure_with_captures(
        |_this, args, captures: &PropertyCaptures, ctx| {
            let freq = args
                .get_or_undefined(0)
                .to_number(ctx)
                .unwrap_or(1.0)
                .max(0.0);
            let amp = args.get_or_undefined(1).to_number(ctx).unwrap_or(0.0);
            let octaves = args
                .get_or_undefined(2)
                .to_number(ctx)
                .unwrap_or(1.0)
                .max(1.0) as i32;
            let amp_mult = args.get_or_undefined(3).to_number(ctx).unwrap_or(0.5);
            let t = args
                .get_or_undefined(4)
                .to_number(ctx)
                .unwrap_or(captures.current_time);

            let prop_ref = unsafe { captures.as_property_ref() };
            let base_value = prop_ref.value_at_time(t);
            let wiggled = wiggle_property_value(&base_value, t, freq, amp, octaves, amp_mult);

            Ok(wiggled.to_js_value(ctx))
        },
        captures,
    );

    let value_at_time_fn = NativeFunction::from_copy_closure_with_captures(
        |_this, args, captures: &PropertyCaptures, ctx| {
            let t = args
                .get_or_undefined(0)
                .to_number(ctx)
                .unwrap_or(captures.current_time);

            let prop_ref = unsafe { captures.as_property_ref() };
            let value = prop_ref.value_at_time(t);
            Ok(value.to_js_value(ctx))
        },
        captures,
    );

    let velocity_at_time_fn = NativeFunction::from_copy_closure_with_captures(
        |_this, args, captures: &PropertyCaptures, ctx| {
            let t = args
                .get_or_undefined(0)
                .to_number(ctx)
                .unwrap_or(captures.current_time);

            let prop_ref = unsafe { captures.as_property_ref() };
            let velocity = prop_ref.velocity_at_time(t);
            Ok(velocity.to_js_value(ctx))
        },
        captures,
    );

    // Now create ObjectInitializer and build the object
    let mut obj_init = ObjectInitializer::new(context);

    obj_init.property(js_string!("value"), value_js, Attribute::all());
    obj_init.property(js_string!("velocity"), velocity_js, Attribute::all());
    obj_init.property(js_string!("speed"), speed_js, Attribute::all());
    obj_init.function(wiggle_fn, js_string!("wiggle"), 2);
    obj_init.function(value_at_time_fn, js_string!("valueAtTime"), 1);
    obj_init.function(velocity_at_time_fn, js_string!("velocityAtTime"), 1);

    let obj = obj_init.build();
    Ok(JsValue::from(obj))
}

#[cfg(feature = "expressions")]
fn create_layer_object(
    layer: &LayerObject,
    context: &mut Context,
    time: f64,
) -> Result<JsValue, String> {
    // Pre-create the transform object before creating ObjectInitializer
    // to avoid double mutable borrow of context
    let transform = layer.transform();
    let transform_obj = create_transform_object(transform, context, time)?;

    // Pre-create name JS value
    let name_js = JsValue::from(js_string!(layer.name()));

    // Now create ObjectInitializer after all dependent objects are created
    let mut obj_init = ObjectInitializer::new(context);

    obj_init.property(
        js_string!("index"),
        JsValue::new(layer.index() as f64),
        Attribute::all(),
    );

    obj_init.property(js_string!("name"), name_js, Attribute::all());

    // Add coordinate transformation methods
    // For now, these are identity transforms - full implementation would need
    // access to the layer's transform hierarchy
    let to_world_fn = NativeFunction::from_fn_ptr(|_this, args, _ctx| {
        let point = args.get_or_undefined(0);
        // Identity for now - just return the point
        // Full implementation would apply layer transform matrix
        Ok(point.clone())
    });

    let from_world_fn = NativeFunction::from_fn_ptr(|_this, args, _ctx| {
        let point = args.get_or_undefined(0);
        // Inverse of toWorld
        Ok(point.clone())
    });

    let to_comp_fn = NativeFunction::from_fn_ptr(|_this, args, _ctx| {
        let point = args.get_or_undefined(0);
        // Composition space is typically same as world for 2D
        Ok(point.clone())
    });

    let from_comp_fn = NativeFunction::from_fn_ptr(|_this, args, _ctx| {
        let point = args.get_or_undefined(0);
        Ok(point.clone())
    });

    obj_init.function(to_world_fn, js_string!("toWorld"), 1);
    obj_init.function(from_world_fn, js_string!("fromWorld"), 1);
    obj_init.function(to_comp_fn, js_string!("toComp"), 1);
    obj_init.function(from_comp_fn, js_string!("fromComp"), 1);

    // Add transform property group with AE-exact accessors
    obj_init.property(js_string!("transform"), transform_obj, Attribute::all());

    let obj = obj_init.build();
    Ok(JsValue::from(obj))
}

#[cfg(feature = "expressions")]
fn create_transform_object(
    transform: &TransformObject,
    context: &mut Context,
    time: f64,
) -> Result<JsValue, String> {
    // Pre-convert all values to JsValue before creating ObjectInitializer
    // to avoid borrow checker issues
    let position_js = transform.position(time).to_js_value(context);
    let scale_js = transform.scale(time).to_js_value(context);
    let rotation_js = transform.rotation(time).to_js_value(context);
    let opacity_js = transform.opacity(time).to_js_value(context);
    let anchor_point_js = transform.anchor_point(time).to_js_value(context);

    // Pre-convert 3D rotation values if needed
    let has_3d_rotation = transform.rotation_x(time).as_scalar() != 0.0
        || transform.rotation_y(time).as_scalar() != 0.0
        || transform.rotation_z(time).as_scalar() != 0.0;

    let (rotation_x_js, rotation_y_js, rotation_z_js) = if has_3d_rotation {
        (
            Some(transform.rotation_x(time).to_js_value(context)),
            Some(transform.rotation_y(time).to_js_value(context)),
            Some(transform.rotation_z(time).to_js_value(context)),
        )
    } else {
        (None, None, None)
    };

    // Now create ObjectInitializer after all borrows are done
    let mut obj_init = ObjectInitializer::new(context);

    obj_init.property(js_string!("position"), position_js, Attribute::all());
    obj_init.property(js_string!("scale"), scale_js, Attribute::all());
    obj_init.property(js_string!("rotation"), rotation_js, Attribute::all());
    obj_init.property(js_string!("opacity"), opacity_js, Attribute::all());
    obj_init.property(js_string!("anchorPoint"), anchor_point_js, Attribute::all());

    // Add 3D rotation properties if present
    if let (Some(rx), Some(ry), Some(rz)) = (rotation_x_js, rotation_y_js, rotation_z_js) {
        obj_init.property(js_string!("xRotation"), rx, Attribute::all());
        obj_init.property(js_string!("yRotation"), ry, Attribute::all());
        obj_init.property(js_string!("zRotation"), rz, Attribute::all());
    }

    let obj = obj_init.build();
    Ok(JsValue::from(obj))
}

#[cfg(feature = "expressions")]
fn create_comp_object(
    comp: &CompObject,
    context: &mut Context,
    time: f64,
) -> Result<JsValue, String> {
    let mut obj_init = ObjectInitializer::new(context);

    // Create captures struct with comp and time
    #[derive(Clone, Finalize)]
    struct CompCaptures {
        comp: CompObject,
        time: f64,
    }

    unsafe impl Trace for CompCaptures {
        unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
            // CompObject doesn't contain GC references
        }
        unsafe fn trace_non_roots(&self) {}
        fn run_finalizer(&self) {}
    }

    let captures = CompCaptures {
        comp: comp.clone(),
        time,
    };

    // layer(index) - get layer by index number
    let layer_by_index_fn = NativeFunction::from_copy_closure_with_captures(
        |_this, args, captures: &CompCaptures, ctx| {
            let arg = args.get_or_undefined(0);

            // Check if argument is a number (index) or string (name)
            if let Some(index) = arg.as_number() {
                // Access by index: thisComp.layer(1)
                let idx = index as i32;
                if let Some(layer) = captures.comp.layer_by_index(idx) {
                    match create_layer_object(layer, ctx, captures.time) {
                        Ok(layer_js) => return Ok(layer_js),
                        Err(_) => return Ok(JsValue::undefined()),
                    }
                }
            } else if let Some(name_str) = arg.as_string() {
                // Access by name: thisComp.layer("Circle")
                let name = name_str.to_std_string_escaped();
                if let Some(layer) = captures.comp.layer_by_name(&name) {
                    match create_layer_object(layer, ctx, captures.time) {
                        Ok(layer_js) => return Ok(layer_js),
                        Err(_) => return Ok(JsValue::undefined()),
                    }
                }
            }

            Ok(JsValue::undefined())
        },
        captures,
    );

    obj_init.function(layer_by_index_fn, js_string!("layer"), 1);

    // numLayers property - total number of layers in the composition
    obj_init.property(
        js_string!("numLayers"),
        JsValue::new(comp.num_layers() as f64),
        Attribute::all(),
    );

    let obj = obj_init.build();
    Ok(JsValue::from(obj))
}

#[cfg(feature = "expressions")]
fn create_effect_object(effect: &EffectObject, context: &mut Context) -> Result<JsValue, String> {
    let mut obj_init = ObjectInitializer::new(context);

    // Store effect name for display
    let effect_name = js_string!(effect.name());
    obj_init.property(
        js_string!("name"),
        JsValue::from(effect_name),
        Attribute::all(),
    );

    // Create captures struct with effect data
    #[derive(Clone, Finalize)]
    struct EffectCaptures {
        effect: EffectObject,
    }

    unsafe impl Trace for EffectCaptures {
        unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
            // EffectObject doesn't contain GC references
        }
        unsafe fn trace_non_roots(&self) {}
        fn run_finalizer(&self) {}
    }

    let captures = EffectCaptures {
        effect: effect.clone(),
    };

    // The effect object is called like a function: effect("Name")("Control Name")
    // This returns the control value. We implement this by making the object callable.
    let control_accessor_fn = NativeFunction::from_copy_closure_with_captures(
        |_this, args, captures: &EffectCaptures, ctx| {
            let arg = args.get_or_undefined(0);

            // Check if argument is a string (control name) or number (control index)
            let control_value = if let Some(name_str) = arg.as_string() {
                // Access by name: effect("Slider Control")("Slider")
                let name = name_str.to_std_string_escaped();
                captures.effect.control_by_name(&name)
            } else if let Some(index) = arg.as_number() {
                // Access by index: effect("Slider Control")(0)
                let idx = index as u32;
                captures.effect.control_by_index(idx)
            } else {
                None
            };

            if let Some(control) = control_value {
                // Return the control value as a PropertyValue
                Ok(control.value().to_js_value(ctx))
            } else {
                // Return undefined if control not found
                Ok(JsValue::undefined())
            }
        },
        captures,
    );

    // Register the callable property - in AE, effect("Name")("Control") returns the value directly
    // We implement this by adding the function as "__call" and also making it the default behavior
    obj_init.function(control_accessor_fn, js_string!("__call"), 1);

    // Also expose numControls property
    obj_init.property(
        js_string!("numControls"),
        JsValue::new(effect.num_controls() as f64),
        Attribute::all(),
    );

    let obj = obj_init.build();
    Ok(JsValue::from(obj))
}

/// Register the global `effect()` function that returns effect objects by name
#[cfg(feature = "expressions")]
fn register_effect_function(effects: &[EffectObject], context: &mut Context) -> Result<(), String> {
    // Create captures struct with effects list
    #[derive(Clone, Finalize)]
    struct EffectsCaptures {
        effects: Vec<EffectObject>,
    }

    unsafe impl Trace for EffectsCaptures {
        unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
            // EffectObject doesn't contain GC references
        }
        unsafe fn trace_non_roots(&self) {}
        fn run_finalizer(&self) {}
    }

    let captures = EffectsCaptures {
        effects: effects.to_vec(),
    };

    // effect(name_or_index) - returns an EffectObject
    let effect_fn = NativeFunction::from_copy_closure_with_captures(
        |_this, args, captures: &EffectsCaptures, ctx| {
            let arg = args.get_or_undefined(0);

            // Find effect by name or index
            let effect = if let Some(name_str) = arg.as_string() {
                // Access by name: effect("Slider Control")
                let name = name_str.to_std_string_escaped();
                captures.effects.iter().find(|e| e.name() == name)
            } else if let Some(index) = arg.as_number() {
                // Access by index: effect(0) (1-based in AE, 0-based in our Vec)
                let idx = (index as usize).saturating_sub(1);
                captures.effects.get(idx)
            } else {
                None
            };

            if let Some(effect_obj) = effect {
                // Return the effect object
                match create_effect_object(effect_obj, ctx) {
                    Ok(effect_js) => Ok(effect_js),
                    Err(_) => Ok(JsValue::undefined()),
                }
            } else {
                // Return undefined if effect not found
                Ok(JsValue::undefined())
            }
        },
        captures,
    );

    // Register the function globally
    context
        .register_global_callable(js_string!("effect"), 1, effect_fn)
        .map_err(|e| format!("Failed to register effect function: {}", e))?;

    Ok(())
}

#[cfg(feature = "expressions")]
fn helper_add(a: &JsValue, b: &JsValue, context: &mut Context) -> JsResult<JsValue> {
    if let (Some(obj_a), Some(obj_b)) = (a.as_object(), b.as_object()) {
        if obj_a.is_array() && obj_b.is_array() {
            let len_a = obj_a
                .get(js_string!("length"), context)?
                .to_number(context)? as u64;
            let len_b = obj_b
                .get(js_string!("length"), context)?
                .to_number(context)? as u64;
            let len = std::cmp::min(len_a, len_b);
            let mut result = Vec::new();
            for i in 0..len {
                let val_a = obj_a.get(i, context)?.to_number(context)?;
                let val_b = obj_b.get(i, context)?.to_number(context)?;
                result.push(JsValue::new(val_a + val_b));
            }
            return Ok(JsArray::from_iter(result, context).into());
        }
    }
    let num_a = a.to_number(context)?;
    let num_b = b.to_number(context)?;
    Ok(JsValue::new(num_a + num_b))
}

#[cfg(feature = "expressions")]
fn helper_sub(a: &JsValue, b: &JsValue, context: &mut Context) -> JsResult<JsValue> {
    if let (Some(obj_a), Some(obj_b)) = (a.as_object(), b.as_object()) {
        if obj_a.is_array() && obj_b.is_array() {
            let len_a = obj_a
                .get(js_string!("length"), context)?
                .to_number(context)? as u64;
            let len_b = obj_b
                .get(js_string!("length"), context)?
                .to_number(context)? as u64;
            let len = std::cmp::min(len_a, len_b);
            let mut result = Vec::new();
            for i in 0..len {
                let val_a = obj_a.get(i, context)?.to_number(context)?;
                let val_b = obj_b.get(i, context)?.to_number(context)?;
                result.push(JsValue::new(val_a - val_b));
            }
            return Ok(JsArray::from_iter(result, context).into());
        }
    }
    let num_a = a.to_number(context)?;
    let num_b = b.to_number(context)?;
    Ok(JsValue::new(num_a - num_b))
}

#[cfg(feature = "expressions")]
fn helper_mul(a: &JsValue, b: &JsValue, context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj_a) = a.as_object() {
        if obj_a.is_array() {
            let scalar_b = b.to_number(context)?;
            let len = obj_a
                .get(js_string!("length"), context)?
                .to_number(context)? as u64;
            let mut result = Vec::new();
            for i in 0..len {
                let val_a = obj_a.get(i, context)?.to_number(context)?;
                result.push(JsValue::new(val_a * scalar_b));
            }
            return Ok(JsArray::from_iter(result, context).into());
        }
    }
    if let Some(obj_b) = b.as_object() {
        if obj_b.is_array() {
            let scalar_a = a.to_number(context)?;
            let len = obj_b
                .get(js_string!("length"), context)?
                .to_number(context)? as u64;
            let mut result = Vec::new();
            for i in 0..len {
                let val_b = obj_b.get(i, context)?.to_number(context)?;
                result.push(JsValue::new(scalar_a * val_b));
            }
            return Ok(JsArray::from_iter(result, context).into());
        }
    }

    let num_a = a.to_number(context)?;
    let num_b = b.to_number(context)?;
    Ok(JsValue::new(num_a * num_b))
}

#[cfg(feature = "expressions")]
fn helper_div(a: &JsValue, b: &JsValue, context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj_a) = a.as_object() {
        if obj_a.is_array() {
            let scalar_b = b.to_number(context)?;
            if scalar_b == 0.0 {
                return Ok(JsValue::nan());
            }
            let len = obj_a
                .get(js_string!("length"), context)?
                .to_number(context)? as u64;
            let mut result = Vec::new();
            for i in 0..len {
                let val_a = obj_a.get(i, context)?.to_number(context)?;
                result.push(JsValue::new(val_a / scalar_b));
            }
            return Ok(JsArray::from_iter(result, context).into());
        }
    }
    let num_a = a.to_number(context)?;
    let num_b = b.to_number(context)?;
    Ok(JsValue::new(num_a / num_b))
}

/// Interpolation types for ease functions
#[cfg(feature = "expressions")]
#[derive(Clone, Copy)]
enum InterpolationType {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
}

#[cfg(feature = "expressions")]
fn helper_interpolate(
    args: &[JsValue],
    context: &mut Context,
    interp_type: InterpolationType,
) -> JsResult<JsValue> {
    // AE interpolation functions: fn(t, tMin, tMax, value1, value2)
    // t: current time/input value
    // tMin: minimum input range
    // tMax: maximum input range
    // value1: output when t <= tMin
    // value2: output when t >= tMax

    if args.len() < 5 {
        return Ok(JsValue::undefined());
    }

    let t = args.get_or_undefined(0).to_number(context)?;
    let t_min = args.get_or_undefined(1).to_number(context)?;
    let t_max = args.get_or_undefined(2).to_number(context)?;
    let value1 = &args.get_or_undefined(3);
    let value2 = &args.get_or_undefined(4);

    // Calculate normalized position (0 to 1)
    let range = t_max - t_min;
    let normalized = if range == 0.0 {
        0.0
    } else {
        ((t - t_min) / range).clamp(0.0, 1.0)
    };

    // Apply easing to the normalized value
    let eased = match interp_type {
        InterpolationType::Linear => normalized,
        InterpolationType::Ease => ease_in_out(normalized),
        InterpolationType::EaseIn => ease_in(normalized),
        InterpolationType::EaseOut => ease_out(normalized),
    };

    // Interpolate between value1 and value2
    interpolate_values(value1, value2, eased, context)
}

/// Linear interpolation between two values
#[cfg(feature = "expressions")]
fn interpolate_values(
    a: &JsValue,
    b: &JsValue,
    t: f64,
    context: &mut Context,
) -> JsResult<JsValue> {
    // Handle array interpolation (vectors)
    if let (Some(obj_a), Some(obj_b)) = (a.as_object(), b.as_object()) {
        if obj_a.is_array() && obj_b.is_array() {
            let len_a = obj_a
                .get(js_string!("length"), context)?
                .to_number(context)? as u64;
            let len_b = obj_b
                .get(js_string!("length"), context)?
                .to_number(context)? as u64;
            let len = std::cmp::min(len_a, len_b);

            let mut result = Vec::new();
            for i in 0..len {
                let val_a = obj_a.get(i, context)?.to_number(context)?;
                let val_b = obj_b.get(i, context)?.to_number(context)?;
                let interpolated = val_a + (val_b - val_a) * t;
                result.push(JsValue::new(interpolated));
            }
            return Ok(JsArray::from_iter(result, context).into());
        }
    }

    // Scalar interpolation
    let val_a = a.to_number(context)?;
    let val_b = b.to_number(context)?;
    Ok(JsValue::new(val_a + (val_b - val_a) * t))
}

/// Standard ease function (easeInOut) using cubic-bezier-like interpolation
#[cfg(feature = "expressions")]
fn ease_in_out(t: f64) -> f64 {
    // Cubic ease in-out: smoother than quadratic
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - ((-2.0 * t + 2.0).powi(3) / 2.0)
    }
}

/// Ease in function (slow start, fast end)
#[cfg(feature = "expressions")]
fn ease_in(t: f64) -> f64 {
    // Quadratic ease in
    t * t
}

/// Ease out function (fast start, slow end)
#[cfg(feature = "expressions")]
fn ease_out(t: f64) -> f64 {
    // Quadratic ease out
    1.0 - (1.0 - t) * (1.0 - t)
}
