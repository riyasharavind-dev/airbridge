use std::sync::{Arc,Mutex,atomic::{AtomicBool,Ordering}};
#[derive(Clone,serde::Serialize,Default)]
pub struct Status {pub running:bool,pub frames:u64,pub error:Option<String>}
#[derive(Default)]pub struct Capture {stop:Arc<AtomicBool>,status:Arc<Mutex<Status>>}
#[tauri::command]
pub fn preview_status(state:tauri::State<'_,Capture>)->Result<Status,String>{state.status.lock().map(|s|s.clone()).map_err(|_|"Preview status unavailable".into())}
#[tauri::command]
pub fn stop_preview(state:tauri::State<'_,Capture>){state.stop.store(true,Ordering::Relaxed);}
#[tauri::command]
pub fn start_preview(display_id:u32,state:tauri::State<'_,Capture>)->Result<(),String>{
 let mut status=state.status.lock().map_err(|_|"Preview status unavailable")?;
 if status.running{return Err("Preview already running".into())}*status=Status{running:true,..Default::default()};
 state.stop.store(false,Ordering::Relaxed);let stop=state.stop.clone();let status=state.status.clone();
 std::thread::spawn(move||{
  let result=run(stop,display_id);
  if let Ok(mut s)=status.lock(){s.running=false;match result{Ok(frames)=>s.frames=frames,Err(error)=>s.error=Some(error)}}
 });Ok(())
}
fn run(stop:Arc<AtomicBool>,display_id:u32)->Result<u64,String>{
 let screens=screenshots::Screen::all().map_err(|e|e.to_string())?;let screen=screens.iter().find(|s|s.display_info.id==display_id).ok_or("Selected display is no longer available; refresh displays")?;
 let first=screen.capture().map_err(|e|e.to_string())?;let width=first.width() as usize;let height=first.height() as usize;
 let mut window=minifb::Window::new("AirBridge - local screen preview (Escape to close)",960,540,minifb::WindowOptions{resize:true,..Default::default()}).map_err(|e|e.to_string())?;
 window.set_target_fps(30);let mut frames=0;
 while window.is_open()&&!window.is_key_down(minifb::Key::Escape)&&!stop.load(Ordering::Relaxed){
  let image=screen.capture().map_err(|e|e.to_string())?;
  if image.width() as usize!=width||image.height() as usize!=height{return Err("Display dimensions changed; restart preview".into())}
  let buffer:Vec<u32>=image.as_raw().chunks_exact(4).map(|p|((p[0] as u32)<<16)|((p[1] as u32)<<8)|p[2] as u32).collect();
  window.update_with_buffer(&buffer,width,height).map_err(|e|e.to_string())?;frames+=1;
 }Ok(frames)
}

#[cfg(test)]mod tests {#[test]#[ignore="Requires an interactive Windows desktop; captures one frame without saving"]fn real_capture_smoke(){let screens=screenshots::Screen::all().expect("screen enumeration");let image=screens.first().expect("display").capture().expect("screen capture");assert!(image.width()>0&&image.height()>0);assert_eq!(image.as_raw().len(),image.width() as usize*image.height() as usize*4);}}

#[derive(serde::Serialize)]pub struct Display {id:u32,width:u32,height:u32,x:i32,y:i32,primary:bool}
#[tauri::command]
pub async fn list_displays()->Result<Vec<Display>,String>{tauri::async_runtime::spawn_blocking(||screenshots::Screen::all().map(|screens|screens.into_iter().map(|screen|{let d=screen.display_info;Display{id:d.id,width:d.width,height:d.height,x:d.x,y:d.y,primary:d.is_primary}}).collect()).map_err(|e|e.to_string())).await.map_err(|e|e.to_string())?}
