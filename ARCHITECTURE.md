# AirBridge architecture
Phase 1 baseline. Tauri 2 hosts React/TypeScript controls; Rust owns OS inspection, discovery, authorization and sessions. Native capture, H.264 codecs and native receiver surfaces remain outside the webview. No browser viewer or cloud account.

Capture -> bounded frame queue -> encoder -> authenticated encrypted transport -> decoder -> native renderer. Drop stale frames; never grow queues without bounds. UI receives status events, not raw video frames.

Transport decision remains open: benchmark native WebRTC against QUIC before Phase 4. WebRTC offers congestion control and media timing; QUIC requires application media framing, pacing and loss recovery design. No custom cryptography. Discovery is untrusted and never authorizes a stream.

Platform adapters: Windows Graphics Capture/Desktop Duplication; Linux PipeWire/portal with explicit X11 fallback. Extended display requires a verified IddCx or compositor virtual output backend. Unsupported remains unavailable.

Risks: driver signing and WDK, Wayland backend differences, GPU compatibility, H.264 distribution licensing, codec packaging, firewall and Wi-Fi isolation, native rendering integration. Validate on both target laptops before claiming support.
