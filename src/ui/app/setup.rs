use tracing::{info, warn};
use vnox_client::audio::{self, PipelineConfig};
use vnox_client::net::voice;

use super::VnoxApp;

impl VnoxApp {
    pub(crate) fn pipeline_config(&self, channel_id: &str) -> PipelineConfig {
        PipelineConfig {
            channel_key: voice::channel_key(channel_id),
            mic_device_index: self.ui.mic_device,
            output_device_index: self.ui.output_device,
            voice_bitrate_kbps: self.ui.voice_bitrate,
            packet_interval: self.ui.packet_interval,
        }
    }

    pub(crate) fn start_voice(&mut self, channel_id: &str) {
        let cfg = self.pipeline_config(channel_id);
        if self.voice_channel.as_deref() == Some(channel_id)
            && self.audio.is_some()
            && self.last_pipeline_config.as_ref() == Some(&cfg)
        {
            return;
        }
        self.audio = None;
        match audio::start(cfg.clone()) {
            Ok(pipeline) => {
                info!("voice active on channel {channel_id}");
                self.audio = Some(pipeline);
                self.voice_channel = Some(channel_id.to_string());
                self.last_pipeline_config = Some(cfg);
                self.ui.voice = crate::state::VoiceUiState::Active {
                    channel: channel_id.to_string(),
                };
                if self.ui.voice_joined_at.is_none() {
                    self.ui.voice_joined_at = Some(std::time::Instant::now());
                }
                self.ui.status_msg = None;
            }
            Err(e) => {
                warn!("audio start failed: {e}");
                self.voice_channel = None;
                self.last_pipeline_config = None;
                self.ui.voice = crate::state::VoiceUiState::Error(e.to_string());
                self.ui.status_msg = Some(format!("microphone error: {e}"));
            }
        }
    }
}
