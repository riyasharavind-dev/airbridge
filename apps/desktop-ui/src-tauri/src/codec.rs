use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
#[derive(serde::Serialize)]
pub struct EncodingResult {
    pub encoder: String,
    pub width: u32,
    pub height: u32,
    pub encoded_bytes: usize,
    pub elapsed_ms: f64,
}
fn encode(width: u32, height: u32, rgba: Vec<u8>) -> Result<EncodingResult, String> {
    encode_packet(width, height, rgba).map(|(result, _)| result)
}
pub fn encode_packet(
    width: u32,
    height: u32,
    rgba: Vec<u8>,
) -> Result<(EncodingResult, Vec<u8>), String> {
    encode_with_bitrate(width, height, rgba, 8)
}
pub fn encode_with_bitrate(
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    bitrate: u32,
) -> Result<(EncodingResult, Vec<u8>), String> {
    if width == 0
        || height == 0
        || width > 16384
        || height > 16384
        || rgba.len() != width as usize * height as usize * 4
    {
        return Err("Invalid frame dimensions or pixel buffer".into());
    }
    let bitrate_arg = format!("{}M", bitrate.clamp(1, 50));
    let started = Instant::now();
    let size = format!("{}x{}", width, height);
    let mut command = Command::new("ffmpeg");
    command
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "rawvideo",
            "-pixel_format",
            "rgba",
            "-video_size",
            &size,
            "-framerate",
            "1",
            "-i",
            "pipe:0",
            "-frames:v",
            "1",
            "-an",
            "-vf",
            "pad=ceil(iw/2)*2:ceil(ih/2)*2",
            "-c:v",
            "libx264",
            "-b:v",
            &bitrate_arg,
            "-preset",
            "ultrafast",
            "-tune",
            "zerolatency",
            "-pix_fmt",
            "yuv420p",
            "-f",
            "h264",
            "pipe:1",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child=command.spawn().map_err(|e|format!("FFmpeg unavailable: {}. Install an FFmpeg build with libx264 from https://ffmpeg.org/download.html",e))?;
    let mut input = child.stdin.take().ok_or("Encoder input unavailable")?;
    let output = child.stdout.take().ok_or("Encoder output unavailable")?;
    let errors = child
        .stderr
        .take()
        .ok_or("Encoder diagnostics unavailable")?;
    let writer = std::thread::spawn(move || input.write_all(&rgba));
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        output
            .take(64 * 1024 * 1024)
            .read_to_end(&mut bytes)
            .map(|_| bytes)
    });
    let error_reader = std::thread::spawn(move || {
        let mut text = String::new();
        errors.take(65536).read_to_string(&mut text).map(|_| text)
    });
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let write_result = writer.join().map_err(|_| "Encoder input worker failed")?;
                let bytes = reader
                    .join()
                    .map_err(|_| "Encoder output worker failed")?
                    .map_err(|e| e.to_string())?;
                let errors = error_reader
                    .join()
                    .map_err(|_| "Encoder diagnostic worker failed")?
                    .map_err(|e| e.to_string())?;
                if !status.success() {
                    return Err(format!("libx264 encoding failed: {}", errors.trim()));
                }
                write_result.map_err(|e| e.to_string())?;
                if !(bytes.starts_with(&[0, 0, 0, 1]) || bytes.starts_with(&[0, 0, 1])) {
                    return Err("Encoder returned no Annex B stream".into());
                }
                return Ok((
                    EncodingResult {
                        encoder: "libx264 software".into(),
                        width: width + width % 2,
                        height: height + height % 2,
                        encoded_bytes: bytes.len(),
                        elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
                    },
                    bytes,
                ));
            }
            Ok(None) => {}
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e.to_string());
            }
        }
        if started.elapsed() > Duration::from_secs(15) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Encoding timed out after 15 seconds".into());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}
#[tauri::command]
pub async fn test_h264(display_id: u32) -> Result<EncodingResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let screens = screenshots::Screen::all().map_err(|e| e.to_string())?;
        let screen = screens
            .iter()
            .find(|s| s.display_info.id == display_id)
            .ok_or("Selected display unavailable; refresh displays")?;
        let frame = screen.capture().map_err(|e| e.to_string())?;
        encode(frame.width(), frame.height(), frame.into_raw())
    })
    .await
    .map_err(|e| e.to_string())?
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_input_before_starting_encoder() {
        assert!(encode(0, 1, vec![]).is_err());
        assert!(encode(2, 2, vec![0; 4]).is_err());
        assert!(encode(20000, 1, vec![]).is_err());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    #[test]
    #[ignore = "Requires FFmpeg/libx264 and interactive desktop"]
    fn actual_screen_h264() {
        let screens = screenshots::Screen::all().unwrap();
        let frame = screens.first().unwrap().capture().unwrap();
        let result = encode(frame.width(), frame.height(), frame.into_raw()).unwrap();
        assert!(result.encoded_bytes > 0);
        println!(
            "Actual capture encoded: {}x{}, {} bytes, {:.1} ms",
            result.width, result.height, result.encoded_bytes, result.elapsed_ms
        );
    }
}

