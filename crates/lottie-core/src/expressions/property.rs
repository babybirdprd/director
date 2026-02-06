//! Property Object System for AE-Exact Expressions
//!
//! Implements AE Property objects with methods like wiggle(), valueAtTime()
//! and attributes like value, velocity, speed.

use std::sync::Arc;

/// Represents a property value - either scalar or vector
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyValue {
    Scalar(f64),
    Vector(Vec<f64>),
}

impl PropertyValue {
    /// Get value as scalar (returns first component for vectors)
    pub fn as_scalar(&self) -> f64 {
        match self {
            PropertyValue::Scalar(v) => *v,
            PropertyValue::Vector(v) => v.first().copied().unwrap_or(0.0),
        }
    }

    /// Get value as vector (wraps scalar in single-element vec)
    pub fn as_vector(&self) -> Vec<f64> {
        match self {
            PropertyValue::Scalar(v) => vec![*v],
            PropertyValue::Vector(v) => v.clone(),
        }
    }

    /// Get the dimension of the value
    pub fn dimension(&self) -> usize {
        match self {
            PropertyValue::Scalar(_) => 1,
            PropertyValue::Vector(v) => v.len(),
        }
    }

    /// Add two property values
    pub fn add(&self, other: &PropertyValue) -> PropertyValue {
        match (self, other) {
            (PropertyValue::Scalar(a), PropertyValue::Scalar(b)) => PropertyValue::Scalar(a + b),
            (PropertyValue::Vector(a), PropertyValue::Vector(b)) => {
                let result: Vec<f64> = a.iter().zip(b.iter()).map(|(x, y)| x + y).collect();
                PropertyValue::Vector(result)
            }
            (PropertyValue::Scalar(a), PropertyValue::Vector(b)) => {
                let result: Vec<f64> = b.iter().map(|x| x + a).collect();
                PropertyValue::Vector(result)
            }
            (PropertyValue::Vector(a), PropertyValue::Scalar(b)) => {
                let result: Vec<f64> = a.iter().map(|x| x + b).collect();
                PropertyValue::Vector(result)
            }
        }
    }

    /// Scale property value by factor
    pub fn scale(&self, factor: f64) -> PropertyValue {
        match self {
            PropertyValue::Scalar(v) => PropertyValue::Scalar(v * factor),
            PropertyValue::Vector(v) => {
                PropertyValue::Vector(v.iter().map(|x| x * factor).collect())
            }
        }
    }

    /// Convert to boa_engine JsValue
    #[cfg(feature = "expressions")]
    pub fn to_js_value(&self, context: &mut boa_engine::Context) -> boa_engine::JsValue {
        match self {
            PropertyValue::Scalar(v) => boa_engine::JsValue::new(*v),
            PropertyValue::Vector(vec) => {
                let js_values: Vec<boa_engine::JsValue> =
                    vec.iter().map(|v| boa_engine::JsValue::new(*v)).collect();
                boa_engine::object::builtins::JsArray::from_iter(js_values, context).into()
            }
        }
    }

    /// Convert from boa_engine JsValue
    #[cfg(feature = "expressions")]
    pub fn from_js_value(
        value: &boa_engine::JsValue,
        context: &mut boa_engine::Context,
    ) -> Option<Self> {
        if let Some(num) = value.as_number() {
            return Some(PropertyValue::Scalar(num));
        }

        if let Some(obj) = value.as_object() {
            if obj.is_array() {
                if let Ok(length) = obj.get(boa_engine::js_string!("length"), context) {
                    if let Ok(len) = length.to_number(context) {
                        let len = len as u64;
                        let mut vec = Vec::with_capacity(len as usize);
                        for i in 0..len {
                            if let Ok(val) = obj.get(i, context) {
                                if let Ok(num) = val.to_number(context) {
                                    vec.push(num);
                                }
                            }
                        }
                        return Some(PropertyValue::Vector(vec));
                    }
                }
            }
        }

        None
    }
}

impl Default for PropertyValue {
    fn default() -> Self {
        PropertyValue::Scalar(0.0)
    }
}

