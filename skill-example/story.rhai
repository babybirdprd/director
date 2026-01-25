// story.rhai - The Eternal Wanderer
// A 60-second cinematic journey through diverse landscapes.

let movie = new_director(1920, 1080, 30);
movie.add_audio("assets/music.mp3");

// -----------------------------------------------------------------------------
// INTRO SCENE (5.0s)
// -----------------------------------------------------------------------------
let intro = movie.add_scene(5.0);

// Background Video with blur effect
let bg_video = intro.add_box(#{ 
    width: "100%", height: "100%", 
    bg_color: "#000000" 
});
let vid = bg_video.add_image("assets/background.mp4", #{ 
    width: "100%", height: "100%", 
    object_fit: "cover" 
});
vid.animate("blur", 10.0, 0.0, 2.0, "cubic_out");

// Title Overlay
let title_container = intro.add_box(#{
    width: "100%", height: "100%",
    justify_content: "center", align_items: "center",
    z_index: 10
});

let title = title_container.add_text(#{
    content: "THE ETERNAL WANDERER",
    size: 100.0,
    color: "#ffffff",
    weight: "bold",
    letter_spacing: 5.0
});
title.spring("scale", 0.5, 1.0, #{ stiffness: 120, damping: 10 });
title.add_animator(0, 20, "opacity", 0.0, 1.0, 2.0, "linear", 0.1);
title.add_animator(0, 20, "y", 50.0, 0.0, 1.5, "back_out", 0.08);

// -----------------------------------------------------------------------------
// SCENE 1 - THE ANCIENT FOREST (8.0s)
// -----------------------------------------------------------------------------
let s1 = movie.add_scene(8.0);
let img1 = s1.add_image("assets/1.jpg", #{ width: "100%", height: "100%", object_fit: "cover" });
img1.animate("scale", 1.0, 1.15, 8.0, "linear");
img1.animate("x", 0.0, -50.0, 8.0, "linear");

let txt1_box = s1.add_box(#{
    width: "100%", height: "100%",
    justify_content: "flex_end", align_items: "center",
    padding: 100
});
let txt1 = txt1_box.add_text(#{
    content: "Through the ancient whispers of the forest...",
    size: 45.0, color: "#ffffff", bg_color: "rgba(0,0,0,0.4)", padding: 20
});
txt1.animate("opacity", 0.0, 1.0, 1.5, "cubic_out");
txt1.animate("y", 30.0, 0.0, 1.5, "cubic_out");

// -----------------------------------------------------------------------------
// SCENE 2 - THE WILD LANDS (8.0s)
// -----------------------------------------------------------------------------
let s2 = movie.add_scene(8.0);
let img2 = s2.add_image("assets/2.jpg", #{ width: "100%", height: "100%", object_fit: "cover" });
img2.animate("scale", 1.15, 1.0, 8.0, "linear");
img2.animate("y", 0.0, -30.0, 8.0, "linear");

let txt2_box = s2.add_box(#{
    width: "100%", height: "100%",
    justify_content: "flex_start", align_items: "center",
    padding: 100
});
let txt2 = txt2_box.add_text(#{
    content: "Across the wild, untamed territories...",
    size: 45.0, color: "#ffffff", bg_color: "rgba(0,0,0,0.4)", padding: 20
});
txt2.animate("opacity", 0.0, 1.0, 1.5, "cubic_out");

// -----------------------------------------------------------------------------
// SCENE 3 - THE SNOWY PEAKS (8.0s)
// -----------------------------------------------------------------------------
let s3 = movie.add_scene(8.0);
let img3 = s3.add_image("assets/3.jpg", #{ width: "100%", height: "100%", object_fit: "cover" });
img3.animate("scale", 1.0, 1.2, 8.0, "linear");

let txt3 = s3.add_text(#{
    content: "Past the peaks of eternal snow.",
    size: 50.0, color: "#ffffff", weight: "bold",
    x: 200, y: 800
});
txt3.add_animator(0, 30, "opacity", 0.0, 1.0, 2.0, "linear", 0.05);

// -----------------------------------------------------------------------------
// SCENE 4 - THE DESERT (8.0s)
// -----------------------------------------------------------------------------
let s4 = movie.add_scene(8.0);
let img4 = s4.add_image("assets/4.jpg", #{ width: "100%", height: "100%", object_fit: "cover" });
img4.animate("x", -50.0, 50.0, 8.0, "linear");
img4.animate("blur", 5.0, 0.0, 2.0, "cubic_out");

let txt4 = s4.add_text(#{
    content: "Searching for the truth in the golden sands.",
    size: 45.0, color: "#ffd700",
    width: "100%", text_align: "center", y: 900
});
txt4.animate("opacity", 0.0, 1.0, 2.0, "ease_in");

// -----------------------------------------------------------------------------
// SCENE 5 - THE CRYSTAL REALM (8.0s)
// -----------------------------------------------------------------------------
let s5 = movie.add_scene(8.0);
let img5 = s5.add_image("assets/5.jpg", #{ width: "100%", height: "100%", object_fit: "cover" });
img5.animate("rotation", -2.0, 2.0, 8.0, "linear");
img5.animate("scale", 1.2, 1.0, 8.0, "linear");

let txt5 = s5.add_text(#{
    content: "Where light bends to the will of time.",
    size: 45.0, color: "#a5f3fc",
    width: "100%", text_align: "center", y: 100
});
txt5.animate("blur", 10.0, 0.0, 3.0, "cubic_out");

// -----------------------------------------------------------------------------
// SCENE 6 - THE FINAL DESTINATION (8.0s)
// -----------------------------------------------------------------------------
let s6 = movie.add_scene(8.0);
let img6 = s6.add_image("assets/6.jpg", #{ width: "100%", height: "100%", object_fit: "cover" });
img6.animate("scale", 1.0, 1.1, 8.0, "linear");

let txt6 = s6.add_text(#{
    content: "To the place where it all begins again.",
    size: 55.0, color: "#ffffff", weight: "bold",
    width: "100%", text_align: "center", y: 540
});
txt6.spring("scale", 0.0, 1.0, #{ stiffness: 100, damping: 15 });

// -----------------------------------------------------------------------------
// OUTRO (7.0s)
// -----------------------------------------------------------------------------
let outro = movie.add_scene(7.0);
outro.add_box(#{ width: "100%", height: "100%", bg_color: "#000000" });

let credits = outro.add_text(#{
    content: "A DIRECTOR ENGINE PRODUCTION",
    size: 40.0, color: "#888888",
    width: "100%", text_align: "center", y: 500
});
credits.animate("opacity", 0.0, 1.0, 2.0, "linear");
credits.animate("opacity", 1.0, 0.0, 2.0, "linear", 3.0); // Fade out after 3s delay

// -----------------------------------------------------------------------------
// TRANSITIONS
// -----------------------------------------------------------------------------
movie.add_transition(intro, s1, "fade", 1.5, "ease_in_out");
movie.add_transition(s1, s2, "slide_right", 1.0, "cubic_out");
movie.add_transition(s2, s3, "circle_open", 1.5, "ease_in_out");
movie.add_transition(s3, s4, "wipe_left", 1.0, "ease_in_out");
movie.add_transition(s4, s5, "slide_left", 1.0, "cubic_out");
movie.add_transition(s5, s6, "fade", 2.0, "ease_in_out");
movie.add_transition(s6, outro, "fade", 2.0, "ease_in_out");

movie
