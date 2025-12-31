// Elastic Easings Demo (Fixed)
// Showcases new easing types with correct layout and scaling

let movie = new_director(1080, 1920, 60);
let scene = movie.add_scene(4.0);

// Root container - vertical stack covering whole screen
let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "column",
    bg_color: "#0a0a0a",
    padding: 60.0,
    align_items: "center",
    justify_content: "space_between" // Distribute title and list
});

// Title
root.add_text(#{
    content: "Elastic Easings",
    size: 80.0,
    color: "#ffffff",
    weight: "bold"
});

// List container
let list = root.add_box(#{
    width: "100%",
    flex_grow: 1.0,
    flex_direction: "column",
    justify_content: "space_around",
    padding: 20.0
});

fn create_row(parent, label, easing_name) {
    let row = parent.add_box(#{
        width: "100%",
        height: 100,
        flex_direction: "row",
        align_items: "center"
    });
    
    row.add_text(#{
        content: label,
        size: 32.0,
        color: "#9ca3af",
        width: 280
    });
    
    let track = row.add_box(#{
        flex_grow: 1.0,
        height: 60,
        bg_color: "#1f2937",
        border_radius: 30.0,
        padding: 5.0
    });
    
    let ball = track.add_box(#{
        width: 50,
        height: 50,
        bg_color: "#10b981",
        border_radius: 25.0
    });
    
    // Animate from left to right
    ball.animate("x", 0.0, 600.0, 2.0, easing_name);
    // Add rotation test
    ball.animate("rotation", 0.0, 360.0, 2.0, "linear");
    // Add opacity test (fade in)
    ball.animate("opacity", 0.0, 1.0, 1.0, "linear");
}

// Add easing variants
create_row(list, "bounce_out", "bounce_out");
create_row(list, "bounce_in", "bounce_in");
create_row(list, "elastic_out", "elastic_out");
create_row(list, "elastic_in", "elastic_in");
create_row(list, "elastic_in_out", "elastic_in_out");
create_row(list, "back_out", "back_out");
create_row(list, "back_in", "back_in");
create_row(list, "back_in_out", "back_in_out");
create_row(list, "ease_out (ref)", "ease_out");

movie
