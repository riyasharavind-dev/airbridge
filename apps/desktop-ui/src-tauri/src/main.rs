#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod discovery;
#[derive(serde::Serialize)]
struct Diagnostics { os: String, architecture: String, device_name: String, session_type: String }
#[tauri::command]
fn diagnostics() -> Diagnostics {
 Diagnostics { os: std::env::consts::OS.into(), architecture: std::env::consts::ARCH.into(), device_name: std::env::var("COMPUTERNAME").or_else(|_| std::env::var("HOSTNAME")).unwrap_or_else(|_| "Unknown".into()), session_type: std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| if cfg!(windows) { "Windows desktop".into() } else { "Unknown".into() }) }
}
fn main() { let discovery = discovery::Discovery::start(); tauri::Builder::default().manage(discovery).invoke_handler(tauri::generate_handler![diagnostics, discovery::nearby_devices]).run(tauri::generate_context!()).expect("AirBridge startup failed"); }
