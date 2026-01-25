// animated_hero.rhai - Advanced Animation and Layout
let movie = new_director(1920, 1080, 60);
let s1 = movie.add_scene(5.0);

// Background
s1.add_box(#{ 
    width: "100%", height: "100%", 
    bg_color: "#0f172a",
    justify_content: "center", 
    align_items: "center" 
});

// Hero Container
let hero = s1.add_box(#{
    width: 800,
    height: 400,
    bg_color: "rgba(30, 41, 59, 0.8)",
    border_radius: 20.0,
    border_width: 2.0,
    border_color: "#3b82f6",
    justify_content: "center",
    align_items: "center"
});

// Hero Animation
hero.spring("scale", 0.0, 1.0, #{ stiffness: 150.0, damping: 12.0 });
hero.animate("rotation", -10.0, 0.0, 1.0, "back_out");

// Text with staggered character animation
let title = hero.add_text(#{
    content: "DIRECTOR ENGINE",
    size: 80.0,
    color: "#ffffff",
    weight: "bold"
});
title.add_animator(0, 15, "y", 50.0, 0.0, 1.0, "cubic_out", 0.05);
title.add_animator(0, 15, "opacity", 0.0, 1.0, 0.5, "linear", 0.05);

movie
