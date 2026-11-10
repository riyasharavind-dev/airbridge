use std::{
    collections::HashMap,
    net::UdpSocket,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
const PORT: u16 = 45873;
const QUERY: &[u8] = b"AIRBRIDGE_DISCOVER_V1";
#[derive(Clone, serde::Serialize)]
pub struct Peer {
    pub address: String,
    pub name: String,
    pub os: String,
}
pub struct Discovery {
    peers: Arc<Mutex<HashMap<String, (Peer, Instant)>>>,
    error: Option<String>,
}
impl Discovery {
    pub fn start() -> Self {
        let peers = Arc::new(Mutex::new(HashMap::new()));
        let socket = match UdpSocket::bind(("0.0.0.0", PORT)) {
            Ok(s) => s,
            Err(e) => {
                return Self {
                    peers,
                    error: Some(e.to_string()),
                }
            }
        };
        if let Err(e) = socket
            .set_broadcast(true)
            .and_then(|_| socket.set_read_timeout(Some(Duration::from_millis(250))))
        {
            return Self {
                peers,
                error: Some(e.to_string()),
            };
        }
        let state = peers.clone();
        std::thread::spawn(move || {
            let name = std::env::var("COMPUTERNAME")
                .or_else(|_| std::env::var("HOSTNAME"))
                .unwrap_or_else(|_| "AirBridge device".into())
                .replace(['\n', '\r', '|'], " ");
            let identity = format!("AIRBRIDGE_PEER_V1|{}|{}", name, std::env::consts::OS);
            let mut last = Instant::now() - Duration::from_secs(5);
            let mut buffer = [0u8; 512];
            loop {
                if last.elapsed() > Duration::from_secs(3) {
                    let _ = socket.send_to(QUERY, ("255.255.255.255", PORT));
                    last = Instant::now();
                }
                if let Ok((n, source)) = socket.recv_from(&mut buffer) {
                    if n == 32 && buffer[..8] == *b"ABPING01" {
                        let _ = socket.send_to(&buffer[..n], source);
                    } else if &buffer[..n] == QUERY {
                        let _ = socket.send_to(identity.as_bytes(), source);
                    } else if let Some(peer) = parse(&buffer[..n], source.ip().to_string()) {
                        if let Ok(mut map) = state.lock() {
                            map.insert(peer.address.clone(), (peer, Instant::now()));
                        }
                    }
                }
                if let Ok(mut map) = state.lock() {
                    map.retain(|_, (_, seen)| seen.elapsed() < Duration::from_secs(12));
                }
            }
        });
        Self { peers, error: None }
    }
}
fn parse(bytes: &[u8], address: String) -> Option<Peer> {
    let text = std::str::from_utf8(bytes).ok()?;
    let parts: Vec<_> = text.split('|').collect();
    if parts.len() != 3
        || parts[0] != "AIRBRIDGE_PEER_V1"
        || parts[1].is_empty()
        || parts[1].len() > 128
        || !["windows", "linux"].contains(&parts[2])
        || parts[1].chars().any(char::is_control)
    {
        return None;
    }
    Some(Peer {
        address,
        name: parts[1].into(),
        os: parts[2].into(),
    })
}
#[tauri::command]
pub fn nearby_devices(state: tauri::State<'_, Discovery>) -> Result<Vec<Peer>, String> {
    if let Some(error) = &state.error {
        return Err(format!("Discovery unavailable: {}", error));
    }
    let peers = state
        .peers
        .lock()
        .map_err(|_| "Discovery state unavailable")?;
    Ok(peers.values().map(|(peer, _)| peer.clone()).collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_untrusted_packets() {
        for data in [
            b"garbage".as_slice(),
            b"AIRBRIDGE_PEER_V1||linux",
            b"AIRBRIDGE_PEER_V1|device|linux|extra",
            b"AIRBRIDGE_PEER_V1|device|unknown",
            b"AIRBRIDGE_PEER_V1|bad\nname|linux",
        ] {
            assert!(parse(data, "127.0.0.1".into()).is_none());
        }
    }
    #[test]
    fn accepts_supported_advertisement() {
        let peer = parse(b"AIRBRIDGE_PEER_V1|Lenovo|linux", "192.168.1.2".into()).unwrap();
        assert_eq!(peer.name, "Lenovo");
        assert_eq!(peer.address, "192.168.1.2");
    }
}

#[derive(serde::Serialize)]
pub struct ProbeResult {
    address: String,
    round_trip_ms: f64,
}
fn probe(address: std::net::SocketAddr) -> Result<f64, String> {
    let socket = UdpSocket::bind(("0.0.0.0", 0)).map_err(|e| e.to_string())?;
    socket.connect(address).map_err(|e| e.to_string())?;
    socket
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;
    let mut packet = [0u8; 32];
    packet[..8].copy_from_slice(b"ABPING01");
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos()
        .to_be_bytes();
    packet[8..24].copy_from_slice(&nonce);
    let started = Instant::now();
    socket.send(&packet).map_err(|e| e.to_string())?;
    let mut response = [0u8; 64];
    let size = socket.recv(&mut response).map_err(|e| {
        format!(
            "No valid reply within 2 seconds. Check AirBridge, Wi-Fi isolation and firewall: {}",
            e
        )
    })?;
    if size != packet.len() || response[..size] != packet {
        return Err("Unexpected diagnostic reply".into());
    }
    Ok(started.elapsed().as_secs_f64() * 1000.0)
}
#[tauri::command]
pub async fn check_device(address: String) -> Result<ProbeResult, String> {
    let ip: std::net::Ipv4Addr = address
        .parse()
        .map_err(|_| "Enter a valid local IPv4 address")?;
    if !(ip.is_private() || ip.is_loopback() || ip.is_link_local())
        || ip.is_broadcast()
        || ip.is_unspecified()
    {
        return Err("Only local network IPv4 addresses are allowed".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let round_trip_ms = probe((ip, PORT).into())?;
        Ok(ProbeResult {
            address: ip.to_string(),
            round_trip_ms,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}
#[cfg(test)]
mod probe_tests {
    use super::*;
    #[test]
    fn measures_real_udp_reply() {
        let server = UdpSocket::bind("127.0.0.1:0").unwrap();
        let address = server.local_addr().unwrap();
        server
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let worker = std::thread::spawn(move || {
            let mut b = [0u8; 64];
            let (n, peer) = server.recv_from(&mut b).unwrap();
            server.send_to(&b[..n], peer).unwrap();
        });
        assert!(probe(address).unwrap() >= 0.0);
        worker.join().unwrap();
    }
    #[test]
    fn rejects_invalid_reply() {
        let server = UdpSocket::bind("127.0.0.1:0").unwrap();
        let address = server.local_addr().unwrap();
        server
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let worker = std::thread::spawn(move || {
            let mut b = [0u8; 64];
            let (_, peer) = server.recv_from(&mut b).unwrap();
            server.send_to(b"bad reply", peer).unwrap();
        });
        assert!(probe(address).unwrap_err().contains("Unexpected"));
        worker.join().unwrap();
    }
}
