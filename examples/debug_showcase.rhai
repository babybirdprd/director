// Debug Layout Version
print("🎬 Debug Layout Check...");

let width = 1920;
let height = 1080;
let fps = 1; // 1 FPS for quick debugging

let movie = new_director(width, height, fps, #{ mode: "export" });

// ============================================================
// SCENE 1
// ============================================================
let scene1 = movie.add_scene(1.0); // 1 Frame

// Background
let bg1_container = scene1.add_box(#{
    width: "100%",
    height: "100%",
    position: "absolute"
});
// Skip image loading to focus on layout boxes, or keep it to check ImageNode
let bg1 = bg1_container.add_image("assets/3.jpg", #{
    width: "100%",
    height: "100%"
});

// Title container
let title_container = scene1.add_box(#{
    width: "100%",
    height: "100%",
    align_items: "center",
    justify_content: "center",
    position: "absolute",
    left: 0.0,
    top: 0.0
});

// Title Text
let intro_text = title_container.add_text(#{
    content: "GENESIS",
    size: 180.0,
    color: "#FFFFFF",
    weight: "bold",
    flex_shrink: 0,
    fit: "none"
});

// ============================================================
// SCENE 2
// ============================================================
let scene2 = movie.add_scene(1.0);

let text_container = scene2.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "column",
    align_items: "center",
    justify_content: "center"
});

let wave_text = text_container.add_text(#{
    content: "DIRECTOR",
    size: 160.0,
    weight: "bold",
    color: "#FFFFFF",
    flex_shrink: 0,
    fit: "none"
});

let subtitle = text_container.add_text(#{
    content: "ENGINE",
    size: 100.0,
    weight: "bold",
    flex_shrink: 0,
    fit: "none"
});

movie
