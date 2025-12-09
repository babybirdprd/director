//! Debug showcase test - isolate performance issues
//!
//! Run this test in release mode to identify what causes the hang

use director_core::systems::renderer::render_export;
use director_core::{
    scripting::{register_rhai_api, MovieHandle},
    DefaultAssetLoader,
};
use rhai::Engine;
use std::fs;
use std::sync::Arc;

#[test]
fn test_debug_showcase() {
    let mut engine = Engine::new();
    let loader = Arc::new(DefaultAssetLoader);
    register_rhai_api(&mut engine, loader);

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    std::env::set_current_dir(workspace_root).ok();

    let script_path = workspace_root.join("examples/debug_showcase.rhai");
    let script = fs::read_to_string(&script_path).expect("Failed to read debug_showcase.rhai");

    println!("🔍 Executing debug showcase...");
    let start = std::time::Instant::now();

    let result = engine.eval::<MovieHandle>(&script).expect("Script failed");
    println!("Script execution: {:?}", start.elapsed());

    let mut director = result.director.lock().unwrap();

    let output_dir = workspace_root.join("output");
    fs::create_dir_all(&output_dir).ok();

    let out_path = output_dir.join("debug_showcase.mp4");
    if out_path.exists() {
        fs::remove_file(&out_path).unwrap();
    }

    println!("🎥 Rendering debug showcase...");
    let render_start = std::time::Instant::now();
    render_export(&mut director, out_path.clone(), None, None).expect("Export failed");
    println!("Render time: {:?}", render_start.elapsed());

    assert!(out_path.exists(), "Output video should exist");
    println!("✅ Done! Total time: {:?}", start.elapsed());
}
