// Ultimate Motion Showcase
// Combining Kinetic Typography with Elastic & Bounce Easings
// A demonstration of the full range of motion capabilities in the engine.

let movie = new_director(1920, 1080, 60);
let scene = movie.add_scene(7.0);

// Set deep aesthetic background
let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    bg_color: "#05070a",
    flex_direction: "column",
    align_items: "center",
    justify_content: "center",
    padding: 100.0
});

// 1. Kinetic Typography with Elastic Pop
// "ELASTIC" with staggered elastic_out scale
let title = root.add_text(#{
    content: "ELASTIC",
    size: 220.0,
    color: "#4f46e5",
    weight: "bold",
    margin_bottom: 20.0
});

// Staggered per-letter pop-up from zero scale with overshoot
// Prop: scale, Start: 0.0, End: 1.0, Dur: 1.5, Ease: elastic_out
title.add_animator(0, 7, "scale", 0.0, 1.0, 1.5, "elastic_out");

// Secondary text with bounce drop
let sub = root.add_text(#{
    content: "MOTION ENGINE",
    size: 64.0,
    color: "#94a3b8",
    weight: "bold",
    margin_bottom: 120.0
});
// Fade in and drop from above per-letter
sub.add_animator(0, 13, "opacity", 0.0, 1.0, 1.0, "ease_out");
sub.add_animator(0, 13, "offset_y", -100.0, 0.0, 1.5, "bounce_out");

// 2. Interactive-style Cards showing different Physics
let card_row = root.add_box(#{
    width: "100%",
    height: 400,
    flex_direction: "row",
    justify_content: "space_between",
    align_items: "center"
});

fn create_showcase_card(parent, label, color, delay, easing) {
    let card = parent.add_box(#{
        width: 480,
        height: 320,
        bg_color: "#0f172a",
        border_radius: 32.0,
        border_width: 2.0,
        border_color: "#1e293b",
        padding: 40.0,
        flex_direction: "column",
        align_items: "center",
        justify_content: "center"
    });

    // Animate Card Entrance with Back easing (overshoot start/end)
    card.animate("scale", 0.0, 1.0, 1.0 + delay, "back_out");
    card.animate("opacity", 0.0, 1.0, 0.5 + delay, "ease_out");

    let indicator = card.add_box(#{
        width: 80,
        height: 80,
        bg_color: color,
        border_radius: 40.0,
        margin_bottom: 30.0
    });

    // Internal indicator uses the specific showcasing easing
    indicator.animate("y", 60.0, -60.0, 2.0 + delay, easing);
    
    card.add_text(#{
        content: label,
        size: 32.0,
        color: "#f8fafc",
        weight: "bold"
    });
}

create_showcase_card(card_row, "BOUNCE PHYSICS", "#ef4444", 0.0, "bounce_out");
create_showcase_card(card_row, "ELASTIC SPRING", "#10b981", 0.2, "elastic_out");
create_showcase_card(card_row, "BACK OVERSHOOT", "#f59e0b", 0.4, "back_out");

// Footer credit
let footer = root.add_box(#{
    width: "100%",
    margin_top: 80,
    align_items: "center"
});

let foot_text = footer.add_text(#{
    content: "Crafted with Rust & Skia",
    size: 24.0,
    color: "#475569"
});
foot_text.animate("opacity", 0.0, 0.6, 3.0, "linear");

movie
