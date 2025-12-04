let movie = new_director(1920, 1080, 60);
let scene = movie.add_scene(5.0);

let container = scene.add_box(#{
    width: 800,
    height: 800,
    bg_color: "#222222",
    justify_content: "center",
    align_items: "center",
    margin: "auto"
});

// Load from fixtures
let lottie = container.add_lottie("tests/fixtures/lottie_simple.json", #{
    width: 500,
    height: 500,
    speed: 1.0
});

// Animate the lottie node to prove it's being updated/rendered
lottie.animate("rotation", 0.0, 360.0, 5.0, "linear");
lottie.animate("scale", 0.5, 1.5, 2.5, "ease_in_out");

print("Lottie script evaluated.");
movie
