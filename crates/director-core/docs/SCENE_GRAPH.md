# Scene Graph

The Scene Graph is the backbone of the Director engine. Unlike a traditional pointer-based tree, `director-core` uses a flat **Arena** (or ECS-lite) approach for memory management and performance.

## Storage: The Arena

The `SceneGraph` struct (`src/scene.rs`) manages all nodes in a flat vector:

```rust
pub struct SceneGraph {
    nodes: Vec<Option<SceneNode>>,
    free_indices: Vec<usize>,
    // ...
}
```

*   **`NodeId`**: A simple wrapper around `usize`. It is an index into the `nodes` vector.
*   **`Option<SceneNode>`**: Nodes can be deleted. When destroyed, the slot becomes `None`, and the index is added to `free_indices` for reuse.
*   **Generational Indices**: *Not currently implemented.* Be careful holding onto `NodeId`s across frames if nodes are being destroyed and recreated, as IDs are recycled immediately.

## The `SceneNode` Wrapper

A `SceneNode` is a container that composes generic data with specific behavior. It holds:

1.  **Hierarchy**: `parent: Option<NodeId>`, `children: Vec<NodeId>`.
2.  **Layout Data**: `style: taffy::Style` (Flexbox properties).
3.  **Transform Data**: `transform: Transform` (Translation, Rotation, Scale, Skew).
4.  **Behavior**: `element: Box<dyn Element>`.
5.  **Meta**: `name`, `z_index`, `mask_node`.

## The `Element` Trait

The `Element` trait (`src/element.rs`) defines the specific behavior of a node type (e.g., displaying text, playing video).

```rust
pub trait Element: Any + Debug + Send + Sync + ElementClone {
    // 1. Lifecycle
    fn update(&mut self, time: f32, duration: f32) -> Result<()>;

    // 2. Rendering
    fn render(
        &mut self,
        canvas: &Canvas,
        ctx: &RenderContext
    ) -> Result<()>;

    // 3. Layout
    fn needs_measure(&self) -> bool { false }
    fn measure(&self, known_dims: Size<Option<f32>>, ...) -> Size<f32>;
    fn post_layout(&mut self, rect: Rect) { ... }
}
```

### Common Implementations
*   **`BoxNode`**: Draws a rectangle with background/border. Acts as a container.
*   **`TextNode`**: Renders rich text using Skia Paragraph.
*   **`ImageNode`**: Renders a static bitmap.
*   **`VideoNode`**: Decodes and renders video frames.
*   **`CompositionNode`**: Contains a nested `Director` instance (nested timeline).

## Transforms and Coordinate Systems

*   **Order of Operations**: Translate -> Rotate -> Scale -> Skew.
*   **Pivot**: Defaults to `(0.5, 0.5)` (Center).
*   **Z-Index**: Controls draw order *within the same sibling group*. It does not flatten the tree; it is a local sort.

## Adding a New Node

To add a new visual element:
1.  Create a struct implementing `Element`.
2.  Implement `render()` to draw to the Skia Canvas.
3.  (Optional) Implement `measure()` if it has intrinsic size.
4.  Register it in `src/scripting/api/nodes.rs` to expose it to Rhai.
