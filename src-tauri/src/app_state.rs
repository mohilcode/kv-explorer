use std::path::PathBuf;
use std::sync::Mutex;
use crate::models::kv::RemoteConnection;

pub struct AppState {
    pub current_path: Mutex<Option<PathBuf>>,
    pub remote_connections: Mutex<Vec<RemoteConnection>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            current_path: Mutex::new(None),
            remote_connections: Mutex::new(Vec::new()),
        }
    }
}