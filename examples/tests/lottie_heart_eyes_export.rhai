// Test: Export heart_eyes.json Lottie to MP4 at 1920x1920
// Matching official export specs: 1920x1920, 30fps, 91 frames = 3.033s

let movie = new_director(1920, 1920, 30);

// Create scene for exactly 91 frames at 30fps = 3.033 seconds
// The Lottie is 800x800 native, but we'll scale it up
let scene = movie.add_scene(3.033);

// Add a full-size container
let container = scene.add_box(#{
    width: 1920.0,
    height: 1920.0,
    bg_color: "#F1F2F5"  // Light gray to match official LottieLab export
});

// Load the heart_eyes.json Lottie animation
// Native: 800x800 at 30fps with 91 frames
let lottie = container.add_lottie("crates/lottie-data/tests/heart_eyes.json", #{
    width: 1920.0,
    height: 1920.0,
    speed: 1.0,
    "loop": false,
    z_index: 1
});

// Return movie for export
movie
