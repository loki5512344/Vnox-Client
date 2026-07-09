pub mod config;
pub mod devices;
mod pipeline;
mod processing;

pub use devices::{DeviceLabel, DeviceLists};
pub use pipeline::AudioPipeline;
pub use pipeline::JitterConfig;
pub use processing::{DenoiseState, TransmitGate};

#[derive(Debug, Clone, Copy)]
pub struct AudioSync {
    pub mic_enabled: bool,
    pub vad_mode: usize,
    pub vad_threshold: f32,
    pub input_volume: f32,
    pub ptt_held: bool,
    pub output_volume: f32,
    pub jitter_size: usize,
    pub adaptive_buffer: bool,
    pub voice_bitrate_kbps: f32,
    pub noise_suppress: bool,
}

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub channel_key: u64,
    pub mic_device_index: usize,
    pub output_device_index: usize,
    pub voice_bitrate_kbps: f32,
    pub packet_interval: usize,
}

impl PartialEq for PipelineConfig {
    fn eq(&self, other: &Self) -> bool {
        self.channel_key == other.channel_key
            && self.mic_device_index == other.mic_device_index
            && self.output_device_index == other.output_device_index
            && self.packet_interval == other.packet_interval
    }
}

#[derive(Debug, Clone)]
pub struct IncomingVoice {
    pub voice_seq: u32,
    pub timestamp: u32,
    pub channel_id: u64,
    pub opus_data: Vec<u8>,
}

pub struct EncodedFrame {
    pub channel_id: u64,
    pub voice_seq: u32,
    pub timestamp: u32,
    pub data: Vec<u8>,
}

pub fn start(cfg: PipelineConfig) -> anyhow::Result<AudioPipeline> {
    pipeline::start(cfg)
}
