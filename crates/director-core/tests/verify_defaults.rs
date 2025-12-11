use director_core::{scripting::{register_rhai_api, MovieHandle}, DefaultAssetLoader};
use rhai::Engine;
use std::sync::Arc;
use std::fs;

#[test]
fn test_verify_defaults() {
    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let script_path = std::path::Path::new(manifest_dir).join("tests/test_defaults.rhai");
    let script = fs::read_to_string(&script_path).expect("Failed to read script");

    // Set current dir to workspace root for asset paths in script
    let workspace_root = std::path::Path::new(manifest_dir).parent().unwrap().parent().unwrap();
    std::env::set_current_dir(workspace_root).ok();

    let result = engine.eval::<MovieHandle>(&script).expect("Script failed");
    let mut director = result.director.lock().unwrap();

    // Trigger Layout (Frame 0)
    let mut surface = skia_safe::surfaces::raster_n32_premul((1000, 1000)).unwrap();
    let _ = director_core::systems::renderer::render_frame(&mut director, 0.0, surface.canvas());

    // Traverse to container
    let scene_root_id = director.timeline[0].scene_root;
    let scene_root = director.scene.get_node(scene_root_id).unwrap();

    // scene_root -> container (child 0)
    let container_id = scene_root.children[0];
    let container = director.scene.get_node(container_id).unwrap();

    let t1_id = container.children[0];
    let t2_id = container.children[1];
    let t1 = director.scene.get_node(t1_id).unwrap();
    let t2 = director.scene.get_node(t2_id).unwrap();

    println!("Container: {:?}", container.layout_rect);
    println!("T1: {:?}", t1.layout_rect);
    println!("T2: {:?}", t2.layout_rect);

    // Assert Vertical Stack (Local Coordinates)
    // T1 should be above T2 (T1.bottom <= T2.top)
    assert!(t1.layout_rect.bottom <= t2.layout_rect.top,
        "Text A should be above Text B (Column layout). T1 Bottom: {}, T2 Top: {}",
        t1.layout_rect.bottom, t2.layout_rect.top);

    // Assert Horizontal Center Alignment (Local Coordinates)
    // Nodes should be centered within the Container's width
    let container_half_width = container.layout_rect.width() / 2.0;

    let t1_center_x = t1.layout_rect.left + t1.layout_rect.width() / 2.0;
    let t2_center_x = t2.layout_rect.left + t2.layout_rect.width() / 2.0;

    assert!((t1_center_x - container_half_width).abs() < 1.0,
        "Text A should be horizontally centered in container. T1 X: {}, Cont Half Width: {}", t1_center_x, container_half_width);
    assert!((t2_center_x - container_half_width).abs() < 1.0,
        "Text B should be horizontally centered in container. T2 X: {}, Cont Half Width: {}", t2_center_x, container_half_width);
}
