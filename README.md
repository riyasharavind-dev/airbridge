# AirBridge

Windows wireless display development prototype. Genuine extended display is unavailable.

## Run the mirror prototype
1. Install the development build on both Windows computers on the same local network.
2. Mirror encoding/decoding is built into the application. FFmpeg is optional, used only by the separate FFmpeg diagnostic buttons.
3. On the receiver choose **Accept connections for 60 seconds**. On the sender use a discovered device's **Connect** button or its local IPv4 address.
4. Compare the entire verification code through a trusted channel. Approve on both computers only if every character matches.
5. Choose the sender display and **Share selected display**. The receiver opens a native window. Use **Stop sharing**, **Disconnect**, or close the receiver to end.

Mirroring reuses built-in OpenH264 encoder and decoder instances. Independent keyframes currently allow safe frame dropping at the cost of extra bandwidth. FPS targets are limits, not guarantees; low-latency performance and two-device operation are not yet validated. Each session requires fresh verification; peer trust is not persisted.

## Build
Rust MSVC, Node/npm and official Tauri prerequisites are required. In apps/desktop-ui run npm ci then npm run tauri build -- --bundles nsis. For native tests run cargo test --manifest-path apps/desktop-ui/src-tauri/Cargo.toml --release from the repository root.

See PENDING.md for release blockers and WORK_PROGRESS.md for actual results. No complete-product claim is made.
