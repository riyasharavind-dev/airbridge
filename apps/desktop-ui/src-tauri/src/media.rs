use std::sync::mpsc;
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub packet: Vec<u8>,
}
#[derive(Default)]
pub struct Assembly {
    width: u32,
    height: u32,
    total: usize,
    bytes: Vec<u8>,
}
impl Assembly {
    pub fn accept(&mut self, message: &[u8]) -> Result<Option<Frame>, String> {
        if message.starts_with(b"FRAME") && message.len() == 17 {
            if self.total != 0 {
                return Err("Overlapping video frames".into());
            }
            let w = u32::from_be_bytes(message[5..9].try_into().unwrap());
            let h = u32::from_be_bytes(message[9..13].try_into().unwrap());
            let n = u32::from_be_bytes(message[13..17].try_into().unwrap()) as usize;
            if w == 0 || h == 0 || w > 4096 || h > 4096 || n == 0 || n > 16 * 1024 * 1024 {
                return Err("Invalid video frame header".into());
            }
            self.width = w;
            self.height = h;
            self.total = n;
            self.bytes.clear();
            return Ok(None);
        }
        if message.starts_with(b"CHUNK") {
            if self.total == 0 || self.bytes.len() + message.len() - 5 > self.total {
                return Err("Unexpected or oversized video chunk".into());
            }
            self.bytes.extend_from_slice(&message[5..]);
            return Ok(None);
        }
        if message == b"END" {
            if self.total == 0 || self.bytes.len() != self.total {
                return Err("Incomplete video frame".into());
            }
            self.total = 0;
            return Ok(Some(Frame {
                width: self.width,
                height: self.height,
                packet: std::mem::take(&mut self.bytes),
            }));
        }
        Err("Unknown video message".into())
    }
}
pub fn receiver() -> mpsc::SyncSender<Frame> {
    let (tx, rx) = mpsc::sync_channel::<Frame>(1);
    std::thread::spawn(move || {
        let first = match rx.recv() {
            Ok(frame) => frame,
            Err(_) => return,
        };
        let mut pixels = match crate::codec::decode_packet(first.packet, first.width, first.height)
        {
            Ok(p) => p,
            Err(_) => return,
        };
        let (mut width, mut height) = (first.width, first.height);
        let mut window = match minifb::Window::new(
            "AirBridge - encrypted mirror (Escape to close)",
            960,
            540,
            minifb::WindowOptions {
                resize: true,
                ..Default::default()
            },
        ) {
            Ok(w) => w,
            Err(_) => return,
        };
        window.set_target_fps(30);
        while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
            match rx.try_recv() {
                Ok(frame) => {
                    match crate::codec::decode_packet(frame.packet, frame.width, frame.height) {
                        Ok(p) => {
                            pixels = p;
                            width = frame.width;
                            height = frame.height
                        }
                        Err(_) => break,
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => break,
                Err(mpsc::TryRecvError::Empty) => {}
            }
            if window
                .update_with_buffer(&pixels, width as usize, height as usize)
                .is_err()
            {
                break;
            }
        }
    });
    tx
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_incomplete_and_oversized_frames() {
        let mut a = Assembly::default();
        assert!(a.accept(b"CHUNKabc").is_err());
        let mut header = b"FRAME".to_vec();
        header.extend_from_slice(&64u32.to_be_bytes());
        header.extend_from_slice(&48u32.to_be_bytes());
        header.extend_from_slice(&3u32.to_be_bytes());
        a.accept(&header).unwrap();
        assert!(a.accept(b"END").is_err());
        a.accept(b"CHUNKabc").unwrap();
        let f = a.accept(b"END").unwrap().unwrap();
        assert_eq!(f.packet, b"abc");
        assert!(a.accept(b"END").is_err());
    }
}
