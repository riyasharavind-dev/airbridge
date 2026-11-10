# Work progress — 11 November 2026

Phase 1 architecture and scaffold completed. Await next instruction before Phase 2.

Created architecture, requirements, dependency matrix, security, protocol draft and roadmap. Scaffolded React/TypeScript + Tauri 2 and a Rust capability executable. Reserved native platform/module boundaries explicitly have no implementations.

Environment: Node 24.19.0, npm 11.17.0, rustc 1.97.1, cargo 1.97.1; Windows x86_64 MSVC target installed.

Actual checks:
- cargo build: passed.
- cargo run -p airbridge-capabilities: passed, reported windows/x86_64.
- npm install: passed, audit reported zero vulnerabilities.
- TypeScript + Vite production build: passed.
- Tauri Windows release build --no-bundle: passed, executable at apps/desktop-ui/src-tauri/target/release/airbridge-desktop.exe.
- Initial Tauri build failed because Cargo was absent from child PATH; corrected build environment. Missing icon corrected with placeholder ICO.

Not verified: desktop launch/UI rendering, Linux build/runtime, GPU/driver capabilities, capture permissions, dependency readiness, two-laptop operation. No performance measurements. No streaming, discovery, pairing, dependency installer, settings persistence or virtual monitor implementation. No installer package produced. Native executable is a foundation screen only.

Next phase: real OS/GPU/dependency inspection, settings and desktop startup testing on Windows and Linux. Confirm Linux session type and GPU on Lenovo before capture backend decisions.

## Continued development requested
User authorized remaining phases and Windows/Linux packaging. Added native device diagnostics (OS, architecture, device name, session type) with refresh/error states. Frontend production build passed. Windows installer build in progress. Linux environment check: WSL absent, Docker unavailable; requested Linux OS/session/GPU information and build-environment preference. Linux build helper prepared but not executed. Streaming/driver implementation remains pending.

Windows NSIS installer build passed: apps/desktop-ui/src-tauri/target/release/bundle/nsis/AirBridge_0.1.0_x64-setup.exe (1.85 MiB). Installer execution/uninstallation not tested. Linux setup attempts (wsl --install and Ubuntu-specific install) both failed with WSL not installed; no Linux package produced. User prefers broad Linux compatibility; AppImage plus deb targets selected, universal Linux support not claimed.

## LAN discovery milestone
Implemented UDP broadcast discovery on port 45873, bounded advertisement parsing, 12-second peer expiration, live list and error reporting. Two Rust packet validation tests passed; frontend build passed. Discovery is untrusted and may include the local device. No two-device validation yet. Added GitHub Actions Windows/Linux development packaging workflow; execution not yet verified. Pairing, media transport, native capture and virtual displays remain unimplemented.

## Windows-first development
Added persistent validated device/video preferences and Windows GPU/driver inspection through CIM. Four native tests passed; TypeScript/Vite production build passed. These preferences do not enable streaming. Secure pairing, native capture, H.264 encoding/decoding, encrypted transport, native receiver rendering and genuine virtual display remain unimplemented. Latest installer must be rebuilt before distributing these changes. Full Windows completion has not been achieved.

## Native screen capture
Added native capture of the first display with a separate framebuffer preview window, explicit start/stop controls and Escape-to-close. Captures stay local. Frontend production build passed; four existing native tests passed; the explicit real_capture_smoke test passed on the actual Windows desktop (one frame captured in memory, not saved or transmitted). Native preview visual rendering, sustained FPS, network streaming, secure pairing and genuine extended display remain unverified/unimplemented. Windows installer rebuild in progress.

## Display selection
Added actual monitor enumeration (ID, dimensions, position, primary flag), display selection, refresh and capture of selected monitor. Hot-unplug selection returns an explicit error. Four native tests and frontend production build passed. Multiple physical monitor behavior and updated preview UI not manually verified. Installer not rebuilt for this change. Secure pairing remains unimplemented.

## Selected display PNG export
Added asynchronous screenshot export to Pictures/AirBridge with unique filenames, create-new protection, explicit flush, actual saved path, busy state and error reporting. Native tests passed (four; interactive capture check skipped in this run). Export UI and file output not manually exercised. Installer not rebuilt. Secure pairing and streaming remain unimplemented.

## LAN reachability diagnostics
Added per-device and manual local IPv4 connection checks using UDP echo and measured round-trip time. Two-second timeout, exact response matching, fixed packet size, source filtering through connected socket, and local-address validation. Results explicitly do not authenticate identity. Six native tests passed including real loopback UDP and invalid reply rejection; interactive capture test skipped. Two-machine firewall/network behavior not verified. Installer not rebuilt for this change.

## Live capture diagnostics
Added live captured-frame count, achieved average FPS, sampled capture-plus-conversion time and target FPS. Preview now loads the validated saved 30/60 FPS preference on start. Target FPS is a limit, not a guaranteed rate. Capture worker panic recovery clears running state and reports error. Six native tests passed; interactive capture smoke skipped. Live preview metrics not manually measured; no performance results claimed. Installer not rebuilt.

## Dependency readiness inspection
Added explicit dependency inspection for connected displays and FFmpeg compiled H.264 encoder inventory. Missing FFmpeg is handled without blocking local preview; official references are shown, no installers executed. Query timeout is five seconds. Compiled encoder names do not establish runtime GPU capability. Unimplemented secure streaming and extended display are clearly reported. Seven native tests passed; interactive capture test skipped. Interactive dependency UI not manually verified; installer not rebuilt.

## One-frame H264 diagnostic
Added selected-display capture into a local FFmpeg/libx264 software encoding check. Uses in-memory pipes, 15-second process timeout, bounded output and Annex B prefix check. Odd dimensions are padded for YUV420. UI reports encoded dimensions, bytes and time including process startup. No saving/transmission. FFmpeg is not installed on this host, so real encoding execution is not verified; no encoding success claimed. Decode, sustained throughput, secure pairing and streaming remain unimplemented. Installer not rebuilt. Reference: https://ffmpeg.org/ffmpeg-formats.html.

## Encrypted control connection and actual encoder validation
Installed FFmpeg through winget (installer hash verified). Explicit actual desktop capture-to-libx264 test passed: 1366x768, 144934 encoded bytes, 116.9 ms including process startup; no frame saved/transmitted. This is one-frame data, not sustained FPS.
Implemented opt-in TCP control listener on LAN port 45874, Noise NN X25519/ChaChaPoly/BLAKE2s handshake, full transcript verification code, explicit fresh mutual approval, encrypted heartbeats, reject/cancel/disconnect, bounded messages, 60-second pairing expiration and peer timeout. No identity persistence; users must compare the entire code through a trusted channel each session. No media is sent. Tests cover matching transcript, tampered ciphertext rejection, both approvals required and disconnect using actual loopback sockets. Windows frontend build passed. Two physical devices and security review remain unverified. Full Windows product remains incomplete; media pipeline, native receiver integration and virtual display driver remain outstanding.