/// Trait for animated properties that can be sampled
pub trait AnimatedProperty: Send + Sync {
    /// Get value at specific time
    fn value_at_time(&self, time: f64) -> PropertyValue;

    /// Get velocity at specific time
    fn velocity_at_time(&self, time: f64) -> PropertyValue;

    /// Get speed at specific time
    fn speed_at_time(&self, time: f64) -> f64;
}

/// AE Property Object - represents a property with methods and attributes
pub struct PropertyObject {
    /// Current value
    value: PropertyValue,
    /// Current time
    time: f64,
    /// Current velocity
    velocity: PropertyValue,
    /// Current speed
    speed: f64,
    /// Reference to underlying animated property for sampling
    property_ref: Option<Arc<dyn AnimatedProperty>>,
}

impl PropertyObject {
    /// Create a new PropertyObject
    pub fn new(
        value: PropertyValue,
        time: f64,
        velocity: PropertyValue,
        speed: f64,
        property_ref: Option<Arc<dyn AnimatedProperty>>,
    ) -> Self {
        Self {
            value,
            time,
            velocity,
            speed,
            property_ref,
        }
    }

    /// Get the value attribute (AE: property.value)
    pub fn value(&self) -> &PropertyValue {
        &self.value
    }

    /// Get the velocity attribute (AE: property.velocity)
    pub fn velocity(&self) -> &PropertyValue {
        &self.velocity
    }

    /// Get the speed attribute (AE: property.speed)
    pub fn speed(&self) -> f64 {
        self.speed
    }

    /// Get value at specific time (AE: property.valueAtTime(t))
    pub fn value_at_time(&self, t: f64) -> PropertyValue {
        if let Some(prop) = &self.property_ref {
            prop.value_at_time(t)
        } else {
            self.value.clone()
        }
    }

    /// Get velocity at specific time (AE: property.velocityAtTime(t))
    pub fn velocity_at_time(&self, t: f64) -> PropertyValue {
        if let Some(prop) = &self.property_ref {
            prop.velocity_at_time(t)
        } else {
            PropertyValue::Scalar(0.0)
        }
    }

    /// Wiggle the property value (AE: property.wiggle(freq, amp, octaves, amp_mult, t))
    pub fn wiggle(
        &self,
        freq: f64,
        amp: f64,
        octaves: i32,
        amp_mult: f64,
        t: f64,
    ) -> PropertyValue {
        // Import wiggle function from wiggle module
        use super::wiggle::wiggle_property_value;

        wiggle_property_value(&self.value, t, freq, amp, octaves, amp_mult)
    }

    /// Get the current time
    pub fn time(&self) -> f64 {
        self.time
    }
}

/// Transform property sampler - used for on-demand sampling of transform values
#[cfg(feature = "expressions")]
pub type TransformSampler = Arc<dyn Fn(f64) -> PropertyValue + Send + Sync>;

/// AE Transform Object - represents layer transform properties
/// Supports access like: thisLayer.transform.position, thisLayer.transform.scale, etc.
#[cfg(feature = "expressions")]
#[derive(Clone)]
pub struct TransformObject {
    /// Position sampler (returns [x, y] or [x, y, z])
    position_sampler: Option<TransformSampler>,
    /// Scale sampler (returns [x, y] or [x, y, z] in percent)
    scale_sampler: Option<TransformSampler>,
    /// Rotation sampler (returns degrees for Z rotation)
    rotation_sampler: Option<TransformSampler>,
    /// Opacity sampler (returns 0-100)
    opacity_sampler: Option<TransformSampler>,
    /// Anchor point sampler (returns [x, y] or [x, y, z])
    anchor_point_sampler: Option<TransformSampler>,
    /// X/Y/Z rotation samplers for 3D layers
    rotation_x_sampler: Option<TransformSampler>,
    rotation_y_sampler: Option<TransformSampler>,
    rotation_z_sampler: Option<TransformSampler>,
}

/// AE Transform Object - represents layer transform properties (non-expressions version)
#[cfg(not(feature = "expressions"))]
#[derive(Clone)]
pub struct TransformObject;

