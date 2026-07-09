use anyhow::{Result, anyhow};
use cpal::traits::DeviceTrait;
use cpal::{SampleFormat, StreamConfig};
use tracing::info;

pub const OPUS_RATE: u32 = 48_000;
pub const FRAME_MS: u32 = 20;

/// Maps settings `packet_interval` index to frame duration in ms.
pub fn frame_ms_from_interval(interval: usize) -> u32 {
    match interval {
        0 => 10,
        1 => 20,
        _ => 40,
    }
}

/// Maps settings `jitter_size` index to playout delay in ms.
pub fn jitter_ms_from_setting(jitter_size: usize) -> u32 {
    match jitter_size {
        0 => 20,
        1 => 40,
        _ => 80,
    }
}

pub fn bitrate_bps_from_kbps(kbps: f32) -> u32 {
    (kbps.clamp(8.0, 512.0) * 1000.0) as u32
}

pub struct AudioConfig {
    pub stream: StreamConfig,
    pub sample_rate: u32,
}

impl AudioConfig {
    pub fn find(device: &cpal::Device, is_input: bool) -> Result<Self> {
        let ranges: Vec<_> = if is_input {
            device.supported_input_configs()?.collect()
        } else {
            device.supported_output_configs()?.collect()
        };

        // Prefer 48 kHz mono F32 — matches Opus native rate.
        for r in &ranges {
            if r.channels() == 1
                && r.sample_format() == SampleFormat::F32
                && r.min_sample_rate() <= OPUS_RATE
                && r.max_sample_rate() >= OPUS_RATE
            {
                let stream = r.with_sample_rate(OPUS_RATE).config();
                info!(
                    "audio {}: {} Hz mono",
                    if is_input { "in" } else { "out" },
                    OPUS_RATE
                );
                return Ok(Self {
                    stream,
                    sample_rate: OPUS_RATE,
                });
            }
        }

        // Fallback: device native rate, resample to/from Opus in capture/playback.
        let default = if is_input {
            device.default_input_config()
        } else {
            device.default_output_config()
        }
        .map_err(|e| anyhow!("no default audio config: {e}"))?;

        let sample_rate = default.sample_rate();
        let stream: StreamConfig = default.config();
        info!(
            "audio {}: {} Hz {}ch (will resample for Opus)",
            if is_input { "in" } else { "out" },
            sample_rate,
            stream.channels
        );
        Ok(Self {
            stream,
            sample_rate,
        })
    }
}

pub fn frame_samples(sample_rate: u32) -> usize {
    (sample_rate as usize * FRAME_MS as usize) / 1000
}

/// Linear resample mono F32 buffer to target sample count.
pub fn resample_linear(input: &[f32], out_len: usize) -> Vec<f32> {
    if input.is_empty() {
        return vec![0.0; out_len];
    }
    if input.len() == out_len {
        return input.to_vec();
    }
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src = i as f64 * (input.len() - 1) as f64 / (out_len - 1).max(1) as f64;
        let lo = src.floor() as usize;
        let hi = (lo + 1).min(input.len() - 1);
        let t = src - lo as f64;
        out.push(input[lo] * (1.0 - t) as f32 + input[hi] * t as f32);
    }
    out
}

/// Resample between two sample rates (mono).
pub fn resample_rate(input: &[f32], from_hz: u32, to_hz: u32) -> Vec<f32> {
    if from_hz == to_hz {
        return input.to_vec();
    }
    let out_len = ((input.len() as u64 * to_hz as u64) / from_hz as u64) as usize;
    resample_linear(input, out_len.max(1))
}

#[cfg(test)]
mod tests {
    use super::{bitrate_bps_from_kbps, frame_ms_from_interval, jitter_ms_from_setting};

    #[test]
    fn jitter_size_maps_to_ms() {
        assert_eq!(jitter_ms_from_setting(0), 20);
        assert_eq!(jitter_ms_from_setting(1), 40);
        assert_eq!(jitter_ms_from_setting(2), 80);
    }

    #[test]
    fn packet_interval_maps_to_frame_ms() {
        assert_eq!(frame_ms_from_interval(0), 10);
        assert_eq!(frame_ms_from_interval(1), 20);
        assert_eq!(frame_ms_from_interval(2), 40);
    }

    #[test]
    fn bitrate_kbps_to_bps() {
        assert_eq!(bitrate_bps_from_kbps(64.0), 64_000);
    }
}
