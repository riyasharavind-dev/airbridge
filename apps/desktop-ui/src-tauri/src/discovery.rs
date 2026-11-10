use std::{collections::HashMap, net::UdpSocket, sync::{Arc,Mutex}, time::{Duration,Instant}};
const PORT:u16=45873;
const QUERY:&[u8]=b"AIRBRIDGE_DISCOVER_V1";
#[derive(Clone,serde::Serialize)]
pub struct Peer { pub address:String, pub name:String, pub os:String }
pub struct Discovery { peers:Arc<Mutex<HashMap<String,(Peer,Instant)>>>, error:Option<String> }
impl Discovery {
 pub fn start()->Self {
  let peers=Arc::new(Mutex::new(HashMap::new()));
  let socket=match UdpSocket::bind(("0.0.0.0",PORT)){Ok(s)=>s,Err(e)=>return Self{peers,error:Some(e.to_string())}};
  if let Err(e)=socket.set_broadcast(true).and_then(|_|socket.set_read_timeout(Some(Duration::from_millis(250)))) {return Self{peers,error:Some(e.to_string())};}
  let state=peers.clone();
  std::thread::spawn(move||{
   let name=std::env::var("COMPUTERNAME").or_else(|_|std::env::var("HOSTNAME")).unwrap_or_else(|_|"AirBridge device".into()).replace(['\n','\r','|']," ");
   let identity=format!("AIRBRIDGE_PEER_V1|{}|{}",name,std::env::consts::OS);
   let mut last=Instant::now()-Duration::from_secs(5); let mut buffer=[0u8;512];
   loop {
    if last.elapsed()>Duration::from_secs(3){let _=socket.send_to(QUERY,("255.255.255.255",PORT));last=Instant::now();}
    if let Ok((n,source))=socket.recv_from(&mut buffer){
     if &buffer[..n]==QUERY {let _=socket.send_to(identity.as_bytes(),source);}
     else if let Some(peer)=parse(&buffer[..n],source.ip().to_string()) {if let Ok(mut map)=state.lock(){map.insert(peer.address.clone(),(peer,Instant::now()));}}
    }
    if let Ok(mut map)=state.lock(){map.retain(|_,(_,seen)|seen.elapsed()<Duration::from_secs(12));}
   }
  }); Self{peers,error:None}
 }
}
fn parse(bytes:&[u8],address:String)->Option<Peer>{
 let text=std::str::from_utf8(bytes).ok()?;let parts:Vec<_>=text.split('|').collect();
 if parts.len()!=3 || parts[0]!="AIRBRIDGE_PEER_V1" || parts[1].is_empty() || parts[1].len()>128 || !["windows","linux"].contains(&parts[2]) || parts[1].chars().any(char::is_control){return None;}
 Some(Peer{address,name:parts[1].into(),os:parts[2].into()})
}
#[tauri::command]
pub fn nearby_devices(state:tauri::State<'_,Discovery>)->Result<Vec<Peer>,String>{
 if let Some(error)=&state.error{return Err(format!("Discovery unavailable: {}",error));}
 let peers=state.peers.lock().map_err(|_|"Discovery state unavailable")?;
 Ok(peers.values().map(|(peer,_)|peer.clone()).collect())
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn rejects_untrusted_packets(){for data in [b"garbage".as_slice(),b"AIRBRIDGE_PEER_V1||linux",b"AIRBRIDGE_PEER_V1|device|linux|extra",b"AIRBRIDGE_PEER_V1|device|unknown",b"AIRBRIDGE_PEER_V1|bad\nname|linux"]{assert!(parse(data,"127.0.0.1".into()).is_none());}}
 #[test] fn accepts_supported_advertisement(){let peer=parse(b"AIRBRIDGE_PEER_V1|Lenovo|linux","192.168.1.2".into()).unwrap();assert_eq!(peer.name,"Lenovo");assert_eq!(peer.address,"192.168.1.2");}
}