impl TransformObject {
    /// Create a new TransformObject with all samplers
    #[cfg(feature = "expressions")]
    pub fn new(
        position_sampler: Option<TransformSampler>,
        scale_sampler: Option<TransformSampler>,
        rotation_sampler: Option<TransformSampler>,
        opacity_sampler: Option<TransformSampler>,
        anchor_point_sampler: Option<TransformSampler>,
    ) -> Self {
        Self {
            position_sampler,
            scale_sampler,
            rotation_sampler,
            opacity_sampler,
            anchor_point_sampler,
            rotation_x_sampler: None,
            rotation_y_sampler: None,
            rotation_z_sampler: None,
        }
    }

    /// Create a new TransformObject for 3D layers with separate rotation axes
    #[cfg(feature = "expressions")]
    pub fn new_3d(
        position_sampler: Option<TransformSampler>,
        scale_sampler: Option<TransformSampler>,
        rotation_x_sampler: Option<TransformSampler>,
        rotation_y_sampler: Option<TransformSampler>,
        rotation_z_sampler: Option<TransformSampler>,
        opacity_sampler: Option<TransformSampler>,
        anchor_point_sampler: Option<TransformSampler>,
    ) -> Self {
        Self {
            position_sampler,
            scale_sampler,
            rotation_sampler: None, // Use rotation_z instead for 3D
            opacity_sampler,
            anchor_point_sampler,
            rotation_x_sampler,
            rotation_y_sampler,
            rotation_z_sampler,
        }
    }

    /// Get position at time (AE: transform.position)
    pub fn position(&self, time: f64) -> PropertyValue {
        #[cfg(feature = "expressions")]
        if let Some(ref sampler) = self.position_sampler {
            return sampler(time);
        }
        PropertyValue::Vector(vec![0.0, 0.0])
    }

    /// Get scale at time (AE: transform.scale)
    pub fn scale(&self, time: f64) -> PropertyValue {
        #[cfg(feature = "expressions")]
        if let Some(ref sampler) = self.scale_sampler {
            return sampler(time);
        }
        PropertyValue::Vector(vec![100.0, 100.0])
    }

    /// Get rotation (Z) at time (AE: transform.rotation)
    pub fn rotation(&self, time: f64) -> PropertyValue {
        #[cfg(feature = "expressions")]
        {
            if let Some(ref sampler) = self.rotation_sampler {
                return sampler(time);
            }
            if let Some(ref sampler) = self.rotation_z_sampler {
                return sampler(time);
            }
        }
        PropertyValue::Scalar(0.0)
    }

    /// Get rotation X at time for 3D layers (AE: transform.orientation or transform.xRotation)
    pub fn rotation_x(&self, time: f64) -> PropertyValue {
        #[cfg(feature = "expressions")]
        if let Some(ref sampler) = self.rotation_x_sampler {
            return sampler(time);
        }
        PropertyValue::Scalar(0.0)
    }

    /// Get rotation Y at time for 3D layers
    pub fn rotation_y(&self, time: f64) -> PropertyValue {
        #[cfg(feature = "expressions")]
        if let Some(ref sampler) = self.rotation_y_sampler {
            return sampler(time);
        }
        PropertyValue::Scalar(0.0)
    }

    /// Get rotation Z at time for 3D layers
    pub fn rotation_z(&self, time: f64) -> PropertyValue {
        #[cfg(feature = "expressions")]
        if let Some(ref sampler) = self.rotation_z_sampler {
            return sampler(time);
        }
        PropertyValue::Scalar(0.0)
    }

    /// Get opacity at time (AE: transform.opacity)
    pub fn opacity(&self, time: f64) -> PropertyValue {
        #[cfg(feature = "expressions")]
        if let Some(ref sampler) = self.opacity_sampler {
            return sampler(time);
        }
        PropertyValue::Scalar(100.0)
    }

    /// Get anchor point at time (AE: transform.anchorPoint)
    pub fn anchor_point(&self, time: f64) -> PropertyValue {
        #[cfg(feature = "expressions")]
        if let Some(ref sampler) = self.anchor_point_sampler {
            return sampler(time);
        }
        PropertyValue::Vector(vec![0.0, 0.0])
    }
}

