use anyhow::Result;
use cpal::traits::{DeviceTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

use crate::audio::{
    EncodedFrame,
    config::{self, AudioConfig, OPUS_RATE},
    processing::{DenoiseState, TransmitGate},
};

const MAX_OPUS: usize = 4000;

#[allow(clippy::too_many_arguments)]
pub fn run(
    device: cpal::Device,
    cfg: AudioConfig,
    channel_id: u64,
    tx: std::sync::mpsc::Sender<EncodedFrame>,
    gate: Arc<Mutex<TransmitGate>>,
    frame_ms: u32,
    bitrate_bps: Arc<AtomicU32>,
    local_speaking: Arc<AtomicBool>,
) -> Result<()> {
    let mut encoder = opus::Encoder::new(OPUS_RATE, opus::Channels::Mono, opus::Application::Voip)?;
    let mut last_bitrate_bps: u32 = 0;
    apply_bitrate(&mut encoder, &bitrate_bps, &mut last_bitrate_bps)?;

    let mut denoise = DenoiseState::new()?;

    let capture_frame = frame_samples_at(cfg.sample_rate, frame_ms);
    let opus_frame = frame_samples_at(OPUS_RATE, frame_ms);
    let needs_resample = cfg.sample_rate != OPUS_RATE;
    let in_channels = cfg.stream.channels as usize;

    let buf: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let buf2 = buf.clone();
    let mut voice_seq: u32 = 0;
    let mut timestamp: u32 = 0;

    let stream = device.build_input_stream(
        cfg.stream,
        move |data: &[f32], _| {
            let mut b = buf2.lock().unwrap();
            if in_channels <= 1 {
                b.extend_from_slice(data);
            } else {
                for frame in data.chunks(in_channels) {
                    let sum: f32 = frame.iter().sum();
                    b.push(sum / in_channels as f32);
                }
            }
        },
        |e| tracing::error!("input: {e}"),
        None,
    )?;
    stream.play()?;
    info!("capture started ({} Hz)", cfg.sample_rate);

    loop {
        std::thread::sleep(std::time::Duration::from_millis(5));
        let mut b = buf.lock().unwrap();
        while b.len() >= capture_frame {
            let raw: Vec<f32> = b.drain(..capture_frame).collect();
            drop(b);

            let frame = if needs_resample {
                config::resample_rate(&raw, cfg.sample_rate, OPUS_RATE)
            } else {
                raw
            };
            let mut pcm = if frame.len() >= opus_frame {
                frame[..opus_frame].to_vec()
            } else {
                let mut padded = frame;
                padded.resize(opus_frame, 0.0);
                padded
            };

            let gate_snapshot = gate.lock().unwrap().clone();
            TransmitGate::apply_input_volume(&mut pcm, gate_snapshot.input_volume);
            if gate_snapshot.noise_suppress {
                denoise.process(&mut pcm);
            }
            let speaking = TransmitGate::should_transmit(&gate_snapshot, &pcm);
            local_speaking.store(speaking, Ordering::Relaxed);
            if !speaking {
                b = buf.lock().unwrap();
                continue;
            }

            apply_bitrate(&mut encoder, &bitrate_bps, &mut last_bitrate_bps)?;

            let mut out = vec![0u8; MAX_OPUS];
            match encoder.encode_float(&pcm, &mut out) {
                Ok(n) => {
                    out.truncate(n);
                    if tx
                        .send(EncodedFrame {
                            channel_id,
                            voice_seq,
                            timestamp,
                            data: out,
                        })
                        .is_err()
                    {
                        return Ok(());
                    }
                    voice_seq = voice_seq.wrapping_add(1);
                    timestamp = timestamp.wrapping_add(opus_frame as u32);
                }
                Err(e) => warn!("encode: {e}"),
            }
            b = buf.lock().unwrap();
        }
    }
}

fn frame_samples_at(sample_rate: u32, frame_ms: u32) -> usize {
    (sample_rate as usize * frame_ms as usize) / 1000
}

fn apply_bitrate(
    encoder: &mut opus::Encoder,
    bitrate_bps: &AtomicU32,
    last: &mut u32,
) -> Result<()> {
    let bps = bitrate_bps.load(Ordering::Relaxed).clamp(8_000, 512_000);
    if bps == *last {
        return Ok(());
    }
    *last = bps;
    encoder.set_bitrate(opus::Bitrate::Bits(bps as i32))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::frame_samples_at;

    #[test]
    fn frame_sizes_scale_with_ms() {
        assert_eq!(frame_samples_at(48_000, 10), 480);
        assert_eq!(frame_samples_at(48_000, 20), 960);
        assert_eq!(frame_samples_at(48_000, 40), 1920);
    }
}
