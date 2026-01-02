# Director Engine: Code-Based Capabilities Audit

**Date**: 2026-01-01  
**Source of Truth**: Codebase analysis (NOT docs/issues)

---

## Node Types (`node/`)

| Node | File | Capabilities |
|------|------|--------------|
| **BoxNode** | `box_node.rs` | Container, flexbox, bg_color, border, shadow, overflow, border_radius |
| **TextNode** | `text.rs` | SkParagraph text, multi-span, per-glyph animators |
| **ImageNode** | `image_node.rs` | Static images, object_fit (cover/contain/fill) |
| **VideoNode** | `video_node.rs` | Video playback, object_fit, audio extraction |
| **VectorNode** | `vector.rs` | SVG rendering |
| **LottieNode** | `lottie.rs` | Lottie JSON animations, speed, looping |
| **EffectNode** | `effect.rs` | Visual effects wrapper (applies to children) |
| **CompositionNode** | `composition.rs` | Nested timeline with internal Director |

---

## Animation System (`animation.rs`)

### Easing Types (13)
```
Linear, EaseIn, EaseOut, EaseInOut,
BounceIn, BounceOut, BounceInOut,
ElasticIn, ElasticOut, ElasticInOut,
BackIn, BackOut, BackInOut
```

### Animation Types
- **Keyframe** — `Animated<T>` with add_keyframe(), add_segment()
- **Spring** — `SpringConfig { stiffness, damping, mass, initial_velocity }`
- **Text Animators** — per-glyph opacity, offset_y, offset_x, scale, rotation

---

## Effects (`node/effect.rs`)

| Effect | Properties |
|--------|------------|
| **Blur** | sigma (radius) |
| **DropShadow** | blur, offset_x, offset_y, color |
| **ColorMatrix** | 4x5 matrix (20 floats) |
| **DirectionalBlur** | strength, angle, samples |
| **FilmGrain** | intensity, size |

*Presets: Grayscale, Sepia*

---

## Transitions (`systems/transitions.rs`)

| Type | Description |
|------|-------------|
| Fade | Alpha crossfade |
| SlideLeft | Scene slides left |
| SlideRight | Scene slides right |
| WipeLeft | Hard edge wipe left |
| WipeRight | Hard edge wipe right |
| CircleOpen | Circular reveal from center |

---

## Audio (`audio.rs`)

| Component | Capabilities |
|-----------|--------------|
| **AudioMixer** | Multi-track mixing, stereo output |
| **AudioTrack** | Volume, start_time, looping |
| **AudioAnalyzer** | FFT-based spectrum, bass/mids/highs bands |
| **Audio Reactivity** | bind_audio → map bands to properties |

---

## Layout (`systems/layout.rs`)

- **Taffy** integration (CSS Flexbox + Grid)
- Supports: flex_direction, justify_content, align_items, flex_wrap, gap
- Grid: grid_template_columns/rows, grid_row, grid_column
- Positioning: absolute/relative, top/left/right/bottom insets
- Sizing: width, height, min/max, aspect_ratio

---

## Rhai Scripting API

### Lifecycle
| Function | Description |
|----------|-------------|
| `new_director(w, h, fps)` | Create Director |
| `add_scene(movie, duration)` | Add scene |
| `rand_float(min, max)` | Random number |

### Nodes
| Function | Description |
|----------|-------------|
| `add_box(parent, props)` | Create container |
| `add_text(parent, props)` | Create text |
| `add_image(parent, path)` | Load image |
| `add_video(parent, path)` | Load video |
| `add_svg(parent, path)` | Load SVG |
| `add_lottie(parent, path, props)` | Load Lottie |
| `add_composition(parent, props)` | Nested timeline |
| `destroy(node)` | Remove node |

### Animation
| Function | Description |
|----------|-------------|
| `animate(node, prop, end, duration, easing)` | Keyframe animation |
| `animate(node, prop, end, config)` | Spring animation |
| `path_animate(node, svg_path, ...)` | Follow SVG path |
| `add_text_animator(...)` | Per-glyph animation |

### Effects
| Function | Description |
|----------|-------------|
| `apply_effect(node, "blur", sigma)` | Gaussian blur |
| `apply_effect(node, "grayscale")` | Grayscale |
| `apply_effect(node, "sepia")` | Sepia tone |
| `apply_effect(node, "directional_blur", #{...})` | Motion blur |
| `apply_effect(node, "grain", #{...})` | Film grain |
| `apply_effect(node, "shader", #{...})` | Custom SkSL |

### Audio
| Function | Description |
|----------|-------------|
| `add_audio(movie/scene, path)` | Add audio track |
| `bass(track, time)` | Get bass energy |
| `mids(track, time)` | Get mids energy |
| `highs(track, time)` | Get highs energy |
| `bind_audio(node, track_id, band, prop)` | Audio reactivity |

### Properties
| Function | Description |
|----------|-------------|
| `set_style(node, {...})` | Update layout style |
| `set_pivot(node, x, y)` | Set rotation pivot |
| `set_z_index(node, z)` | Layer ordering |
| `set_mask(node, mask_node)` | Apply mask |
| `set_blend_mode(node, mode)` | Blend mode |

---

## Export (`export/video.rs`)

- FFmpeg encoding via video-rs
- Configurable resolution, FPS
- Audio muxing from AudioMixer
