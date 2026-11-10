use std::{io::{Read,Write},net::{TcpListener,TcpStream,Ipv4Addr},sync::{Arc,Mutex,mpsc},time::{Duration,Instant}};
const PORT:u16=45874;const PATTERN:&str="Noise_NN_25519_ChaChaPoly_BLAKE2s";
#[derive(Clone,Default,serde::Serialize)]pub struct Status{pub phase:String,pub peer:String,pub verification:String,pub local_approved:bool,pub remote_approved:bool,pub error:Option<String>}
#[derive(Default)]pub struct Pairing{status:Arc<Mutex<Status>>,commands:Mutex<Option<mpsc::Sender<String>>>}
fn local(ip:Ipv4Addr)->bool{ip.is_private()||ip.is_loopback()||ip.is_link_local()}
fn update(state:&Arc<Mutex<Status>>,f:impl FnOnce(&mut Status)){if let Ok(mut s)=state.lock(){f(&mut s)}}
fn send(socket:&mut TcpStream,data:&[u8])->Result<(),String>{if data.len()>4096{return Err("Control message too large".into())}socket.write_all(&(data.len() as u16).to_be_bytes()).and_then(|_|socket.write_all(data)).map_err(|e|e.to_string())}
fn receive(socket:&mut TcpStream)->Result<Vec<u8>,String>{let mut header=[0;2];socket.read_exact(&mut header).map_err(|e|e.to_string())?;let n=u16::from_be_bytes(header)as usize;if n==0||n>4096{return Err("Invalid control message length".into())}let mut data=vec![0;n];socket.read_exact(&mut data).map_err(|e|e.to_string())?;Ok(data)}
fn handshake(socket:&mut TcpStream,initiator:bool)->Result<(snow::TransportState,String),String>{
 let builder=snow::Builder::new(PATTERN.parse().map_err(|e:snow::Error|e.to_string())?).prologue(b"AirBridge control v1").map_err(|e|e.to_string())?;
 let mut noise=if initiator{builder.build_initiator()}else{builder.build_responder()}.map_err(|e|e.to_string())?;let mut buffer=[0;4096];
 if initiator{let n=noise.write_message(&[],&mut buffer).map_err(|e|e.to_string())?;send(socket,&buffer[..n])?;noise.read_message(&receive(socket)?,&mut buffer).map_err(|e|e.to_string())?;}else{noise.read_message(&receive(socket)?,&mut buffer).map_err(|e|e.to_string())?;let n=noise.write_message(&[],&mut buffer).map_err(|e|e.to_string())?;send(socket,&buffer[..n])?;}
 let verification=noise.get_handshake_hash().iter().map(|b|format!("{:02X}",b)).collect::<Vec<_>>().join("");Ok((noise.into_transport_mode().map_err(|e|e.to_string())?,verification))
}
fn secure_send(socket:&mut TcpStream,noise:&mut snow::TransportState,message:&[u8])->Result<(),String>{let mut b=[0;4096];let n=noise.write_message(message,&mut b).map_err(|e|e.to_string())?;send(socket,&b[..n])}
fn session(mut socket:TcpStream,initiator:bool,state:Arc<Mutex<Status>>,commands:mpsc::Receiver<String>)->Result<(),String>{
 socket.set_read_timeout(Some(Duration::from_secs(5))).map_err(|e|e.to_string())?;socket.set_write_timeout(Some(Duration::from_secs(2))).map_err(|e|e.to_string())?;
 update(&state,|s|{s.phase="handshaking".into();s.peer=socket.peer_addr().map(|p|p.ip().to_string()).unwrap_or_default()});
 let(mut noise,code)=handshake(&mut socket,initiator)?;update(&state,|s|{s.phase="verify".into();s.verification=code});
 socket.set_read_timeout(Some(Duration::from_millis(100))).map_err(|e|e.to_string())?;
 let started=Instant::now();let mut last_received=Instant::now();let mut last_ping=Instant::now();let(mut approved,mut remote)=(false,false);let mut incoming=Vec::new();let mut bytes=[0;4096];
 loop{
  if let Ok(command)=commands.try_recv(){match command.as_str(){"approve"=>{if !approved{secure_send(&mut socket,&mut noise,b"APPROVE")?;approved=true;update(&state,|s|s.local_approved=true);}},"disconnect"|"reject"=>{let _=secure_send(&mut socket,&mut noise,b"CLOSE");return Ok(())},_=>return Err("Unknown session command".into())}}
  if !(approved&&remote)&&started.elapsed()>Duration::from_secs(60){return Err("Pairing approval expired after 60 seconds".into())}
  if last_received.elapsed()>Duration::from_secs(15){return Err("Peer connection timed out".into())}
  if last_ping.elapsed()>Duration::from_secs(3){secure_send(&mut socket,&mut noise,b"PING")?;last_ping=Instant::now();}
  match socket.read(&mut bytes){Ok(0)=>return Ok(()),Ok(n)=>incoming.extend_from_slice(&bytes[..n]),Err(e)if [std::io::ErrorKind::WouldBlock,std::io::ErrorKind::TimedOut].contains(&e.kind())=>{},Err(e)=>return Err(e.to_string())}
  while incoming.len()>=2{let n=u16::from_be_bytes([incoming[0],incoming[1]])as usize;if n<16||n>4096{return Err("Invalid encrypted control length".into())}if incoming.len()<n+2{break}let mut plain=[0;4096];let size=noise.read_message(&incoming[2..n+2],&mut plain).map_err(|_|"Encrypted control authentication failed")?;incoming.drain(..n+2);last_received=Instant::now();match &plain[..size]{b"APPROVE"=>{remote=true;update(&state,|s|s.remote_approved=true)},b"PING"=>{},b"CLOSE"=>return Ok(()),_=>return Err("Unknown encrypted control message".into())}}
  if approved&&remote{update(&state,|s|s.phase="connected".into())}
 }
}
fn launch(pairing:&Pairing,address:Option<String>)->Result<(),String>{
 let mut slot=pairing.commands.lock().map_err(|_|"Session state unavailable")?;
 let mut status=pairing.status.lock().map_err(|_|"Session state unavailable")?;
 if ["listening","connecting","handshaking","verify","connected"].contains(&status.phase.as_str()){return Err("Disconnect the current session first".into())}
 let target=match address{Some(text)=>{let ip:Ipv4Addr=text.parse().map_err(|_|"Invalid IPv4 address")?;if !local(ip){return Err("Only local network addresses are supported".into())}Some(ip)},None=>None};
 let listener=if target.is_none(){let l=TcpListener::bind(("0.0.0.0",PORT)).map_err(|e|e.to_string())?;l.set_nonblocking(true).map_err(|e|e.to_string())?;Some(l)}else{None};
 let(tx,rx)=mpsc::channel();*slot=Some(tx);*status=Status{phase:if target.is_some(){"connecting"}else{"listening"}.into(),..Default::default()};drop(status);
 let state=pairing.status.clone();std::thread::spawn(move||{
  let result=(||{let socket=if let Some(ip)=target{TcpStream::connect_timeout(&(ip,PORT).into(),Duration::from_secs(5)).map_err(|e|e.to_string())?}else{let listener=listener.ok_or("Listener unavailable")?;let deadline=Instant::now();loop{if rx.try_recv().is_ok(){return Ok(())}if deadline.elapsed()>Duration::from_secs(60){return Err("No incoming connection within 60 seconds".into())}match listener.accept(){Ok((socket,peer))=>{if let std::net::IpAddr::V4(ip)=peer.ip(){if local(ip){break socket}}},Err(e)if e.kind()==std::io::ErrorKind::WouldBlock=>{},Err(e)=>return Err(e.to_string())}std::thread::sleep(Duration::from_millis(50));}};session(socket,target.is_some(),state.clone(),rx)})();
  update(&state,|s|{s.phase="disconnected".into();s.verification.clear();s.local_approved=false;s.remote_approved=false;s.error=result.err();});
 });Ok(())
}
#[tauri::command]pub fn listen_pairing(state:tauri::State<'_,Pairing>)->Result<(),String>{launch(&state,None)}
#[tauri::command]pub fn connect_pairing(address:String,state:tauri::State<'_,Pairing>)->Result<(),String>{launch(&state,Some(address))}
#[tauri::command]pub fn pairing_status(state:tauri::State<'_,Pairing>)->Result<Status,String>{state.status.lock().map(|s|s.clone()).map_err(|_|"Session status unavailable".into())}
#[tauri::command]pub fn pairing_action(action:String,state:tauri::State<'_,Pairing>)->Result<(),String>{if !["approve","reject","disconnect"].contains(&action.as_str()){return Err("Unknown action".into())}let status=state.status.lock().map_err(|_|"Session state unavailable")?;if action=="approve"&&status.phase!="verify"{return Err("No verification pending".into())}drop(status);state.commands.lock().map_err(|_|"Session channel unavailable")?.as_ref().ok_or("No session")?.send(action).map_err(|_|"Session already closed".into())}
#[cfg(test)]mod tests{use super::*;#[test]fn encrypted_handshake_and_tamper_rejection(){let listener=TcpListener::bind("127.0.0.1:0").unwrap();let addr=listener.local_addr().unwrap();let responder=std::thread::spawn(move||{let(mut socket,_)=listener.accept().unwrap();socket.set_read_timeout(Some(Duration::from_secs(3))).unwrap();let(mut noise,code)=handshake(&mut socket,false).unwrap();let data=receive(&mut socket).unwrap();let mut b=[0;4096];assert!(noise.read_message(&data,&mut b).is_err());code});let mut socket=TcpStream::connect(addr).unwrap();socket.set_read_timeout(Some(Duration::from_secs(3))).unwrap();let(mut noise,code)=handshake(&mut socket,true).unwrap();let mut b=[0;4096];let n=noise.write_message(b"APPROVE",&mut b).unwrap();b[n-1]^=1;send(&mut socket,&b[..n]).unwrap();assert_eq!(code,responder.join().unwrap());}}

