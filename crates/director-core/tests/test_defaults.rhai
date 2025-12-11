let movie = new_director(1000, 1000, 30);
let scene = movie.add_scene(1.0);

// Explicit container
let container = scene.add_box(#{
    width: 500,
    height: 500,
    bg_color: "#CCCCCC"
});

let t1 = container.add_text(#{
    content: "Text A",
    size: 50
});

let t2 = container.add_text(#{
    content: "Text B",
    size: 50
});

return movie;
