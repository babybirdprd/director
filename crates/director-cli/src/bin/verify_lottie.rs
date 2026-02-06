// Lottie Frame Verification Tool
// Compares our rendered output with official reference frames

use image::{ImageBuffer, Rgba, RgbaImage};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug)]
struct FrameComparison {
    frame_num: u32,
    timestamp: f32,
    reference_path: PathBuf,
    our_path: PathBuf,
    diff_path: PathBuf,
    mse: f64,
    max_diff: u8,
    pixel_diff_count: usize,
}

fn main() {
    let lottie_path = Path::new("crates/lottie-data/tests/heart_eyes.json");
    let reference_dir = Path::new("examples/tests/reference_frames");
    let output_dir = Path::new("examples/tests/comparisons");
    let our_frames_dir = Path::new("examples/tests/our_frames");

    // Ensure directories exist
    fs::create_dir_all(output_dir).expect("Failed to create comparisons directory");
    fs::create_dir_all(our_frames_dir).expect("Failed to create our_frames directory");

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║      Lottie Frame Verification - heart_eyes.json              ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    // Frame indices and timestamps (0, 15, 30, 45, 60, 75, 90 at 30fps = 0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0s)
    let frame_indices = vec![0u32, 15, 30, 45, 60, 75, 90];
    let timestamps: Vec<f32> = frame_indices.iter().map(|&f| f as f32 / 30.0).collect();

    // First, render our frames at these timestamps
    println!("📸 Rendering our frames...");
    for (&frame_num, &timestamp) in frame_indices.iter().zip(&timestamps) {
        println!("  Rendering frame {} at {:.2}s", frame_num, timestamp);
        let output_path = our_frames_dir.join(format!("frame_{:03}.png", frame_num + 1));

        if let Err(e) = render_lottie_frame(lottie_path, frame_num, &output_path) {
            eprintln!("    ❌ Failed to render: {}", e);
        }
    }

    println!();
    println!("🔍 Comparing frames...");

    // Compare each frame
    let mut comparisons = Vec::new();
    for (&frame_num, &timestamp) in frame_indices.iter().zip(&timestamps) {
        let ref_path = reference_dir.join(format!("frame_{:03}.png", frame_num + 1));
        let our_path = our_frames_dir.join(format!("frame_{:03}.png", frame_num + 1));
        let diff_path = output_dir.join(format!("diff_{:03}.png", frame_num + 1));
        let report_path = output_dir.join(format!("report_{:03}.png", frame_num + 1));

        match compare_frames(
            &ref_path,
            &our_path,
            &diff_path,
            &report_path,
            frame_num,
            timestamp,
        ) {
            Ok(comp) => {
                comparisons.push(comp);
            }
            Err(e) => {
                eprintln!("  ❌ Frame {} comparison failed: {}", frame_num, e);
            }
        }
    }

    // Print summary
    println!();
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                    COMPARISON REPORT                          ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    for comp in &comparisons {
        print_frame_report(comp);
    }

    // Overall summary
    println!();
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                      SUMMARY                                  ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    let total_mse: f64 = comparisons.iter().map(|c| c.mse).sum();
    let avg_mse = total_mse / comparisons.len() as f64;
    let max_diff = comparisons.iter().map(|c| c.max_diff).max().unwrap_or(0);
    let total_pixel_diffs: usize = comparisons.iter().map(|c| c.pixel_diff_count).sum();

    println!("  Average MSE: {:.2}", avg_mse);
    println!("  Max Pixel Difference: {} (0-255)", max_diff);
    println!("  Total Pixels Different: {}", total_pixel_diffs);
    println!();

    // Determine pass/fail
    let pass_threshold = 100.0; // MSE threshold
    if avg_mse < pass_threshold {
        println!("  ✅ OVERALL: PASS (Average MSE < {})", pass_threshold);
    } else {
        println!("  ❌ OVERALL: FAIL (Average MSE >= {})", pass_threshold);
        println!();
        println!("  Detailed reports saved to: {}", output_dir.display());
        println!("    - diff_###.png: Visual diff (red = different)");
        println!("    - report_###.png: Side-by-side comparison");
    }
}

fn render_lottie_frame(
    lottie_path: &Path,
    frame_num: u32,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use lottie_core::{LottieAsset, LottiePlayer};
    use lottie_data::model::LottieJson;
    use lottie_skia::SkiaRenderer;
    use skia_safe::{surfaces, EncodedImageFormat, Rect};

    // Load the JSON
    let json_str = fs::read_to_string(lottie_path)?;
    let model: LottieJson = serde_json::from_str(&json_str)?;

    // Create asset and player
    let asset = Arc::new(LottieAsset::from_model(model));
    let mut player = LottiePlayer::new();
    player.load(asset);

    // Set frame
    player.current_frame = frame_num as f32;

    // Render to render tree
    let render_tree = player.render_tree();

    // Create Skia surface
    let mut surface =
        surfaces::raster_n32_premul((1920, 1920)).ok_or("Failed to create surface")?;
    let canvas = surface.canvas();

    // Clear background
    canvas.clear(skia_safe::Color::TRANSPARENT);

    // Draw the render tree
    let dest_rect = Rect::from_xywh(0.0, 0.0, 1920.0, 1920.0);
    SkiaRenderer::draw(canvas, &render_tree, dest_rect, 1.0, &());

    // Save to PNG
    let image = surface.image_snapshot();
    let encoded = image
        .encode(None, EncodedImageFormat::PNG, 100)
        .ok_or("Failed to encode image")?;
    fs::write(output_path, encoded.as_bytes())?;

    Ok(())
}

