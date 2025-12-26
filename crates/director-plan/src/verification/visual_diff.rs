use std::path::Path;
use std::process::Command;
use anyhow::{Context, Result, anyhow};
use image::{GenericImageView, ImageReader, Pixel, Rgba};
use serde::Serialize;
use std::fs;

#[derive(Debug, Serialize)]
pub struct VisualDiffReport {
    pub diff_detected: bool,
    pub mismatch_percentage: f64,
    pub diff_bounds: Option<Rect>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub fn verify_visual(
    workspace_root: &Path,
    golden_path: &str,
) -> Result<VisualDiffReport> {
    let proof_dir = workspace_root.join("proof");
    if !proof_dir.exists() {
        fs::create_dir_all(&proof_dir).context("Failed to create proof directory")?;
    }

    let actual_path = proof_dir.join("actual.png");
    let golden_full_path = workspace_root.join(golden_path);

    // 1. Capture Screenshot via Playwright
    // Note: This assumes the frontend server is running on localhost:3000
    // In a real scenario, we might need to start it or ensure it's up.
    // For now, we rely on the environment being set up.

    // We need to run this from apps/director-plan directory because that's where playwright config/deps are
    let frontend_dir = workspace_root.join("apps/director-plan");

    // TARGET_URL=http://localhost:3000 OUTPUT=proof/actual.png npx playwright test scripts/snapshot.spec.ts
    // output path needs to be absolute or relative to apps/director-plan?
    // Playwright test runs relative to the config/project root.
    // Let's pass absolute path for output to be safe.

    let output_arg = actual_path.to_string_lossy().to_string();

    let status = Command::new("npx")
        .current_dir(&frontend_dir)
        .arg("playwright")
        .arg("test")
        .arg("scripts/snapshot.spec.ts")
        .env("TARGET_URL", "http://localhost:3000") // TODO: Make configurable?
        .env("OUTPUT", &output_arg)
        .status()
        .context("Failed to execute playwright script")?;

    if !status.success() {
        return Err(anyhow!("Playwright screenshot capture failed"));
    }

    if !actual_path.exists() {
         return Err(anyhow!("Playwright finished but actual.png was not created at {:?}", actual_path));
    }

    // 2. Compare Images
    if !golden_full_path.exists() {
        // If no golden image exists, we can't compare.
        // Maybe we should treat this as "Pass" but warn?
        // Or "Fail" because verification requires golden image?
        // The user said: "If golden_image is present in the ticket ... Calculates pixel diff".
        // The caller of this function checks if golden_image is present.
        // If the FILE is missing, that's an error.
        return Err(anyhow!("Golden image not found at {:?}", golden_full_path));
    }

    let img1 = ImageReader::open(&golden_full_path)?.decode().context("Failed to decode golden image")?;
    let img2 = ImageReader::open(&actual_path)?.decode().context("Failed to decode actual image")?;

    if img1.dimensions() != img2.dimensions() {
        return Ok(VisualDiffReport {
            diff_detected: true,
            mismatch_percentage: 100.0,
            diff_bounds: None,
            reason: Some(format!(
                "Dimensions mismatch: Golden {:?} vs Actual {:?}",
                img1.dimensions(),
                img2.dimensions()
            )),
        });
    }

    let (width, height) = img1.dimensions();
    let mut mismatch_count = 0;
    let mut min_x = width;
    let mut max_x = 0;
    let mut min_y = height;
    let mut max_y = 0;

    for y in 0..height {
        for x in 0..width {
            let p1 = img1.get_pixel(x, y);
            let p2 = img2.get_pixel(x, y);

            if !pixels_match(p1, p2, 0) { // Tolerance 0 for now
                mismatch_count += 1;
                if x < min_x { min_x = x; }
                if x > max_x { max_x = x; }
                if y < min_y { min_y = y; }
                if y > max_y { max_y = y; }
            }
        }
    }

    let total_pixels = (width * height) as f64;
    let mismatch_percentage = (mismatch_count as f64 / total_pixels) * 100.0;

    if mismatch_count > 0 {
        Ok(VisualDiffReport {
            diff_detected: true,
            mismatch_percentage,
            diff_bounds: Some(Rect {
                x: min_x,
                y: min_y,
                width: max_x - min_x + 1,
                height: max_y - min_y + 1,
            }),
            reason: Some("Pixel mismatch detected".to_string()),
        })
    } else {
        Ok(VisualDiffReport {
            diff_detected: false,
            mismatch_percentage: 0.0,
            diff_bounds: None,
            reason: None,
        })
    }
}

fn pixels_match(p1: impl Pixel<Subpixel = u8>, p2: impl Pixel<Subpixel = u8>, tolerance: u8) -> bool {
    let p1_channels = p1.channels();
    let p2_channels = p2.channels();

    for (c1, c2) in p1_channels.iter().zip(p2_channels.iter()) {
        if (*c1 as i16 - *c2 as i16).abs() > tolerance as i16 {
            return false;
        }
    }
    true
}
