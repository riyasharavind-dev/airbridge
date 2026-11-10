#[derive(serde::Serialize)]pub struct Check {name:String,status:String,details:String,source:Option<String>}
fn item(name:&str,status:&str,details:String,source:Option<&str>)->Check{Check{name:name.into(),status:status.into(),details,source:source.map(String::from)}}
fn encoders(output:&str)->Vec<String>{output.lines().filter_map(|line|{let mut fields=line.split_whitespace();let flags=fields.next()?;let name=fields.next()?;if flags.starts_with('V')&&(name=="libx264"||name=="libopenh264"||name.starts_with("h264_")){Some(name.to_owned())}else{None}}).collect()}
fn ffmpeg()->Result<String,String>{
 use std::process::{Command,Stdio};use std::time::{Duration,Instant};
 let mut command=Command::new("ffmpeg");command.args(["-hide_banner","-encoders"]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null());
 #[cfg(windows)]{use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
 let mut child=command.spawn().map_err(|e|format!("FFmpeg could not be started: {}",e))?;
 let stdout=child.stdout.take().ok_or("FFmpeg output unavailable")?;
 let reader=std::thread::spawn(move||{use std::io::Read;let mut data=String::new();stdout.take(1024*1024).read_to_string(&mut data).map(|_|data)});
 let start=Instant::now();
 loop{match child.try_wait(){Ok(Some(status))=>{let output=reader.join().map_err(|_|"FFmpeg output worker failed")?.map_err(|e|e.to_string())?;return if status.success(){Ok(output)}else{Err("FFmpeg encoder query failed".into())}},Ok(None)=>{},Err(e)=>{let _=child.kill();let _=child.wait();return Err(e.to_string())}}
 if start.elapsed()>Duration::from_secs(5){let _=child.kill();let _=child.wait();return Err("FFmpeg query timed out after 5 seconds".into())}std::thread::sleep(Duration::from_millis(20));}
}
#[tauri::command]
pub async fn dependency_report()->Result<Vec<Check>,String>{tauri::async_runtime::spawn_blocking(||{
 let mut checks=Vec::new();
 checks.push(match screenshots::Screen::all(){Ok(screens)if !screens.is_empty()=>item("Display enumeration","available",format!("{} displays found. Capture permission is verified only when capture succeeds.",screens.len()),None),Ok(_)=>item("Display enumeration","unavailable","No displays found".into(),None),Err(e)=>item("Display enumeration","error",e.to_string(),None)});
 checks.push(match ffmpeg(){Ok(output)=>{let names=encoders(&output);if names.is_empty(){item("H.264 encoder inventory","unavailable","FFmpeg runs, but no recognized H.264 encoder is compiled in.".into(),Some("https://ffmpeg.org/download.html"))}else{item("H.264 encoder inventory","detected",format!("Compiled encoders: {}. Runtime encoding and GPU support are not verified; streaming is not implemented.",names.join(", ")),Some("https://ffmpeg.org/download.html"))}},Err(e)=>item("FFmpeg inspection","unavailable",format!("{}. Optional for local preview; required by this diagnostic only. No components installed automatically.",e),Some("https://ffmpeg.org/download.html"))});
 checks.push(item("Secure network streaming","not implemented","Verified encrypted control-channel pairing is implemented. Media transport and screen streaming are not implemented.".into(),None));
 checks.push(item("Extended display","not implemented","No virtual display driver integration. A preview window is not an extended monitor.".into(),Some("https://learn.microsoft.com/en-us/windows-hardware/drivers/display/indirect-display-driver-model-overview")));
 checks
 }).await.map_err(|e|e.to_string())}
#[cfg(test)]mod tests{use super::*;#[test]fn distinguishes_video_encoders(){assert_eq!(encoders(" V....D libx264 H.264\n V....D h264_amf AMD\n A..... aac Audio\n V..... hevc_nvenc HEVC\n malformed"),vec!["libx264","h264_amf"]);}}
