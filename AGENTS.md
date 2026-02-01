---
trigger: always_on
---

# Agent Instructions for `director-engine`

> [!IMPORTANT]
> **Agent Protocol - MANDATORY**
>
> 1. **Read First**: Before editing any file, check if it has a `//!` doc block at the top. Read it to understand the module's responsibilities.
> 2. **Find via Map**: Use the **Codebase Map** below to locate the correct file for your task. Do not guess file locations.
> 3. **Update In-Code Docs**: When adding/removing/renaming major functions or changing a module's responsibilities, **you MUST update** the `//!` doc block in that file.
> 4. **Update Codebase Map**: When adding a new file, deleting a file, or shifting responsibilities between modules, **you MUST update** the Codebase Map tables below.

---

## Codebase Map

Use this map to locate the correct file for a specific task.

### Core Systems (`crates/director-core/src`)
| Responsibility | Primary File | Key Structs |
| :--- | :--- | :--- |
| **Orchestration** | `director.rs` | `Director`, `TimelineItem` |
| **Scene Graph** | `scene.rs` | `SceneGraph` (Arena), `SceneNode` |
| **Element Trait** | `element.rs` | `Element` trait (all nodes implement) |
| **Shared Types** | `types.rs` | `Color`, `Transform`, `NodeId` |
| **Design System** | `tokens.rs` | `DesignSystem`, spacing, safe areas |
| **Rendering** | `systems/renderer.rs` | `render_recursive`, `render_frame` |
| **Transitions** | `systems/transitions.rs` | `TransitionType`, `Transition`, shaders |
| **Video Export** | `export/video.rs` | `render_export`, FFmpeg encoding |
| **Layout** | `systems/layout.rs` | `LayoutEngine`, Taffy integration |
| **Assets** | `systems/assets.rs` | `AssetManager`, fonts, shaders |
| **Scripting** | `scripting/mod.rs` | Rhai engine, `register_rhai_api` |
| **Scripting Types** | `scripting/types.rs` | `MovieHandle`, `SceneHandle`, `NodeHandle` |
| **Scripting Utils** | `scripting/utils.rs` | Parsers (`parse_easing`, `parse_layout_style`) |
| **Scripting API** | `scripting/api/*.rs` | Lifecycle, nodes, animation, audio, effects, properties |
| **Animation** | `animation.rs` | `Animated`, `EasingType`, springs |
| **Audio** | `audio.rs` | `AudioMixer`, `AudioTrack`, `AudioAnalyzer` |
| **Video Encoding** | `video_wrapper.rs` | FFMPEG wrapper |

### Node Types (`crates/director-core/src/node`)
| Node | File | Use Case |
| :--- | :--- | :--- |
| **Box** | `box_node.rs` | Container with flexbox, borders, backgrounds |
| **Text** | `text.rs` | Rich text rendering (SkParagraph) |
| **Text Animator** | `text_animator.rs` | Kinetic typography and per-character animations |
| **Image**| `image_node.rs` | Static image display |
| **Video** | `video_node.rs` | Video playback |
| **Lottie** | `lottie.rs` | Lottie animation embedding |
| **Vector** | `vector.rs` | SVG-like vector graphics |
| **Effect** | `effect.rs` | Visual effects/shaders |
| **Composition** | `composition.rs` | Nested scene composition |

### Supporting Crates (`crates/`)
| Responsibility | Primary File | Notes |
| :--- | :--- | :--- |
| **DSL Types** | `director-schema/src/lib.rs` | `NodeKind`, `StyleMap`, JSON serialization |
| **Asset Pipeline** | `director-pipeline/src/lib.rs` | `build_node_recursive`, DSL to SceneGraph |
| **Developer Tools** | `director-developer/src/main.rs` | Live reflection, spec generation, and monitoring |
| **Live Preview** | `director-view/src/main.rs` | Real-time Skia window for script previewing |
| **CLI Renderer** | `director-cli/src/main.rs` | Command-line interface for video production |

### Lottie System (`crates/lottie-*`)
| Responsibility | Primary File | Notes |
| :--- | :--- | :--- |
| **Lottie Parsing** | `lottie-core/src/lib.rs` | JSON model, keyframe evaluation |
| **Lottie Data** | `lottie-data/src/model.rs` | Raw Lottie JSON types |
| **Lottie Rendering** | `lottie-skia/src/lib.rs` | Skia path drawing |

