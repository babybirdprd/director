// Test: Multi-frame trim path sampling
// Samples trim values at multiple frames to verify animation works

let movie = new_director(400, 400, 30);

// Create scene for 2 seconds (60 frames) to sample multiple points
let scene = movie.add_scene(2.0);

// Add a container with white background
let container = scene.add_box(#{
    width: 400.0,
    height: 400.0,
    bg_color: "#FFFFFF"
});

// Load the trim path test Lottie - animates from 0% to 100% over 30 frames
let lottie = container.add_lottie("crates/lottie-data/tests/trim_path_test.json", #{
    width: 400.0,
    height: 400.0,
    speed: 1.0,
    "loop": true,
    z_index: 1
});

print("Testing trim path animation across multiple frames...");

// The lottie has trim animation:
// - Frame 0: end=0% (no line visible)
// - Frame 15: end=50% (half line visible)
// - Frame 30: end=100% (full line visible)

// Return movie for export
movie
