# director-core

The heart of Director Engine. Contains rendering, layout, animation, and scripting logic.

## Documentation

**[📚 Internal Documentation (docs/)](docs/INDEX.md)**

*   **[Architecture](docs/ARCHITECTURE.md)**
*   **[Scene Graph](docs/SCENE_GRAPH.md)**
*   **[Rendering](docs/RENDERING.md)**
*   **[Layout](docs/LAYOUT.md)**
*   **[Scripting](docs/SCRIPTING.md)**
*   **[Audio](docs/AUDIO.md)**

## Overview

| Component | Purpose |
|-----------|---------|
| **Scene Graph** | Arena-based node storage with `NodeId` handles |
| **Layout** | Taffy-powered Flexbox (Grid planned) |
| **Rendering** | Skia 2D rasterization |
| **Animation** | Keyframes, springs, easings |
| **Scripting** | Rhai API bindings |

## Usage

```toml
[dependencies]
director-core = "1.1"
```

```rust
use director_core::{scripting, DefaultAssetLoader};
use rhai::Engine;
use std::sync::Arc;

let mut engine = Engine::new();
scripting::register_rhai_api(&mut engine, Arc::new(DefaultAssetLoader));

let movie = engine.eval::<scripting::MovieHandle>(script)?;
```

## Feature Flags

| Flag | Purpose |
|------|---------|
| `mock_video` | Build without FFmpeg |
| `vulkan` | Enable Vulkan Skia backend |
