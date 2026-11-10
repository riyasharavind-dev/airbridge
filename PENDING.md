# AirBridge Windows completion audit

Current state: development prototype. Full product acceptance is not complete.

## Implemented
- Native shell, real OS/GPU inspection, persisted validated settings.
- LAN discovery and reachability diagnostics.
- Fresh encrypted control connection with full-code comparison and both approvals.
- Selected-display capture and H.264 software encode/decode.
- Encrypted framed mirror transport with bounded queues and native receiver window.
- Explicit share/stop and disconnect; resolution, bitrate and FPS targets applied.

## Must finish before a reliable Windows release
- Verify continuous mirroring on two physical computers, both directions.
- Validate native window startup/rendering, close, Escape and disconnect cleanup.
- Replace per-frame FFmpeg processes with persistent codec pipeline; measure sustained FPS, latency, CPU and memory.
- Hardware encoder/decoder runtime probes, software fallback selection and adaptive bitrate.
- Reconnection after Wi-Fi loss; persistent peer identity/credential storage and reviewed pairing protocol.
- App shutdown must cancel/join all capture, discovery, session and codec workers.
- Capture backend migration/evaluation for Windows Graphics Capture/Desktop Duplication; current screenshots capture is a fallback prototype.
- Dependency deployment for other computers: current media requires FFmpeg/libx264 on PATH; it is not bundled.
- Firewall onboarding and testing, network-isolation errors and rate limits.
- Fresh installer/uninstaller tests, cleanup policy, signing and offline WebView2 strategy.
- Genuine Windows IddCx virtual-display driver, WDK integration, monitor lifecycle tests and driver signing. Extended mode unavailable until verified.
- UI accessibility, visual QA, fullscreen receiver and useful session logs.

## Deferred Linux work
- Build failures/dependencies, distro baseline, X11/Wayland portal capture, packages and real-device testing.

External acceptance needs two devices. Driver signing needs an appropriate signing workflow. None of these are marked passed without evidence.
