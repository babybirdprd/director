// test_preview.rhai
// Simple script to test video loading and timeline seeking

let director = new_director(1920, 1080, 30);
let scene = director.add_scene(30.0); // 30 seconds scene
// Replace with a valid path on your system to test!
print("Please edit this script to point to a real video file!");
// add_video(scene, "C:/path/to/your/video.mp4", #{ object_fit: "contain" });

// For now, let's add some generated content to prove the engine works
let box = add_box(scene, #{
    bg_color: "#FF0000",
    width: 500.0,
    height: 500.0,
    x: 100.0,
    y: 100.0
});

// Animate it
box.animate("x", 100.0, 800.0, 5.0, "linear");
print("Scene initialized. Load a video to test Proxy generation.");

director
