use director_core::animation::SpringConfig;
use director_core::AssetLoader;
use director_pipeline::load_movie;
use director_schema::{
    AudioBand, AudioReactiveBinding, AudioReactiveProperty, AudioTrack, MovieRequest, Node,
    NodeKind, SpringAnimation, StyleMap, TransformMap,
};
use std::sync::Arc;

struct MockLoader;
impl AssetLoader for MockLoader {
    fn load_bytes(&self, _path: &str) -> std::result::Result<Vec<u8>, anyhow::Error> {
        // Return dummy valid wav header
        let wav = vec![
            0x52, 0x49, 0x46, 0x46, // RIFF
            36, 0, 0, 0, // ChunkSize
            0x57, 0x41, 0x56, 0x45, // WAVE
            0x66, 0x6d, 0x74, 0x20, // fmt
            16, 0, 0, 0, // Subchunk1Size (16 for PCM)
            1, 0, // AudioFormat (1 = PCM)
            1, 0, // NumChannels (1)
            68, 172, 0, 0, // SampleRate (44100)
            136, 88, 1, 0, // ByteRate
            2, 0, // BlockAlign
            16, 0, // BitsPerSample
            0x64, 0x61, 0x74, 0x61, // data
            0, 0, 0, 0, // Subchunk2Size (0)
        ];
        Ok(wav)
    }
}

#[test]
fn test_pipeline_parity() {
    let req = MovieRequest {
        width: 1920,
        height: 1080,
        fps: 30,
        default_font: None,
        asset_search_paths: vec![],
        scenes: vec![director_schema::Scene {
            id: "scene1".to_string(),
            name: Some("Scene 1".to_string()),
            duration_secs: 5.0,
            z_index: 7,
            background: None,
            root: Node {
                id: "root".to_string(),
                kind: NodeKind::Box { border_radius: 0.0 },
                style: StyleMap {
                    z_index: Some(11),
                    ..StyleMap::default()
                },
                transform: TransformMap::default(),
                animations: vec![],
                spring_animations: vec![SpringAnimation {
                    property: "scale_x".to_string(),
                    target: 1.5,
                    config: SpringConfig {
                        stiffness: 100.0,
                        damping: 10.0,
                        mass: 1.0,
                        velocity: 0.0,
                    },
                }],
                audio_bindings: vec![AudioReactiveBinding {
                    audio_id: "track1".to_string(),
                    band: AudioBand::Bass,
                    property: AudioReactiveProperty::ScaleY,
                    min_value: 1.0,
                    max_value: 2.0,
                    smoothing: 0.5,
                }],
                children: vec![],
            },
            transition: None,
        }],
        audio_tracks: vec![AudioTrack {
            id: "track1".to_string(),
            src: "audio.mp3".to_string(),
            start_time: 0.0,
            volume: 1.0,
            loop_audio: true,
        }],
    };

    let loader = Arc::new(MockLoader);
    let director = load_movie(req, loader).expect("Failed to load movie");

    // 1. Verify Audio Track Loaded
    // We expect 1 track in the mixer
    let mixer_track_count = director
        .audio_mixer
        .tracks
        .iter()
        .filter(|t| t.is_some())
        .count();
    assert_eq!(
        mixer_track_count, 1,
        "Should have 1 global audio track loaded"
    );

    // 2. Verify Bindings
    // Get root node from scene 0
    // Director.timeline[0].scene_root is the ID
    let root_id = director.timeline[0].scene_root;
    assert_eq!(director.timeline[0].name.as_deref(), Some("Scene 1"));
    assert_eq!(director.timeline[0].z_index, 7);
    let root_node = director
        .scene
        .get_node(root_id)
        .expect("Root node should exist");
    assert_eq!(root_node.z_index, 11);

    assert_eq!(
        root_node.audio_bindings.len(),
        1,
        "Audio binding should be mapped"
    );
    assert_eq!(root_node.audio_bindings[0].property, "scale_y");
    // Verify track_id resolves to something valid (likely 0)
    assert_eq!(root_node.audio_bindings[0].track_id, 0);

    // 3. Verify Springs
    // scale_x should be animated.
    // Spring "baking" creates keyframes in the Animated<f32> struct.
    assert!(
        root_node.transform.scale_x.raw_keyframes.len() > 1,
        "Spring should generate keyframes for scale_x"
    );

    // Check values
    let first = root_node.transform.scale_x.raw_keyframes.first().unwrap();
    let last = root_node.transform.scale_x.raw_keyframes.last().unwrap();

    assert!((first.0 - 1.0).abs() < 0.001, "Start value should be 1.0");
    // Start (implied 1.0) -> Target 1.5
    // Note: Default scale is 1.0. Appending spring at t=0 might add jump if start!=last?
    // Just verify we have movement.
    assert!((last.0 - 1.5).abs() < 0.001, "Target value should be 1.5");
}
