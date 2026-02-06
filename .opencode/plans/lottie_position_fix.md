# Lottie Rendering Position Fix - Implementation Plan

## Current Status

**Visual Test Results:**
- ❌ **Frame 30**: Yellow face appears in top-left corner (SHOULD BE CENTERED)
- ✅ **Frame 60**: Red heart alone appears correctly positioned

**Root Cause Hypothesis:**
The precomposition layer transform is not being applied correctly. The layer has:
- Position: (400, 400) - center of 800x800 canvas
- Anchor: (400, 400) - center of precomp
- Scale: 82%

But it's rendering at (0, 0) instead of center.

---

## Phase 1: Precomposition Layer Audit

### 1.1 Verify Precomposition Layer Schema Compliance

From `lottie-schema.json`, a Precomposition Layer (type 0) extends:
- `visual-layer` → `layer` → `visual-object`

**Required properties we have:**
- ✅ `ty` = 0 (layer type)
- ✅ `refId` (precomp asset reference)
- ✅ `ks` (transform) 
- ✅ `ip`, `op` (in/out points)

**Optional properties to verify:**
- ✅ `w`, `h` - clipping rect dimensions (we have these)
- ✅ `st` - start time (default: 0, we fixed this)
- ✅ `tm` - time remapping (we have this)
- ⚠️ `ct` - collapse transform (default: 0, NOT IMPLEMENTED)
- ⚠️ `cp` - collapse transform deprecated (NOT IMPLEMENTED)

### 1.2 Critical Missing Properties Check

**From visual-layer in schema:**
```json
{
  "ct": {
    "title": "Collapse Transform",
    "description": "Marks that transforms should be applied before masks",
    "$ref": "#/$defs/values/int-boolean",
    "default": 0
  }
}
```

**Status:** We don't have `ct` (Collapse Transform) property in our Layer struct!

**Impact:** According to Lottie spec, when `ct: 1`, transforms should be applied BEFORE masks. If a precomp has this set, we might be applying transforms in wrong order.

### 1.3 Coordinate System Verification

**Question:** Are we handling coordinate systems correctly?
- Lottie: Origin at top-left, Y increases downward
- Skia: Origin at top-left, Y increases downward
- Result: Should match! ✅

**Potential Issue:** Precomp anchor point vs position calculation

---

## Phase 2: Transform Application Analysis

### 2.1 Current Transform Flow

**Code location:** `crates/lottie-core/src/lib.rs:963-993`

```rust
let mut sub_builder = SceneGraphBuilder::new(self.asset, local_frame);
let root = sub_builder.build_composition(layers, &sub_layer_map, evaluator);

// Apply layer transform to precomp result
let mut precomp_node = root;
precomp_node.transform = self.resolve_transform(layer, layer_map, evaluator);
```

**Potential Bug:** The `root` returned from `build_composition` may already have a transform from the precomp's internal structure, and we're overwriting it completely!

### 2.2 Transform Resolution Check

**Function:** `resolve_transform()` (lines 1655-1670)
- Gets local transform from `get_layer_transform()`
- Multiplies by parent transform if parent exists
- Returns: `parent_world × local`

**Function:** `get_layer_transform()` (lines 1672+)
- Builds matrix: `T × R × Skew × S × (-A)`
- Applied right-to-left (correct)
- Frame calculation: `t_frame = self.frame - layer.st`

**Verdict:** Transform math appears correct.

### 2.3 Precomp Internal Root Transform Issue

**Hypothesis:** When `build_composition()` is called for a precomp, it creates a root node. If that root node has children positioned relative to (0,0), and we then apply the layer's transform (which includes position at 400,400), it should work. But if the root node already has an offset...

**Action:** Check if `build_composition()` applies any default transform to the root node.

---

## Phase 3: Comparison Test Implementation

### 3.1 Create Debug Rendering Test

Create a test that renders both:
1. Our renderer output (PNG)
2. Expected transform values printed to console

**Test file:** `crates/lottie-skia/tests/precomp_position_debug.rs`

### 3.2 Add Transform Debugging

Add debug logging to trace:
- Precomp layer position, anchor, scale values
- Calculated transform matrix
- Final node transform after build

---

## Phase 4: Systematic Fixes

