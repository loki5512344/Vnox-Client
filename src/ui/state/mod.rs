mod default;
mod models;
mod types;

pub use models::{
    Channel, ChatMessage, DmConversation, FriendState, GuildState, NodeBookmark, SettingsPage,
};
pub use types::{ConnState, TYPING_TIMEOUT, UiState, VoiceUiState};

use std::time::{Duration, Instant};

use models::bookmark_label_from_address;

impl UiState {
    pub fn refresh_audio_devices(&mut self) {
        let now = Instant::now();
        if let Some(last) = self.last_device_scan
            && now.duration_since(last) < Duration::from_millis(2500)
        {
            return;
        }
        self.last_device_scan = Some(now);

        match vnox_client::audio::devices::enumerate() {
            Ok(lists) => {
                self.input_device_labels = lists.input_labels;
                self.input_device_metas = lists.input_devices;
                self.output_device_labels = lists.output_labels;
                self.output_device_metas = lists.output_devices;
            }
            Err(e) => {
                tracing::warn!("audio device enumeration: {e}");
            }
        }

        if self.input_device_labels.is_empty() {
            self.input_device_labels.push("Default".into());
        }
        if self.output_device_labels.is_empty() {
            self.output_device_labels.push("Default".into());
        }

        self.mic_device = self
            .mic_device
            .min(self.input_device_labels.len().saturating_sub(1));
        self.output_device = self
            .output_device
            .min(self.output_device_labels.len().saturating_sub(1));
    }

    pub fn force_refresh_audio_devices(&mut self) {
        self.last_device_scan = None;
        self.refresh_audio_devices();
    }

    pub fn remove_bookmark(&mut self, index: usize) {
        if index < self.bookmarks.len() {
            self.bookmarks.remove(index);
            if self.selected_bookmark >= self.bookmarks.len() && !self.bookmarks.is_empty() {
                self.selected_bookmark = self.bookmarks.len() - 1;
            }
        }
    }

    pub fn add_bookmark_from_input(&mut self) -> bool {
        let address = self.connect_input.trim().to_string();
        if address.is_empty() || self.bookmarks.iter().any(|b| b.address == address) {
            return false;
        }
        let label = bookmark_label_from_address(&address);
        self.bookmarks.push(NodeBookmark { label, address });
        self.selected_bookmark = self.bookmarks.len().saturating_sub(1);
        true
    }

    pub fn voice_elapsed(&self) -> Option<String> {
        let secs = self.voice_joined_at?.elapsed().as_secs();
        Some(format!("{}:{:02}", secs / 60, secs % 60))
    }

    pub fn nick_for<'a>(&'a self, sender_id: &'a str) -> &'a str {
        self.user_names
            .get(sender_id)
            .map(|s| s.as_str())
            .unwrap_or(sender_id)
    }

    pub fn active_voice_channel(&self) -> Option<&Channel> {
        self.channels
            .iter()
            .find(|c| matches!(&self.voice, VoiceUiState::Active { channel } if channel == &c.id))
    }

    pub fn seed_default_channels(&mut self) {
        if !self.channels.is_empty() {
            return;
        }
        self.channels = vec![
            Channel {
                id: "general".into(),
                name: "general".into(),
                kind: "text".into(),
                members: Vec::new(),
            },
            Channel {
                id: "dev-talk".into(),
                name: "dev-talk".into(),
                kind: "text".into(),
                members: Vec::new(),
            },
            Channel {
                id: "plugins".into(),
                name: "plugins".into(),
                kind: "text".into(),
                members: Vec::new(),
            },
            Channel {
                id: "lobby".into(),
                name: "lobby".into(),
                kind: "voice".into(),
                members: Vec::new(),
            },
            Channel {
                id: "gaming".into(),
                name: "gaming".into(),
                kind: "voice".into(),
                members: Vec::new(),
            },
        ];
    }
}
