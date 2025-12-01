let movie = new_director(1000, 1000, 30);
let scene = movie.add_scene(5.0);

let root = scene.add_box(#{
    width: "100%",
    height: "100%",
    flex_grow: 1.0,
    flex_direction: "row",
    align_items: "stretch"
});

// Image
// Should take (1000 - 200) = 800px width
root.add_image("test.jpg", #{
    flex_grow: 1.0,
    height: "auto"
});

// Video
root.add_video("test.mp4", #{
    width: 200,
    height: "100%"
});

movie