pub fn decode_packet(data: Vec<u8>, width: u32, height: u32) -> Result<Vec<u32>, String> {
    if width == 0
        || height == 0
        || width > 4096
        || height > 4096
        || data.is_empty()
        || data.len() > 16 * 1024 * 1024
    {
        return Err("Invalid or oversized encoded frame".into());
    }
    let expected = width as usize * height as usize * 4;
    let mut command = Command::new("ffmpeg");
    command
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-max_pixels",
            "16777216",
            "-f",
            "h264",
            "-i",
            "pipe:0",
            "-frames:v",
            "1",
            "-an",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "pipe:1",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command
        .spawn()
        .map_err(|e| format!("FFmpeg decoder unavailable: {}", e))?;
    let mut input = child.stdin.take().ok_or("Decoder input unavailable")?;
    let output = child.stdout.take().ok_or("Decoder output unavailable")?;
    let writer = std::thread::spawn(move || input.write_all(&data));
    let reader = std::thread::spawn(move || {
        let mut b = Vec::new();
        output
            .take(expected as u64 + 1)
            .read_to_end(&mut b)
            .map(|_| b)
    });
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let write_result = writer.join().map_err(|_| "Decoder input worker failed")?;
                let rgba = reader
                    .join()
                    .map_err(|_| "Decoder output worker failed")?
                    .map_err(|e| e.to_string())?;
                if !status.success() {
                    return Err("FFmpeg failed to decode frame".into());
                }
                write_result.map_err(|e| e.to_string())?;
                if rgba.len() != expected {
                    return Err("Decoded dimensions do not match expected frame size".into());
                }
                return Ok(rgba
                    .chunks_exact(4)
                    .map(|p| ((p[0] as u32) << 16) | ((p[1] as u32) << 8) | p[2] as u32)
                    .collect());
            }
            Ok(None) => {}
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e.to_string());
            }
        }
        if start.elapsed() > Duration::from_secs(15) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Decoding timed out after 15 seconds".into());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}
#[tauri::command]
pub async fn receiver_preview(display_id: u32) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move||{
 let screens=screenshots::Screen::all().map_err(|e|e.to_string())?;let screen=screens.iter().find(|s|s.display_info.id==display_id).ok_or("Selected display unavailable")?;let image=screen.capture().map_err(|e|e.to_string())?;
 let started=Instant::now();let(info,packet)=encode_packet(image.width(),image.height(),image.into_raw())?;let pixels=decode_packet(packet,info.width,info.height)?;let elapsed=started.elapsed().as_secs_f64()*1000.0;
 let mut window=minifb::Window::new("AirBridge - decoded H.264 frame (Escape to close)",960,540,minifb::WindowOptions{resize:true,..Default::default()}).map_err(|e|e.to_string())?;window.set_target_fps(30);
 while window.is_open()&&!window.is_key_down(minifb::Key::Escape){window.update_with_buffer(&pixels,info.width as usize,info.height as usize).map_err(|e|e.to_string())?;}
 Ok(format!("Decoded and displayed one local {} x {} H.264 frame. Codec round trip: {:.1} ms including process startup. No network streaming.",info.width,info.height,elapsed))
}).await.map_err(|e|e.to_string())?
}
#[cfg(test)]
mod roundtrip_tests {
    use super::*;
    #[test]
    fn rejects_unsafe_decode_input() {
        assert!(decode_packet(vec![], 1920, 1080).is_err());
        assert!(decode_packet(vec![1], 5000, 1080).is_err());
    }
    #[test]
    #[ignore = "Requires FFmpeg/libx264"]
    fn actual_encode_decode() {
        let mut rgba = Vec::new();
        for _ in 0..(64 * 48) {
            rgba.extend_from_slice(&[200, 30, 60, 255]);
        }
        let (info, encoded) = encode_packet(64, 48, rgba).unwrap();
        let pixels = decode_packet(encoded, info.width, info.height).unwrap();
        assert_eq!(pixels.len(), 64 * 48);
        let pixel = pixels[0];
        assert!((((pixel >> 16) & 255) as i32 - 200).abs() < 10);
        assert!((((pixel >> 8) & 255) as i32 - 30).abs() < 10);
        assert!(((pixel & 255) as i32 - 60).abs() < 10);
        println!(
            "Real libx264 encode/decode round trip passed, {} encoded bytes",
            info.encoded_bytes
        );
    }
}