impl Default for TransformObject {
    fn default() -> Self {
        Self {
            #[cfg(feature = "expressions")]
            position_sampler: None,
            #[cfg(feature = "expressions")]
            scale_sampler: None,
            #[cfg(feature = "expressions")]
            rotation_sampler: None,
            #[cfg(feature = "expressions")]
            opacity_sampler: None,
            #[cfg(feature = "expressions")]
            anchor_point_sampler: None,
            #[cfg(feature = "expressions")]
            rotation_x_sampler: None,
            #[cfg(feature = "expressions")]
            rotation_y_sampler: None,
            #[cfg(feature = "expressions")]
            rotation_z_sampler: None,
        }
    }
}

/// AE Layer Object - represents layer context
#[derive(Clone)]
pub struct LayerObject {
    index: i32,
    name: String,
    /// Transform properties for this layer
    transform: TransformObject,
}

impl LayerObject {
    /// Create a new LayerObject
    pub fn new(index: i32, name: impl Into<String>) -> Self {
        Self {
            index,
            name: name.into(),
            transform: TransformObject::default(),
        }
    }

    /// Create a new LayerObject with transform
    pub fn with_transform(index: i32, name: impl Into<String>, transform: TransformObject) -> Self {
        Self {
            index,
            name: name.into(),
            transform,
        }
    }

    /// Get layer index (AE: layer.index or index)
    pub fn index(&self) -> i32 {
        self.index
    }

    /// Get layer name (AE: layer.name)
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get transform object (AE: layer.transform)
    pub fn transform(&self) -> &TransformObject {
        &self.transform
    }

    /// Get mutable transform object for building
    pub fn transform_mut(&mut self) -> &mut TransformObject {
        &mut self.transform
    }
}

/// AE Composition Object - represents the composition context
/// Provides access to all layers in the composition via `thisComp.layer(index)` or `thisComp.layer(name)`
#[derive(Clone)]
pub struct CompObject {
    /// Map of layer index to LayerObject
    layers_by_index: std::collections::HashMap<i32, LayerObject>,
    /// Map of layer name to layer index for name-based lookups
    layers_by_name: std::collections::HashMap<String, i32>,
}

impl CompObject {
    /// Create a new empty CompObject
    pub fn new() -> Self {
        Self {
            layers_by_index: std::collections::HashMap::new(),
            layers_by_name: std::collections::HashMap::new(),
        }
    }

    /// Add a layer to the composition
    pub fn add_layer(&mut self, layer: LayerObject) {
        let index = layer.index();
        let name = layer.name().to_string();
        self.layers_by_index.insert(index, layer);
        self.layers_by_name.insert(name, index);
    }

    /// Get a layer by index (AE: thisComp.layer(index))
    pub fn layer_by_index(&self, index: i32) -> Option<&LayerObject> {
        self.layers_by_index.get(&index)
    }

    /// Get a layer by name (AE: thisComp.layer(name))
    pub fn layer_by_name(&self, name: &str) -> Option<&LayerObject> {
        self.layers_by_name
            .get(name)
            .and_then(|&index| self.layers_by_index.get(&index))
    }

    /// Get the number of layers in the composition
    pub fn num_layers(&self) -> usize {
        self.layers_by_index.len()
    }
}

impl Default for CompObject {
    fn default() -> Self {
        Self::new()
    }
}

/// AE Effect Control Value - represents a single control value within an effect
/// Supports different control types: Slider, Angle, Checkbox, Color, Point
#[derive(Clone)]
pub struct ControlValue {
    /// Control name (e.g., "Slider", "Angle", "Checkbox")
    name: String,
    /// Current value at the evaluated time
    value: PropertyValue,
}

impl ControlValue {
    /// Create a new ControlValue
    pub fn new(name: impl Into<String>, value: PropertyValue) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    /// Get the control name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the current value
    pub fn value(&self) -> &PropertyValue {
        &self.value
    }
}

