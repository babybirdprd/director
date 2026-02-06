use anyhow::Result;
use director_core::AssetLoader;
use director_pipeline::load_movie;
use director_schema::{
    AudioBand, AudioReactiveBinding, AudioReactiveProperty, MovieRequest, Node, NodeKind, Scene,
};
use std::sync::Arc;

struct MockAssetLoader;

impl AssetLoader for MockAssetLoader {
    fn load_bytes(&self, path: &str) -> Result<Vec<u8>> {
        if path == "missing.png" {
            anyhow::bail!("File not found");
        }
        Ok(vec![])
    }
}

#[test]
fn test_load_movie_error_propagation() {
    let loader = Arc::new(MockAssetLoader);
    let request = MovieRequest {
        width: 1920,
        height: 1080,
        fps: 30,
        default_font: None,
        asset_search_paths: vec![],
        scenes: vec![Scene {
            id: "scene_1".to_string(),
            name: None,
            duration_secs: 5.0,
            z_index: 0,
            background: None,
            root: Node {
                id: "root".to_string(),
                kind: NodeKind::Image {
                    src: "missing.png".to_string(),
                    object_fit: None,
                },
                style: Default::default(),
                transform: Default::default(),
                animations: vec![],
                spring_animations: vec![],
                audio_bindings: vec![],
                children: vec![],
            },
            transition: None,
        }],
        audio_tracks: vec![],
    };

    let result = load_movie(request, loader);
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(err_msg.contains("Failed to load image asset: missing.png"));
    assert!(err_msg.contains("Failed to build scene graph for scene: scene_1"));
}

#[test]
fn test_load_movie_rejects_missing_audio_binding_track() {
    let loader = Arc::new(MockAssetLoader);
    let request = MovieRequest {
        width: 1920,
        height: 1080,
        fps: 30,
        default_font: None,
        asset_search_paths: vec![],
        scenes: vec![Scene {
            id: "scene_1".to_string(),
            name: None,
            duration_secs: 5.0,
            z_index: 0,
            background: None,
            root: Node {
                id: "root".to_string(),
                kind: NodeKind::Box { border_radius: 0.0 },
                style: Default::default(),
                transform: Default::default(),
                animations: vec![],
                spring_animations: vec![],
                audio_bindings: vec![AudioReactiveBinding {
                    audio_id: "missing_track".to_string(),
                    band: AudioBand::Bass,
                    property: AudioReactiveProperty::Scale,
                    min_value: 0.5,
                    max_value: 1.5,
                    smoothing: 0.1,
                }],
                children: vec![],
            },
            transition: None,
        }],
        audio_tracks: vec![],
    };

    let result = load_movie(request, loader);
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(err_msg.contains("unknown audio track id"));
}
