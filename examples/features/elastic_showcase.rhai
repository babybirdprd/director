// Elastic & Bounce Showcase
// A high-end demonstration of the new physics-based easing engines.

let movie = new_director(1920, 1080, 60);
let scene = movie.add_scene(5.0);

// Deep metallic background
let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    bg_color: "#0f172a",
    flex_direction: "column",
    align_items: "center",
    justify_content: "center",
    padding: 80.0
});

// Header
let header = root.add_text(#{
    content: "Premium Motion Engine",
    size: 100.0,
    color: "#f8fafc",
    weight: "bold",
    margin_bottom: 100.0
});
header.animate("opacity", 0.0, 1.0, 1.0, "ease_out");
header.animate("y", -100.0, 0.0, 1.5, "elastic_out");

// Card Container
let container = root.add_box(#{
    width: "100%",
    height: 600,
    flex_direction: "row",
    justify_content: "space_between",
    align_items: "center"
});

fn create_card(parent, title, color, delay, easing) {
    let card_outer = parent.add_box(#{
        width: 450,
        height: 600,
        bg_color: "#1e293b",
        border_radius: 40.0,
        padding: 40.0,
        justify_content: "center",
        align_items: "center",
        border_width: 2.0,
        border_color: "#334155"
    });

    let icon = card_outer.add_box(#{
        width: 150,
        height: 150,
        bg_color: color,
        border_radius: 75.0,
        margin_bottom: 40.0
    });

    let text = card_outer.add_text(#{
        content: title,
        size: 50.0,
        color: "#f1f5f9",
        weight: "bold"
    });

    // Animate Entrance
    // Initial state (outside match logic for some props)
    card_outer.animate("opacity", 0.0, 1.0, 1.0 + delay, "linear");
    
    // Y-Slide with Bounce
    card_outer.animate("y", 1000.0, 0.0, 2.0 + delay, easing);
    
    // Scale Pop with Elastic
    card_outer.animate("scale", 0.0, 1.0, 1.5 + delay, "elastic_out");
}

create_card(container, "Bounce", "#38bdf8", 0.0, "bounce_out");
create_card(container, "Elastic", "#818cf8", 0.3, "elastic_out");
create_card(container, "Back", "#fb7185", 0.6, "back_out");

movie