#[cfg(test)]mod session_tests{use super::*;
fn wait(state:&Arc<Mutex<Status>>,phase:&str){let start=Instant::now();while state.lock().unwrap().phase!=phase{assert!(start.elapsed()<Duration::from_secs(5),"session did not reach {}",phase);std::thread::sleep(Duration::from_millis(10));}}
#[test]fn requires_both_approvals_and_disconnects(){let listener=TcpListener::bind("127.0.0.1:0").unwrap();let address=listener.local_addr().unwrap();let a=Arc::new(Mutex::new(Status::default()));let b=Arc::new(Mutex::new(Status::default()));let(at,ar)=mpsc::channel();let(bt,br)=mpsc::channel();let bs=b.clone();let responder=std::thread::spawn(move||session(listener.accept().unwrap().0,false,bs,br));let ast=a.clone();let initiator=std::thread::spawn(move||session(TcpStream::connect(address).unwrap(),true,ast,ar));wait(&a,"verify");wait(&b,"verify");assert_eq!(a.lock().unwrap().verification,b.lock().unwrap().verification);at.send("approve".into()).unwrap();std::thread::sleep(Duration::from_millis(200));assert_eq!(a.lock().unwrap().phase,"verify");assert_eq!(b.lock().unwrap().phase,"verify");bt.send("approve".into()).unwrap();wait(&a,"connected");wait(&b,"connected");at.send("disconnect".into()).unwrap();assert!(initiator.join().unwrap().is_ok());assert!(responder.join().unwrap().is_ok());}
}
