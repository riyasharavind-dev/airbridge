#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../apps/desktop-ui"
command -v cargo >/dev/null || { echo 'Rust toolchain required'; exit 1; }
command -v npm >/dev/null || { echo 'Node/npm required'; exit 1; }
pkg-config --exists gtk+-3.0 webkit2gtk-4.1 || { echo 'GTK3/WebKitGTK 4.1 development packages required; see DEPENDENCIES.md'; exit 1; }
npm ci
npm run tauri build -- --bundles deb,appimage
