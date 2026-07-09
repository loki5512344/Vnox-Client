mod frame;
mod net_poll;
mod setup;

use vnox_client::{
    audio::{AudioPipeline, AudioSync},
    identity::Identity,
    net::{NetCommand, NetHandle},
};

use crate::keybind;
use crate::state::{ConnState, VoiceUiState};
use crate::theme::apply_visuals;

pub struct VnoxApp {
    pub(crate) identity: Identity,
    pub(crate) net: NetHandle,
    pub(crate) ui: crate::state::UiState,
    pub(crate) audio: Option<AudioPipeline>,
    pub(crate) voice_channel: Option<String>,
    pub(crate) last_pipeline_config: Option<vnox_client::audio::PipelineConfig>,
}

impl VnoxApp {
    pub fn new(cc: &eframe::CreationContext, identity: Identity, net: NetHandle) -> Self {
        apply_visuals(&cc.egui_ctx);
        Self {
            identity,
            net,
            ui: crate::state::UiState::default(),
            audio: None,
            voice_channel: None,
            last_pipeline_config: None,
        }
    }

    pub(crate) fn stop_voice(&mut self) {
        self.audio = None;
        self.voice_channel = None;
        self.last_pipeline_config = None;
        self.ui.voice = VoiceUiState::Off;
        self.ui.voice_joined_at = None;
    }

    pub(crate) fn sync_voice_audio(&mut self) {
        if !matches!(self.ui.conn, ConnState::Connected { .. }) {
            self.stop_voice();
            return;
        }
        let active_voice = self
            .ui
            .active_channel
            .as_ref()
            .and_then(|id| self.ui.channels.iter().find(|c| &c.id == id))
            .filter(|c| c.kind == "voice")
            .map(|c| c.id.clone());

        match active_voice {
            Some(id) => self.start_voice(&id),
            None => self.stop_voice(),
        }
    }

    pub(crate) fn sync_audio_controls(&mut self, ctx: &eframe::egui::Context) {
        let Some(pipeline) = self.audio.as_ref() else {
            return;
        };
        let ptt_held = if self.ui.vad_mode == 0 {
            keybind::binding_held(ctx, &self.ui.kb_ptt)
        } else {
            false
        };
        pipeline.sync_controls(AudioSync {
            mic_enabled: self.ui.mic_enabled,
            vad_mode: self.ui.vad_mode,
            vad_threshold: self.ui.vad_threshold,
            input_volume: self.ui.input_volume,
            ptt_held,
            output_volume: self.ui.output_volume,
            jitter_size: self.ui.jitter_size,
            adaptive_buffer: self.ui.adaptive_buffer,
            voice_bitrate_kbps: self.ui.voice_bitrate,
            noise_suppress: self.ui.noise_suppress,
        });

        if let Some(ch) = self.voice_channel.clone() {
            let cfg = self.pipeline_config(&ch);
            if self.last_pipeline_config.as_ref() != Some(&cfg) {
                self.start_voice(&ch);
            }
        }
    }

    pub(crate) fn handle_voice_keybinds(&mut self, ctx: &eframe::egui::Context) {
        if ctx.wants_keyboard_input() {
            return;
        }
        if keybind::binding_pressed(ctx, &self.ui.kb_mute) {
            self.ui.mic_enabled = !self.ui.mic_enabled;
        }
        if keybind::binding_pressed(ctx, &self.ui.kb_deafen) {
            self.ui.deafen = !self.ui.deafen;
        }
    }

    pub(crate) fn flush_encoded_voice(&mut self) {
        let Some(pipeline) = self.audio.as_ref() else {
            return;
        };
        while let Some(frame) = pipeline.poll_encoded() {
            let net = self.net.clone();
            tokio::spawn(async move {
                net.send(NetCommand::SendVoice {
                    channel_id: frame.channel_id,
                    voice_seq: frame.voice_seq,
                    timestamp: frame.timestamp,
                    opus_data: frame.data,
                })
                .await;
            });
        }
    }
}
