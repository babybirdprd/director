//! Audio Processing Tests
//!
//! Tests for audio resampling and mixing.

use director_core::audio::resample_audio;
use director_core::director::TimelineItem;
use director_core::node::BoxNode;
use director_core::scene::AudioBinding;
use director_core::video_wrapper::RenderMode;
use director_core::{DefaultAssetLoader, Director};
use std::f32::consts::PI;
use std::sync::Arc;

/// Test audio resampling from 44.1kHz to 48kHz.
///
/// Generates a sine sweep and verifies resampling produces valid output.
#[test]
fn audio_resampling_44100_to_48000() {
    let source_rate = 44100u32;
    let target_rate = 48000u32;
    let duration_secs = 1; // Shorter for faster test
    let num_samples = (source_rate as usize) * duration_secs;

    // Generate sine sweep (stereo)
    let mut samples: Vec<f32> = Vec::with_capacity(num_samples * 2);
    let start_freq = 440.0f32;

    for i in 0..num_samples {
        let t = i as f32 / source_rate as f32;
        let sample = (2.0 * PI * start_freq * t).sin();
        samples.push(sample); // Left
        samples.push(sample); // Right
    }

    // Resample
    let resampled = resample_audio(&samples, source_rate, target_rate).expect("Resampling failed");

    // Verify output
    let expected_samples = (target_rate as usize) * duration_secs * 2; // stereo
    let tolerance = expected_samples / 10; // 10% tolerance

    assert!(
        (resampled.len() as i64 - expected_samples as i64).abs() < tolerance as i64,
        "Resampled length {} should be close to expected {}",
        resampled.len(),
        expected_samples
    );

    // Verify no NaN or Inf values
    for (i, &sample) in resampled.iter().enumerate() {
        assert!(sample.is_finite(), "Sample {} is not finite: {}", i, sample);
    }
}

/// Test that resampling preserves approximate amplitude.
#[test]
fn audio_resampling_preserves_amplitude() {
    let source_rate = 44100u32;
    let target_rate = 48000u32;

    // Generate simple sine wave at 440Hz
    let num_samples = 4410; // 0.1 seconds
    let mut samples: Vec<f32> = Vec::with_capacity(num_samples * 2);

    for i in 0..num_samples {
        let t = i as f32 / source_rate as f32;
        let sample = (2.0 * PI * 440.0 * t).sin() * 0.5; // Half amplitude
        samples.push(sample);
        samples.push(sample);
    }

    let resampled = resample_audio(&samples, source_rate, target_rate).expect("Resampling failed");

    // Find max amplitude
    let max_amplitude = resampled
        .iter()
        .map(|s| s.abs())
        .fold(0.0f32, |a, b| a.max(b));

    // Should be approximately 0.5 (with some tolerance for interpolation)
    assert!(
        max_amplitude > 0.4 && max_amplitude < 0.6,
        "Max amplitude {} should be close to 0.5",
        max_amplitude
    );
}

// ============================================================================
// FFT / Spectrum Analysis Tests
// ============================================================================

use director_core::audio::AudioAnalyzer;

/// Test FFT correctly identifies a 440Hz sine wave peak.
#[test]
fn audio_fft_sine_wave_peak() {
    let sample_rate = 48000u32;
    let fft_size = 2048usize;
    let freq = 440.0f32;

    // Generate 1 second of 440Hz sine wave (stereo, interleaved)
    let num_frames = sample_rate as usize;
    let mut samples: Vec<f32> = Vec::with_capacity(num_frames * 2);

    for i in 0..num_frames {
        let t = i as f32 / sample_rate as f32;
        let sample = (2.0 * PI * freq * t).sin();
        samples.push(sample); // Left
        samples.push(sample); // Right
    }

    let analyzer = AudioAnalyzer::new(fft_size, sample_rate);
    let spectrum = analyzer.compute_spectrum(&samples, 0.0);

    // Find the bin with the maximum magnitude
    let (peak_bin, peak_mag) = spectrum
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap();

    // Calculate expected bin for 440Hz
    let bin_hz = sample_rate as f32 / fft_size as f32;
    let expected_bin = (freq / bin_hz).round() as usize;

    // Peak should be within 2 bins of expected (due to spectral leakage)
    assert!(
        (peak_bin as i32 - expected_bin as i32).abs() <= 2,
        "Peak at bin {} ({}Hz), expected near bin {} ({}Hz)",
        peak_bin,
        peak_bin as f32 * bin_hz,
        expected_bin,
        freq
    );

    // Peak magnitude should be significant
    assert!(
        *peak_mag > 0.1,
        "Peak magnitude {} should be significant",
        peak_mag
    );
}

