# Slint UI Migration Plan — VNOX Client

## Current Architecture (egui)

```
main.rs
  └─ eframe::run_native() → VnoxApp::update() called every frame
       ├─ titlebar::show()
       ├─ sidebar::guild_bar()
       ├─ sidebar::show()
       ├─ members::show()
       ├─ CentralPanel:
       │    ├─ Disconnected → connect::show()
       │    └─ Connected    → chat::show()
       │         ├─ dm::dm_chat()
       │         ├─ friends::show()
       │         └─ text chat + voice panel
       └─ settings::show_window()
```

State: `UiState` — single large struct (131+ lines) with conn, channels, messages, DMs, guilds, friends, audio settings, voice state. Mutated directly in place every frame.

UI: 80+ Rust files in `ui/`, all immediate-mode egui. Every widget redrawn every frame.

---

## Target Architecture (Slint)

```
main.rs
  └─ slint::MainWindow::new() → .run()
       │
       ├─ .slint files (declarative UI)
       │   ├─ main.slint     — window layout
       │   ├─ sidebar.slint  — guild bar + channel list
       │   ├─ chat.slint     — message list + input bar
       │   ├─ voice.slint    — voice channel panel
       │   ├─ connect.slint  — connect/bookmarks screen
       │   ├─ settings.slint — settings dialog
       │   └─ theme.slint    — color/font/spacing definitions
       │
       └─ Rust code (logic only)
           ├─ app.rs          — VnoxApp, init, event wiring
           ├─ bridge.rs       — UiState ↔ Slint property sync
           ├─ state/          — UiState (unchanged, slimmed)
           └─ net/ audio/     — no changes
```

Key difference: Slint only re-renders changed properties. UI is declared once in `.slint` files. Rust code pushes state updates via `in-out property` setters.

---

## Migration Phases

### Phase 1 — Infrastructure (day 1)

1. Add `slint` crate to `Cargo.toml`:
   ```toml
   slint = { version = "1", features = ["backend-winit", "compat-1-2"] }
   ```

2. Create `ui/main.slint` — empty window with a placeholder text

3. Rewrite `main.rs`:
   - Remove `eframe::run_native()`
   - Create `slint::MainWindow`
   - Spawn tokio runtime alongside slint event loop

4. Keep all egui code in repo (deleted only at the end)

**Result:** App boots in Slint window, shows placeholder. All real UI still in egui waiting to be ported.

---

### Phase 2 — Sidebar (day 2)

Sidebar is the most isolated component — no complex interaction, just a list of channels + guild bar + user bar.

```slint
// theme.slint
export global Theme {
    in-out property <color> bg-surface: #1e1e2e;
    in-out property <color> bg-base: #181825;
    in-out property <color> accent: #cba6f7;
    in-out property <color> text-primary: #cdd6f4;
    in-out property <length> radius: 8px;
}

// sidebar.slint
component Sidebar { /* guild icons, channel list, user bar */ }
```

Rust bridge:
```rust
fn sync_sidebar(window: &MainWindow, state: &UiState) {
    let channels: Vec<Channel> = state.channels.iter().map(|c| Channel { .. }).collect();
    window.set_channels(ModelRc::from(VecModel::from(channels)));
    window.on_channel_selected(|id| { /* net.send(JoinChannel { id }) */ });
}
```

**Files to create:** `ui/theme.slint`, `ui/sidebar.slint`
**Files to remove (end of phase):** `ui/sidebar/mod.rs`, `ui/sidebar/channel_list/`, `ui/sidebar/sections/`, `ui/sidebar/guild_bar/`

---

### Phase 3 — Chat (days 3-4)

The complex one: text messages, reactions, editing, typing indicators, reply chain, DMs, friends list.

```slint
component ChatPanel {
    in-out property <[Message]> messages;
    in-out property <string> input-text;
    signal send-message(string);
    signal load-history(string);

    VerticalLayout {
        ListView { /* scrollable messages */ }
        HorizontalLayout {
            LineEdit { text <=> input-text; }
            Button { text: "Send"; }
        }
    }
}
```

**Sub-steps:**
1. Text chat panel (messages + input)
2. Message context menu (react/edit/delete/reply/copy)
3. DM panel
4. Friends list + requests
5. Voice channel panel (speaking indicators, members)
6. Member list (right sidebar)

**Files to create:** `ui/chat.slint`, `ui/voice.slint`
**Files to remove (end of phase):** `ui/chat/`, `ui/members/`

---

### Phase 4 — Connect Screen (day 4)

```slint
component ConnectScreen {
    in-out property <[Bookmark]> bookmarks;
    in-out property <string> address-input;
    signal connect(string);
    signal remove-bookmark(int);
}
```

**Files to create:** `ui/connect.slint`
**Files to remove (end of phase):** `ui/connect/`

---

### Phase 5 — Settings + Polish (day 5)

Settings dialog with tabs: Audio, Network, Identity, Appearance.

**Files to create:** `ui/settings.slint`
**Files to remove (end of phase):** `ui/settings/`, `ui/theme/`, `ui/state/` (partially)

Final cleanup: remove `eframe` from `Cargo.toml`, delete all `ui/*mod.rs` that re-export egui components.

---

## Data Flow

```
┌─────────────────────────────────────────────────────────┐
│                    slint::MainWindow                     │
│  ┌───────────────────┐  ┌────────────────────────────┐  │
│  │   .slint UI        │  │   Rust Backend             │  │
│  │                    │  │                            │  │
│  │  property <value>  │←──→  window.set_value()       │  │
│  │  signal event()    │───→  window.on_event(cb)      │  │
│  └───────────────────┘  └────────────────────────────┘  │
│                                      │                   │
│                                      ▼                   │
│                              ┌──────────────┐           │
│                              │   UiState     │           │
│                              │  (Rust struct)│           │
│                              └──────────────┘           │
│                                      │                   │
│                                      ▼                   │
│                              ┌──────────────┐           │
│                              │  NetHandle    │           │
│                              │  (tokio task) │           │
│                              └──────────────┘           │
└─────────────────────────────────────────────────────────┘
```

The bridge pattern:
- `NetHandle` receives events from gateway → updates `UiState`
- Every 16ms (60fps) or on event, `sync_ui()` reads `UiState` and calls `window.set_*()` methods
- User clicks in Slint → `signal` fires → Rust callback → `net.send(NetCommand::...)`

---

## Dependency changes

| Dependency | Action |
|------------|--------|
| `eframe`   | Remove (end of Phase 5) |
| `egui`     | Remove (end of Phase 5) |
| `slint`    | Add (Phase 1) |
| `tokio`    | Keep |
| `cpal`     | Keep |
| `rodio`    | Keep |
| `opus`     | Keep |

---

## What stays the same

- **`net/`** — all networking code, `NetHandle`, `NetCommand`, `NetEvent`
- **`audio/`** — capture/playback pipeline, Opus encode/decode, RNNoise
- **`identity.rs`**, **`identity_vault.rs`** — keypair management
- **`jitter/`** — jitter buffer (copied from voice-node)

---

## Timeline estimate

| Phase | Scope | Time |
|-------|-------|------|
| 1 | Infrastructure (slint init, window, wiring) | 1 day |
| 2 | Sidebar (guild bar + channel list + user bar) | 1 day |
| 3 | Chat + DMs + Friends + Voice panel | 2 days |
| 4 | Connect screen + bookmarks | 0.5 day |
| 5 | Settings + theme + cleanup | 0.5 day |
| **Total** | | **~5 days** |
