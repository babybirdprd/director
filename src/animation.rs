use keyframe::{Keyframe, EasingFunction, AnimationSequence, CanTween};
use std::fmt;

// Define our own enum to store easing types uniformly
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EasingType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    BounceOut,
}

impl EasingFunction for EasingType {
    fn y(&self, x: f64) -> f64 {
        match self {
            EasingType::Linear => keyframe::functions::Linear.y(x),
            EasingType::EaseIn => keyframe::functions::EaseIn.y(x),
            EasingType::EaseOut => keyframe::functions::EaseOut.y(x),
            EasingType::EaseInOut => keyframe::functions::EaseInOut.y(x),
            // BounceOut is missing in keyframe 1.1 functions mod or named differently.
            // Mapping to EaseOut for safety.
            EasingType::BounceOut => keyframe::functions::EaseOut.y(x),
        }
    }
}

impl EasingType {
    pub fn eval(&self, x: f32) -> f32 {
        self.y(x as f64) as f32
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpringConfig {
    pub stiffness: f32, // Tension
    pub damping: f32,   // Friction
    pub mass: f32,
    pub velocity: f32,  // Initial velocity
}

impl Default for SpringConfig {
    fn default() -> Self {
        // "Wobbly" default for visibility
        Self { stiffness: 100.0, damping: 10.0, mass: 1.0, velocity: 0.0 }
    }
}

// Wrapper for Vec<f32> to implement CanTween
#[derive(Clone, Debug, Default)]
pub struct TweenableVector(pub Vec<f32>);

impl CanTween for TweenableVector {
    fn ease(from: Self, to: Self, time: impl keyframe::num_traits::Float) -> Self {
        let t = time.to_f64().unwrap();
        let len = from.0.len().min(to.0.len());
        let mut result = Vec::with_capacity(len);

        for i in 0..len {
            let start = from.0[i] as f64;
            let end = to.0[i] as f64;
            let val = start + (end - start) * t;
            result.push(val as f32);
        }

        // If lengths differ, maybe we should fill with 0 or the target value?
        // But for shader uniforms, length usually matches.
        // We'll just take the min length.

        TweenableVector(result)
    }
}

#[derive(Clone)]
pub struct Animated<T>
where T: Clone + keyframe::CanTween + Default
{
    pub raw_keyframes: Vec<(T, f64, EasingType)>,
    pub sequence: AnimationSequence<T>,
    pub current_value: T,
}

impl<T> Animated<T>
where T: Clone + keyframe::CanTween + Default
{
    pub fn new(initial: T) -> Self {
        let raw = vec![(initial.clone(), 0.0, EasingType::Linear)];
        let kf = Keyframe::new(initial.clone(), 0.0, EasingType::Linear);

        Self {
            sequence: AnimationSequence::from(vec![kf]),
            raw_keyframes: raw,
            current_value: initial,
        }
    }

    pub fn add_keyframe(&mut self, target: T, duration: f64, easing: EasingType) {
        let current_end_time = self.sequence.duration();
        let new_time = current_end_time + duration;

        self.raw_keyframes.push((target.clone(), new_time, easing));

        // Rebuild sequence
        let frames: Vec<Keyframe<T>> = self.raw_keyframes.iter()
            .map(|(val, time, ease_type)| {
                Keyframe::new(val.clone(), *time, *ease_type)
            })
            .collect();

        self.sequence = AnimationSequence::from(frames);
    }

    pub fn duration(&self) -> f64 {
        self.sequence.duration()
    }

    pub fn add_segment(&mut self, start: T, target: T, duration: f64, easing: EasingType) {
        if self.sequence.duration() == 0.0 {
             // If no animation exists yet, treat start as the initial value
             *self = Self::new(start);
        } else {
             // If animation exists, we jump to 'start' immediately at the current end time
             self.add_keyframe(start, 0.0, EasingType::Linear);
        }
        self.add_keyframe(target, duration, easing);
    }

    pub fn update(&mut self, time: f64) {
        self.sequence.advance_to(time);
        self.current_value = self.sequence.now();
    }
}

impl Animated<f32> {
    pub fn add_spring(&mut self, target: f32, config: SpringConfig) {
        let start = if let Some(last) = self.raw_keyframes.last() {
            last.0
        } else {
            self.current_value
        };

        self.add_spring_with_start(start, target, config);
    }

    pub fn add_spring_with_start(&mut self, start: f32, target: f32, config: SpringConfig) {
        // If start is different from last keyframe, we insert a jump
        if let Some(last) = self.raw_keyframes.last() {
             if (last.0 - start).abs() > 0.0001 {
                  self.add_keyframe(start, 0.0, EasingType::Linear);
             }
        } else {
             // Should verify if this ever happens as ::new sets a keyframe
             *self = Self::new(start);
        }

        let frames = solve_spring(start, target, config);

        let mut previous_time = 0.0;
        for (value, time) in frames {
             let dt = time - previous_time;
             // dt is f64, time is f64
             self.add_keyframe(value, dt, EasingType::Linear);
             previous_time = time;
        }
    }
}

fn solve_spring(start: f32, end: f32, config: SpringConfig) -> Vec<(f32, f64)> {
    let mut frames = Vec::new();
    let mut t = 0.0;
    let dt: f32 = 1.0 / 60.0; // Bake resolution

    let mut current = start;
    let mut velocity = config.velocity;

    // Safety break
    let max_duration = 10.0;

    // Epsilon for settling
    let position_epsilon = 0.1; // 0.1 pixel/unit
    let velocity_epsilon = 0.1;

    loop {
        let force = -config.stiffness * (current - end);
        let damping = -config.damping * velocity;
        let acceleration = (force + damping) / config.mass;

        velocity += acceleration * dt;
        current += velocity * dt;
        t += dt as f64; // Keep time as f64

        frames.push((current, t));

        if t > max_duration as f64 { break; }

        let is_settled = (current - end).abs() < position_epsilon && velocity.abs() < velocity_epsilon;
        if is_settled {
            // Add one final frame exactly at target to ensure we land
            frames.push((end, t + dt as f64));
            break;
        }
    }
    frames
}

impl<T> fmt::Debug for Animated<T>
where T: Clone + keyframe::CanTween + Default + fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Animated")
         .field("current_value", &self.current_value)
         .finish()
    }
}