---

## Project Overview

**Director Engine** is a programmatic video rendering engine in Rust. It combines:
- **Taffy** — CSS Flexbox layout
- **Skia** — 2D rasterization
- **Rhai** — Scripting language
- **FFmpeg** — Video encoding

---

## Key Concepts

### Director & Timeline
- `Director` manages a `Vec<TimelineItem>` (scenes)
- Each scene has a root `NodeId` and time range
- Transitions create overlap between scenes

### Scene Graph
- **Arena storage**: `Vec<Option<SceneNode>>` in `SceneGraph`
- **NodeId**: `usize` index
- **Hierarchy**: `children: Vec<NodeId>`, `parent: Option<NodeId>`
- **Element trait**: All nodes implement `Element` (render, update, measure)

### Layout (Taffy)
- Flexbox layout computed every frame
- Transforms (scale, rotation) are visual-only, don't affect layout
- `needs_measure()` nodes report intrinsic size to Taffy

### Rendering Pipeline
1. `Director::update(time)` — Update animations
2. `LayoutEngine::compute_layout()` — Taffy pass
3. `Director::run_post_layout()` — Post-layout hooks
4. `render_recursive()` — Skia drawing

---

## Common Tasks

### Add a Rhai API
1. Identify the appropriate sub-module in `crates/director-core/src/scripting/api/`:
   - `lifecycle.rs` - Director/scene management
   - `nodes.rs` - Node creation functions
   - `animation.rs` - Animation functions
   - `audio.rs` - Audio functions
   - `effects.rs` - Visual effects
   - `properties.rs` - Node property setters
2. Add `engine.register_fn("name", |...| { ... })` in the appropriate module
3. If adding a new utility parser, add it to `scripting/utils.rs`
4. Update `docs/SCRIPTING.md`

### Add a Node Type
1. Create `crates/director-core/src/node/my_node.rs`
2. Implement `Element` trait
3. Add to `node/mod.rs`
4. Add Rhai binding in `scripting/api/nodes.rs`
5. **Update the Codebase Map** (Node Types table)

### Run Tests
```bash
cargo test -p director-core           # All tests
cargo test -p director-core --test examples  # Example validation
cargo test -p director-core layout    # Specific test
```

### Update Snapshots
```bash
$env:UPDATE_SNAPSHOTS="1"; cargo test -p director-core
```

---

## Constraints

### Threading
- `AssetManager` is `!Send` (shader cache)
- Use `Arc<dyn AssetLoader>` for thread-safe asset loading
- `Director` is wrapped in `Arc<Mutex<>>` for Rhai handles

### Text Rendering
- Uses `skia_safe::textlayout::Paragraph` (SkParagraph)
- NOT cosmic-text
- Text animators enabled via `add_text_animator`

### Performance
- Avoid logging in per-pixel or per-frame loops
- Use `tracing::debug!` for development-only logs
- Large assets not in git — use `assets/` folder

---

## Logging

Uses `tracing` ecosystem:
```rust
tracing::info!(width, height, "Director initialized");
tracing::warn!("Feature disabled: {}", name);
tracing::debug!(frame, elapsed_ms, "Frame rendered");
```

For tests:
```rust
let _ = tracing_subscriber::fmt()
    .with_test_writer()
    .try_init();
```

---

## Documentation

> Start here: [DOCS_INDEX.md](../../DOCS_INDEX.md) is the canonical navigation index.

| Doc | Purpose |
|-----|---------|
| `DOCS_INDEX.md` | Documentation navigation |
| `docs/user/scripting-guide.md` | Rhai API reference |
| `docs/architecture/overview.md` | Engine design |
| `docs/architecture/roadmap.md` | Development milestones |
| `docs/contributing/development.md` | Build guide & contributing |
| `docs/specs/` | Design specifications |
| `examples/` | Working Rhai scripts |


<!-- AGENTS-INDEX-START -->

