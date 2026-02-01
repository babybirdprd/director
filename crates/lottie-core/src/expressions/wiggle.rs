//! Wiggle implementation for AE-exact expression behavior
//!
//! Implements AE wiggle(freq, amp, octaves, amp_mult, t) with Perlin noise

use noise::{NoiseFn, Perlin};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::property::PropertyValue;

/// Wiggle state for deterministic noise generation
pub struct WiggleState {
    perlin: Perlin,
    seed: u32,
}

impl WiggleState {
    /// Create new wiggle state with random seed
    pub fn new() -> Self {
        Self::with_seed(0)
    }

    /// Create new wiggle state with specific seed
    pub fn with_seed(seed: u32) -> Self {
        Self {
            perlin: Perlin::new(seed),
            seed,
        }
    }

    /// Create wiggle state from string hash (for dimension-specific seeds)
    pub fn from_string(s: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        let seed = (hasher.finish() & 0xFFFFFFFF) as u32;
        Self::with_seed(seed)
    }

    /// Generate wiggle value for a single dimension
    ///
    /// AE wiggle algorithm:
    /// 1. Sample noise at time * frequency
    /// 2. Combine multiple octaves at increasing frequencies and decreasing amplitudes
    /// 3. Result is in range [-amp, +amp]
    pub fn wiggle(&self, time: f64, freq: f64, amp: f64, octaves: i32, amp_mult: f64) -> f64 {
        let sample_time = time * freq;
        let mut result = 0.0;
        let mut current_amp = amp;
        let mut current_freq = 1.0;

        let octaves = octaves.max(1);

        for _ in 0..octaves {
            // Sample Perlin noise (range [-1, 1])
            // Scale coordinates to avoid large integer values that return 0
            // Use fractional coordinates in a reasonable range
            let noise_val = self
                .perlin
                .get([sample_time * current_freq * 0.1, self.seed as f64 * 0.01]);

            // Scale by current octave amplitude and add to result
            result += noise_val * current_amp;

            // Prepare next octave
            current_amp *= amp_mult;
            current_freq *= 2.0;
        }

        // Normalize result to [-amp, +amp] range
        // With multiple octaves, the sum can exceed the base amplitude
        // We clamp to ensure it stays within bounds
        result.clamp(-amp, amp)
    }

    /// Generate wiggle for a vector property (each dimension gets different noise)
    pub fn wiggle_vector(
        &self,
        time: f64,
        freq: f64,
        amp: &[f64],
        octaves: i32,
        amp_mult: f64,
    ) -> Vec<f64> {
        amp.iter()
            .enumerate()
            .map(|(i, a)| {
                // Each dimension gets a different seed offset for independent noise
                let dim_state = WiggleState::with_seed(self.seed.wrapping_add(i as u32));
                dim_state.wiggle(time, freq, *a, octaves, amp_mult)
            })
            .collect()
    }
}

impl Default for WiggleState {
    fn default() -> Self {
        Self::new()
    }
}

/// Wiggle a property value according to AE semantics
pub fn wiggle_property_value(
    base_value: &PropertyValue,
    time: f64,
    freq: f64,
    amp: f64,
    octaves: i32,
    amp_mult: f64,
) -> PropertyValue {
    let wiggle_state = WiggleState::new();

    match base_value {
        PropertyValue::Scalar(_) => {
            let wiggled = wiggle_state.wiggle(time, freq, amp, octaves, amp_mult);
            PropertyValue::Scalar(base_value.as_scalar() + wiggled)
        }
        PropertyValue::Vector(components) => {
            // For vector properties, wiggle each dimension with the same amplitude
            let amps: Vec<f64> = components.iter().map(|_| amp).collect();
            let wiggled_components =
                wiggle_state.wiggle_vector(time, freq, &amps, octaves, amp_mult);

            // Add wiggle to original values
            let result: Vec<f64> = components
                .iter()
                .zip(wiggled_components.iter())
                .map(|(base, w)| base + w)
                .collect();

            PropertyValue::Vector(result)
        }
    }
}