### 4.1 Fix 1: Add Missing `ct` (Collapse Transform) Property

**File:** `crates/lottie-data/src/model.rs`

Add to Layer struct:
```rust
#[serde(default, rename = "ct")]
pub collapse_transform: Option<u8>, // Collapse transform: 0=off, 1=on
```

### 4.2 Fix 2: Verify Precomp Root Transform Handling

**File:** `crates/lottie-core/src/lib.rs`

Current code overwrites `precomp_node.transform` completely. This might be wrong if the root already has a transform.

**Investigate:** Should we COMBINE transforms instead of overwrite?
```rust
// Instead of:
precomp_node.transform = layer_transform;

// Maybe:
precomp_node.transform = layer_transform * precomp_node.transform;
```

### 4.3 Fix 3: Check Shape Group Transforms

**From schema:** Shape groups have their own `tr` (transform) property.

**Question:** Are shape group transforms being applied correctly within precomps?

**File to check:** `crates/lottie-core/src/lib.rs` in shape processing

### 4.4 Fix 4: Verify Time Remapping Impact

**Code:** Lines 942-954

```rust
let local_frame = if let Some(tm_prop) = &layer.tm {
    tm_sec * self.asset._frame_rate
} else {
    self.frame - layer.st
};
```

**Question:** Is this calculating the correct frame for the precomp timeline?

**Test:** At frame 30, what local_frame are we computing?

---

## Phase 5: Schema Property Audit

### 5.1 Create Automated Schema Comparison Script

**Script:** `scripts/audit_lottie_spec.py`

Purpose: Extract all properties from `lottie-schema.json` and compare with our Rust model.

**Key areas to audit:**
1. All layer properties (visual-layer, precomposition-layer, shape-layer, etc.)
2. Transform properties
3. Shape properties
4. Mask properties

### 5.2 High Priority Missing Properties

Based on visual-layer schema section (lines 2800-2900):

| Property | Status | Impact |
|----------|--------|--------|
| `ct` | ❌ Missing | Collapse transform - HIGH |
| `cp` | ❌ Missing | Deprecated collapse - LOW |
| `td` | ❌ Missing | Matte target - MEDIUM |
| `tp` | ❌ Missing | Matte parent - MEDIUM |
| `sy` | ✅ Have | Layer styles |
| `ef` | ✅ Have | Effects |
| `mb` | ❌ Missing | Motion blur - LOW |
| `tg` | ❌ Missing | XML tag - LOW |
| `cl` | ❌ Missing | CSS class - LOW |
| `ln` | ❌ Missing | XML ID - LOW |

---

## Phase 6: Implementation Sequence

### Immediate (Fix Positioning)
1. ✅ **Add debug output** to trace precomp transform
2. 🔧 **Verify precomp root node handling** - check if we're overwriting vs combining transforms
3. 🔧 **Add `ct` property** to Layer struct
4. 🔧 **Fix transform application order** if `ct` is enabled

### Short-term (Schema Compliance)
5. 🔧 **Implement matte properties** (`tt`, `tp`, `td`)
6. 🔧 **Add motion blur support** (`mb`)
7. 🔧 **Implement remaining visual-layer properties**

### Testing
8. 🔧 **Create comprehensive visual tests** for precomps
9. 🔧 **Add parity tests** comparing with reference renders
10. 🔧 **Run full test suite** after each fix

---

## Questions to Resolve

1. **Transform Combination:** Should we multiply precomp root transform with layer transform instead of overwriting?

2. **Coordinate Origin:** Are precomp layers positioned relative to their own (0,0) or relative to anchor point?

3. **Time Remapping:** Is our frame calculation for precomps correct when `tm` is not present?

4. **Collapse Transform:** Do any of our test files use `ct: 1`? If so, we need to handle transform-before-mask logic.

---

## Success Criteria

- ✅ Frame 30: Yellow face centered in canvas (not top-left)
- ✅ Frame 60: Heart still correctly positioned  
- ✅ All existing tests pass
- ✅ New precomp position test passes
- ✅ No regressions in other Lottie files

---

## Next Actions

**Ready to implement:**
1. Add debug tracing to precomp rendering
2. Test transform combination hypothesis
3. Add `ct` property to Layer struct
4. Run tests and verify frame 30 positioning
