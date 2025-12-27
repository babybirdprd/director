# SDK Architecture

> **Status:** Draft  
> **Author:** babybirdprd  
> **Created:** 2024-12-27  
> **Updated:** 2024-12-27  
> **Epic:** gr-jtweqo

## Summary

The SDK layer provides a token-efficient, capability-aware abstraction over `director-core`. It unifies three existing crates (`director-core`, `director-schema`, `director-pipeline`) into a cohesive API surface for humans, AI agents, and external applications.

---

## The Abstraction Stack

```
┌─────────────────────────────────────────────────────────────┐
│  L3: Recipes                                                │
│  Full video orchestration scripts                           │
│  ("The Faceless Historian", "The Podcast Clipper")          │
├─────────────────────────────────────────────────────────────┤
│  L2: Smart Components                                       │
│  Self-contained compositions with internal logic            │
│  ("Smart Lower Third", "Auto-B-Roll Frame")                 │
├─────────────────────────────────────────────────────────────┤
│  L1: Macros (Functional Templates)                          │
│  Token-efficient animation/effect helpers                   │
│  (fly_in_up, typewriter_effect, pulse_on_beat)              │
├─────────────────────────────────────────────────────────────┤
│  L0: director-core + director-schema + director-pipeline    │
│  Raw engine, JSON schema, schema→Director conversion        │
└─────────────────────────────────────────────────────────────┘
```

---

## Existing Infrastructure

| Crate | Purpose | Status |
|-------|---------|--------|
| `director-core` | Rendering, layout, animation, scripting | ✅ Mature |
| `director-schema` | JSON-serializable `MovieRequest` structure | ✅ Exists |
| `director-pipeline` | `load_movie(MovieRequest) → Director` | ✅ Exists |

**The SDK's role:** Wrap these into a unified API that exposes L1-L3 abstractions.

---

## Design Principles

### 1. Token Efficiency
```rhai
// ❌ L0 (verbose) — 127 tokens
let box = scene.add_box(#{
    width: "100%", height: "100%",
    justify_content: "center", align_items: "center",
    bg_color: "#1a1a2e"
});
let title = box.add_text(#{
    content: "Hello", size: 72.0, color: "#FFF", weight: "bold"
});
title.animate("scale", 0.8, 1.0, 0.5, "bounce_out");
title.animate("opacity", 0.0, 1.0, 0.3, "ease_out");

// ✅ L1 (SDK) — 31 tokens
let title = sdk.centered_text("Hello", 72);
title.pop_in(0.5);
```

### 2. Capability-Aware
```rhai
fn remove_background(video_node) {
    if engine.capabilities.has_gpu && engine.capabilities.sam3_loaded {
        // High-End: SAM 3 segmentation
        let mask = video.track_object("person");
        video_node.set_mask(mask);
    } else {
        // Fallback: Simple center crop
        video_node.set_mask(create_circle_mask());
    }
}
```

### 3. Multi-Frontend Compatible

| Frontend | Input Format | SDK Role |
|----------|--------------|----------|
| Rhai scripts | Native SDK calls | Direct API |
| JSON/REST | `MovieRequest` | `director-schema` → `director-pipeline` |
| AI Agents | Natural language → structured | Generate L1/L2 calls |
| GUI (Dioxus) | Visual nodes | Generate `MovieRequest` |

---

## Proposed Structure

```
crates/
├── director-core/        # L0: Engine (unchanged)
├── director-schema/      # L0: JSON types (expand)
├── director-pipeline/    # L0: Schema→Director (expand)
└── director-sdk/         # NEW: L1-L3 abstractions
    ├── src/
    │   ├── lib.rs
    │   ├── macros/       # L1: Animation helpers
    │   │   ├── mod.rs
    │   │   ├── animations.rs  # fly_in, pop_in, fade_in
    │   │   └── effects.rs     # remove_background, add_grain
    │   ├── components/   # L2: Smart Components
    │   │   ├── mod.rs
    │   │   └── lower_third.rs
    │   └── recipes/      # L3: Orchestration helpers
    │       └── mod.rs
    └── assets/
        └── scripts/std/  # Rhai standard library
```

---

## Implementation Phases

### Phase 1: Extract scripting.rs
Move Rhai bindings from `director-core` to `director-sdk`.

### Phase 2: L1 Macros
Implement core animation presets:
- `pop_in(duration)` — scale + opacity bounce
- `fly_in_up/down/left/right(duration)`
- `typewriter_effect(speed)`
- `pulse_on_beat(bpm)`

### Phase 3: L2 Smart Components
Implement reusable compositions:
- Smart Lower Third (auto-resize, SAM-aware)
- Quote Card (styled container + text)
- Progress Bar (animated fill)

### Phase 4: Capability System
Expose `engine.capabilities` to Rhai:
- `has_gpu`
- `sam3_loaded`
- `memory_available`

---

## Open Questions

- [ ] Should `director-sdk` depend on `director-pipeline` or vice versa?
- [ ] Where do `.dirproj` component files live? (assets/ vs separate?)
- [ ] How are community templates distributed? (asset store API?)

---

## Related Documents

- [TEMPLATE_STRATEGY.md](TEMPLATE_STRATEGY.md) — Original 3-layer vision
- [RHAI_API_SPEC.md](RHAI_API_SPEC.md) — Proposed standard library
