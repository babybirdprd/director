// Test: Basic Lottie Positioning
// A static blue square positioned at [400, 400] in an 800x800 composition
// Expected: Blue 100x100 square centered at (400, 400)

let movie = new_director(800, 800, 30);

// Create scene for 1 second
let scene = movie.add_scene(1.0);

// Add a container with white background for visibility
let container = scene.add_box(#{
    width: 800.0,
    height: 800.0,
    bg_color: "#FFFFFF"
});

// Load the basic position test Lottie
// The lottie has a 100x100 blue square centered at position [400, 400]
let lottie = container.add_lottie("crates/lottie-data/tests/basic_position_test.json", #{
    width: 800.0,
    height: 800.0,
    speed: 1.0,
    "loop": false,
    z_index: 1
});

print("Testing basic positioning: Blue square at [400, 400]");
print("Expected: 100x100 blue square centered at composition center (400, 400)");

// Return movie for export
movie