/// AE Effect Object - represents an effect with controls
/// Accessed via: effect("Effect Name")("Control Name")
#[derive(Clone)]
pub struct EffectObject {
    /// Effect name
    name: String,
    /// Effect index
    index: u32,
    /// Controls by name
    controls_by_name: std::collections::HashMap<String, ControlValue>,
    /// Controls by index
    controls_by_index: std::collections::HashMap<u32, ControlValue>,
}

impl EffectObject {
    /// Create a new EffectObject
    pub fn new(name: impl Into<String>, index: u32) -> Self {
        Self {
            name: name.into(),
            index,
            controls_by_name: std::collections::HashMap::new(),
            controls_by_index: std::collections::HashMap::new(),
        }
    }

    /// Add a control to the effect
    pub fn add_control(&mut self, index: u32, control: ControlValue) {
        let name = control.name().to_string();
        self.controls_by_name.insert(name, control.clone());
        self.controls_by_index.insert(index, control);
    }

    /// Get a control by name (AE: effect("Name")("Control Name"))
    pub fn control_by_name(&self, name: &str) -> Option<&ControlValue> {
        self.controls_by_name.get(name)
    }

    /// Get a control by index
    pub fn control_by_index(&self, index: u32) -> Option<&ControlValue> {
        self.controls_by_index.get(&index)
    }

    /// Get effect name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get effect index
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Get number of controls
    pub fn num_controls(&self) -> usize {
        self.controls_by_name.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_value_scalar() {
        let val = PropertyValue::Scalar(10.0);
        assert_eq!(val.as_scalar(), 10.0);
        assert_eq!(val.dimension(), 1);
    }

    #[test]
    fn test_property_value_vector() {
        let val = PropertyValue::Vector(vec![1.0, 2.0, 3.0]);
        assert_eq!(val.as_vector(), vec![1.0, 2.0, 3.0]);
        assert_eq!(val.dimension(), 3);
    }

    #[test]
    fn test_property_value_add() {
        let a = PropertyValue::Scalar(5.0);
        let b = PropertyValue::Scalar(3.0);
        assert_eq!(a.add(&b), PropertyValue::Scalar(8.0));
    }

    #[test]
    fn test_property_value_scale() {
        let val = PropertyValue::Vector(vec![1.0, 2.0, 3.0]);
        let scaled = val.scale(2.0);
        assert_eq!(scaled, PropertyValue::Vector(vec![2.0, 4.0, 6.0]));
    }

    #[test]
    fn test_layer_object() {
        let layer = LayerObject::new(5, "Test Layer");
        assert_eq!(layer.index(), 5);
        assert_eq!(layer.name(), "Test Layer");
    }

    #[test]
    fn test_comp_object() {
        let mut comp = CompObject::new();

        // Add multiple layers
        let layer1 = LayerObject::new(1, "Background");
        let layer2 = LayerObject::new(2, "Circle");
        let layer3 = LayerObject::new(3, "Text Layer");

        comp.add_layer(layer1);
        comp.add_layer(layer2);
        comp.add_layer(layer3);

        // Verify num_layers
        assert_eq!(comp.num_layers(), 3);

        // Access by index
        let by_index = comp.layer_by_index(2);
        assert!(by_index.is_some());
        assert_eq!(by_index.unwrap().name(), "Circle");

        // Access by name
        let by_name = comp.layer_by_name("Text Layer");
        assert!(by_name.is_some());
        assert_eq!(by_name.unwrap().index(), 3);

        // Non-existent access
        assert!(comp.layer_by_index(999).is_none());
        assert!(comp.layer_by_name("NonExistent").is_none());
    }

    #[test]
    fn test_comp_object_clone() {
        let mut comp = CompObject::new();
        comp.add_layer(LayerObject::new(1, "Layer1"));
        comp.add_layer(LayerObject::new(2, "Layer2"));

        let cloned = comp.clone();

        // Verify clone works correctly
        assert_eq!(cloned.num_layers(), 2);
        assert_eq!(cloned.layer_by_index(1).unwrap().name(), "Layer1");
        assert_eq!(cloned.layer_by_name("Layer2").unwrap().index(), 2);
    }
}
