use std::path::PathBuf;
use std::sync::Mutex;
use std::collections::HashMap;
use crate::models::kv::{RemoteConnection, LocalFolder};
use crate::persistence::Database;

pub struct AppState {
    pub db: Mutex<Database>,
    pub folders: Mutex<HashMap<i64, LocalFolder>>,
    pub remote_connections: Mutex<Vec<RemoteConnection>>,
}

impl AppState {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let db = match Database::new(&app_data_dir) {
            Ok(db) => db,
            Err(e) => {
                eprintln!("Failed to initialize database: {}", e);
                Database::new(&std::env::temp_dir()).unwrap()
            }
        };

        let folders = match db.get_folders() {
            Ok(folders) => {
                let mut folder_map = HashMap::new();
                for (id, path, name) in folders {
                    folder_map.insert(id, LocalFolder {
                        id,
                        path: PathBuf::from(path),
                        name,
                    });
                }
                folder_map
            },
            Err(e) => {
                eprintln!("Failed to load folders: {}", e);
                HashMap::new()
            }
        };

        let remote_connections = match db.get_remote_connections() {
            Ok(connections) => connections,
            Err(e) => {
                eprintln!("Failed to load remote connections: {}", e);
                Vec::new()
            }
        };

        AppState {
            db: Mutex::new(db),
            folders: Mutex::new(folders),
            remote_connections: Mutex::new(remote_connections),
        }
    }
}