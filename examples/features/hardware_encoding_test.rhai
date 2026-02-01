// Hardware Acceleration Test Script
// Tests encoder detection and hardware encoding on NVIDIA 1050

let movie = new_director(1920, 1080, 30);

// Test 1: Check encoder detection
print("=== Encoder Detection Test ===");
let info = movie.get_encoder_info();
print(info);

// Test 2: Configure different encoders and verify
print("\n=== Encoder Configuration Tests ===");

// Test Auto mode (should pick NVENC on your 1050)
movie.configure_encoder("auto");
print("Auto mode: " + movie.get_encoder_info());

// Test explicit NVENC
movie.configure_encoder("nvenc");
print("NVENC mode: " + movie.get_encoder_info());

// Test software fallback
movie.configure_encoder("software");
print("Software mode: " + movie.get_encoder_info());

// Restore auto for actual export
movie.configure_encoder("auto");

// Test 3: Create a test scene
print("\n=== Creating Test Scene ===");
let scene = movie.add_scene(5.0);

// Create animated content to test encoding
let box = scene.add_box(#{
    width: 400,
    height: 400,
    bg_color: "#FF0000",
    x: 100,
    y: 100
});

// Animate box across screen
box.animate("x", 100, 1400, 5.0, "linear");

// Add text to verify quality
let text = scene.add_text(#{
    content: "Hardware Encoding Test",
    font_size: 48,
    color: "#FFFFFF",
    x: 960,
    y: 540
});
text.animate("opacity", 0.0, 1.0, 1.0, "ease_out");

print("Scene created with 5-second animation");
print("Expected: NVENC should provide 3-5x speedup vs software");

movie
