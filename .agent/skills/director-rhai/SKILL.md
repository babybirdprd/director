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
