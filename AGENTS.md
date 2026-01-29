---
trigger: always_on
---

# Agent Instructions for `director-engine`

This file is the **authoritative source of truth** for AI agents working on this codebase.

> [!IMPORTANT]
> **Agent Protocol - MANDATORY**
>
> 1.  **Read First**: Before editing any file, check for a `//!` doc block at the top. Read it to understand the module's responsibilities.
> 2.  **Locate via Map**: Use the **Codebase Map** below to find the correct files. Do not guess locations.
> 3.  **No `println!`**: strict prohibition on `println!`. Use `tracing::info!`, `tracing::debug!`, etc.
> 4.  **Update Docs**: If you change a module's responsibility, update its `//!` doc block.
> 5.  **Verify Work**: Always run `cargo check` and relevant tests before submitting.

---

## 🛠️ Environment & Build

**System Dependencies (Ubuntu/Debian)**
> You must have these installed to build the project.
```bash
sudo apt-get update && sudo apt-get install -y \
    clang libclang-dev llvm-dev \
    libavutil-dev libavformat-dev libavcodec-dev libswscale-dev libavfilter-dev libavdevice-dev \
    libasound2-dev pkg-config build-essential \
    libfreetype6-dev libfontconfig1-dev
```

**Build Commands**
```bash
# Build the engine
cargo build --release

# Run all tests
cargo test --release --workspace

# Run only Director Core tests
cargo test -p director-core

# Run Visual Regression tests (requires valid environment)
cargo test -p director-core --test visual_tests
```

**Managing Snapshots**
If visual tests fail due to intended changes, update the baselines:
```bash
export UPDATE_SNAPSHOTS=1
cargo test -p director-core --test visual_tests
```

---

## 🗺️ Codebase Map

### Core Systems (`crates/director-core/src`)
| Responsibility | Primary File | Key Structs/Functions |
| :--- | :--- | :--- |
| **Orchestration** | `director.rs` | `Director`, `TimelineItem` |
| **Scene Graph** | `scene.rs` | `SceneGraph` (Arena), `SceneNode` |
| **Element Trait** | `element.rs` | `Element` trait (mandatory for nodes) |
| **Registry** | `registry.rs` | `CapabilityRegistry`, `NodeTypeInfo` |
| **Errors** | `errors.rs` | `RenderError` |
| **Rendering** | `systems/renderer.rs` | `render_recursive`, `render_frame` |
| **Layout** | `systems/layout.rs` | `LayoutEngine`, Taffy integration |
| **Assets** | `systems/assets.rs` | `AssetManager`, `FontCollection` |
| **Scripting** | `scripting/mod.rs` | `Engine`, `register_rhai_api` |
| **Scripting Types** | `scripting/types.rs` | `MovieHandle`, `NodeHandle` |
| **Scripting API** | `scripting/api/*.rs` | `nodes.rs`, `animation.rs`, `effects.rs` |

### Node Implementations (`crates/director-core/src/node`)
| Node Type | File | Notes |
| :--- | :--- | :--- |
| **Box** | `box_node.rs` | Flexbox container, styling base |
| **Text** | `text.rs` | Skia Paragraph, rich text |
| **Text Animator**| `text_animator.rs` | Per-glyph text animation logic |
| **Image** | `image_node.rs` | Static bitmaps |
| **Video** | `video_node.rs` | FFMPEG wrapper, frame control |
| **Lottie** | `lottie.rs` | Vector animation playback |
| **Vector** | `vector.rs` | SVG/Path drawing |
| **Composition** | `composition.rs` | Nested scenes, sub-timelines |
| **Effect** | `effect.rs` | Filter/Shader application |

### Ecosystem
| Crate | Purpose |
| :--- | :--- |
| `director-cli` | Command line tool for rendering video files |
| `director-schema` | JSON data models and serialization |
| `director-pipeline` | Asset processing and hydration |
| `lottie-*` | Lottie parsing and Skia rendering backend |

---

## 🏛️ Architecture

### 1. Scene Graph (Arena)
The engine uses an arena-based scene graph (`Vec<Option<SceneNode>>`) where `NodeId` is a `usize` index.
*   **Parents/Children**: Stored as IDs.
*   **Update**: `Director::update` traverses the graph to update animations and state.
*   **Render**: `render_recursive` traverses the graph to draw.

### 2. Layout (Taffy)
Flexbox layout is computed every frame.
*   **Nodes**: Report intrinsic size via `Element::measure`.
*   **Output**: Taffy calculates final geometry (`Layout` struct).
*   **Transforms**: Applied *after* layout (visual only).

### 3. Scripting (Rhai)
Rhai is the primary user interface.
*   **Handles**: `NodeHandle`, `MovieHandle` wrap internal IDs.
*   **API**: Exposed via `scripting/api/`. New features must be registered here.
*   **Standard Library**: We are moving towards a "Rhai Standard Library" (`std/`) for high-level logic.

---

## 📝 Coding Standards

### Logging
*   **NEVER** use `println!`, `eprintln!`, or `dbg!`.
*   **ALWAYS** use `tracing`.
    ```rust
    tracing::info!(width, height, "Initializing renderer");
    tracing::warn!("Asset not found, using fallback");
    tracing::debug!(?node_id, "Processing node");
    ```

### Error Handling
*   Use `anyhow::Result` for top-level/CLI functions.
*   Use `RenderError` (from `errors.rs`) for core engine logic.
*   Fail gracefully where possible (e.g., missing asset -> placeholder + warning).

### Threading
*   `Director` is often wrapped in `Arc<Mutex<>>`.
*   `AssetManager` contains `!Send` types (Skia resources).
*   Be careful with cross-thread asset loading.

---

## 🪜 Common Workflows

### Adding a New Node Type
1.  Create `crates/director-core/src/node/my_node.rs`.
2.  Implement `Element` trait.
3.  Add to `node/mod.rs` and `registry.rs` (for reflection).
4.  Register in Rhai via `scripting/api/nodes.rs`.

### Adding a Rhai Function
1.  Identify the correct module in `scripting/api/` (e.g., `animation.rs` for motion).
2.  Define the function signature.
3.  Register it in `register_rhai_api`.
4.  Add a test case in `tests/` to verify it works.

### Visual Regression Testing
1.  Create a test in `tests/visual_tests.rs`.
2.  Run with `cargo test`.
3.  If it fails, inspect `target/visual_regression_failures`.
4.  If the new output is correct, run with `UPDATE_SNAPSHOTS=1`.
