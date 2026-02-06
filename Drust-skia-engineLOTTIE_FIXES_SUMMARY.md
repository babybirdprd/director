
### 10. ✅ Auto-Orient Support
**Files:**
- `crates/lottie-core/src/lib.rs:1757-1762` - Check for auto-orient flag and calculate rotation
- `crates/lottie-core/src/lib.rs:1817-1880` - Implemented `calculate_auto_orient_rotation` method

**Problem:** The `ao` (auto-orient) layer property was parsed but not implemented. When enabled, layers should automatically rotate to follow their position path, but this was completely missing.

**Fix:** Implemented auto-orient rotation calculation:
- When `layer.ao == Some(1)`, calculate rotation from position path tangent
- Sample position at current frame and small delta ahead (0.1 frames)
- Calculate direction vector and convert to rotation angle using `atan2`
- Add calculated rotation to the layer's Z rotation
- Supports both unified and split position properties
- Handles both expression-enabled and non-expression builds

**Algorithm:**
```rust
let direction = pos_next - pos_current;
let angle = direction.y.atan2(direction.x);
let rotation = -angle.to_degrees().to_radians();
```

**Spec Reference:** https://lottie.github.io/lottie-spec/1.0.1/specs/layers/#visual-layer

**Impact:** Layers with motion paths now correctly rotate to follow the path direction, fixing positioning/layout issues for animated objects.

### 11. ✅ Shape Rendering Order Fix
**File:**
- `crates/lottie-core/src/lib.rs:2047` - Changed shape iteration to reverse order

**Problem:** According to the Lottie spec, shapes should be rendered in reverse order (bottom->top), with shapes at the beginning of the array rendered on top. The code was processing shapes in forward order, which could cause incorrect visual layering in complex shape groups.

**Fix:** Changed the main shape processing loop from:
```rust
for item in shapes {
```
to:
```rust
for item in shapes.iter().rev() {
```

**Spec Reference:** https://lottie.github.io/lottie-spec/1.0.1/specs/shapes/
> "Shapes are rendered in reverse order, bottom->top. Shapes at the beginning of the array are rendered on top."

**Impact:** Ensures correct z-ordering when multiple shapes overlap within a shape layer, fixing visual layering issues.