/// Test energy bands correctly separate bass from highs.
#[test]
fn audio_energy_bands_separation() {
    let sample_rate = 48000u32;
    let fft_size = 2048usize;

    // Generate low frequency (100Hz) bass signal
    let bass_freq = 100.0f32;
    let num_frames = fft_size * 2; // Enough samples
    let mut bass_samples: Vec<f32> = Vec::with_capacity(num_frames * 2);

    for i in 0..num_frames {
        let t = i as f32 / sample_rate as f32;
        let sample = (2.0 * PI * bass_freq * t).sin();
        bass_samples.push(sample);
        bass_samples.push(sample);
    }

    let analyzer = AudioAnalyzer::new(fft_size, sample_rate);

    let bass_energy = analyzer.bass(&bass_samples, 0.0);
    let highs_energy = analyzer.highs(&bass_samples, 0.0);

    // Bass signal should have higher bass energy than highs
    assert!(
        bass_energy > highs_energy,
        "For 100Hz signal: bass ({}) should be > highs ({})",
        bass_energy,
        highs_energy
    );
}

#[test]
fn audio_binding_uses_track_relative_time_and_duration_window() {
    let loader = Arc::new(DefaultAssetLoader);
    let mut director = Director::new(640, 360, 30, loader, RenderMode::Preview, None);

    let root_id = director.scene.add_node(Box::new(BoxNode::new()));
    director.timeline.push(TimelineItem {
        scene_root: root_id,
        name: None,
        start_time: 0.0,
        duration: 5.0,
        z_index: 0,
        audio_tracks: vec![],
    });

    let sample_rate = director.audio_mixer.sample_rate as usize;
    let freq = 100.0_f32; // Bass band
    let mut samples = Vec::with_capacity(sample_rate * 2);
    for i in 0..sample_rate {
        let t = i as f32 / sample_rate as f32;
        let v = (2.0 * PI * freq * t).sin();
        samples.push(v);
        samples.push(v);
    }

    let track_id = director.add_global_audio(samples);
    if let Some(track) = director.audio_mixer.get_track_mut(track_id) {
        track.start_time = 2.0;
        track.duration = Some(1.0);
    }

    if let Some(node) = director.scene.get_node_mut(root_id) {
        node.audio_bindings.push(AudioBinding {
            track_id,
            band: "bass".to_string(),
            property: "x".to_string(),
            min_value: 0.0,
            max_value: 200.0,
            smoothing: 0.0,
            prev_value: 0.0,
        });
    }

    // Before track start: no energy contribution.
    director.update(1.5);
    let before_start = director
        .scene
        .get_node(root_id)
        .unwrap()
        .transform
        .translate_x
        .current_value;
    assert!(
        before_start.abs() < 1e-5,
        "Expected no audio reactivity before start, got {}",
        before_start
    );

    // During active window: reactive value should move above min.
    director.update(2.1);
    let during_active = director
        .scene
        .get_node(root_id)
        .unwrap()
        .transform
        .translate_x
        .current_value;
    assert!(
        during_active > 0.001,
        "Expected positive reactive value during active track window, got {}",
        during_active
    );

    // After track duration ends: returns to min.
    director.update(3.2);
    let after_end = director
        .scene
        .get_node(root_id)
        .unwrap()
        .transform
        .translate_x
        .current_value;
    assert!(
        after_end.abs() < 1e-5,
        "Expected no reactivity after track end, got {}",
        after_end
    );
}