<!-- PROJECT-INDEX-START -->
[Project Index]|root: ./
|IMPORTANT: Prefer retrieval-led reasoning over pre-training-led reasoning
|assets:{1.jpg,2.jpg,3.jpg,4.jpg,5.jpg,6.jpg,background.mp4,img1.jpg,img2.jpg,img3.jpg,img4.jpg,img5.jpg,img6.jpg,music.mp3}
|assets/fonts:{Inter-Bold.ttf,JetBrainsMono-Bold.ttf,Roboto-Regular.ttf}
|crates/director-cli:{Cargo.toml,README.md}
|crates/director-cli/src:{main.rs}
|crates/director-cli/src/bin:{review_visuals.rs}
|crates/director-core:{Cargo.toml,README.md}
|crates/director-core/src:{animation.rs,audio.rs,director.rs,element.rs,errors.rs,lib.rs,registry.rs,scene.rs,tokens.rs,types.rs,video_wrapper.rs}
|crates/director-core/src/export:{mod.rs,video.rs}
|crates/director-core/src/node:{box_node.rs,composition.rs,effect.rs,image_node.rs,lottie.rs,mod.rs,text.rs,text_animator.rs,vector.rs,video_node.rs}
|crates/director-core/src/scripting:{mod.rs,theme.rs,types.rs,utils.rs}
|crates/director-core/src/scripting/api:{animation.rs,audio.rs,effects.rs,lifecycle.rs,mod.rs,nodes.rs,properties.rs}
|crates/director-core/src/systems:{assets.rs,layout.rs,mod.rs,renderer.rs,transitions.rs}
|crates/director-core/tests:{api_showcase.rs,audio.rs,composition.rs,examples.rs,layout.rs,masking.rs,transforms.rs,transitions.rs,typography.rs,video.rs,visual_suite.rs}
|crates/director-core/tests/assets:{test_image.png,test_video.mp4}
|crates/director-core/tests/fixtures:{lottie_mobilo_a.json,lottie_simple.json,lottie_u4j.json}
|crates/director-core/tests/snapshots:{elements_basic_box_linux.png,elements_basic_box_windows.png,elements_blend_multiply_linux.png,elements_blend_multiply_windows.png,elements_blend_overlay_linux.png,elements_blend_overlay_windows.png,elements_blend_screen_linux.png,elements_blend_screen_windows.png,layout_align_center_linux.png,layout_align_center_windows.png,layout_align_end_linux.png,layout_align_end_windows.png,layout_align_start_linux.png,layout_align_start_windows.png,layout_structure.txt,structure_layout_stability.txt,test_visual_basic_box.png}
|crates/director-core/tests/visual:{elements.rs,layout.rs,mod.rs}
|crates/director-developer:{Cargo.toml,README.md}
|crates/director-developer/examples:{glow_effect.ron}
|crates/director-developer/prompts:{add_rhai_function.prompt,debug_animation.prompt,extend_schema.prompt,implement_effect.prompt,implement_node.prompt}
|crates/director-developer/src:{main.rs,prompts.rs,spec.rs,synthesizer.rs,tests.rs,watch.rs}
|crates/director-developer/src/reflectors:{graph.rs,mod.rs,pipeline.rs,schema.rs,scripting.rs}
|crates/director-pipeline:{Cargo.toml,README.md}
|crates/director-pipeline/src:{lib.rs}
|crates/director-pipeline/tests:{error_handling.rs,parity_test.rs}
|crates/director-schema:{Cargo.toml,README.md}
|crates/director-schema/src:{lib.rs}
|crates/director-view:{Cargo.toml}
|crates/director-view/frontend:{README.md,bun.lock,index.html,package.json,postcss.config.js,tailwind.config.js,tsconfig.json,tsconfig.node.json,vite.config.ts}
|crates/director-view/frontend/public:{director.svg}
|crates/director-view/frontend/src:{App.tsx,main.tsx,vite-env.d.ts}
|crates/director-view/frontend/src/api:{director.ts}
|crates/director-view/frontend/src/components:{Controls.tsx,Preview.tsx,ScriptEditor.tsx,Timeline.tsx,index.ts}
|crates/director-view/frontend/src/stores:{project.ts}
|crates/director-view/frontend/src/styles:{globals.css}
|crates/director-view/src:{main.rs}
|crates/director-view/static:{index.html}
|crates/lottie-core:{Cargo.toml}
|crates/lottie-core/benches:{animator_bench.rs}
|crates/lottie-core/src:{animatable.rs,expressions.rs,lib.rs,modifiers.rs,renderer.rs}
|crates/lottie-core/tests:{spatial_bezier.rs,test_dash.rs}
|crates/lottie-data:{Cargo.toml}
|crates/lottie-data/src:{lib.rs,model.rs}
|crates/lottie-data/tests:{heart_eyes.json,integration_parsing.rs,mobilo_a.json}
|crates/lottie-skia:{Cargo.toml,compositing_test_output.png,output.png}
|crates/lottie-skia/src:{lib.rs}
|crates/lottie-skia/tests:{compositing.rs,context_test.rs,render_test.rs}
|docs/architecture:{overview.md,roadmap.md}
|docs/contributing:{development.md,documentation.md}
|docs/specs:{FRONTEND_ARCHITECTURE.md,RHAI_API_SPEC.md,SAM3_SPEC.md,SDK_ARCHITECTURE.md,TEMPLATE_STRATEGY.md}
|examples:{README.md,complex_preview.rhai,preview_test.rhai,ultimate_mashup.rhai,verification_test.rhai}
|examples/basics:{animation.rhai,hello_world.rhai,layout_flexbox.rhai,text.rhai}
|examples/features:{audio_reactive.mp4,audio_reactive.rhai,audio_visualizer.mp4,audio_visualizer.rhai,cinematic_shaders.rhai,effects.rhai,elastic_easings.rhai,elastic_showcase.rhai,grid_bento.rhai,image.rhai,kinetic_text.rhai,masking.rhai,motion_showcase.rhai,transitions.rhai,z_index.rhai}
|{Cargo.lock,Cargo.toml,DOCS_INDEX.md,README.md,setup_assets.sh,showcase.mp4,test_image.mp4}
<!-- PROJECT-INDEX-END -->

