# VNOX — Client

> Native desktop client for VNOX. Voice, text, no cloud.

[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Status: Phase 1](https://img.shields.io/badge/status-phase%201%20%E2%80%94%20implemented-yellow.svg)](docs/00-status.md)

---

## What is this?

The official desktop client for [VNOX](https://github.com/loki5512344/Vnox) — a self-hosted voice and text communication platform. Connects to a VNOX server over the LNEx protocol.

- Connects to gateway via **TCP** (text, auth, channels, guilds, DMs)
- Connects to voice node via **UDP** (encrypted Opus voice packets)
- Opus audio encoding/decoding with adaptive jitter buffer
- Native UI built with **Slint** — no Electron, no web views

---

## Features

### Implemented
- **Auth:** Ed25519 keypair generation, challenge-response login, reconnect with exponential backoff
- **Encryption:** ChaCha20-Poly1305 AEAD + X25519 ECDH key exchange
- **Text chat:** Persistent history, reactions, replies, edit, delete, typing indicators, read receipts
- **Channel management:** Join, leave, create, delete; text and voice channels
- **Guilds:** Create, list, leave, member list with roles, kick, audit log viewer
- **Roles:** Role list, assign/unassign with permission checks
- **Direct Messages:** 1:1 DMs with persistent history, unread badges, search
- **Friends:** Requests, accept/decline, Online/All/Pending/Blocked tabs
- **Presence:** Online/Idle/DND/Invisible, custom status text, activity display
- **Voice:** PTT / VAD / always-on modes, configurable bitrate, per-user volume
- **Jitter buffer:** Adaptive mode, configurable target latency
- **Noise suppression:** RNNoise (feature-gated, off by default)
- **Identity vault:** Optional Argon2id + ChaCha20-Poly1305 encrypted keyfile
- **Keyfile export/import:** Encrypted or plain JSON with passphrase
- **Bookmarks:** Save/remove server nodes in connect screen

### Planned
- In-game overlay (Phase 2)
- Federation support (Phase 3)
- Mobile client (Phase 3)

---

## Status

Phase 1 is implemented. Not production ready.

| Feature              | Status                     |
|----------------------|----------------------------|
| Connect to server    | ✅ Working                  |
| Text chat            | ✅ Working                  |
| Channel management   | ✅ Working                  |
| Guilds & roles       | ✅ Working                  |
| Direct Messages      | ✅ Working                  |
| Friends & presence   | ✅ Working                  |
| Voice (send/receive) | 🔧 Partial (audio pipeline) |
| Encryption           | ✅ ChaCha20-Poly1305 + X25519 |
| In-game overlay      | 🔲 Phase 2                  |
| Mobile client        | 🔲 Phase 3                  |

See [docs/00-status.md](docs/00-status.md) for a full breakdown.

---

## Quick start

**Requirements:** Rust 1.85+, a running [VNOX Server](https://github.com/loki5512344/Vnox)

```bash
# Terminal 1 — gateway
cargo run -p vnox-gateway -- --config dev/config.toml

# Terminal 2 — voice node
cargo run -p vnox-voice-node -- --config dev/config.toml

# Terminal 3 — client
cargo run -p vnox-client
```

The client connects to `127.0.0.1:7600` by default. You can change the address in the UI on the connect screen.

### Opus on Windows

`audiopus_sys` builds libopus from source via CMake. CMake 4.x policy flag is already set in `.cargo/config.toml` — no manual steps needed.

---

## Tech stack

| Layer       | Technology                     |
|-------------|--------------------------------|
| UI          | Slint                          |
| Networking  | Tokio (async TCP + UDP)        |
| Audio       | cpal + rodio + opus + rnnoise  |
| Identity    | ed25519-dalek + x25519-dalek   |
| Crypto      | ChaCha20-Poly1305 + Argon2id   |
| Protocol    | LNEx v1 (JSON, Phase 1)        |

---

## Project structure

```
src/
├── main.rs             # Entry point
├── ui/                 # Slint UI (slint files + Rust glue)
├── net/                # LNEx TCP + UDP networking
│   ├── crypto/         # Session encryption
│   ├── framing/        # Packet read/write
│   ├── voice/          # UDP voice send/recv
│   └── session/        # Connection lifecycle, reconnection
├── audio/              # Audio pipeline
│   ├── capture.rs      # Mic → Opus encode
│   ├── playback.rs     # Opus decode → speaker
│   ├── config.rs       # Bitrate, VAD, jitter settings
│   └── processing/     # VAD, noise suppression
├── identity_vault.rs   # Argon2id-encrypted keyfile
├── app.rs              # UI state and event dispatch
└── jitter/             # Jitter buffer
```

---

## License

Client code: **GPL-3.0** — see [LICENSE](LICENSE)  
LNEx protocol specification: **CC0** (public domain)
