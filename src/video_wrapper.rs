// Conditional re-export or mock of video-rs types

#[cfg(feature = "video-rs")]
mod real {
    use anyhow::Result;
    use video_rs::ffmpeg::{self, format, codec, software, ChannelLayout};
    use video_rs::{Time, Location as Locator};
    use ndarray::Array3;

    pub struct EncoderSettings {
        pub width: usize,
        pub height: usize,
        pub sample_rate: i32,
    }

    impl EncoderSettings {
        pub fn preset_h264_yuv420p(w: usize, h: usize, _b: bool) -> Self {
             Self { width: w, height: h, sample_rate: 48000 }
        }
    }

    pub struct Encoder {
        output: format::context::Output,
        video_idx: usize,
        audio_idx: usize,
        video_encoder: codec::encoder::video::Encoder,
        audio_encoder: codec::encoder::audio::Encoder,
        scaler: software::scaling::Context,
        audio_buffer: Vec<f32>,
        audio_samples_processed: i64,
    }

    impl Encoder {
        pub fn new(dest: &Locator, settings: EncoderSettings) -> Result<Self> {
            ffmpeg::init().unwrap();

            let path = match dest {
                Locator::File(p) => p,
                _ => return Err(anyhow::anyhow!("Network not supported")),
            };

            let mut output = format::output(&path)?;

            // Video Setup
            let global_header = output.format().flags().contains(format::flag::Flags::GLOBAL_HEADER);
            let codec_v = codec::encoder::find(codec::Id::H264).ok_or(anyhow::anyhow!("H264 not found"))?;
            let mut v_encoder = codec::context::Context::new_with_codec(codec_v).encoder().video()?;

            v_encoder.set_height(settings.height as u32);
            v_encoder.set_width(settings.width as u32);
            v_encoder.set_aspect_ratio((settings.height as i32, settings.width as i32));
            v_encoder.set_format(format::Pixel::YUV420P);
            v_encoder.set_time_base((1, 90000));

            if global_header {
                v_encoder.set_flags(codec::flag::Flags::GLOBAL_HEADER);
            }

            let mut v_encoder = v_encoder.open_as(codec_v)?;
            let mut o_stream_v = output.add_stream(codec_v)?;
            o_stream_v.set_parameters(&v_encoder);
            let video_idx = o_stream_v.index();

            // Audio Setup
            let codec_a = codec::encoder::find(codec::Id::AAC).ok_or(anyhow::anyhow!("AAC not found"))?;
            let mut a_encoder = codec::context::Context::new_with_codec(codec_a).encoder().audio()?;

            a_encoder.set_rate(settings.sample_rate);
            a_encoder.set_channel_layout(ChannelLayout::STEREO);
            a_encoder.set_channels(2);
            a_encoder.set_format(format::Sample::F32(format::sample::Type::Planar));
            a_encoder.set_time_base((1, settings.sample_rate));

            if global_header {
                a_encoder.set_flags(codec::flag::Flags::GLOBAL_HEADER);
            }

            let mut a_encoder = a_encoder.open_as(codec_a)?;
            let mut o_stream_a = output.add_stream(codec_a)?;
            o_stream_a.set_parameters(&a_encoder);
            let audio_idx = o_stream_a.index();

            // Scaler
            let scaler = software::scaling::Context::get(
                format::Pixel::RGBA,
                settings.width as u32,
                settings.height as u32,
                format::Pixel::YUV420P,
                settings.width as u32,
                settings.height as u32,
                software::scaling::flag::Flags::BILINEAR,
            )?;

            output.write_header()?;

            Ok(Self {
                output,
                video_idx,
                audio_idx,
                video_encoder: v_encoder,
                audio_encoder: a_encoder,
                scaler,
                audio_buffer: Vec::new(),
                audio_samples_processed: 0,
            })
        }

        fn write_video_packets(&mut self) -> Result<()> {
             let mut packet = codec::packet::Packet::empty();
             while self.video_encoder.receive_packet(&mut packet).is_ok() {
                 packet.set_stream(self.video_idx);
                 packet.rescale_ts(self.video_encoder.time_base(), self.output.stream(self.video_idx).unwrap().time_base());
                 packet.write_interleaved(&mut self.output)?;
             }
             Ok(())
        }

        fn write_audio_packets(&mut self) -> Result<()> {
             let mut packet = codec::packet::Packet::empty();
             while self.audio_encoder.receive_packet(&mut packet).is_ok() {
                 packet.set_stream(self.audio_idx);
                 packet.rescale_ts(self.audio_encoder.time_base(), self.output.stream(self.audio_idx).unwrap().time_base());
                 packet.write_interleaved(&mut self.output)?;
             }
             Ok(())
        }

