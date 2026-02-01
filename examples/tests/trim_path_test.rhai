// Test: Simple trim path animation
// A horizontal line that draws from left to right over 1 second (30 frames)

let movie = new_director(400, 400, 30);

// Create scene for exactly 30 frames at 30fps = 1 second
let scene = movie.add_scene(1.0);

// Add a container with white background
let container = scene.add_box(#{
    width: 400.0,
    height: 400.0,
    bg_color: "#FFFFFF"
});

// Load the trim path test Lottie
let lottie = container.add_lottie("crates/lottie-data/tests/trim_path_test.json", #{
    width: 400.0,
    height: 400.0,
    speed: 1.0,
    "loop": false,
    z_index: 1
});

// Return movie for export
movie
