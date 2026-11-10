use openh264::{
    decoder::Decoder,
    encoder::{BitRate, Encoder, EncoderConfig, FrameRate},
    formats::{RgbaSliceU8, YUVBuffer, YUVSource},
    OpenH264API,
};
pub struct NativeEncoder {
    encoder: Encoder,
}
impl NativeEncoder {
    pub fn new(bitrate_mbps: u32, fps: u32) -> Result<Self, String> {
        let config = EncoderConfig::new()
            .bitrate(BitRate::from_bps(bitrate_mbps.clamp(1, 50) * 1_000_000))
            .max_frame_rate(FrameRate::from_hz(fps.clamp(1, 60) as f32))
            .skip_frames(false);
        Ok(Self {
            encoder: Encoder::with_api_config(OpenH264API::from_source(), config)
                .map_err(|e| e.to_string())?,
        })
    }
    pub fn encode(&mut self, width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, String> {
        if width == 0
            || height == 0
            || width > 4096
            || height > 4096
            || width % 2 != 0
            || height % 2 != 0
            || rgba.len() != width as usize * height as usize * 4
        {
            return Err("Native codec requires even dimensions and a valid RGBA frame".into());
        }
        let yuv =
            YUVBuffer::from_rgb_source(RgbaSliceU8::new(rgba, (width as usize, height as usize)));
        self.encoder.force_intra_frame();
        let packet = self
            .encoder
            .encode(&yuv)
            .map_err(|e| e.to_string())?
            .to_vec();
        if packet.is_empty() {
            return Err("Native encoder produced no frame".into());
        }
        Ok(packet)
    }
}
pub struct NativeDecoder {
    decoder: Decoder,
}
impl NativeDecoder {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            decoder: Decoder::new().map_err(|e| e.to_string())?,
        })
    }
    pub fn decode(&mut self, packet: &[u8], width: u32, height: u32) -> Result<Vec<u32>, String> {
        if width == 0
            || height == 0
            || width > 4096
            || height > 4096
            || packet.is_empty()
            || packet.len() > 16 * 1024 * 1024
        {
            return Err("Invalid native decoder input".into());
        }
        let image = self
            .decoder
            .decode(packet)
            .map_err(|e| e.to_string())?
            .ok_or("Decoder returned no picture")?;
        if image.dimensions() != (width as usize, height as usize) {
            return Err("Decoded dimensions mismatch".into());
        }
        let mut rgba = vec![0; width as usize * height as usize * 4];
        image.write_rgba8(&mut rgba);
        Ok(rgba
            .chunks_exact(4)
            .map(|p| ((p[0] as u32) << 16) | ((p[1] as u32) << 8) | p[2] as u32)
            .collect())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn persistent_codec_roundtrip() {
        let mut encoder = NativeEncoder::new(8, 30).unwrap();
        let mut decoder = NativeDecoder::new().unwrap();
        let started = std::time::Instant::now();
        for i in 0..30 {
            let rgba = [(100 + i) as u8, 30, 60, 255].repeat(64 * 48);
            let packet = encoder.encode(64, 48, &rgba).unwrap();
            let pixels = decoder.decode(&packet, 64, 48).unwrap();
            assert_eq!(pixels.len(), 64 * 48);
            assert!((((pixels[0] >> 16) & 255) as i32 - (100 + i)).abs() < 12);
        }
        println!(
            "30 native encode/decode frames in {:.1} ms (64x48 test image, not desktop FPS)",
            started.elapsed().as_secs_f64() * 1000.0
        );
    }
    #[test]
    fn rejects_invalid_frames() {
        let mut e = NativeEncoder::new(8, 30).unwrap();
        assert!(e.encode(3, 2, &[0; 24]).is_err());
        assert!(e.encode(2, 2, &[0; 4]).is_err());
    }
}

#[cfg(test)]
mod desktop_tests {
    use super::*;
    #[test]
    #[ignore = "Captures actual desktop frames in memory"]
    fn actual_desktop_native_pipeline() {
        let screens = screenshots::Screen::all().unwrap();
        let screen = screens.first().unwrap();
        let mut encoder = NativeEncoder::new(8, 30).unwrap();
        let mut decoder = NativeDecoder::new().unwrap();
        let start = std::time::Instant::now();
        let mut total = 0;
        for _ in 0..10 {
            let frame = screen.capture().unwrap();
            let ratio = (1280.0 / frame.width() as f64)
                .min(720.0 / frame.height() as f64)
                .min(1.0);
            let w = ((frame.width() as f64 * ratio) as u32).max(2) & !1;
            let h = ((frame.height() as f64 * ratio) as u32).max(2) & !1;
            let image = screenshots::image::imageops::resize(
                &frame,
                w,
                h,
                screenshots::image::imageops::FilterType::Triangle,
            );
            let bytes = encoder.encode(w, h, image.as_raw()).unwrap();
            total += bytes.len();
            assert_eq!(
                decoder.decode(&bytes, w, h).unwrap().len(),
                w as usize * h as usize
            );
        }
        println!("10 real desktop capture/resize/native encode/decode frames: {:.1} ms, {:.1} FPS locally, {} encoded bytes; no rendering/network",start.elapsed().as_secs_f64()*1000.0,10.0/start.elapsed().as_secs_f64(),total);
    }
}

#[cfg(test)]mod drop_tests{use super::*;#[test]fn later_keyframe_decodes_after_dropped_frames(){let mut encoder=NativeEncoder::new(8,30).unwrap();let mut last=Vec::new();for i in 0..10{last=encoder.encode(64,48,&[150+i,30,60,255].repeat(64*48)).unwrap();}let mut decoder=NativeDecoder::new().unwrap();let pixels=decoder.decode(&last,64,48).unwrap();assert_eq!(pixels.len(),64*48);assert!((((pixels[0]>>16)&255)as i32-159).abs()<12);}}
