// verification_test.rhai - A complex test script generated using the director-rhai skill
let movie = new_director(1920, 1080, 60);

// Scene 1: Intro (3s)
let s1 = movie.add_scene(3.0);
s1.add_box(#{ width: "100%", height: "100%", bg_color: "#000000" });

let logo_container = s1.add_box(#{
    width: 200, height: 200, bg_color: "#3b82f6",
    border_radius: 100.0,
    justify_content: "center", align_items: "center",
    position: "relative"
});

// Bouncing entrance
logo_container.spring("scale", 0.0, 1.2, #{ stiffness: 200.0, damping: 10.0 });
logo_container.animate("y", -500.0, 0.0, 1.0, "bounce_out");

let logo_text = logo_container.add_text(#{
    content: "G",
    size: 100.0,
    color: "#ffffff",
    weight: "bold"
});

// Scene 2: Feature List (5s)
let s2 = movie.add_scene(5.0);
s2.add_box(#{ width: "100%", height: "100%", bg_color: "#111111" });

let list_container = s2.add_box(#{
    flex_direction: "column",
    gap: 30.0,
    padding: 100.0,
    width: "100%", height: "100%",
    justify_content: "center"
});

let features = ["Rust Powered", "Skia Rendering", "Rhai Scripting", "Flexbox Layout"];
for i in 0..features.len() {
    let item = list_container.add_text(#{
        content: features[i],
        size: 60.0,
        color: "#ffffff",
        margin_left: 50.0
    });
    // Staggered slide in
    item.animate("x", -1000.0, 0.0, 0.8, "cubic_out");
    item.animate("opacity", 0.0, 1.0, 0.5, "linear");
}

// Transition
movie.add_transition(s1, s2, "circle_open", 1.5, "ease_in_out");

movie
