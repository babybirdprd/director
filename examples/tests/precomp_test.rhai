let movie = new_director(800, 800, 30);
let scene = movie.add_scene(1.0);

// White background
let container = scene.add_box(#{
    width: 800.0,
    height: 800.0,
    bg_color: "#FFFFFF"
});

// Load test lottie
let lottie = container.add_lottie("crates/lottie-data/tests/precomp_test.json", #{
    width: 800.0,
    height: 800.0,
    speed: 1.0,
    "loop": false,
    z_index: 1
});

movie
