use director_core::{Director, DefaultAssetLoader, video_wrapper::RenderMode};
use director_core::node::BoxNode;
use director_core::director::TimelineItem;
use std::sync::{Arc, Mutex};
use skia_safe::{ColorType, AlphaType, ColorSpace, EncodedImageFormat};
use std::path::PathBuf;
use std::env;
use std::fs;
use taffy::style::Dimension;
use image::{GenericImageView, Rgba};

/// Helper function to save a difference map between two images.
fn save_diff_image(reference: &image::RgbaImage, actual: &image::RgbaImage, path: &PathBuf) {
    let width = reference.width();
    let height = reference.height();

    let mut diff_img = image::RgbaImage::new(width, height);

    for x in 0..width {
        for y in 0..height {
            let p1 = reference.get_pixel(x, y);
            let p2 = actual.get_pixel(x, y);

            if p1 != p2 {
                // Mismatch: Magenta (Full Opacity)
                diff_img.put_pixel(x, y, Rgba([255, 0, 255, 255]));
            } else {
                // Match: Ghost (Dimmed Original)
                let mut dim = *p1;
                dim.0[3] = 64; // ~25% Alpha
                diff_img.put_pixel(x, y, dim);
            }
        }
    }

    diff_img.save(path).expect("Failed to save diff image");
}

/// Helper function to perform visual regression testing.
pub fn assert_frame_match(director: &mut Director, time: f64, snapshot_name: &str) {
    let width = director.width;
    let height = director.height;

    // 1. Setup Skia Surface
    let info = skia_safe::ImageInfo::new(
        (width, height),
        ColorType::RGBA8888,
        AlphaType::Premul,
        Some(ColorSpace::new_srgb()),
    );

    let mut surface = skia_safe::surfaces::raster(&info, None, None)
        .expect("Failed to create Skia surface");

    // 2. Render Frame
    director_core::systems::renderer::render_frame(director, time, surface.canvas());

    // 3. Encode to PNG
    let image = surface.image_snapshot();
    let data = image.encode(None, EncodedImageFormat::PNG, 100)
        .expect("Failed to encode image to PNG");
    let rendered_bytes = data.as_bytes();

    // Paths
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let snapshot_dir = PathBuf::from(manifest_dir.clone()).join("tests/snapshots");
    let snapshot_path = snapshot_dir.join(format!("{}.png", snapshot_name));

    // 4. Handle Snapshot Update
    if env::var("UPDATE_SNAPSHOTS").is_ok() {
        if !snapshot_dir.exists() {
            fs::create_dir_all(&snapshot_dir).expect("Failed to create snapshot directory");
        }
        fs::write(&snapshot_path, rendered_bytes).expect("Failed to write snapshot");
        println!("Updated snapshot: {:?}", snapshot_path);
        return;
    }

    // 5. Load Reference
    if !snapshot_path.exists() {
        panic!("Snapshot not found: {:?}. Run with UPDATE_SNAPSHOTS=1 to generate.", snapshot_path);
    }

    let reference_img = image::open(&snapshot_path)
        .expect("Failed to open reference snapshot")
        .to_rgba8();

    let rendered_img = image::load_from_memory(rendered_bytes)
        .expect("Failed to load rendered image")
        .to_rgba8();

    // 6. Compare Dimensions
    if reference_img.dimensions() != rendered_img.dimensions() {
        panic!(
            "Dimension mismatch! Reference: {:?}, Rendered: {:?}",
            reference_img.dimensions(),
            rendered_img.dimensions()
        );
    }

    // 7. Pixel Comparison
    let mut diff_pixels: u64 = 0;
    let total_pixels = (width * height) as u64;

    for (x, y, ref_pixel) in reference_img.enumerate_pixels() {
        let render_pixel = rendered_img.get_pixel(x, y);
        if ref_pixel != render_pixel {
            diff_pixels += 1;
        }
    }

    let diff_percent = (diff_pixels as f64 / total_pixels as f64) * 100.0;
    println!("Visual Difference: {:.4}% ({} / {} pixels)", diff_percent, diff_pixels, total_pixels);

    // 8. Handle Failure
    if diff_percent > 0.1 {
        let fail_dir = PathBuf::from(manifest_dir).join("target/visual_regression_failures");
        if !fail_dir.exists() {
            fs::create_dir_all(&fail_dir).ok();
        }

        let actual_path = fail_dir.join(format!("{}_actual.png", snapshot_name));
        let diff_path = fail_dir.join(format!("{}_diff.png", snapshot_name));

        fs::write(&actual_path, rendered_bytes).expect("Failed to save failure artifact");

        // Generate Diff Map
        save_diff_image(&reference_img, &rendered_img, &diff_path);

        panic!(
            "Visual regression failed! Image differed by {:.4}%. \nArtifacts:\n  Actual: {:?}\n  Diff:   {:?}",
            diff_percent,
            actual_path,
            diff_path
        );
    }
}

#[test]
fn test_visual_basic_box() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    let width = 200;
    let height = 200;
    let fps = 30; // u32
    let director = Director::new(
        width,
        height,
        fps,
        Arc::new(DefaultAssetLoader),
        RenderMode::Preview,
        None
    );

    let director_arc = Arc::new(Mutex::new(director));

    // Create Scene manually via Director lock
    let root_id = {
        let mut d = director_arc.lock().unwrap();

        let mut root_node = BoxNode::new();
        root_node.style.size = taffy::geometry::Size {
            width: Dimension::percent(1.0),
            height: Dimension::percent(1.0),
        };
        let root_id = d.scene.add_node(Box::new(root_node));

        d.timeline.push(TimelineItem {
            scene_root: root_id,
            start_time: 0.0,
            duration: 5.0,
            z_index: 0,
            audio_tracks: vec![],
        });
        root_id
    };

    // Add Content Box manually
    {
        let mut d = director_arc.lock().unwrap();
        let mut box_node = BoxNode::new();

        // Apply Style manually (simulating rhai parsing)
        // 100x100, Margin 50
        box_node.style.size.width = Dimension::length(100.0);
        box_node.style.size.height = Dimension::length(100.0);
        box_node.style.margin.left = taffy::style::LengthPercentageAuto::length(50.0);
        box_node.style.margin.top = taffy::style::LengthPercentageAuto::length(50.0);

        box_node.bg_color = Some(director_core::animation::Animated::new(director_core::types::Color::new(0.0, 0.0, 1.0, 1.0)));

        let id = d.scene.add_node(Box::new(box_node));
        d.scene.add_child(root_id, id);
    }

    // Run verification
    let mut d = director_arc.lock().unwrap();
    assert_frame_match(&mut *d, 0.0, "test_visual_basic_box");
}
