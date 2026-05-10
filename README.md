# Voiceland

A terminal-based voice communication application for local area networks. Talk to your friends or colleagues over LAN with low latency and minimal setup.

## Configuration

Create a `voiceland.toml` in the working directory:

```toml
name = "Alice"        # Your display name
peers = ["10.0.0.2"]  # IP addresses of peers to connect to
port = 6413           # UDP port to listen on
bitrate = 512000      # Audio quality
buffer_len = 5        # Jitter buffer size (frames)
denoise = true        # Enable noise suppression
```
