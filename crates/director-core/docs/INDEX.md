# Director Core Internals

This documentation covers the internal architecture and implementation details of the `director-core` crate.

## Table of Contents

*   **[Architecture](ARCHITECTURE.md)**: High-level overview of the `Director` struct, the update loop, and system interactions.
*   **[Scene Graph](SCENE_GRAPH.md)**: The Arena-based node storage, `NodeId`s, the `Element` trait, and the `SceneNode` wrapper.
*   **[Rendering](RENDERING.md)**: The Skia-based rendering pipeline, handling of effects (shaders/layers), and thread-safety constraints.
*   **[Layout](LAYOUT.md)**: Integration with `Taffy` for flexbox layout, intrinsic sizing, and coordinate systems.
*   **[Scripting](SCRIPTING.md)**: How the Rust engine binds to Rhai, the module structure, and extending the API.
*   **[Audio](AUDIO.md)**: The audio mixing pipeline, `AudioTrack`s, and synchronization.

## Quick Links

*   **[Root Documentation Index](../../../DOCS_INDEX.md)**: Back to the main repository documentation.
*   **[Source Code](../src/)**: Browse the Rust source code.
