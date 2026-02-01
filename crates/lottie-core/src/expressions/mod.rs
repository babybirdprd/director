//! AE-Exact Expression System for Lottie
//!
//! Implements After Effects expression evaluation with proper Property objects
//! supporting methods like wiggle(), valueAtTime() and attributes like value, velocity.

pub mod property;
pub mod wiggle;

pub use property::{AnimatedProperty, LayerObject, PropertyObject, PropertyValue};
pub use wiggle::{wiggle_property_value, wiggle_property_value_with_amps, WiggleState};

#[cfg(feature = "expressions")]
use boa_engine::{
    context::ContextBuilder, js_string, native_function::NativeFunction, object::builtins::JsArray,
    object::ObjectInitializer, property::Attribute, Context, JsArgs, JsResult, JsValue, Source,
};
#[cfg(feature = "expressions")]
use boa_gc::{Finalize, Trace};

/// Loop type for expressions
#[derive(Clone, Copy, Debug, PartialEq)]
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
                    Ok(vec)
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
                            let get_component =
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
                            let get_component =
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

        // LoopOut with type support
        let loop_out_type = self.loop_out_type;
        self.context
            .register_global_callable(
                js_string!("loopOut"),
                0,
                NativeFunction::from_copy_closure_with_captures(
                    |_this, args, loop_type: &LoopType, context| {
                        let requested_type = if args.len() > 0 {
                            if let Ok(s) = args.get_or_undefined(0).to_string(context) {
                                LoopType::from_str(&s.to_std_string().unwrap_or_default())
                            } else {
                                *loop_type
                            }
                        } else {
                            *loop_type
                        };

                        // For now, we return the loop value - the actual loop type
                        // is handled in the pre-calculation phase in Animator::resolve
                        let val = context
                            .global_object()
                            .get(js_string!("__loop_value"), context)
                            .unwrap_or_default();
                        Ok(val)
                    },
                    loop_out_type,
                ),
            )
            .map_err(|e| format!("Failed to register loopOut: {}", e))?;

        // LoopIn with type support
        let loop_in_type = self.loop_in_type;
        self.context
            .register_global_callable(
                js_string!("loopIn"),
                0,
                NativeFunction::from_copy_closure_with_captures(
                    |_this, args, loop_type: &LoopType, context| {
                        let requested_type = if args.len() > 0 {
                            if let Ok(s) = args.get_or_undefined(0).to_string(context) {
                                LoopType::from_str(&s.to_std_string().unwrap_or_default())
                            } else {
                                *loop_type
                            }
                        } else {
                            *loop_type
                        };

                        // Return the loop value for loopIn
                        let val = context
                            .global_object()
                            .get(js_string!("__loop_in_value"), context)
                            .unwrap_or_default();
                        Ok(val)
                    },
                    loop_in_type,
                ),
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
    pub fn evaluate_on_property(
        &mut self,
        script: &str,
        property: &dyn AnimatedProperty,
        current_value: &PropertyValue,
        time: f64,
        layer_index: i32,
        layer_name: &str,
        loop_out_value: Option<&PropertyValue>,
        loop_in_value: Option<&PropertyValue>,
        director_vars: &DirectorVariableContext,
    ) -> Result<PropertyValue, String> {
        let velocity = property.velocity_at_time(time);
        let speed = property.speed_at_time(time);

        let prop_obj = PropertyObject::new(current_value.clone(), time, velocity, speed, None);

        let layer_obj = LayerObject::new(layer_index, layer_name);

        let prop_js = create_property_object(&prop_obj, property, &mut self.context)?;
        let layer_js = create_layer_object(&layer_obj, &mut self.context)?;

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
                JsValue::new(layer_index as f64),
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
    let mut obj_init = ObjectInitializer::new(context);

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
    for (key, value) in all_vars {
        let js_val = value.to_js_value(context);
        obj_init.property(js_string!(&key), js_val, Attribute::all());
    }

    let obj = obj_init.build();
    Ok(JsValue::from(obj))
}

#[cfg(feature = "expressions")]
fn create_director_context_object(
    ctx: &DirectorVariableContext,
    context: &mut Context,
) -> Result<JsValue, String> {
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

    let scene_init = ObjectInitializer::new(context);
    let scene_obj = scene_init
        .property(
            js_string!("name"),
            JsValue::from(js_string!(&ctx.scene_name)),
            Attribute::all(),
        )
        .property(
            js_string!("duration"),
            JsValue::new(ctx.scene_duration),
            Attribute::all(),
        )
        .build();

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
fn create_layer_object(layer: &LayerObject, context: &mut Context) -> Result<JsValue, String> {
    let mut obj_init = ObjectInitializer::new(context);

    obj_init.property(
        js_string!("index"),
        JsValue::new(layer.index() as f64),
        Attribute::all(),
    );

    obj_init.property(
        js_string!("name"),
        JsValue::from(js_string!(layer.name())),
        Attribute::all(),
    );

    // Add coordinate transformation methods
    // For now, these are identity transforms - full implementation would need
    // access to the layer's transform hierarchy
    let to_world_fn = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let point = args.get_or_undefined(0);
        // Identity for now - just return the point
        // Full implementation would apply layer transform matrix
        Ok(point)
    });

    let from_world_fn = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let point = args.get_or_undefined(0);
        // Inverse of toWorld
        Ok(point)
    });

    let to_comp_fn = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let point = args.get_or_undefined(0);
        // Composition space is typically same as world for 2D
        Ok(point)
    });

    let from_comp_fn = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let point = args.get_or_undefined(0);
        Ok(point)
    });

    obj_init.function(to_world_fn, js_string!("toWorld"), 1);
    obj_init.function(from_world_fn, js_string!("fromWorld"), 1);
    obj_init.function(to_comp_fn, js_string!("toComp"), 1);
    obj_init.function(from_comp_fn, js_string!("fromComp"), 1);

    let obj = obj_init.build();
    Ok(JsValue::from(obj))
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
