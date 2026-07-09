mod capture;
mod playback;

use cpal::traits::DeviceTrait;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    mpsc::{self, Receiver, Sender},
};
use tracing::info;

use crate::audio::{
    AudioSync, EncodedFrame, IncomingVoice, PipelineConfig, config, devices,
    processing::TransmitGate,
};

pub use playback::JitterConfig;

pub struct AudioPipeline {
    encoded_rx: Receiver<EncodedFrame>,
    voice_tx: Sender<IncomingVoice>,
    _encoded_tx: Sender<EncodedFrame>,
    gate: Arc<Mutex<TransmitGate>>,
    output_volume: Arc<Mutex<f32>>,
    jitter_cfg: Arc<Mutex<JitterConfig>>,
    bitrate_bps: Arc<AtomicU32>,
    /// Local user is currently transmitting (PTT held or VAD above threshold and mic enabled).
    local_speaking: Arc<AtomicBool>,
    /// UNIX millisecond timestamp of the last voice packet received from a remote peer.
    last_remote_voice_ms: Arc<AtomicU64>,
}

impl AudioPipeline {
    pub fn poll_encoded(&self) -> Option<EncodedFrame> {
        self.encoded_rx.try_recv().ok()
    }

    pub fn push_voice(&self, packet: IncomingVoice) {
        let _ = self.voice_tx.send(packet);
        self.last_remote_voice_ms.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            Ordering::Relaxed,
        );
    }

    /// Returns true if the local user is currently speaking / transmitting.
    pub fn local_speaking(&self) -> bool {
        self.local_speaking.load(Ordering::Relaxed)
    }

    /// Returns true if we received a voice packet from a remote peer within the
    /// last `within_ms` milliseconds.
    pub fn remote_speaking(&self, within_ms: u64) -> bool {
        let last = self.last_remote_voice_ms.load(Ordering::Relaxed);
        if last == 0 {
            return false;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        now.saturating_sub(last) <= within_ms
    }

    pub fn sync_controls(&self, sync: AudioSync) {
        let mut g = self.gate.lock().unwrap();
        g.mic_enabled = sync.mic_enabled;
        g.vad_mode = sync.vad_mode.min(2) as u8;
        g.vad_threshold = sync.vad_threshold;
        g.input_volume = sync.input_volume;
        g.ptt_held = sync.ptt_held;
        g.noise_suppress = sync.noise_suppress;
        *self.output_volume.lock().unwrap() = sync.output_volume;
        *self.jitter_cfg.lock().unwrap() = JitterConfig {
            target_ms: config::jitter_ms_from_setting(sync.jitter_size),
            adaptive: sync.adaptive_buffer,
        };
        self.bitrate_bps.store(
            config::bitrate_bps_from_kbps(sync.voice_bitrate_kbps),
            Ordering::Relaxed,
        );
    }
}

pub fn start(cfg: PipelineConfig) -> anyhow::Result<AudioPipeline> {
    let input = devices::input_at(cfg.mic_device_index)?;
    let output = devices::output_at(cfg.output_device_index)?;
    info!(
        "audio in: {}  out: {}",
        input
            .description()
            .map(|d| d.name().to_owned())
            .unwrap_or_default(),
        output
            .description()
            .map(|d| d.name().to_owned())
            .unwrap_or_default()
    );

    let in_cfg = config::AudioConfig::find(&input, true)?;
    let out_cfg = config::AudioConfig::find(&output, false)?;
    let frame_ms = config::frame_ms_from_interval(cfg.packet_interval);

    let (encoded_tx, encoded_rx) = mpsc::channel::<EncodedFrame>();
    let (voice_tx, voice_rx) = mpsc::channel::<IncomingVoice>();
    let gate = Arc::new(Mutex::new(TransmitGate::default()));
    let gate_cap = gate.clone();
    let output_volume = Arc::new(Mutex::new(90.0f32));
    let vol_play = output_volume.clone();
    let jitter_cfg = Arc::new(Mutex::new(JitterConfig {
        target_ms: 40,
        adaptive: true,
    }));
    let jitter_play = jitter_cfg.clone();
    let bitrate_bps = Arc::new(AtomicU32::new(config::bitrate_bps_from_kbps(
        cfg.voice_bitrate_kbps,
    )));
    let bitrate_cap = bitrate_bps.clone();

    let local_speaking = Arc::new(AtomicBool::new(false));
    let local_speaking_cap = local_speaking.clone();
    let last_remote_voice_ms = Arc::new(AtomicU64::new(0));
    let last_remote_voice_play = last_remote_voice_ms.clone();

    let channel_id = cfg.channel_key;
    let enc_tx = encoded_tx.clone();
    std::thread::spawn(move || {
        if let Err(e) = capture::run(
            input,
            in_cfg,
            channel_id,
            enc_tx,
            gate_cap,
            frame_ms,
            bitrate_cap,
            local_speaking_cap,
        ) {
            tracing::error!("capture: {e}");
        }
    });
    std::thread::spawn(move || {
        if let Err(e) = playback::run(
            output,
            out_cfg,
            voice_rx,
            vol_play,
            jitter_play,
            last_remote_voice_play,
        ) {
            tracing::error!("playback: {e}");
        }
    });

    Ok(AudioPipeline {
        encoded_rx,
        voice_tx,
        _encoded_tx: encoded_tx,
        gate,
        output_volume,
        jitter_cfg,
        bitrate_bps,
        local_speaking,
        last_remote_voice_ms,
    })
}
