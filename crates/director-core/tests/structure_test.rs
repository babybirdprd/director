use director_core::{Director, DefaultAssetLoader, video_wrapper::RenderMode};
use director_core::node::BoxNode;
use director_core::director::TimelineItem;
use director_core::scene::SceneGraph;
use director_core::types::NodeId;
use std::sync::{Arc, Mutex};
use std::env;
use std::fs;
use std::path::PathBuf;
use taffy::style::Dimension;

fn dump_scene_tree(director: &Director) -> String {
    let mut output = String::new();

    // Find active timeline item (assuming first for this test)
    if let Some(item) = director.timeline.first() {
        recursive_dump(&director.scene, item.scene_root, 0, &mut output);
    } else {
        output.push_str("No active timeline item found.");
    }

    output
}

fn recursive_dump(graph: &SceneGraph, node_id: NodeId, depth: usize, output: &mut String) {
    if let Some(node) = graph.get_node(node_id) {
        let indent = "  ".repeat(depth);
        let children_ids = node.children.iter().map(|id| format!("{}", id)).collect::<Vec<_>>().join(", ");

        let rect_str = format!(
            "Rect(x:{:.1}, y:{:.1}, w:{:.1}, h:{:.1})",
            node.layout_rect.left,
            node.layout_rect.top,
            node.layout_rect.width(),
            node.layout_rect.height()
        );

        output.push_str(&format!(
            "{}Node[{}] (Children: [{}]) - {} Z: {}\n",
            indent, node_id, children_ids, rect_str, node.z_index
        ));

        for &child_id in &node.children {
            recursive_dump(graph, child_id, depth + 1, output);
        }
    }
}

#[test]
fn test_layout_structure_stability() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    let width = 1000;
    let height = 1000;
    let fps = 30;
    let director = Director::new(
        width,
        height,
        fps,
        Arc::new(DefaultAssetLoader),
        RenderMode::Preview,
        None
    );

    let director_arc = Arc::new(Mutex::new(director));

    // Kitchen Sink Construction
    {
        let mut d = director_arc.lock().unwrap();

        // 1. Root Container (Full Size)
        let mut root_node = BoxNode::new();
        root_node.style.size = taffy::geometry::Size {
            width: Dimension::length(1000.0), // Hardcoded size for stability
            height: Dimension::length(1000.0),
        };
        root_node.style.display = taffy::style::Display::Flex;
        root_node.style.flex_direction = taffy::style::FlexDirection::Column;
        // Center content to verify complex layout calculations (margin distribution)
        root_node.style.justify_content = Some(taffy::style::JustifyContent::Center);
        root_node.style.align_items = Some(taffy::style::AlignItems::Center);

        let root_id = d.scene.add_node(Box::new(root_node));

        d.timeline.push(TimelineItem {
            scene_root: root_id,
            start_time: 0.0,
            duration: 10.0,
            z_index: 0,
            audio_tracks: vec![],
        });

        // 2. Child 1: Flex Row with internal items
        let mut row_node = BoxNode::new();
        row_node.style.size.width = Dimension::percent(1.0);
        row_node.style.size.height = Dimension::length(200.0);
        row_node.style.flex_direction = taffy::style::FlexDirection::Row;
        row_node.style.justify_content = Some(taffy::style::JustifyContent::SpaceBetween);
        let row_id = d.scene.add_node(Box::new(row_node));
        d.scene.add_child(root_id, row_id);

        // Row Child A
        let mut box_a = BoxNode::new();
        box_a.style.size.width = Dimension::length(50.0);
        box_a.style.size.height = Dimension::length(50.0);
        let box_a_id = d.scene.add_node(Box::new(box_a));
        d.scene.add_child(row_id, box_a_id);

        // Row Child B
        let mut box_b = BoxNode::new();
        box_b.style.size.width = Dimension::length(50.0);
        box_b.style.size.height = Dimension::length(50.0);
        let box_b_id = d.scene.add_node(Box::new(box_b));
        d.scene.add_child(row_id, box_b_id);

        // 3. Child 2: Absolute Positioning
        let mut abs_node = BoxNode::new();
        abs_node.style.position = taffy::style::Position::Absolute;
        abs_node.style.size.width = Dimension::length(100.0);
        abs_node.style.size.height = Dimension::length(100.0);
        abs_node.style.inset.bottom = taffy::style::LengthPercentageAuto::length(10.0);
        abs_node.style.inset.right = taffy::style::LengthPercentageAuto::length(10.0);
        let abs_id = d.scene.add_node(Box::new(abs_node));
        d.scene.add_child(root_id, abs_id);

        // 4. Child 3: Z-Index test
        let mut z_node = BoxNode::new();
        z_node.style.size.width = Dimension::length(100.0);
        z_node.style.size.height = Dimension::length(100.0);

        let z_id = d.scene.add_node(Box::new(z_node));
        // Modify z-index in place
        if let Some(node) = d.scene.get_node_mut(z_id) {
            node.z_index = 10;
        }
        d.scene.add_child(root_id, z_id);
    }

    // Trigger Layout & Update
    {
        let mut d = director_arc.lock().unwrap();
        // Mimic render_frame logic to trigger layout
        let mut layout_engine = director_core::systems::layout::LayoutEngine::new();
        d.update(0.0);
        let width = d.width;
        let height = d.height;
        layout_engine.compute_layout(&mut d.scene, width, height, 0.0);
        d.run_post_layout(0.0);
    }

    // Generate Dump
    let d = director_arc.lock().unwrap();
    let actual_dump = dump_scene_tree(&d);

    // Snapshot Logic
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let snapshot_path = PathBuf::from(manifest_dir).join("tests/snapshots/structure_layout_stability.txt");

    if env::var("UPDATE_SNAPSHOTS").is_ok() {
        if let Some(parent) = snapshot_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create snapshot directory");
        }
        fs::write(&snapshot_path, &actual_dump).expect("Failed to write snapshot");
        println!("Updated snapshot: {:?}", snapshot_path);
    } else {
        if !snapshot_path.exists() {
            panic!("Snapshot not found: {:?}. Run with UPDATE_SNAPSHOTS=1 to generate.", snapshot_path);
        }
        let expected_dump = fs::read_to_string(&snapshot_path).expect("Failed to read snapshot");

        assert_eq!(actual_dump.trim(), expected_dump.trim(), "Layout structure mismatch! \nActual:\n{}\nExpected:\n{}", actual_dump, expected_dump);
    }
}
