use tauri::Manager;
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Settings {
    pub device_name: String,
    pub fps: u32,
    pub bitrate_mbps: u32,
    pub resolution: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            device_name: std::env::var("COMPUTERNAME").unwrap_or_else(|_| "AirBridge".into()),
            fps: 30,
            bitrate_mbps: 8,
            resolution: "1080p".into(),
        }
    }
}
fn validate(s: &Settings) -> Result<(), String> {
    if s.device_name.trim().is_empty()
        || s.device_name.len() > 64
        || s.device_name.chars().any(char::is_control)
    {
        return Err("Device name must contain 1-64 bytes without control characters".into());
    }
    if ![30, 60].contains(&s.fps)
        || !(1..=50).contains(&s.bitrate_mbps)
        || !["720p", "1080p"].contains(&s.resolution.as_str())
    {
        return Err("Unsupported video settings".into());
    }
    Ok(())
}
#[tauri::command]
pub fn load_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    let path = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?
        .join("settings.json");
    if !path.exists() {
        return Ok(Settings::default());
    }
    let s: Settings = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    validate(&s)?;
    Ok(s)
}
#[tauri::command]
pub fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    validate(&settings)?;
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("settings.json"), bytes).map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_settings() {
        let mut s = Settings::default();
        assert!(validate(&s).is_ok());
        s.fps = 144;
        assert!(validate(&s).is_err());
        s.fps = 30;
        s.device_name = "\n".into();
        assert!(validate(&s).is_err());
    }
    #[test]
    fn settings_round_trip() {
        let s = Settings::default();
        assert_eq!(
            s,
            serde_json::from_slice::<Settings>(&serde_json::to_vec(&s).unwrap()).unwrap()
        );
    }
}
