// NEON AUDIO VISUALIZER
// Polished visualizer with glow effects and proper composition

let movie = new_director(1920, 1080, 60);  // 60fps for smoothness
let scene = movie.add_scene(10.0);

// Load audio
let music = movie.add_audio("assets/music.mp3");
let track_id = music.id();

// === DARK BACKGROUND ===
let bg = scene.add_box(#{
    width: "100%",
    height: "100%",
    bg_color: "#05050a"
});

// === CENTERED EQUALIZER ===
// Main container - centered
let main_container = bg.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "column",
    align_items: "center",
    justify_content: "center"
});

// Title
let title = main_container.add_text(#{
    content: "NEON VISUALIZER",
    size: 56.0,
    color: "#ffffff",
    weight: "bold",
    margin: 40.0
});

// Equalizer container
let eq_container = main_container.add_box(#{
    width: 800,
    height: 400,
    flex_direction: "row",
    align_items: "flex-end",
    justify_content: "center",
    padding: 40.0
});

// Create 16 equalizer bars with gradient colors
// Each bar has a glow layer behind it

// --- BASS Section (4 bars - Magenta/Pink) ---
let bass1_glow = eq_container.add_box(#{width: 32, height: 200, bg_color: "#ff006640", border_radius: 16.0, margin: 6.0});
bass1_glow.set_pivot(0.5, 1.0);
bass1_glow.bind_audio(track_id, "bass", "scale_y");
let bass1 = bass1_glow.add_box(#{width: 24, height: "100%", bg_color: "#ff0066", border_radius: 12.0, position: "absolute", left: 4.0});

let bass2_glow = eq_container.add_box(#{width: 32, height: 220, bg_color: "#ff338840", border_radius: 16.0, margin: 6.0});
bass2_glow.set_pivot(0.5, 1.0);
bass2_glow.bind_audio(track_id, "bass", "scale_y");
let bass2 = bass2_glow.add_box(#{width: 24, height: "100%", bg_color: "#ff3388", border_radius: 12.0, position: "absolute", left: 4.0});

let bass3_glow = eq_container.add_box(#{width: 32, height: 240, bg_color: "#ff66aa40", border_radius: 16.0, margin: 6.0});
bass3_glow.set_pivot(0.5, 1.0);
bass3_glow.bind_audio(track_id, "bass", "scale_y");
let bass3 = bass3_glow.add_box(#{width: 24, height: "100%", bg_color: "#ff66aa", border_radius: 12.0, position: "absolute", left: 4.0});

let bass4_glow = eq_container.add_box(#{width: 32, height: 260, bg_color: "#ff99cc40", border_radius: 16.0, margin: 6.0});
bass4_glow.set_pivot(0.5, 1.0);
bass4_glow.bind_audio(track_id, "bass", "scale_y");
let bass4 = bass4_glow.add_box(#{width: 24, height: "100%", bg_color: "#ff99cc", border_radius: 12.0, position: "absolute", left: 4.0});

// --- MIDS Section (8 bars - Cyan/Green) ---
let mid1_glow = eq_container.add_box(#{width: 32, height: 280, bg_color: "#00ffcc40", border_radius: 16.0, margin: 6.0});
mid1_glow.set_pivot(0.5, 1.0);
mid1_glow.bind_audio(track_id, "mids", "scale_y");
let mid1 = mid1_glow.add_box(#{width: 24, height: "100%", bg_color: "#00ffcc", border_radius: 12.0, position: "absolute", left: 4.0});

let mid2_glow = eq_container.add_box(#{width: 32, height: 300, bg_color: "#00ffaa40", border_radius: 16.0, margin: 6.0});
mid2_glow.set_pivot(0.5, 1.0);
mid2_glow.bind_audio(track_id, "mids", "scale_y");
let mid2 = mid2_glow.add_box(#{width: 24, height: "100%", bg_color: "#00ffaa", border_radius: 12.0, position: "absolute", left: 4.0});

let mid3_glow = eq_container.add_box(#{width: 32, height: 320, bg_color: "#00ff8840", border_radius: 16.0, margin: 6.0});
mid3_glow.set_pivot(0.5, 1.0);
mid3_glow.bind_audio(track_id, "mids", "scale_y");
let mid3 = mid3_glow.add_box(#{width: 24, height: "100%", bg_color: "#00ff88", border_radius: 12.0, position: "absolute", left: 4.0});

let mid4_glow = eq_container.add_box(#{width: 32, height: 340, bg_color: "#33ff6640", border_radius: 16.0, margin: 6.0});
mid4_glow.set_pivot(0.5, 1.0);
mid4_glow.bind_audio(track_id, "mids", "scale_y");
let mid4 = mid4_glow.add_box(#{width: 24, height: "100%", bg_color: "#33ff66", border_radius: 12.0, position: "absolute", left: 4.0});

let mid5_glow = eq_container.add_box(#{width: 32, height: 340, bg_color: "#33ff6640", border_radius: 16.0, margin: 6.0});
mid5_glow.set_pivot(0.5, 1.0);
mid5_glow.bind_audio(track_id, "mids", "scale_y");
let mid5 = mid5_glow.add_box(#{width: 24, height: "100%", bg_color: "#33ff66", border_radius: 12.0, position: "absolute", left: 4.0});

let mid6_glow = eq_container.add_box(#{width: 32, height: 320, bg_color: "#00ff8840", border_radius: 16.0, margin: 6.0});
mid6_glow.set_pivot(0.5, 1.0);
mid6_glow.bind_audio(track_id, "mids", "scale_y");
let mid6 = mid6_glow.add_box(#{width: 24, height: "100%", bg_color: "#00ff88", border_radius: 12.0, position: "absolute", left: 4.0});

let mid7_glow = eq_container.add_box(#{width: 32, height: 300, bg_color: "#00ffaa40", border_radius: 16.0, margin: 6.0});
mid7_glow.set_pivot(0.5, 1.0);
mid7_glow.bind_audio(track_id, "mids", "scale_y");
let mid7 = mid7_glow.add_box(#{width: 24, height: "100%", bg_color: "#00ffaa", border_radius: 12.0, position: "absolute", left: 4.0});

let mid8_glow = eq_container.add_box(#{width: 32, height: 280, bg_color: "#00ffcc40", border_radius: 16.0, margin: 6.0});
mid8_glow.set_pivot(0.5, 1.0);
mid8_glow.bind_audio(track_id, "mids", "scale_y");
let mid8 = mid8_glow.add_box(#{width: 24, height: "100%", bg_color: "#00ffcc", border_radius: 12.0, position: "absolute", left: 4.0});

// --- HIGHS Section (4 bars - Blue/Purple) ---
let high1_glow = eq_container.add_box(#{width: 32, height: 260, bg_color: "#6666ff40", border_radius: 16.0, margin: 6.0});
high1_glow.set_pivot(0.5, 1.0);
high1_glow.bind_audio(track_id, "highs", "scale_y");
let high1 = high1_glow.add_box(#{width: 24, height: "100%", bg_color: "#6666ff", border_radius: 12.0, position: "absolute", left: 4.0});

let high2_glow = eq_container.add_box(#{width: 32, height: 240, bg_color: "#9966ff40", border_radius: 16.0, margin: 6.0});
high2_glow.set_pivot(0.5, 1.0);
high2_glow.bind_audio(track_id, "highs", "scale_y");
let high2 = high2_glow.add_box(#{width: 24, height: "100%", bg_color: "#9966ff", border_radius: 12.0, position: "absolute", left: 4.0});

let high3_glow = eq_container.add_box(#{width: 32, height: 220, bg_color: "#cc66ff40", border_radius: 16.0, margin: 6.0});
high3_glow.set_pivot(0.5, 1.0);
high3_glow.bind_audio(track_id, "highs", "scale_y");
let high3 = high3_glow.add_box(#{width: 24, height: "100%", bg_color: "#cc66ff", border_radius: 12.0, position: "absolute", left: 4.0});

let high4_glow = eq_container.add_box(#{width: 32, height: 200, bg_color: "#ff66ff40", border_radius: 16.0, margin: 6.0});
high4_glow.set_pivot(0.5, 1.0);
high4_glow.bind_audio(track_id, "highs", "scale_y");
let high4 = high4_glow.add_box(#{width: 24, height: "100%", bg_color: "#ff66ff", border_radius: 12.0, position: "absolute", left: 4.0});

// === BAND LABELS ===
let label_row = main_container.add_box(#{
    width: 700,
    height: 50,
    flex_direction: "row",
    justify_content: "space-between",
    margin: 20.0
});

label_row.add_text(#{
    content: "BASS",
    size: 24.0,
    color: "#ff0066"
});

label_row.add_text(#{
    content: "MIDS",
    size: 24.0,
    color: "#00ff88"
});

label_row.add_text(#{
    content: "HIGHS",
    size: 24.0,
    color: "#9966ff"
});

// === PULSING CENTER ORB (behind eq) ===
let orb_container = bg.add_box(#{
    width: "100%",
    height: "100%",
    position: "absolute",
    top: 0.0,
    left: 0.0,
    justify_content: "center",
    align_items: "center"
});

// Outer glow
let orb_glow = orb_container.add_box(#{
    width: 600,
    height: 600,
    bg_color: "#ff006610",
    border_radius: 300.0
});
orb_glow.set_pivot(0.5, 0.5);
orb_glow.bind_audio(track_id, "bass", "scale");

// Inner orb
let orb = orb_glow.add_box(#{
    width: 300,
    height: 300,
    bg_color: "#ff006620",
    border_radius: 150.0,
    position: "absolute",
    top: 150.0,
    left: 150.0
});
orb.set_pivot(0.5, 0.5);
orb.bind_audio(track_id, "bass", "scale");

movie
