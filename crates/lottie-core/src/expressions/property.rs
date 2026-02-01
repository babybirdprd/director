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

/// AE Layer Object - represents layer context
pub struct LayerObject {
    index: i32,
    name: String,
}

impl LayerObject {
    /// Create a new LayerObject
    pub fn new(index: i32, name: impl Into<String>) -> Self {
        Self {
            index,
            name: name.into(),
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
}
