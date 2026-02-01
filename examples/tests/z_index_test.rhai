// Z-Index Test
// Tests that nodes render in correct z-order

let movie = new_director(1920, 1080, 30);
let scene = movie.add_scene(2.0);

// Black background
let bg = scene.add_box(#{
    width: "100%",
    height: "100%",
    bg_color: "#0a0a0a"
});

// Red box at z_index 1 - bottom layer
let red = bg.add_box(#{
    width: 300,
    height: 300,
    bg_color: "#ff0000",
    position: "absolute",
    top: 300,
    left: 300
});
red.set_z_index(1);

// Green box at z_index 2 - middle layer, partially covering red
let green = bg.add_box(#{
    width: 300,
    height: 300,
    bg_color: "#00ff00",
    position: "absolute",
    top: 450,
    left: 450
});
green.set_z_index(2);

// Blue box at z_index 3 - top layer, partially covering green
let blue = bg.add_box(#{
    width: 300,
    height: 300,
    bg_color: "#0000ff",
    position: "absolute",
    top: 600,
    left: 600
});
blue.set_z_index(3);

// Labels for each box
red.add_text(#{
    content: "z=1",
    size: 24,
    color: "#ffffff",
    justify_content: "center",
    align_items: "center"
});

green.add_text(#{
    content: "z=2",
    size: 24,
    color: "#ffffff",
    justify_content: "center",
    align_items: "center"
});

blue.add_text(#{
    content: "z=3",
    size: 24,
    color: "#ffffff",
    justify_content: "center",
    align_items: "center"
});

movie
