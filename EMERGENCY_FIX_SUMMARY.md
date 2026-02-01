# Emergency Frame Calculation Fix - Summary

## Problem
The Lottie animation was rendering only ~3 static frames instead of 91 fluid animation frames because:
1. The `frame` property was wrapped in `Animated<f32>` with a single keyframe at value 0.0
2. Calling `self.frame.update(time)` kept the value at 0.0 for all times
3. The frame calculation from time was correct, but the `Animated` wrapper was interfering

## Fix Applied
**File:** `crates/director-core/src/node/lottie.rs`

### Changes Made:

1. **Line 70:** Changed field type
   ```rust
   // BEFORE:
   pub frame: Animated<f32>,
   
   // AFTER:
   pub current_frame: f32,
   ```

2. **Line 137:** Changed initialization
   ```rust
   // BEFORE:
   frame: Animated::new(0.0),
   
   // AFTER:
   current_frame: 0.0,
   ```

3. **Lines 249-285:** Fixed update() method
   ```rust
   // BEFORE (BROKEN):
   fn update(&mut self, time: f64) -> bool {
       self.opacity.update(time);
       self.frame.update(time);  // ← Keeps value at 0.0!
       
       let mut player = self.player.lock().unwrap();
       
       if self.frame.raw_keyframes.len() > 1 {  // Never true
           player.current_frame = self.frame.current_value;
       } else {
           // Calculate from time... (correct but rarely reached)
       }
   }
   
   // AFTER (FIXED):
   fn update(&mut self, time: f64) -> bool {
       self.opacity.update(time);
       
       let mut player = self.player.lock().unwrap();
       
       // Calculate frame directly from Director time
       let fps = self.asset._frame_rate;
       let start_frame = self.asset.model.ip;
       let end_frame = self.asset.model.op;
       let total_frames = end_frame - start_frame;
       
       let current_raw = (time * fps as f64 * self.speed as f64) + start_frame as f64;
       
       let calculated_frame = if self.loop_anim && total_frames > 0.0 {
           ((current_raw - start_frame as f64).rem_euclid(total_frames as f64)) 
               + start_frame as f64
       } else {
           current_raw.clamp(start_frame as f64, end_frame as f64)
       };
       
       player.current_frame = calculated_frame as f32;
       self.current_frame = calculated_frame as f32;
       
       tracing::debug!("Lottie frame: time={} → frame={}", time, calculated_frame);
       
       true
   }
   ```

4. **Line 89:** Fixed Debug implementation
   ```rust
   // BEFORE:
   .field("frame", &self.frame)
   
   // AFTER:
   .field("current_frame", &self.current_frame)
   ```

5. **Line 109:** Fixed Clone implementation
   ```rust
   // BEFORE:
   frame: self.frame.clone(),
   
   // AFTER:
   current_frame: self.current_frame,
   ```

6. **Lines 390-395:** Updated animate_property()
   ```rust
   // BEFORE:
   } else if property == "frame" {
       self.frame.add_segment(start, target, duration, ease);
   }
   
   // AFTER:
   } else if property == "frame" {
       // Frame animation via segments not yet supported
       tracing::warn!("Frame animation via add_segment not supported. Use seek() instead.");
   }
   ```

## Key Improvements

1. **Removed Animated wrapper interference** - Frame is now a simple f32 calculated directly from time
2. **Added debug logging** - Can trace frame calculation: `time={} → frame={}`
3. **Simplified logic** - Single code path, no confusing if/else branches
4. **Fixed rem_euclid** - Proper modulo for negative numbers (though unlikely with positive time)

## Testing

### New Test Files Created:
1. `crates/lottie-skia/tests/animation_test.rs` - Renders multiple frames, detects frozen animation
2. `examples/tests/validate_animation.py` - Quick validation script

### To Test:
```bash
# Build the fixed version
cargo build --release -p director-cli

# Render the test video
cargo run --release -p director-cli --bin director-engine -- \
  examples/tests/lottie_heart_eyes_export.rhai \
  examples/tests/lottie_heart_eyes_output_v4.mp4

# Validate it's not frozen
python examples/tests/validate_animation.py
```

## Expected Result
With this fix, the animation should now:
1. Render all 91 frames with fluid motion
2. Show the heart eyes burst animation starting at frame 76
3. Be visually comparable to the official LottieLab export

## Next Steps

1. Build and test the fix
2. If working, proceed to Phase 2: Comprehensive test suite
3. If issues remain, check layer timing (ip vs st handling)
4. Download reference Lottie files from LottieFiles for broader testing

## Notes

- This is an emergency fix to get basic animation working
- Time remapping via keyframes (animating the frame property) is temporarily disabled
- Can be re-added later if needed via proper implementation
- The fix prioritizes correctness over features (95% of Lottie files don't need time remapping)
