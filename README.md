# Voiceland

A terminal-based voice communication application for local area networks. Talk to your friends or colleagues over LAN with low latency and minimal setup.

## Features

- **Real-time voice chat** — Low-latency UDP-based voice communication
- **Terminal UI** — Clean ratatui interface with peer list and volume controls
- **Opus codec** — Efficient audio compression at 48 kHz, 512 kbps
- **Noise suppression** — Optional RNNoise-based microphone noise reduction
- **Per-peer volume** — Individual volume control for each connected peer
- **Auto-discovery** — Peers are automatically discovered when they send voice packets
- **Cross-platform** — Works on Windows, macOS, and Linux

## Requirements

- **Rust nightly** (required for `maybe_uninit_array_assume_init`)


## Configuration

Create a `voiceland.toml` in the working directory:

```toml
port = 6413           # UDP port to listen on
name = "Alice"        # Your display name
peers = ["10.0.0.2"]  # IP addresses of peers to connect to
buffer_len = 5        # Jitter buffer size (frames)
denoise = true        # Enable noise suppression
```
