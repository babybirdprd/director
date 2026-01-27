# Layout Engine

Layout is handled by [Taffy](https://github.com/DioxusLabs/taffy), a high-performance Flexbox and Grid implementation in Rust.

## The `LayoutEngine` Struct

Located in `src/systems/layout.rs`, this struct wraps the `Taffy` tree.

```rust
pub struct LayoutEngine {
    taffy: TaffyTree<NodeId>,
    root_node: NodeId,
    // ...
}
```

## The Layout Process

1.  **Sync**: Before layout computation, the engine synchronizes the `taffy::Style` from the `SceneGraph` into the `TaffyTree`.
2.  **Compute**: `taffy.compute_layout(...)` is called with the root dimensions (e.g., 1920x1080).
3.  **Measure**: Taffy asks for intrinsic sizes for leaf nodes (see below).
4.  **Write Back**: The computed `Layout` (x, y, width, height) is written back to the `SceneNode`.

## Intrinsic Sizing (`Measure`)

Some nodes, like `TextNode` and `ImageNode`, have natural dimensions based on their content.

1.  The `Element` trait has a method `needs_measure()`.
2.  If `true`, `LayoutEngine` registers a measure callback with Taffy.
3.  **Callback**:
    *   Taffy provides `known_dimensions` (constraints).
    *   The node calculates its size (e.g., by shaping text or checking image dims).
    *   Returns `Size<f32>`.

## Post-Layout Hook

After layout is computed, the engine runs a `post_layout` pass.

```rust
fn post_layout(&mut self, rect: Rect)
```

This is used for **Layout-Dependent State**.
*   **Example**: `TextNode` with `TextFit::Shrink`.
    *   The node checks if the text fits in the computed `rect`.
    *   If not, it reduces font size and re-shapes.
    *   *Note*: This does not trigger a re-layout in the same frame (for performance), so it strictly affects drawing content *within* the allocated box.

## Coordinate System

*   **Origin**: Top-Left `(0, 0)`.
*   **Y-Axis**: Down.
*   **Units**: Logical pixels (corresponding to Skia coordinates).