### director-developer



---
name: director-developer
description: Use the director-developer CLI to introspect the Director engine for Spec-Driven Development. Run this tool to get accurate, ground-truth information about available types, APIs, and capabilities.
---

# Director-Developer Skill

This skill enables AI agents to work with **verified system truth** about the Director engine. Instead of relying on potentially outdated documentation, use `director-dev` to introspect the actual codebase.

## When to Use This Skill

- When implementing new features across multiple Director layers
- When you need accurate JSON schemas for data types
- When you need the exact Rhai function signatures
- When validating a feature spec before implementation
- When generating context for AI-assisted development

## Available Commands

### Quick Reference

```bash
# Show system overview
director-dev info

# List available types, functions, nodes, or effects
director-dev list types
director-dev list functions --filter "animate"
director-dev list nodes
director-dev list effects

# Create a new feature spec from template
director-dev new "My Feature" --template effect --output specs

# Validate a feature spec
director-dev validate --spec specs/my_feature.ron

# Generate context from a spec
director-dev generate --spec specs/my_feature.ron --output CONTEXT.md

# Dump all system truth to files
director-dev dump --output .director-dev

# Watch for spec changes (interactive mode)
director-dev watch --dir specs --output .director-dev/contexts

# Generate AI prompts from templates
director-dev prompt --list
director-dev prompt implement_effect --var EFFECT_NAME=Glow

# Show workspace dependency graph
director-dev graph
director-dev graph --impact director-core
director-dev graph --format mermaid
```

## Workflow

### 1. Get System Info First

Before implementing a feature, run:

```bash
cargo run -p director-developer -- info
```

This shows:
- Number of registered Rhai functions
- Available schema types
- Supported node types, effects, and transitions
- Animation support matrix

### 2. List Specific Resources

Query for specific types or functions:

```bash
# Find all animation-related functions
cargo run -p director-developer -- list functions --filter "anim"

# Find all schema types
cargo run -p director-developer -- list types
```

### 3. Create a Feature Spec

Use the `new` command to scaffold a spec from a template:

```bash
# Templates: effect, node, animation, api
cargo run -p director-developer -- new "Glow Effect" --template effect --output specs
```

This generates a pre-filled RON spec that you can customize:

```ron
FeatureSpec(
    title: "Glow Effect",
    user_story: "As a video creator, I want to add a glow effect...",
    priority: 2,
    
    related_types: ["EffectConfig", "Node"],
    related_functions: ["effect", "add_"],
    
    schema_changes: [...],
    scripting_requirements: [...],
    pipeline_requirements: [...],
    
    verification: VerificationSpec(...),
)
```

### 4. Validate the Spec

```bash
cargo run -p director-developer -- validate --spec specs/my_feature.ron
```

This checks:
- ✓ All related types exist in the schema
- ✓ All related function patterns have matches
- Reports proposed changes needed

