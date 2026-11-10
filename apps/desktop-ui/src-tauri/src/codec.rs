use std::{io::{Read,Write},process::{Command,Stdio},time::{Duration,Instant}};
#[derive(serde::Serialize)]pub struct EncodingResult{pub encoder:String,pub width:u32,pub height:u32,pub encoded_bytes:usize,pub elapsed_ms:f64}
fn encode(width:u32,height:u32,rgba:Vec<u8>)->Result<EncodingResult,String>{
 if width==0||height==0||width>16384||height>16384||rgba.len()!=width as usize*height as usize*4{return Err("Invalid frame dimensions or pixel buffer".into())}
 let started=Instant::now();let size=format!("{}x{}",width,height);let mut command=Command::new("ffmpeg");
 command.args(["-hide_banner","-loglevel","error","-f","rawvideo","-pixel_format","rgba","-video_size",&size,"-framerate","1","-i","pipe:0","-frames:v","1","-an","-vf","pad=ceil(iw/2)*2:ceil(ih/2)*2","-c:v","libx264","-preset","ultrafast","-tune","zerolatency","-pix_fmt","yuv420p","-f","h264","pipe:1"]).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
 #[cfg(windows)]{use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
 let mut child=command.spawn().map_err(|e|format!("FFmpeg unavailable: {}. Install an FFmpeg build with libx264 from https://ffmpeg.org/download.html",e))?;
 let mut input=child.stdin.take().ok_or("Encoder input unavailable")?;let output=child.stdout.take().ok_or("Encoder output unavailable")?;let errors=child.stderr.take().ok_or("Encoder diagnostics unavailable")?;
 let writer=std::thread::spawn(move||input.write_all(&rgba));
 let reader=std::thread::spawn(move||{let mut bytes=Vec::new();output.take(64*1024*1024).read_to_end(&mut bytes).map(|_|bytes)});
 let error_reader=std::thread::spawn(move||{let mut text=String::new();errors.take(65536).read_to_string(&mut text).map(|_|text)});
 loop{match child.try_wait(){Ok(Some(status))=>{
  let write_result=writer.join().map_err(|_|"Encoder input worker failed")?;
  let bytes=reader.join().map_err(|_|"Encoder output worker failed")?.map_err(|e|e.to_string())?;
  let errors=error_reader.join().map_err(|_|"Encoder diagnostic worker failed")?.map_err(|e|e.to_string())?;
  if !status.success(){return Err(format!("libx264 encoding failed: {}",errors.trim()))}write_result.map_err(|e|e.to_string())?;
  if !(bytes.starts_with(&[0,0,0,1])||bytes.starts_with(&[0,0,1])){return Err("Encoder returned no Annex B stream".into())}
  return Ok(EncodingResult{encoder:"libx264 software".into(),width:width+width%2,height:height+height%2,encoded_bytes:bytes.len(),elapsed_ms:started.elapsed().as_secs_f64()*1000.0})
 },Ok(None)=>{},Err(e)=>{let _=child.kill();let _=child.wait();return Err(e.to_string())}}
 if started.elapsed()>Duration::from_secs(15){let _=child.kill();let _=child.wait();return Err("Encoding timed out after 15 seconds".into())}std::thread::sleep(Duration::from_millis(20));}
}
#[tauri::command]
pub async fn test_h264(display_id:u32)->Result<EncodingResult,String>{tauri::async_runtime::spawn_blocking(move||{
 let screens=screenshots::Screen::all().map_err(|e|e.to_string())?;let screen=screens.iter().find(|s|s.display_info.id==display_id).ok_or("Selected display unavailable; refresh displays")?;
 let frame=screen.capture().map_err(|e|e.to_string())?;encode(frame.width(),frame.height(),frame.into_raw())
}).await.map_err(|e|e.to_string())?}
#[cfg(test)]mod tests{use super::*;#[test]fn rejects_invalid_input_before_starting_encoder(){assert!(encode(0,1,vec![]).is_err());assert!(encode(2,2,vec![0;4]).is_err());assert!(encode(20000,1,vec![]).is_err());}}

#[cfg(test)]mod integration_tests{use super::*;#[test]#[ignore="Requires FFmpeg/libx264 and interactive desktop"]fn actual_screen_h264(){let screens=screenshots::Screen::all().unwrap();let frame=screens.first().unwrap().capture().unwrap();let result=encode(frame.width(),frame.height(),frame.into_raw()).unwrap();assert!(result.encoded_bytes>0);println!("Actual capture encoded: {}x{}, {} bytes, {:.1} ms",result.width,result.height,result.encoded_bytes,result.elapsed_ms);}}