/// Wiggle with per-dimension amplitude control
pub fn wiggle_property_value_with_amps(
    base_value: &PropertyValue,
    time: f64,
    freq: f64,
    amps: &[f64],
    octaves: i32,
    amp_mult: f64,
) -> PropertyValue {
    let wiggle_state = WiggleState::new();

    match base_value {
        PropertyValue::Scalar(_) => {
            let amp = amps.first().copied().unwrap_or(1.0);
            let wiggled = wiggle_state.wiggle(time, freq, amp, octaves, amp_mult);
            PropertyValue::Scalar(base_value.as_scalar() + wiggled)
        }
        PropertyValue::Vector(components) => {
            // Use provided amps for each dimension, default to 1.0 if not enough
            let effective_amps: Vec<f64> = components
                .iter()
                .enumerate()
                .map(|(i, _)| amps.get(i).copied().unwrap_or(1.0))
                .collect();

            let wiggled_components =
                wiggle_state.wiggle_vector(time, freq, &effective_amps, octaves, amp_mult);

            let result: Vec<f64> = components
                .iter()
                .zip(wiggled_components.iter())
                .map(|(base, w)| base + w)
                .collect();

            PropertyValue::Vector(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wiggle_deterministic() {
        let state = WiggleState::with_seed(12345);

        // Same time = same value
        let v1 = state.wiggle(1.0, 5.0, 10.0, 1, 0.5);
        let v2 = state.wiggle(1.0, 5.0, 10.0, 1, 0.5);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_wiggle_time_variation() {
        let state = WiggleState::with_seed(12345);

        // Different time = different value
        let v1 = state.wiggle(1.0, 5.0, 10.0, 1, 0.5);
        let v2 = state.wiggle(2.0, 5.0, 10.0, 1, 0.5);
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_wiggle_amplitude_bounds() {
        let state = WiggleState::with_seed(12345);
        let amp = 10.0;

        // Test many samples
        for i in 0..100 {
            let t = i as f64 / 30.0;
            let v = state.wiggle(t, 5.0, amp, 1, 0.5);
            assert!(
                v >= -amp && v <= amp,
                "Wiggle value {} out of bounds [-{}, {}]",
                v,
                amp,
                amp
            );
        }
    }

    #[test]
    fn test_wiggle_multiple_octaves() {
        let state = WiggleState::with_seed(12345);

        let simple = state.wiggle(1.0, 5.0, 10.0, 1, 0.5);
        let detailed = state.wiggle(1.0, 5.0, 10.0, 4, 0.5);

        // More octaves should produce different result
        assert_ne!(simple, detailed);
    }

    #[test]
    fn test_wiggle_frequency_effect() {
        let state = WiggleState::with_seed(12345);

        // Higher frequency = more variation over same time period
        let low_freq = state.wiggle(1.0, 1.0, 10.0, 1, 0.5);
        let high_freq = state.wiggle(1.0, 10.0, 10.0, 1, 0.5);

        // Just verify they produce different results
        assert_ne!(low_freq, high_freq);
    }

    #[test]
    fn test_wiggle_vector() {
        let state = WiggleState::with_seed(12345);
        let base = PropertyValue::Vector(vec![100.0, 200.0]);

        let wiggled = wiggle_property_value(&base, 1.0, 5.0, 10.0, 1, 0.5);

        match wiggled {
            PropertyValue::Vector(v) => {
                assert_eq!(v.len(), 2);
                // Both dimensions should be within [base-10, base+10]
                assert!(v[0] >= 90.0 && v[0] <= 110.0);
                assert!(v[1] >= 190.0 && v[1] <= 210.0);
            }
            _ => panic!("Expected vector result"),
        }
    }

    #[test]
    fn test_wiggle_scalar() {
        let base = PropertyValue::Scalar(100.0);
        let wiggled = wiggle_property_value(&base, 1.0, 5.0, 10.0, 1, 0.5);

        match wiggled {
            PropertyValue::Scalar(v) => {
                assert!(v >= 90.0 && v <= 110.0);
            }
            _ => panic!("Expected scalar result"),
        }
    }
}
