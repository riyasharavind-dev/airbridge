# Dependency matrix
| Component | Windows | Linux | Status |
|---|---|---|---|
| UI build | Node/npm, Rust MSVC, C++ tools/SDK | Node/npm, Rust, C toolchain | Node/Rust detected locally |
| UI runtime | WebView2 | GTK/WebKitGTK 4.1 distro packages | Not verified |
| Capture | Graphics Capture or Duplication | PipeWire + portal; X11 fallback | Not implemented |
| Codec | H.264 native or packaged fallback | H.264 native or packaged fallback | Not evaluated |
| Extend | WDK, UMDF/IddCx signed driver | Supported virtual output backend | Unavailable |
| Network | Firewall permission, LAN | Firewall permission, LAN | Not verified |

Versions: Tauri 2; exact dependencies resolved in lockfiles when installation succeeds. Runtime checks must report unknown separately from missing. Optional failures cannot prevent startup. Installer must explain required component and official source, request consent, verify publisher/checksum, and verify installation. Never silently replace GPU drivers.

Official prerequisites: https://v2.tauri.app/start/prerequisites/
Driver reference: https://learn.microsoft.com/en-us/windows-hardware/drivers/display/indirect-display-driver-model-overview
