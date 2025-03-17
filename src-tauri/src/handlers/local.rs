use std::fs;
use std::path::{Path, PathBuf};
use rusqlite::Connection;
use serde_json::Value;
use tauri::{command, State};

use crate::app_state::AppState;
use crate::models::kv::{KVEntry, KVNamespace, LocalFolder, LocalFolderInfo};

fn extract_folder_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown Folder")
        .to_string()
}

#[command]
pub fn get_folders(state: State<AppState>) -> Vec<LocalFolderInfo> {
    let folders = state.folders.lock().unwrap();
    folders.values().map(|f| f.into()).collect()
}

#[command]
pub fn add_folder(path: String, state: State<AppState>) -> Result<Vec<KVNamespace>, String> {
    let path = PathBuf::from(path);
    let folder_name = extract_folder_name(&path);

    let folder_id = {
        let db = state.db.lock().unwrap();
        match db.save_folder(&path.to_string_lossy(), &folder_name) {
            Ok(id) => id,
            Err(e) => return Err(format!("Failed to save folder: {}", e))
        }
    };

    let local_folder = LocalFolder {
        id: folder_id,
        path: path.clone(),
        name: folder_name,
    };

    {
        let mut folders = state.folders.lock().unwrap();
        folders.insert(folder_id, local_folder);
    }

    load_namespaces_for_folder(&path, folder_id)
}

#[command]
pub fn remove_folder(folder_id: i64, state: State<AppState>) -> Result<(), String> {
    {
        let db = state.db.lock().unwrap();
        if let Err(e) = db.remove_folder(folder_id) {
            return Err(format!("Failed to remove folder: {}", e));
        }
    }

    {
        let mut folders = state.folders.lock().unwrap();
        folders.remove(&folder_id);
    }

    Ok(())
}

#[command]
pub fn load_folder(folder_id: i64, state: State<AppState>) -> Result<Vec<KVNamespace>, String> {
    let folder_path = {
        let folders = state.folders.lock().unwrap();
        match folders.get(&folder_id) {
            Some(folder) => folder.path.clone(),
            None => return Err("Folder not found".to_string())
        }
    };

    {
        let db = state.db.lock().unwrap();
        if let Err(e) = db.update_folder_timestamp(&folder_path.to_string_lossy()) {
            eprintln!("Failed to update folder timestamp: {}", e);
        }
    }

    load_namespaces_for_folder(&folder_path, folder_id)
}

fn load_namespaces_for_folder(path: &PathBuf, folder_id: i64) -> Result<Vec<KVNamespace>, String> {
    let kv_path = path.join(".wrangler").join("state").join("v3").join("kv");

    if !kv_path.exists() {
        return Err("No Wrangler KV storage found at this location".to_string());
    }

    let mut namespaces = Vec::new();

    let entries = match fs::read_dir(&kv_path) {
        Ok(entries) => entries,
        Err(_) => return Err("Failed to read KV directory".to_string()),
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };

        if !metadata.is_dir() || entry.file_name().to_string_lossy().starts_with("miniflare-") {
            continue;
        }

        let namespace_name = entry.file_name().to_string_lossy().to_string();
        let namespace_id = format!("folder-{}-ns-{}", folder_id, namespace_name);
        let namespace_dir = entry.path();
        let blob_path = namespace_dir.join("blobs");

        let db_dir = kv_path.join("miniflare-KVNamespaceObject");
        let mut db_path = None;

        if let Ok(db_entries) = fs::read_dir(&db_dir) {
            for db_entry in db_entries {
                if let Ok(db_entry) = db_entry {
                    if db_entry.path().extension().unwrap_or_default() == "sqlite" {
                        db_path = Some(db_entry.path());
                        break;
                    }
                }
            }
        }

        if db_path.is_none() {
            continue;
        }

        let conn = match Connection::open(db_path.unwrap()) {
            Ok(conn) => conn,
            Err(_) => continue,
        };

        let mut stmt = match conn.prepare("SELECT key, blob_id, expiration, metadata FROM _mf_entries") {
            Ok(stmt) => stmt,
            Err(_) => continue,
        };

        let entries_iter = match stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let blob_id: String = row.get(1)?;
            let expiration: Option<i64> = row.get(2)?;
            let metadata: Option<String> = row.get(3)?;

            let mut value = None;
            let blob_file = blob_path.join(&blob_id);

            if blob_file.exists() {
                if let Ok(content) = fs::read_to_string(&blob_file) {
                    let json_start = content.find(|c| c == '{' || c == '[');

                    if let Some(start_pos) = json_start {
                        let json_content = &content[start_pos..];

                        if let Ok(parsed) = serde_json::from_str(json_content) {
                            value = Some(parsed);
                        }
                    }
                }
            }

            Ok(KVEntry {
                id: format!("{}-{}", namespace_id, blob_id),
                key,
                blob_id,
                expiration,
                metadata,
                value,
            })
        }) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        let mut entries = Vec::new();
        for entry in entries_iter {
            if let Ok(entry) = entry {
                entries.push(entry);
            }
        }
        let entries_count = entries.len();

        namespaces.push(KVNamespace {
            id: namespace_id.clone(),
            name: namespace_name.to_uppercase(),
            entries,
            r#type: "local".to_string(),
            account_id: None,
            folder_id: Some(folder_id),
            count: Some(entries_count)
        });
    }

    Ok(namespaces)
}

