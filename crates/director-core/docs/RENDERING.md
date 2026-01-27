# Rendering Pipeline

The rendering pipeline converts the state of the Scene Graph into pixels. It utilizes `skia-safe` (bindings to Google Skia) for high-quality 2D vector and raster graphics.

## The Render Loop

The core function is `render_recursive` in `src/systems/renderer.rs`.

```rust
fn render_recursive(
    canvas: &Canvas,
    scene: &SceneGraph,
    node_id: NodeId,
    ctx: &RenderContext
)
```

### Steps per Node
1.  **Transform**: `canvas.concat(transform_matrix)` is applied to move the coordinate system to the node's local space.
2.  **Opacity**: Multiplies the current alpha by the node's opacity.
3.  **Effects (Layering)**:
    *   If the node has effects (Blur, Shadow, Shader), `canvas.save_layer()` is called with an `ImageFilter`.
    *   This redirects drawing into an offscreen buffer.
4.  **Drawing**: `node.element.render(canvas)` is called.
5.  **Compositing (Masks)**:
    *   If a `mask_node` is present, a new layer is saved.
    *   The mask node is rendered.
    *   The blend mode `DstIn` is applied to intersect the mask alpha with the content.
6.  **Children**: The function recurses for all children, sorted by `z_index`.
7.  **Restore**: `canvas.restore()` is called to pop the stack.

## Effects & Shaders

Effects are implemented via `skia_safe::ImageFilter`.

*   **Standard Effects**: Blur, DropShadow, ColorMatrix.
*   **Runtime Shaders**: Custom SkSL (Skia Shading Language) programs.
    *   Shaders are compiled and cached in `AssetManager`.
    *   Uniforms (`float`, `vec2`, `time`) are injected dynamically.

### The "Steal & Fill" Strategy
When applying an effect via Rhai (`node.apply_effect(...)`), a wrapper `EffectNode` is created.
*   The `EffectNode` "steals" the layout properties (width, height, margin) of the target.
*   The target is reparented inside the `EffectNode`.
*   The target's layout is reset to `width: 100%, height: 100%` to fill the effect wrapper.

## Video & Export

### `RenderMode::Preview`
*   Optimized for realtime-ish playback.
*   Video decoding uses a threaded worker to prefetch frames.
*   Skps frames if decoding is too slow.

### `RenderMode::Export`
*   **Frame Perfect**: The engine blocks until the exact video frame for time `t` is decoded.
*   **Sync**: Audio and Video are processed in strict lockstep.
*   **Output**: Frames are piped to FFmpeg via stdin.

## Resolution & Coordinates
*   The canvas uses logical pixels.
*   Default resolution is 1920x1080.
*   Coordinates are floating point (`f32`).