### 5. Generate Context

```bash
cargo run -p director-developer -- generate --spec specs/my_feature.ron
```

This creates a `CURRENT_CONTEXT.md` with:
- Full JSON schemas for related types
- Matching Rhai function signatures
- Pipeline capabilities table
- Proposed changes summary
- Verification checklist

## Output Files

When running `director-dev dump`, three JSON files are generated:

| File | Contents |
|------|----------|
| `rhai_api.json` | All registered Rhai function metadata |
| `schemas.json` | JSON Schemas for all `director-schema` types |
| `pipeline_capabilities.json` | Node types, effects, transitions, constraints |

## Best Practices

1. **Always query before implementing** - Don't assume API shapes, verify them
2. **Use specs for cross-cutting features** - Features touching schema + scripting + pipeline
3. **Regenerate context after engine changes** - Schemas may have changed
4. **Check animation support** - Use `supports_animation(node, property)` logic

## Example: Implementing a Glow Effect

See [examples/glow_effect.ron](file:///d:/rust-skia-engine/crates/director-developer/examples/glow_effect.ron) for a complete feature spec demonstrating:

- Schema changes (adding EffectConfig::Glow variant)
- Scripting requirements (add_glow function)
- Pipeline requirements (implementation notes)
- Verification criteria

## Troubleshooting

**Q: Command output is empty**
A: The CLI uses `println!` for output. Ensure you're running directly, not capturing stderr only.

**Q: Type not found**
A: Check if the type has `#[derive(JsonSchema)]` in `director-schema` or `director-core`.

**Q: Function not found**
A: Check if the function is registered in `director-core/src/scripting/api/`.


---

### director-rhai

[director-rhai Index]|root: ./.agent/skills/director-rhai
|IMPORTANT: Use these tools for director-rhai tasks
|examples:{animated_hero.rhai,simple_text.rhai}
|resources:{API_REFERENCE.md}

---
name: director-rhai
description: Author Rhai scripts for the Director video engine. Use this when the user asks to create or edit video scripts, animations, or cinematic transitions.
---

# Director Rhai Skill

This skill enables the creation of high-performance video scripts using the Director engine's Rhai API. It leverages Skia for rendering and Taffy for layout.

## Goal
To author valid, efficient, and visually rich Rhai scripts that the Director engine can execute to generate video frames or full exports.

## Instructions

1.  **Initialize the Director**: Every script MUST start by creating a `Movie` object.
    - `let movie = new_director(1920, 1080, 30);` (Width, Height, FPS)
2.  **Add Scenes**: Movies are composed of scenes.
    - `let scene = movie.add_scene(5.0);` (Duration in seconds)
3.  **Add Nodes**: Scenes contain hierarchical nodes (Box, Text, Image, Video).
    - `let box = scene.add_box(#{ ... });`
    - `let text = scene.add_text(#{ content: "Hello", ... });`
4.  **Layout**: Use Flexbox or Grid properties in the props map (e.g., `flex_direction`, `justify_content`, `align_items`, `gap`, `padding`).
5.  **Animate**: Nodes support `animate`, `spring`, and `add_animator`.
    - `node.animate("property", start, end, duration, "easing"[, delay]);`
    - `node.spring("property", start, end, #{ stiffness: 100, damping: 10 });`
6.  **Transitions**: Add transitions between scenes.
    - `movie.add_transition(scene1, scene2, "fade", 1.0, "ease_in_out");`
7.  **Return the Movie**: The script MUST return the `movie` object at the end.
    - `movie`

## Common Pitfalls
- **Argument Mismatch**: Ensure `animate` has 5 or 6 arguments. Using too many or too few will cause a "Function not found" error at runtime.
- **Timing**: Sequences are additive. Multiple `animate` calls on the same property will play back-to-back. Use the `delay` argument (6th parameter) to pause before starting an animation.

## Constraints
- **Return Value**: Always end the script with the movie object. Failure to do so will result in no output.
- **Asset Paths**: Use relative paths for assets (e.g., `assets/logo.png`).
- **Performance**: Favor `spring` for UI-like movements and `back_out`/`elastic_out` for premium-feeling entrances.

## References
Consult `resources/API_REFERENCE.md` for a full list of functions and properties.
See `examples/` for reference implementation patterns.


---
<!-- AGENTS-INDEX-END -->