use anyhow::{Result, Context};
use crate::animation::Animated;
use std::io::Cursor;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(Clone, Debug)]
pub struct AudioTrack {
    /// Interleaved stereo samples (L, R, L, R...). Normalized -1.0 to 1.0.
    pub samples: Vec<f32>,
    /// Volume multiplier (animated).
    pub volume: Animated<f32>,
    /// Start time in global seconds.
    pub start_time: f64,
    /// Optional clipping duration (in seconds).
    pub duration: Option<f64>,
    /// Whether to loop the audio.
    pub loop_audio: bool,
}

#[derive(Debug)]
pub struct AudioMixer {
    pub tracks: Vec<Option<AudioTrack>>,
    pub sample_rate: u32,
}

impl AudioMixer {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            tracks: Vec::new(),
            sample_rate,
        }
    }

    pub fn add_track(&mut self, track: AudioTrack) -> usize {
        // Find empty slot
        if let Some(idx) = self.tracks.iter().position(|t| t.is_none()) {
            self.tracks[idx] = Some(track);
            idx
        } else {
            let idx = self.tracks.len();
            self.tracks.push(Some(track));
            idx
        }
    }

    pub fn get_track_mut(&mut self, id: usize) -> Option<&mut AudioTrack> {
        self.tracks.get_mut(id).and_then(|t| t.as_mut())
    }

    /// Mixes audio for a specific time window.
    /// Returns interleaved stereo samples.
    pub fn mix(&mut self, samples_needed: usize, start_time: f64) -> Vec<f32> {
        // Output buffer (stereo)
        let mut output = vec![0.0; samples_needed * 2];
        let dt_per_sample = 1.0 / self.sample_rate as f64;

        for track_opt in self.tracks.iter_mut() {
            if let Some(track) = track_opt {
                // Determine if track is active
                // For looping or simple playback, calculate relative time

                track.volume.update(start_time);
                let vol = track.volume.current_value;

                for i in 0..samples_needed {
                    let t = start_time + i as f64 * dt_per_sample;
                    let relative_time = t - track.start_time;

                    // Check start
                    if relative_time < 0.0 {
                        continue;
                    }

                    // Check duration (clipping)
                    if let Some(dur) = track.duration {
                        if relative_time >= dur {
                            if track.loop_audio {
                                // If looping AND hard clipped? Usually looping means it loops *within* the clip?
                                // Or does it mean the source loops?
                                // RFC: "Scene Audio: Starts at scene.start_time. It is hard clipped to the scene duration."
                                // "Global Audio: ... plays independently".
                                // If hard clipped, we stop.
                                continue;
                            } else {
                                continue;
                            }
                        }
                    }

                    // Determine sample index
                    // If looping, we wrap the sample index relative to the source length.

                    let mut sample_idx = (relative_time * self.sample_rate as f64) as usize;

                    // Convert to stereo frame index
                    let frame_count = track.samples.len() / 2;

                    if track.loop_audio {
                         sample_idx %= frame_count;
                    } else if sample_idx >= frame_count {
                         continue;
                    }

                    let left = track.samples[sample_idx * 2];
                    let right = track.samples[sample_idx * 2 + 1];

                    output[i * 2] += left * vol;
                    output[i * 2 + 1] += right * vol;
                }
            }
        }

        // Clamp
        for s in output.iter_mut() {
            *s = s.clamp(-1.0, 1.0);
        }

        output
    }
}

pub fn load_audio_bytes(data: &[u8], target_sample_rate: u32) -> Result<Vec<f32>> {
    let mss = MediaSourceStream::new(Box::new(Cursor::new(data.to_vec())), Default::default());
    let hint = Hint::new();
    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();
    let decoder_opts = DecoderOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .context("Unsupported format")?;

    let mut format = probed.format;
    let track = format.default_track().context("No track found")?;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &decoder_opts)
        .context("Unsupported codec")?;

    let track_id = track.id;
    let source_rate = track.codec_params.sample_rate.unwrap_or(44100);

    let mut samples = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(_)) => break,
            Err(symphonia::core::errors::Error::ResetRequired) => break,
            Err(_) => break,
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = *decoded.spec();
                let duration = decoded.capacity() as u64;
                let mut buf = symphonia::core::audio::SampleBuffer::<f32>::new(duration, spec);
                buf.copy_interleaved_ref(decoded);

                let buf_samples = buf.samples();
                let channels = spec.channels.count();

                if channels == 1 {
                    for s in buf_samples {
                        samples.push(*s);
                        samples.push(*s);
                    }
                } else if channels >= 2 {
                    // Taking first two channels if more than 2
                    for chunk in buf_samples.chunks(channels) {
                        samples.push(chunk[0]);
                        samples.push(chunk[1]);
                    }
                }
            }
            Err(_) => break,
        }
    }

    if source_rate != target_sample_rate {
        // Linear Resample
        let ratio = source_rate as f64 / target_sample_rate as f64;
        // Total stereo frames
        let source_frames = samples.len() / 2;
        let new_frames = (source_frames as f64 / ratio).ceil() as usize;
        let mut resampled = Vec::with_capacity(new_frames * 2);

        for i in 0..new_frames {
            let pos = i as f64 * ratio;
            let idx = pos.floor() as usize;
            let frac = (pos - pos.floor()) as f32;

            let idx1 = idx;
            let idx2 = idx + 1;

            if idx1 < source_frames {
                let l1 = samples[idx1 * 2];
                let r1 = samples[idx1 * 2 + 1];

                let (l2, r2) = if idx2 < source_frames {
                    (samples[idx2 * 2], samples[idx2 * 2 + 1])
                } else {
                    (0.0, 0.0) // Or duplicate last? 0.0 is fine for padding
                };

                let l = l1 + (l2 - l1) * frac;
                let r = r1 + (r2 - r1) * frac;
                resampled.push(l);
                resampled.push(r);
            } else {
                 resampled.push(0.0);
                 resampled.push(0.0);
            }
        }
        Ok(resampled)
    } else {
        Ok(samples)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixing_logic() {
        let mut mixer = AudioMixer::new(48000);
        let track = AudioTrack {
            samples: vec![0.5; 48000 * 2], // 1 sec stereo
            volume: Animated::new(1.0),
            start_time: 0.0,
            duration: None,
            loop_audio: false,
        };
        mixer.add_track(track);

        let mixed = mixer.mix(100, 0.0);
        assert_eq!(mixed.len(), 200);
        // Check first sample (Left)
        assert!((mixed[0] - 0.5).abs() < 1e-5);
    }
}
