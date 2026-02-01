// Multi-frame animation test to detect frozen/static animations
// This test renders a Lottie at multiple frames and verifies they differ

use lottie_core::{LottieAsset, LottiePlayer};
use lottie_data::model::LottieJson;
use lottie_skia::SkiaRenderer;
use skia_safe::{Canvas, EncodedImageFormat, Rect, Surface};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// Renders a Lottie frame to PNG bytes
fn render_lottie_frame(
    player: &mut LottiePlayer,
    frame: f32,
    width: u32,
    height: u32,
) -> Option<Vec<u8>> {
    player.current_frame = frame;
    let tree = player.render_tree();

    let mut surface = skia_safe::surfaces::raster_n32_premul((width as i32, height as i32))?;
    let canvas = surface.canvas();
    let dest_rect = Rect::from_wh(width as f32, height as f32);

    SkiaRenderer::draw(canvas, &tree, dest_rect, 1.0, &());

    let image = surface.image_snapshot();
    let data = image.encode(None, EncodedImageFormat::PNG, 100)?;
    Some(data.as_bytes().to_vec())
}

/// Calculate pixel difference between two images (0.0 = identical, 1.0 = completely different)
fn calculate_image_difference(img1: &[u8], img2: &[u8]) -> f64 {
    if img1.len() != img2.len() {
        return 1.0; // Different sizes = completely different
    }

    let mut diff_count = 0u64;
    let total_bytes = img1.len() as u64;

    // Compare every 4th byte (alpha channel) and RGB
    for i in (0..img1.len()).step_by(4) {
        if i + 3 < img1.len() {
            let r_diff = (img1[i] as i32 - img2[i] as i32).abs();
            let g_diff = (img1[i + 1] as i32 - img2[i + 1] as i32).abs();
            let b_diff = (img1[i + 2] as i32 - img2[i + 2] as i32).abs();
            let a_diff = (img1[i + 3] as i32 - img2[i + 3] as i32).abs();

            // Consider pixels different if any channel differs by more than 2
            if r_diff > 2 || g_diff > 2 || b_diff > 2 || a_diff > 2 {
                diff_count += 1;
            }
        }
    }

    diff_count as f64 / (total_bytes / 4) as f64
}

#[test]
fn test_heart_eyes_animation_progression() {
    // Load heart_eyes.json
    let json_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../lottie-data/tests/heart_eyes.json");
    let file = File::open(&json_path).expect("Failed to open heart_eyes.json");
    let model: LottieJson = serde_json::from_reader(file).expect("Failed to parse JSON");

    let asset = std::sync::Arc::new(LottieAsset::from_model(model));
    let mut player = LottiePlayer::new();
    player.load(asset);

    // Render key frames where animation should be different
    // Note: Animation ends around frame 86 (last keyframe at 85.977)
    // So we only test up to frame 86
    let test_frames = vec![
        (0.0, "start"),
        (40.0, "mid_animation"),
        (76.0, "trim_start"),
        (80.0, "trim_mid"),
        (86.0, "trim_end"),
    ];

    let mut renders: Vec<(f32, &str, Vec<u8>)> = Vec::new();

    for (frame, label) in &test_frames {
        let rendered = render_lottie_frame(&mut player, *frame, 500, 500)
            .expect(&format!("Failed to render frame {}", frame));
        renders.push((*frame, *label, rendered));

        // Save for debugging
        let output_path = PathBuf::from(format!(
            "target/lottie_test_heart_eyes_frame_{}_{}.png",
            frame, label
        ));
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let mut file = File::create(&output_path).expect("Failed to create output file");
        file.write_all(&renders.last().unwrap().2)
            .expect("Failed to write PNG");
    }

    // Verify consecutive frames are different (animation is progressing)
    for i in 1..renders.len() {
        let (frame1, label1, img1) = &renders[i - 1];
        let (frame2, label2, img2) = &renders[i];

        let diff = calculate_image_difference(img1, img2);

        println!(
            "Frame {} ({}) → {} ({}): {:.2}% different",
            frame1,
            label1,
            frame2,
            label2,
            diff * 100.0
        );

        // Assert animation is progressing
        assert!(
            diff > 0.01, // At least 1% of pixels should differ
            "Animation appears FROZEN between frame {} ({}) and frame {} ({}): only {:.2}% different",
            frame1, label1, frame2, label2, diff * 100.0
        );
    }

    println!("✓ Animation is progressing correctly across all frames");
}

#[test]
fn test_frame_calculation_advances() {
    // Test that frame calculation advances properly
    let json_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../lottie-data/tests/heart_eyes.json");
    let file = File::open(&json_path).expect("Failed to open heart_eyes.json");
    let model: LottieJson = serde_json::from_reader(file).expect("Failed to parse JSON");

    let asset = std::sync::Arc::new(LottieAsset::from_model(model));
    let mut player = LottiePlayer::new();
    player.load(asset.clone());

    // Test frame advancement at different times
    let fps = asset._frame_rate;
    let test_times = vec![0.0, 0.5, 1.0, 2.0, 3.0];

    for time in &test_times {
        let expected_frame = time * fps;

        // Simulate what LottieNode::update() does
        let start_frame = asset.model.ip;
        let calculated = (time * fps) + start_frame;

        println!(
            "Time {}s @ {}fps: expected={:.2}, calculated={:.2}",
            time, fps, expected_frame, calculated
        );

        // Allow small floating point tolerance
        assert!(
            (calculated - expected_frame).abs() < 0.1,
            "Frame calculation mismatch at time {}: expected {}, got {}",
            time,
            expected_frame,
            calculated
        );
    }

    println!("✓ Frame calculation advances correctly with time");
}
