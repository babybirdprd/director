// Test: Minimal scale animation
// A red square that scales from 50% to 150% over 30 frames (1 second)
// Expected: Square starts at 100px and grows to 300px

let movie = new_director(400, 400, 30);

// Create scene for exactly 30 frames at 30fps = 1 second
let scene = movie.add_scene(1.0);

// Add a container with white background
let container = scene.add_box(#{
    width: 400.0,
    height: 400.0,
    bg_color: "#FFFFFF"
});

// Load the scale-only test Lottie
let lottie = container.add_lottie("crates/lottie-data/tests/scale_only_test.json", #{
    width: 400.0,
    height: 400.0,
    speed: 1.0,
    "loop": false,
    z_index: 1
});

print("Testing scale animation: 50% -> 150% over 30 frames");
print("Expected: Red square grows from 100px to 300px");

// Return movie for export
movie
