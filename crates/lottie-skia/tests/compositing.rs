use lottie_core::LottiePlayer;
use lottie_data::model::LottieJson;
use lottie_skia::SkiaRenderer;
use skia_safe::{EncodedImageFormat, Rect};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[test]
fn test_render_masks_and_mattes() {
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../lottie-data/tests");

    let json_path = assets_dir.join("heart_eyes.json");
    let file = File::open(&json_path).expect("Failed to open JSON file: heart_eyes.json");
    let model: LottieJson = serde_json::from_reader(file).expect("Failed to parse JSON");

    let mut player = LottiePlayer::new();
    player.load_json(model);
    player.current_frame = 0.0; // Frame 0

    let tree = player.render_tree();

    let width = 500;
    let height = 500;

    let mut surface =
        skia_safe::surfaces::raster_n32_premul((width, height)).expect("Failed to create surface");
    let canvas = surface.canvas();

    let dest_rect = Rect::from_wh(width as f32, height as f32);

    SkiaRenderer::draw(canvas, &tree, dest_rect, 1.0, &());

    let image = surface.image_snapshot();
    let data = image
        .encode(None, EncodedImageFormat::PNG, 100)
        .expect("Failed to encode image");

    let mut file = File::create("compositing_test_output.png").expect("Failed to create file");
    file.write_all(data.as_bytes())
        .expect("Failed to write to file");

    println!("Rendered to compositing_test_output.png");

    assert!(std::path::Path::new("compositing_test_output.png").exists());
}
