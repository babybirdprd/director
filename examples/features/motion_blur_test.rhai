// Motion Blur Test Script
// Tests the configure_motion_blur API with fast-moving elements

let movie = new_director(1920, 1080, 30);

// Configure motion blur: 4 samples per frame, 180 degree shutter angle
movie.configure_motion_blur(4, 180.0);

let scene = movie.add_scene(3.0);

// Create a fast-moving red box
let box = scene.add_box(#{
    width: 100,
    height: 100,
    bg_color: "#FF0000",
    x: 0,
    y: 540
});

// Animate quickly across screen - motion blur should be visible
box.animate("x", 0, 1820, 3.0, "linear");

movie
