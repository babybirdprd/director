use lottie_core::{LottieAsset, LottiePlayer};
use lottie_data::model::LottieJson;
use lottie_skia::SkiaRenderer;
use skia_safe::{EncodedImageFormat, Rect};
use std::fs::File;
use std::io::{Read, Write};
use std::sync::Arc;

#[test]
fn test_render_heart_eyes() {
    // Load heart_eyes.json
    let mut file =
        File::open("../lottie-data/tests/heart_eyes.json").expect("Failed to open heart_eyes.json");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read file");

    let lottie_json: LottieJson = serde_json::from_str(&contents).expect("Failed to parse JSON");

    // Create player
    let asset = LottieAsset::from_model(lottie_json);
    let mut player = LottiePlayer::new();
    player.load(Arc::new(asset));

    // Render at frame 30
    player.current_frame = 30.0;
    let tree = player.render_tree();

    println!("Rendering heart_eyes at frame 30");
    println!("  Canvas: {}x{}", tree.width, tree.height);

    // Create surface
    let width = tree.width as i32;
    let height = tree.height as i32;
    let mut surface =
        skia_safe::surfaces::raster_n32_premul((width, height)).expect("Failed to create surface");
    let canvas = surface.canvas();

    let dest_rect = Rect::from_wh(tree.width, tree.height);

    // Clear with white background
    canvas.clear(skia_safe::Color::WHITE);

    // Render
    SkiaRenderer::draw(canvas, &tree, dest_rect, 1.0, &());

    // Save to file
    let image = surface.image_snapshot();
    let data = image
        .encode(None, EncodedImageFormat::PNG, 100)
        .expect("Failed to encode image");

    let mut file = File::create("heart_eyes_frame30.png").expect("Failed to create file");
    file.write_all(data.as_bytes()).expect("Failed to write");

    println!("Saved to heart_eyes_frame30.png");

    // Also render frame 60 (when heart should be visible)
    player.current_frame = 60.0;
    let tree60 = player.render_tree();

    surface =
        skia_safe::surfaces::raster_n32_premul((width, height)).expect("Failed to create surface");
    let canvas60 = surface.canvas();
    canvas60.clear(skia_safe::Color::WHITE);
    SkiaRenderer::draw(canvas60, &tree60, dest_rect, 1.0, &());

    let image60 = surface.image_snapshot();
    let data60 = image60
        .encode(None, EncodedImageFormat::PNG, 100)
        .expect("Failed to encode image");

    let mut file60 = File::create("heart_eyes_frame60.png").expect("Failed to create file");
    file60
        .write_all(data60.as_bytes())
        .expect("Failed to write");

    println!("Saved to heart_eyes_frame60.png");
}
