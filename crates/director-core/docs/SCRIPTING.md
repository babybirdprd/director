# Scripting Internals

Director uses [Rhai](https://rhai.rs/) as its embedded scripting language. This document covers how the Rust engine binds to Rhai. For the user-facing API, see `docs/user/scripting-guide.md`.

## Architecture

The bridging logic is located in `src/scripting/`.

### Handles

We do not expose the `Director` or `SceneGraph` structs directly to Rhai. Instead, we use "Handles":

*   **`MovieHandle`**: Wraps `Arc<Mutex<Director>>`.
*   **`SceneHandle`**: Wraps `MovieHandle` + `scene_id`.
*   **`NodeHandle`**: Wraps `MovieHandle` + `scene_id` + `node_id`.

These handles are **lightweight** and **clonable**. They acts as proxies. When a script calls `node.set_x(100)`, the handle locks the `Director`, finds the node, and mutates it.

### Thread Safety

Since Rhai objects can be held in variables, they must be `Send + Sync`. The `Arc<Mutex<Director>>` pattern ensures that the Director is accessible safely, although the scripting execution itself happens sequentially on the main thread.

## Registration

The entry point is `src/scripting/mod.rs`:

```rust
pub fn register_rhai_api(engine: &mut Engine, ...) {
    api::lifecycle::register(engine);
    api::nodes::register(engine);
    api::animation::register(engine);
    // ...
}
```

The API is split into modules in `src/scripting/api/`:
*   `nodes.rs`: `add_box`, `add_text`, etc.
*   `properties.rs`: Getters/Setters for styles.
*   `animation.rs`: `animate()`, `spring()`.

## Adding a New API

To expose a new Rust function to Rhai:

1.  **Choose the Module**: Pick the relevant file in `src/scripting/api/`.
2.  **Define the Signature**:
    ```rust
    engine.register_fn("my_function", |handle: &mut NodeHandle, value: f32| {
        let mut director = handle.director.lock().unwrap();
        // modify director...
    });
    ```
3.  **Parsers**: If you accept complex maps (e.g., property bags), use parsers in `src/scripting/utils.rs` (e.g., `parse_color`, `parse_layout_style`).

## Shadow Types

We often use "Shadow Types" (defined in `director-schema` or `scripting/types.rs`) to convert Rhai `Map` objects into strongly-typed Rust structs before applying them to the engine.