fn compare_frames(
    ref_path: &Path,
    our_path: &Path,
    diff_path: &Path,
    report_path: &Path,
    frame_num: u32,
    timestamp: f32,
) -> Result<FrameComparison, Box<dyn std::error::Error>> {
    let ref_img = image::open(ref_path)?.to_rgba8();
    let our_img = image::open(our_path)?.to_rgba8();

    // Ensure same dimensions
    let width = ref_img.width().min(our_img.width());
    let height = ref_img.height().min(our_img.height());

    let mut diff_img: RgbaImage = ImageBuffer::new(width, height);
    let mut max_diff: u8 = 0;
    let mut total_mse: f64 = 0.0;
    let mut pixel_diff_count = 0;

    // Calculate per-pixel differences
    for y in 0..height {
        for x in 0..width {
            let ref_pixel = ref_img.get_pixel(x, y);
            let our_pixel = our_img.get_pixel(x, y);

            // Calculate color difference
            let dr = (ref_pixel[0] as i16 - our_pixel[0] as i16).abs() as u8;
            let dg = (ref_pixel[1] as i16 - our_pixel[1] as i16).abs() as u8;
            let db = (ref_pixel[2] as i16 - our_pixel[2] as i16).abs() as u8;

            let diff = dr.max(dg).max(db);
            max_diff = max_diff.max(diff);

            if diff > 5 {
                // Threshold for "different"
                pixel_diff_count += 1;
                // Red for differences
                diff_img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            } else {
                // Gray for similar
                diff_img.put_pixel(x, y, Rgba([diff, diff, diff, 255]));
            }

            // MSE calculation
            total_mse += (dr as f64).powi(2) + (dg as f64).powi(2) + (db as f64).powi(2);
        }
    }

    let mse = total_mse / (width * height * 3) as f64;

    // Save diff image
    diff_img.save(diff_path)?;

    // Create side-by-side report
    create_side_by_side_report(
        &ref_img,
        &our_img,
        &diff_img,
        report_path,
        frame_num,
        timestamp,
        mse,
    )?;

    Ok(FrameComparison {
        frame_num,
        timestamp,
        reference_path: ref_path.to_path_buf(),
        our_path: our_path.to_path_buf(),
        diff_path: diff_path.to_path_buf(),
        mse,
        max_diff,
        pixel_diff_count,
    })
}

fn create_side_by_side_report(
    ref_img: &RgbaImage,
    our_img: &RgbaImage,
    diff_img: &RgbaImage,
    report_path: &Path,
    frame_num: u32,
    timestamp: f32,
    mse: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let width = ref_img.width();
    let height = ref_img.height();

    // Create canvas: 2x2 grid with labels
    let report_width = width * 2 + 60; // Gap between columns
    let report_height = height * 2 + 100; // Space for labels
    let mut report: RgbaImage = ImageBuffer::new(report_width, report_height);

    // Fill background
    for y in 0..report_height {
        for x in 0..report_width {
            report.put_pixel(x, y, Rgba([40, 40, 40, 255]));
        }
    }

    // Copy images into grid
    // Top-left: Reference
    for y in 0..height {
        for x in 0..width {
            report.put_pixel(x + 20, y + 40, *ref_img.get_pixel(x, y));
        }
    }

    // Top-right: Ours
    for y in 0..height {
        for x in 0..width {
            report.put_pixel(x + width + 40, y + 40, *our_img.get_pixel(x, y));
        }
    }

    // Bottom: Diff
    for y in 0..height {
        for x in 0..width {
            report.put_pixel(x + 20, y + height + 60, *diff_img.get_pixel(x, y));
        }
    }

    // Add labels (simplified - just save the layout)
    report.save(report_path)?;

    Ok(())
}

fn print_frame_report(comp: &FrameComparison) {
    let status = if comp.mse < 100.0 { "✅" } else { "❌" };
    println!(
        "  {} Frame {} ({:.2}s)",
        status, comp.frame_num, comp.timestamp
    );
    println!(
        "     MSE: {:.2} | Max Diff: {} | Diff Pixels: {} | {:.1}% different",
        comp.mse,
        comp.max_diff,
        comp.pixel_diff_count,
        (comp.pixel_diff_count as f64 / (1920.0 * 1920.0)) * 100.0
    );
    println!();
}
