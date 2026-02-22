use std::sync::Mutex;

pub struct ActiveRecording {
    pub session_id: String,
    pub stop_tx: std::sync::mpsc::Sender<()>,
    pub join_handle: Option<std::thread::JoinHandle<Result<(), String>>>,
}

pub struct AppState {
    pub active_recording: Mutex<Option<ActiveRecording>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_recording: Mutex::new(None),
        }
    }
}
