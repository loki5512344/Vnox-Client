mod default;
mod models;
mod types;

pub use models::{Channel, ChatMessage, DmConversation, FriendState, GuildState, NodeBookmark};
pub use types::{ConnState, UiState, VoiceUiState};

use models::bookmark_label_from_address;

impl UiState {
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

    pub fn nick_for<'a>(&'a self, sender_id: &'a str) -> &'a str {
        self.user_names
            .get(sender_id)
            .map(|s| s.as_str())
            .unwrap_or(sender_id)
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
                guild_id: None,
                members: Vec::new(),
            },
            Channel {
                id: "dev-talk".into(),
                name: "dev-talk".into(),
                kind: "text".into(),
                guild_id: None,
                members: Vec::new(),
            },
            Channel {
                id: "plugins".into(),
                name: "plugins".into(),
                kind: "text".into(),
                guild_id: None,
                members: Vec::new(),
            },
            Channel {
                id: "lobby".into(),
                name: "lobby".into(),
                kind: "voice".into(),
                guild_id: None,
                members: Vec::new(),
            },
            Channel {
                id: "gaming".into(),
                name: "gaming".into(),
                kind: "voice".into(),
                guild_id: None,
                members: Vec::new(),
            },
        ];
    }
}
