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
container.add_lottie("tests/fixtures/lottie_simple.json", #{
    width: "100%",
    height: "100%",
    speed: 1.0
});

print("Lottie script evaluated.");
movie
