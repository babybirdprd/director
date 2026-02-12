# Core Architecture

The `director-core` crate implements the runtime engine for Director. It is designed as a frame-based, non-realtime rendering engine, prioritizing visual fidelity and deterministic output over interactive frame rates.

## The Director Struct

The `Director` struct (`src/director.rs`) is the central coordinator of the engine. It holds the state of the world and exposes methods to drive the frame loop.

```rust
pub struct Director {
    pub scene: SceneGraph,
    pub layout: LayoutEngine,
    pub assets: Arc<AssetManager>,
    pub audio: AudioMixer,
    // ... configuration (width, height, fps)
}
```

### Responsibilities
1.  **Time Management**: Managing the global timeline and calculating local scene times.
2.  **Resource Management**: Holding the `AssetManager` (images, fonts, shaders).
3.  **Pipeline Coordination**: orchestrating Update -> Layout -> Render -> Encode.

## The Frame Loop

A single frame execution involves four distinct phases, executed sequentially. This is deterministic: given the same inputs and time `t`, the output is bit-exact.

### 1. Update Phase (`Director::update`)
*   **Input**: Target time `t` (seconds).
*   **Action**:
    *   Updates the global clock.
    *   Traverses the active scene graph.
    *   Ticks all `Animated<T>` properties (interpolating keyframes or solving springs).
    *   Calls `Element::update(dt)` on each node to allow custom per-frame logic.

### 2. Layout Phase (`LayoutEngine::compute_layout`)
*   **Action**:
    *   Invokes `taffy` to compute the box model (x, y, width, height) for every node.
    *   **Intrinsic Sizing**: For nodes with `needs_measure()` (Text, Image), Taffy calls back into the engine to measure content size.
    *   **Post-Layout**: Calls `Element::post_layout()` on nodes. This is where `TextNode` performs "Auto-Shrink" logic (reducing font size to fit bounds).

### 3. Render Phase (`render_recursive`)
*   **Action**:
    *   Traverses the node tree (sorted by Z-Index).
    *   Manages the Skia `Canvas` state (`save`, `restore`, `concat` matrix).
    *   Applies **Effects** (Blur, Shadows, Shaders) using `save_layer` with `RuntimeEffect`s.
    *   Calls `Element::render()` to draw the node's visual content.

### 4. Encode/Output Phase (`export/video.rs`)
*   **Action**:
    *   The rendered frame (pixels) is sent to the FFmpeg encoder.
    *   Audio samples for the frame duration are mixed and sent to the audio encoder.
    *   Strict synchronization ensures audio and video streams remain aligned.

## Threading Model

The engine is primarily **single-threaded** during the render loop to ensure safety with Skia's `Canvas` and Taffy's layout tree (which is not thread-safe in the version used).

*   **`AssetManager`**: Is `!Send` because it holds `skia_safe::RuntimeEffect` (for shader caching), which is thread-local in the Skia bindings.
*   **Parallelism**:
    *   We do **not** use `rayon` for the main scene traversal.
    *   Video decoding happens on a separate thread (for `Preview` mode) or synchronously on the main thread (for `Export` mode).

## Key Files

*   `src/director.rs`: Main entry point.
*   `src/systems/renderer.rs`: The rendering pipeline logic.
*   `src/systems/layout.rs`: The layout engine wrapper.
*   `src/video_wrapper.rs`: FFmpeg integration.
