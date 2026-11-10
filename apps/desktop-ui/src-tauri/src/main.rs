#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod discovery;
mod settings;
mod capture;
#[derive(serde::Serialize)]
struct Diagnostics { os: String, architecture: String, device_name: String, session_type: String }
#[tauri::command]
fn diagnostics() -> Diagnostics {
 Diagnostics { os: std::env::consts::OS.into(), architecture: std::env::consts::ARCH.into(), device_name: std::env::var("COMPUTERNAME").or_else(|_| std::env::var("HOSTNAME")).unwrap_or_else(|_| "Unknown".into()), session_type: std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| if cfg!(windows) { "Windows desktop".into() } else { "Unknown".into() }) }
}
#[tauri::command]
async fn hardware_report()->Result<String,String>{
 tauri::async_runtime::spawn_blocking(||{
  #[cfg(windows)] { use std::os::windows::process::CommandExt;let output=std::process::Command::new("powershell.exe").args(["-NoProfile","-NonInteractive","-Command","Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion,VideoProcessor | ConvertTo-Json -Compress"]).creation_flags(0x08000000).output().map_err(|e|e.to_string())?;if !output.status.success(){return Err("Windows GPU query failed".into())}Ok(String::from_utf8_lossy(&output.stdout).into_owned()) }
  #[cfg(not(windows))] { Err("Windows GPU diagnostics are unavailable on this platform".into()) }
 }).await.map_err(|e|e.to_string())?
}
fn main() { let discovery = discovery::Discovery::start(); tauri::Builder::default().manage(discovery).manage(capture::Capture::default()).invoke_handler(tauri::generate_handler![diagnostics, discovery::nearby_devices, settings::load_settings, settings::save_settings, hardware_report, capture::start_preview, capture::stop_preview, capture::preview_status, capture::list_displays, capture::save_screenshot]).run(tauri::generate_context!()).expect("AirBridge startup failed"); }
