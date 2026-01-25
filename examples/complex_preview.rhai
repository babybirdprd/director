// examples/complex_preview.rhai
// A rich example demonstrating Layout, Typography, and Physics-based Animation

let director = new_director(1920, 1080, 30);
let scene = director.add_scene(10.0);

// 1. Background Container (Flexbox)
let container = add_box(scene, #{
    width: "100%",
    height: "100%",
    bg_color: "#111111",
    flex_direction: "column",
    align_items: "center",
    justify_content: "center",
    gap: 20.0
});

// 2. Title Text with Gradient-like effect via multiple spans (simulated)
// Note: Actual gradients on text might need shader support, 
// but we can use rich text spans.
let title = add_text(container, #{
    content: [
        #{ text: "DIRECTOR ", size: 80.0, weight: "bold", color: "#FF4D4D" },
        #{ text: "ENGINE", size: 80.0, weight: "bold", color: "#FFFFFF" }
    ],
    shadow_color: "#FF0000",
    shadow_blur: 20.0,
    margin_bottom: 40.0
});

// Animate Title Scale (Spring)
title.spring("scale", 0.0, 1.0, #{ stiffness: 200.0, damping: 15.0 });
title.spring("rotation", -10.0, 0.0, #{ stiffness: 100.0, damping: 10.0 });

// 3. Grid of Animated Cards
let grid = add_box(container, #{
    flex_direction: "row",
    flex_wrap: "wrap",
    justify_content: "center",
    gap: 20.0,
    width: "80%"
});

let colors = ["#4D4DFF", "#4DFF4D", "#FFFF4D", "#FF4DFF"];

for i in 0..4 {
    let card = add_box(grid, #{
        width: 150.0,
        height: 150.0,
        bg_color: colors[i],
        border_radius: 15.0,
        shadow_color: "#000000aa",
        shadow_blur: 10.0,
        shadow_y: 5.0,
        align_items: "center",
        justify_content: "center"
    });
    
    // Staggered Entrance
    let delay = i * 0.1;
    card.animate("scale", 0.0, 1.0, 0.6, "back_out");
    card.animate("y", 100.0, 0.0, 0.6, "cubic_out"); // Slide up
    
    // Inner Text
    add_text(card, #{
        content: `Item ${i+1}`,
        size: 20.0,
        color: "#000000",
        weight: "bold"
    });
    
    // Hover-like Animation (Ping Pong)
    // We simulate a loop by creating a long sequence or using loop property if available.
    // engine doesn't support 'loop' in animate() directly yet for keyframes without custom logic?
    // Let's just do a simple intro for now.
}

// 4. Footer Text
let footer = add_text(container, #{
    content: "Preview App Verification • High Performance Rust Engine",
    size: 16.0,
    color: "#888888",
    margin_top: 50.0
});

// Staggered Text Reveal
// animate characters in from opacity 0 or y offset?
// Director supports `add_animator` for text.
// footer.add_animator(start_char, end_char, property, start_val, end_val, duration, ease, stagger)
footer.add_animator(0, 50, "y", 20.0, 0.0, 1.0, "cubic_out", 0.02);
footer.add_animator(0, 50, "opacity", 0.0, 1.0, 1.0, "linear", 0.02);

print("Complex scene loaded.");
director
