use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
#[derive(Default)]
pub struct Workers {
    pub stop: Arc<AtomicBool>,
    handles: Mutex<Vec<std::thread::JoinHandle<()>>>,
}
impl Workers {
    pub fn track(&self, handle: std::thread::JoinHandle<()>) {
        if let Ok(mut handles) = self.handles.lock() {
            if self.stop.load(Ordering::Relaxed) {
                drop(handles);
                let _ = handle.join();
                return;
            }
            handles.retain(|h| !h.is_finished());
            handles.push(handle);
        }
    }
    pub fn shutdown(&self) {
        self.stop.store(true, Ordering::Relaxed);
        let handles = if let Ok(mut handles) = self.handles.lock() {
            std::mem::take(&mut *handles)
        } else {
            Vec::new()
        };
        for h in handles {
            let _ = h.join();
        }
    }
}
impl Drop for Workers {
    fn drop(&mut self) {
        self.shutdown()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shutdown_cancels_and_joins() {
        let workers = Workers::default();
        let stop = workers.stop.clone();
        let done = Arc::new(AtomicBool::new(false));
        let finished = done.clone();
        workers.track(std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(1))
            }
            finished.store(true, Ordering::Relaxed)
        }));
        workers.shutdown();
        assert!(done.load(Ordering::Relaxed));
        workers.shutdown();
    }
}