        pub fn encode(&mut self, frame_array: &Array3<u8>, time: Time) -> Result<()> {
            let (h, w, c) = frame_array.dim();
            assert_eq!(c, 4);

            let mut frame = ffmpeg::util::frame::Video::new(format::Pixel::RGBA, w as u32, h as u32);

            let stride = frame.stride(0);
            let width_bytes = w * 4;
            let src = frame_array.as_slice().unwrap();

            if stride == width_bytes {
                frame.data_mut(0)[..src.len()].copy_from_slice(src);
            } else {
                for y in 0..h {
                    let src_row = &src[y*width_bytes .. (y+1)*width_bytes];
                    let dest_row = &mut frame.data_mut(0)[y*stride .. y*stride + width_bytes];
                    dest_row.copy_from_slice(src_row);
                }
            }

            let mut yuv_frame = ffmpeg::util::frame::Video::new(format::Pixel::YUV420P, w as u32, h as u32);
            self.scaler.run(&frame, &mut yuv_frame)?;

            let secs = time.as_secs_f64();
            let pts = (secs * 90000.0) as i64;
            yuv_frame.set_pts(Some(pts));

            self.video_encoder.send_frame(&yuv_frame)?;
            self.write_video_packets()?;
            Ok(())
        }

        pub fn encode_audio(&mut self, samples: &[f32], _time: Time) -> Result<()> {
            self.audio_buffer.extend_from_slice(samples);

            let frame_size = self.audio_encoder.frame_size() as usize;
            let channels = 2;
            let chunk_size = frame_size * channels;

            while self.audio_buffer.len() >= chunk_size {
                 let chunk: Vec<f32> = self.audio_buffer.drain(0..chunk_size).collect();

                 let mut frame = ffmpeg::util::frame::Audio::new(format::Sample::F32(format::sample::Type::Planar), frame_size, ChannelLayout::STEREO);

                 let mut left = Vec::with_capacity(frame_size);
                 let mut right = Vec::with_capacity(frame_size);
                 for i in 0..frame_size {
                     left.push(chunk[i*2]);
                     right.push(chunk[i*2+1]);
                 }

                 frame.plane_mut(0).copy_from_slice(&left);
                 frame.plane_mut(1).copy_from_slice(&right);

                 frame.set_pts(Some(self.audio_samples_processed));
                 self.audio_samples_processed += frame_size as i64;

                 self.audio_encoder.send_frame(&frame)?;
                 self.write_audio_packets()?;
            }
            Ok(())
        }

        pub fn finish(mut self) -> Result<()> {
             self.video_encoder.send_eof()?;
             self.write_video_packets()?;

             if !self.audio_buffer.is_empty() {
                  let frame_size = self.audio_encoder.frame_size() as usize;
                  let channels = 2;
                  let needed = frame_size * channels - self.audio_buffer.len();
                  for _ in 0..needed {
                      self.audio_buffer.push(0.0);
                  }
                  let chunk = std::mem::take(&mut self.audio_buffer);

                  let mut frame = ffmpeg::util::frame::Audio::new(format::Sample::F32(format::sample::Type::Planar), frame_size, ChannelLayout::STEREO);

                  let mut left = Vec::with_capacity(frame_size);
                  let mut right = Vec::with_capacity(frame_size);
                  for i in 0..frame_size {
                     left.push(chunk[i*2]);
                     right.push(chunk[i*2+1]);
                  }
                  frame.plane_mut(0).copy_from_slice(&left);
                  frame.plane_mut(1).copy_from_slice(&right);
                  frame.set_pts(Some(self.audio_samples_processed));

                  self.audio_encoder.send_frame(&frame)?;
                  self.write_audio_packets()?;
             }

             self.audio_encoder.send_eof()?;
             self.write_audio_packets()?;

             self.output.write_trailer()?;
             Ok(())
        }
    }
}

#[cfg(feature = "video-rs")]
pub use real::*;
#[cfg(feature = "video-rs")]
pub use video_rs::{Decoder, Location as Locator, Time, Frame, ffmpeg};

#[cfg(not(feature = "video-rs"))]
pub mod mock {
    use std::path::Path;
    use anyhow::Result;
    use ndarray::Array3;

    #[derive(Debug)]
    pub struct Decoder;
    impl Decoder {
        pub fn new(_path: &Path) -> Result<Self, String> { Ok(Self) }
        pub fn decode(&mut self) -> Result<(Time, Array3<u8>), anyhow::Error> {
             Ok((Time, Array3::zeros((10, 10, 3))))
        }
        pub fn seek(&mut self, _ms: i64) -> Result<(), anyhow::Error> { Ok(()) }
    }

    pub struct Encoder;
    impl Encoder {
        pub fn new(_dest: &Locator, _settings: EncoderSettings) -> Result<Self> { Ok(Self) }
        pub fn finish(self) -> Result<()> { Ok(()) }

        pub fn encode(&mut self, _frame: &Array3<u8>, _time: Time) -> Result<()> {
            Ok(())
        }

        pub fn encode_audio(&mut self, _samples: &[f32], _time: Time) -> Result<()> {
            Ok(())
        }
    }

    pub struct Locator;
    impl From<std::path::PathBuf> for Locator {
        fn from(_: std::path::PathBuf) -> Self { Self }
    }

    pub struct EncoderSettings;
    impl EncoderSettings {
        pub fn preset_h264_yuv420p(_w: usize, _h: usize, _b: bool) -> Self { Self }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Time;
    impl Time {
        pub fn from_nth_of_second(_n: usize, _fps: u32) -> Self { Self }
        pub fn from_secs(_s: f64) -> Self { Self }
        pub fn from_secs_f64(_s: f64) -> Self { Self }
        pub fn as_secs_f64(&self) -> f64 { 0.0 }
    }

    pub struct Frame;
}

#[cfg(not(feature = "video-rs"))]
pub use mock::*;