#[command]
pub fn update_kv(folder_id: i64, namespace_id: String, key: String, value_str: String, state: State<AppState>) -> Result<(), String> {
    let namespace_name = if namespace_id.starts_with("folder-") {
        namespace_id.splitn(4, "-").nth(3)
            .ok_or_else(|| "Invalid namespace ID format".to_string())?
    } else {
        &namespace_id
    };

    let path = {
        let folders = state.folders.lock().unwrap();
        match folders.get(&folder_id) {
            Some(folder) => folder.path.clone(),
            None => return Err("Folder not found".to_string())
        }
    };

    let kv_path = path.join(".wrangler").join("state").join("v3").join("kv");
    let namespace_path = kv_path.join(&namespace_name).join("blobs");

    let db_dir = kv_path.join("miniflare-KVNamespaceObject");
    let mut db_path = None;

    if let Ok(db_entries) = fs::read_dir(&db_dir) {
        for db_entry in db_entries {
            if let Ok(db_entry) = db_entry {
                if db_entry.path().extension().unwrap_or_default() == "sqlite" {
                    db_path = Some(db_entry.path());
                    break;
                }
            }
        }
    }

    if db_path.is_none() {
        return Err("SQLite database not found".to_string());
    }

    let conn = match Connection::open(db_path.unwrap()) {
        Ok(conn) => conn,
        Err(_) => return Err("Failed to open SQLite database".to_string()),
    };

    let mut stmt = match conn.prepare("SELECT blob_id FROM _mf_entries WHERE key = ?") {
        Ok(stmt) => stmt,
        Err(_) => return Err("Failed to prepare SQL statement".to_string()),
    };

    let blob_id: String = match stmt.query_row([&key], |row| row.get(0)) {
        Ok(blob_id) => blob_id,
        Err(_) => return Err("Key not found".to_string()),
    };

    let blob_file = namespace_path.join(&blob_id);
    if !blob_file.exists() {
        return Err("Blob file not found".to_string());
    }

    let _: Value = match serde_json::from_str(&value_str) {
        Ok(value) => value,
        Err(_) => return Err("Invalid JSON value".to_string()),
    };

    match fs::write(&blob_file, value_str) {
        Ok(_) => Ok(()),
        Err(_) => Err("Failed to write to blob file".to_string()),
    }
}

#[command]
pub fn delete_kv(folder_id: i64, namespace_id: String, keys: Vec<String>, state: State<AppState>) -> Result<(), String> {
    let namespace_name = if namespace_id.starts_with("folder-") {
        namespace_id.splitn(4, "-").nth(3)
            .ok_or_else(|| "Invalid namespace ID format".to_string())?
    } else {
        &namespace_id
    };

    let path = {
        let folders = state.folders.lock().unwrap();
        match folders.get(&folder_id) {
            Some(folder) => folder.path.clone(),
            None => return Err("Folder not found".to_string())
        }
    };

    let kv_path = path.join(".wrangler").join("state").join("v3").join("kv");
    let namespace_path = kv_path.join(&namespace_name).join("blobs");

    let db_dir = kv_path.join("miniflare-KVNamespaceObject");
    let mut db_path = None;

    if let Ok(db_entries) = fs::read_dir(&db_dir) {
        for db_entry in db_entries {
            if let Ok(db_entry) = db_entry {
                if db_entry.path().extension().unwrap_or_default() == "sqlite" {
                    db_path = Some(db_entry.path());
                    break;
                }
            }
        }
    }

    if db_path.is_none() {
        return Err("SQLite database not found".to_string());
    }

    let conn = match Connection::open(db_path.unwrap()) {
        Ok(conn) => conn,
        Err(_) => return Err("Failed to open SQLite database".to_string()),
    };

    match conn.execute("BEGIN TRANSACTION", []) {
        Ok(_) => {},
        Err(_) => return Err("Failed to start transaction".to_string()),
    }

    for key in &keys {
        let mut stmt = match conn.prepare("SELECT blob_id FROM _mf_entries WHERE key = ?") {
            Ok(stmt) => stmt,
            Err(_) => {
                conn.execute("ROLLBACK", []).ok();
                return Err("Failed to prepare SQL statement".to_string());
            }
        };

        let blob_id: String = match stmt.query_row([key], |row| row.get(0)) {
            Ok(blob_id) => blob_id,
            Err(_) => {
                conn.execute("ROLLBACK", []).ok();
                return Err(format!("Key not found: {}", key));
            }
        };

        let mut stmt = match conn.prepare("DELETE FROM _mf_entries WHERE key = ?") {
            Ok(stmt) => stmt,
            Err(_) => {
                conn.execute("ROLLBACK", []).ok();
                return Err("Failed to prepare SQL statement".to_string());
            }
        };

        match stmt.execute([key]) {
            Ok(_) => {},
            Err(_) => {
                conn.execute("ROLLBACK", []).ok();
                return Err(format!("Failed to delete key: {}", key));
            }
        }

        let blob_file = namespace_path.join(&blob_id);
        if blob_file.exists() {
            if let Err(_) = fs::remove_file(&blob_file) {
                conn.execute("ROLLBACK", []).ok();
                return Err(format!("Failed to delete blob file for key: {}", key));
            }
        }
    }

    match conn.execute("COMMIT", []) {
        Ok(_) => Ok(()),
        Err(_) => {
            conn.execute("ROLLBACK", []).ok();
            Err("Failed to commit transaction".to_string())
        }
    }
}