use std::fs::File;
use std::io::Read;
use lottie_core::{LottiePlayer, LottieAsset};
use lottie_data::model::LottieJson;

fn main() {
    // Load heart_eyes.json
    let mut file = File::open("crates/lottie-data/tests/heart_eyes.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    
    let lottie_json: LottieJson = serde_json::from_str(&contents).unwrap();
    println!("Loaded heart_eyes.json");
    println!("  Dimensions: {}x{}", lottie_json.w, lottie_json.h);
    println!("  Duration: {} frames ({}s @ {}fps)", 
        lottie_json.op - lottie_json.ip,
        (lottie_json.op - lottie_json.ip) / lottie_json.fr,
        lottie_json.fr);
    println!("  Layers: {}", lottie_json.layers.len());
    
    // Create player and render a frame
    let asset = LottieAsset::from_model(lottie_json);
    let mut player = LottiePlayer::new();
    player.load(std::sync::Arc::new(asset));
    
    // Render at frame 30
    player.current_frame = 30.0;
    let render_tree = player.render_tree();
    
    println!("\nRendered at frame 30:");
    println!("  Output size: {}x{}", render_tree.width, render_tree.height);
    
    // Try to get more details about the render tree
    println!("\nRoot node content: {:?}", render_tree.root.content);
}
