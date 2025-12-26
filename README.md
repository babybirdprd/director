# Director

> ⚠️ **Pre-release** — API may change before v1.0

**A programmatic video rendering engine in Rust.**

Director combines Taffy (CSS Flexbox), Skia (rasterization), and Rhai (scripting) to enable programmatic video generation with a clean, declarative API.

---

## Quick Start

```rhai
let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(5.0);

let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    justify_content: "center",
    align_items: "center",
    bg_color: "#1a1a2e"
});

let title = root.add_text(#{
    content: "Hello, Director!",
    size: 72.0,
    color: "#ffffff",
    weight: "bold"
});

title.animate("scale", 0.8, 1.0, 1.0, "bounce_out");

movie
```

```bash
cargo run --release -- examples/basics/hello_world.rhai output.mp4
```

---

## Features

| Category | Features |
|----------|----------|
| **Layout** | Flexbox via Taffy (justify, align, padding, margin, absolute positioning) |
| **Text** | SkParagraph with rich spans, weights, colors, backgrounds, shrink-to-fit |
| **Animation** | Keyframes + easing, spring physics, per-property animation |
| **Effects** | Blur, grayscale, sepia, invert, custom SkSL shaders |
| **Compositing** | Alpha masking, blend modes (multiply, screen, overlay, etc.) |
| **Media** | Image loading, video embedding, multi-track audio |
| **Transitions** | Scene transitions (fade, slide, wipe) with ripple edit |

---

## Project Structure

```
director/
├── crates/
│   ├── director-core/       # Main engine (rendering, scripting, layout)
│   ├── director-cli/        # Command-line video renderer
│   ├── director-plan/       # Task management CLI for AI agents
│   ├── director-pipeline/   # Asset pipeline utilities
│   ├── director-schema/     # Schema definitions
│   └── lottie-*/            # Lottie animation support
├── apps/
│   └── director-studio/     # Web dashboard (Vite + React)
├── plan/
│   └── tickets/             # TOML task specifications
├── examples/                # Reference Rhai scripts
├── docs/                    # Documentation
└── assets/                  # Test assets
```

---

## Installation

### As a Dependency

```toml
[dependencies]
director-engine = "1.1"
rhai = "1.19"
```

### Building from Source

```bash
# Clone
git clone https://github.com/user/director-engine.git
cd director-engine

# Build (requires FFmpeg)
cargo build --release

# Run example
cargo run --release -- examples/basics/hello_world.rhai output.mp4
```

### System Dependencies

| Dependency | Ubuntu | macOS | Windows |
|------------|--------|-------|---------|
| FFmpeg | `apt install libavutil-dev libavformat-dev libavcodec-dev libswscale-dev` | `brew install ffmpeg` | [gyan.dev build](https://www.gyan.dev/ffmpeg/builds/) |
| Clang (for Skia) | `apt install clang` | Xcode | LLVM |

See [docs/BUILD_GUIDE.md](docs/BUILD_GUIDE.md) for detailed setup.

---

## Documentation

| Document | Description |
|----------|-------------|
| [docs/SCRIPTING.md](docs/SCRIPTING.md) | Complete Rhai API reference |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Engine internals |
| [docs/BUILD_GUIDE.md](docs/BUILD_GUIDE.md) | Build instructions |
| [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) | How to contribute |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Development milestones |
| [examples/README.md](examples/README.md) | Example scripts index |

---

## Director Studio (Task Dashboard)

A web-based Kanban board for managing development tasks with AI agents.

```bash
# Build frontend (once)
cd apps/director-studio && pnpm install && pnpm build

# Start server
cargo run -p director-plan -- serve
# Open http://localhost:3000
```

CLI commands:
```bash
cargo run -p director-plan -- list                    # List tickets
cargo run -p director-plan -- context T-VRE-001       # Get context for LLM
cargo run -p director-plan -- verify T-VRE-001        # Run verification
```

---

## Examples

All examples are tested and serve as API reference:

```bash
# Basics
cargo run --release -- examples/basics/hello_world.rhai output.mp4
cargo run --release -- examples/basics/layout_flexbox.rhai output.mp4
cargo run --release -- examples/basics/animation.rhai output.mp4

# Features
cargo run --release -- examples/features/effects.rhai output.mp4
cargo run --release -- examples/features/masking.rhai output.mp4
cargo run --release -- examples/features/transitions.rhai output.mp4
```

---

## Embedding in Rust

```rust
use director_engine::{scripting, DefaultAssetLoader};
use rhai::Engine;
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    let mut engine = Engine::new();
    scripting::register_rhai_api(&mut engine, Arc::new(DefaultAssetLoader));

    let script = r#"
        let movie = new_director(1920, 1080, 30);
        let scene = movie.add_scene(3.0);
        scene.add_text(#{ content: "Hello", size: 72.0, color: "#FFF" });
        movie
    "#;

    let movie = engine.eval::<scripting::MovieHandle>(script)?;
    let mut director = movie.director.lock().unwrap();
    
    director_engine::systems::renderer::render_export(
        &mut director,
        "output.mp4".into(),
        None,
        None
    )?;
    
    Ok(())
}
```

---

## License

MIT
