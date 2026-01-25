// simple_text.rhai - Basic Director Example
let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(3.0);

// Add a background
scene.add_box(#{
    width: "100%",
    height: "100%",
    bg_color: "#1a1a2e"
});

// Add centered text
scene.add_text(#{
    content: "HELLOWORLD",
    size: 120.0,
    color: "#ffffff",
    weight: "bold",
    width: "100%",
    height: "100%",
    justify_content: "center",
    align_items: "center"
});

movie
