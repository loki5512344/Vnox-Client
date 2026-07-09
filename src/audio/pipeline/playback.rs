use crate::jitter::{BufferedPacket, JitterBuffer};
use anyhow::Result;
use cpal::traits::{DeviceTrait, StreamTrait};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc::TryRecvError,
    Arc, Mutex,
};
use std::time::{Duration, Instant};
use tracing::{info, warn};

use crate::audio::config::{self, AudioConfig, OPUS_RATE};
use crate::audio::IncomingVoice;

#[derive(Debug, Clone)]
pub struct JitterConfig {
    pub target_ms: u32,
    pub adaptive: bool,
}

pub fn run(
    device: cpal::Device,
    cfg: AudioConfig,
    opus_rx: std::sync::mpsc::Receiver<IncomingVoice>,
    output_volume: Arc<Mutex<f32>>,
    jitter_cfg: Arc<Mutex<JitterConfig>>,
    last_remote_voice_ms: Arc<AtomicU64>,
) -> Result<()> {
    let mut decoder = opus::Decoder::new(OPUS_RATE, opus::Channels::Mono)?;
    let pb: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let pb2 = pb.clone();
    let vol2 = output_volume.clone();
    let out_channels = cfg.stream.channels as usize;
    let needs_resample = cfg.sample_rate != OPUS_RATE;
    let opus_frame = config::frame_samples(OPUS_RATE);

    let stream = device.build_output_stream(
        cfg.stream,
        move |out: &mut [f32], _| {
            let gain = (*vol2.lock().unwrap() / 100.0).clamp(0.0, 2.0);
            let mut buf = pb2.lock().unwrap();
            if out_channels == 1 {
                let n = buf.len().min(out.len());
                for (o, s) in out[..n].iter_mut().zip(buf[..n].iter()) {
                    *o = s * gain;
                }
                buf.drain(..n);
                out[n..].fill(0.0);
            } else {
                let frames = out.len() / out_channels;
                for i in 0..frames {
                    let sample = if i < buf.len() { buf[i] * gain } else { 0.0 };
                    for ch in 0..out_channels {
                        out[i * out_channels + ch] = sample;
                    }
                }
                let drain_n = frames.min(buf.len());
                buf.drain(..drain_n);
            }
        },
        |e| tracing::error!("output: {e}"),
        None,
    )?;
    stream.play()?;
    info!("playback started ({} Hz)", cfg.sample_rate);

    let clock = Instant::now();
    let initial = jitter_cfg.lock().unwrap();
    let mut jitter = JitterBuffer::new(initial.target_ms, initial.adaptive);
    drop(initial);
    let mut stream_open = true;

    while stream_open || !jitter.is_empty() {
        let cfg_now = jitter_cfg.lock().unwrap().clone();
        jitter.set_target_ms(cfg_now.target_ms);
        if jitter.is_adaptive() != cfg_now.adaptive {
            let target = jitter.target_ms();
            let now_ms = clock.elapsed().as_millis() as u64;
            let mut new_jb = JitterBuffer::new(target, cfg_now.adaptive);
            while let Some(mut pkt) = jitter.pop_ready(now_ms) {
                pkt.arrived_at = now_ms;
                new_jb.push(pkt);
            }
            jitter = new_jb;
            tracing::info!(adaptive = cfg_now.adaptive, "jitter: adaptive mode toggled");
        }
        loop {
            match opus_rx.try_recv() {
                Ok(pkt) => {
                    last_remote_voice_ms.store(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0),
                        Ordering::Relaxed,
                    );
                    jitter.push(BufferedPacket {
                        voice_seq: pkt.voice_seq,
                        timestamp: pkt.timestamp,
                        channel_id: pkt.channel_id,
                        opus_data: pkt.opus_data,
                        arrived_at: clock.elapsed().as_millis() as u64,
                    });
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    stream_open = false;
                    break;
                }
            }
        }
        let now_ms = clock.elapsed().as_millis() as u64;
        while let Some(pkt) = jitter.pop_ready(now_ms) {
            decode_into_playback(
                &mut decoder,
                &pkt.opus_data,
                opus_frame,
                needs_resample,
                cfg.sample_rate,
                &pb,
            );
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    Ok(())
}

fn decode_into_playback(
    decoder: &mut opus::Decoder,
    data: &[u8],
    opus_frame: usize,
    needs_resample: bool,
    device_rate: u32,
    pb: &Arc<Mutex<Vec<f32>>>,
) {
    let mut pcm = vec![0f32; opus_frame];
    match decoder.decode_float(data, &mut pcm, false) {
        Ok(n) => {
            pcm.truncate(n);
            let samples = if needs_resample {
                config::resample_rate(&pcm, OPUS_RATE, device_rate)
            } else {
                pcm
            };
            pb.lock().unwrap().extend_from_slice(&samples);
        }
        Err(e) => warn!("decode: {e}"),
    }
}
