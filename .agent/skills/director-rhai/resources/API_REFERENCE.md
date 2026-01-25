# Director Rhai API Reference

## Global Functions

### `new_director(width, height, fps[, config])`
Initializes a new `Movie` object.
- `width`: Integer
- `height`: Integer
- `fps`: Integer
- `config`: (Optional) Map with `mode` ("preview" or "export")

## Movie Methods

### `add_scene(duration)`
Adds a new scene to the movie.
- `duration`: Float (seconds)
- Returns: `SceneHandle`

### `add_transition(from_scene, to_scene, type, duration, easing)`
Adds a transition between two scenes.
- `type`: "fade", "slide_left", "slide_right", "wipe_left", "wipe_right", "circle_open"
- `duration`: Float (seconds)
- `easing`: Easing string (e.g., "ease_in_out", "cubic_out")

### `add_audio(path)`
Adds a global audio track.
- `path`: String (asset path)
- Returns: `AudioTrackHandle`

## Scene Methods

### `add_box(props)`
Adds a container node to the scene root.
- `props`: Map of properties
- Returns: `NodeHandle`

### `add_text(props)`
Adds a text node to the scene root.
- `props`: Map of properties (including `content`)
- Returns: `NodeHandle`

## Node Methods

### `add_box(props)` / `add_text(props)` / `add_image(path[, props])`
Adds a child node to the current node.

### `animate(property, start, end, duration, easing[, delay])`
Adds a keyframe animation segment.
- `property`: "x", "y", "scale", "rotation", "opacity", "blur", etc.
- `delay`: (Optional) Float (seconds) to wait before starting.

### `spring(property, start, end, config)`
Adds a physics-based spring animation.
- `config`: Map with `stiffness` and `damping`

### `add_animator(start_idx, end_idx, property, start, end, duration, easing[, stagger])`
(Text only) Adds a per-character animator.

### `apply_effect(name[, value/map])`
Applies a visual effect (grayscale, sepia, invert, contrast, brightness, blur, shader).

### `set_style(props)`
Updates layout and visual style properties.

## Common Properties (Props Map)
- `width`, `height`: Number or percentage (e.g., "100%")
- `bg_color`, `color`: Hex string (e.g., "#ffffff") or RGBA (e.g., "rgba(0,0,0,0.5)")
- `flex_direction`: "row", "column"
- `justify_content`, `align_items`: "center", "flex_start", "flex_end", "space_between"
- `gap`, `padding`, `margin`: Number
- `border_radius`, `border_width`, `border_color`: Styling
- `z_index`: Integer
- `opacity`: 0.0 to 1.0
- `object_fit`: "cover", "contain", "fill"
