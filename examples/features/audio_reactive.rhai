// Audio Reactivity Demo - REAL FFT
// Boxes scale in real-time based on audio frequency bands

let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(8.0);  // 8 seconds

// Add audio track - keep the handle to get track ID
let music = movie.add_audio("assets/music.mp3");
let track_id = music.id();

// Create dark background
let bg = scene.add_box(#{
    width: "100%",
    height: "100%",
    bg_color: "#0a0a0a"
});

// Title
bg.add_text(#{
    content: "AUDIO REACTIVITY",
    size: 72.0,
    color: "#ffffff",
    position: "absolute",
    top: 80.0,
    width: "100%",
    justify_content: "center"
});

// Container for boxes
let container = bg.add_box(#{
    width: "100%",
    height: "100%",
    flex_direction: "row",
    justify_content: "center",
    align_items: "center"
});

// BASS box (Red) - scales 1.0 to 2.5 based on bass
let bass_box = container.add_box(#{
    width: 200.0,
    height: 200.0,
    bg_color: "#ff3366",
    margin: 60.0,
    border_radius: 20.0
});
bass_box.set_pivot(0.5, 0.5);
bass_box.bind_audio(track_id, "bass", "scale");

// MIDS box (Green) - scales based on mids
let mids_box = container.add_box(#{
    width: 200.0,
    height: 200.0,
    bg_color: "#33ff66",
    margin: 60.0,
    border_radius: 20.0
});
mids_box.set_pivot(0.5, 0.5);
mids_box.bind_audio(track_id, "mids", "scale");

// HIGHS box (Blue) - scales based on highs
let highs_box = container.add_box(#{
    width: 200.0,
    height: 200.0,
    bg_color: "#3366ff",
    margin: 60.0,
    border_radius: 20.0
});
highs_box.set_pivot(0.5, 0.5);
highs_box.bind_audio(track_id, "highs", "scale");

// Labels container
let labels = bg.add_box(#{
    width: "100%",
    height: 80,
    position: "absolute",
    bottom: 120.0,
    flex_direction: "row",
    justify_content: "center"
});

labels.add_text(#{
    content: "BASS",
    size: 36.0,
    color: "#ff3366",
    margin: 80.0
});

labels.add_text(#{
    content: "MIDS",
    size: 36.0,
    color: "#33ff66",
    margin: 80.0
});

labels.add_text(#{
    content: "HIGHS",
    size: 36.0,
    color: "#3366ff",
    margin: 80.0
});

movie
