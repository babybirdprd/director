// Global Defaults Test
// Tests that default_font and asset_search_paths work correctly

let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(3.0);

// Dark background
let bg = scene.add_box(#{
    width: "100%",
    height: "100%",
    bg_color: "#1a1a2e"
});

// Test text with default font (should use Roboto-Regular from assets/fonts)
bg.add_text(#{
    content: "Global Defaults Test",
    size: 64,
    color: "#ffffff",
    position: "absolute",
    top: 100,
    width: "100%",
    justify_content: "center"
});

// Test image loading
let img = bg.add_image(#{
    src: "assets/img1.jpg",
    width: 400,
    height: 300,
    position: "absolute",
    top: 250,
    left: 760
});

// Add another text to confirm font rendering
bg.add_text(#{
    content: "Font: Default | Asset: Found via search paths",
    size: 36,
    color: "#a0a0a0",
    position: "absolute",
    top: 600,
    width: "100%",
    justify_content: "center"
});

movie
