// Intel QSV Hardware Encoding Test
// Tests Intel Quick Sync hardware encoding

let movie = new_director(1920, 1080, 30);

// Force Intel QSV encoder
movie.configure_encoder("qsv");

print("Testing Intel QSV Hardware Encoding...");
print(movie.get_encoder_info());

let scene = movie.add_scene(3.0);

// Create animated content
let box = scene.add_box(#{
    width: 400,
    height: 400,
    bg_color: "#00FF00",
    x: 100,
    y: 100
});

box.animate("x", 100, 1400, 3.0, "linear");

let text = scene.add_text(#{
    content: "Intel QSV Test",
    font_size: 48,
    color: "#FFFFFF",
    x: 960,
    y: 540
});

movie
