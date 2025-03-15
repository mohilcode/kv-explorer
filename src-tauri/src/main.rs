#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::fs;
use std::path::PathBuf;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::{command, State};
use std::sync::Mutex;
use serde_json::Value;

struct AppState {
    current_path: Mutex<Option<PathBuf>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct KVEntry {
    key: String,
    blob_id: String,
    expiration: Option<i64>,
    metadata: Option<String>,
    value: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct KVNamespace {
    id: String,
    entries: Vec<KVEntry>,
}

#[command]
fn select_folder(path: String, state: State<AppState>) -> Result<Vec<KVNamespace>, String> {
    let path = PathBuf::from(path);
    *state.current_path.lock().unwrap() = Some(path.clone());

    // Find all KV namespaces
    let kv_path = path.join(".wrangler").join("state").join("v3").join("kv");

    if !kv_path.exists() {
        return Err("No Wrangler KV storage found at this location".to_string());
    }

    let mut namespaces = Vec::new();

    // Read directories in kv folder
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

        // Skip miniflare internal directories
        if !metadata.is_dir() || entry.file_name().to_string_lossy().starts_with("miniflare-") {
            continue;
        }

        let namespace_id = entry.file_name().to_string_lossy().to_string();
        let namespace_dir = entry.path();
        let blob_path = namespace_dir.join("blobs");

        // Find the sqlite database
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

        // Open the SQLite database
        let conn = match Connection::open(db_path.unwrap()) {
            Ok(conn) => conn,
            Err(_) => continue,
        };

        // Get KV entries
        let mut stmt = match conn.prepare("SELECT key, blob_id, expiration, metadata FROM _mf_entries") {
            Ok(stmt) => stmt,
            Err(_) => continue,
        };

        let entries_iter = match stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let blob_id: String = row.get(1)?;
            let expiration: Option<i64> = row.get(2)?;
            let metadata: Option<String> = row.get(3)?;

            // Read blob content
            let mut value = None;
            let blob_file = blob_path.join(&blob_id);

            if blob_file.exists() {
                if let Ok(content) = fs::read_to_string(&blob_file) {
                    // Look for the start of JSON content ('{' or '[')
                    let json_start = content.find(|c| c == '{' || c == '[');

                    if let Some(start_pos) = json_start {
                        let json_content = &content[start_pos..];

                        // Try to parse as JSON
                        if let Ok(parsed) = serde_json::from_str(json_content) {
                            value = Some(parsed);
                        }
                    }
                }
            }

            Ok(KVEntry {
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

        namespaces.push(KVNamespace {
            id: namespace_id,
            entries,
        });
    }

    Ok(namespaces)
}

#[command]
fn update_kv(namespace_id: String, key: String, value_str: String, state: State<AppState>) -> Result<(), String> {
    let path = match state.current_path.lock().unwrap().as_ref() {
        Some(path) => path.clone(),
        None => return Err("No folder selected".to_string()),
    };

    let kv_path = path.join(".wrangler").join("state").join("v3").join("kv");
    let namespace_path = kv_path.join(&namespace_id).join("blobs");

    // Find the sqlite database
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

    // Open the SQLite database
    let conn = match Connection::open(db_path.unwrap()) {
        Ok(conn) => conn,
        Err(_) => return Err("Failed to open SQLite database".to_string()),
    };

    // Get the blob ID for the key
    let mut stmt = match conn.prepare("SELECT blob_id FROM _mf_entries WHERE key = ?") {
        Ok(stmt) => stmt,
        Err(_) => return Err("Failed to prepare SQL statement".to_string()),
    };

    let blob_id: String = match stmt.query_row([&key], |row| row.get(0)) {
        Ok(blob_id) => blob_id,
        Err(_) => return Err("Key not found".to_string()),
    };

    // Update the blob file
    let blob_file = namespace_path.join(&blob_id);
    if !blob_file.exists() {
        return Err("Blob file not found".to_string());
    }

    // Parse the new value to ensure it's valid JSON
    let _: Value = match serde_json::from_str(&value_str) {
        Ok(value) => value,
        Err(_) => return Err("Invalid JSON value".to_string()),
    };

    // Write the updated content to the blob file
    match fs::write(&blob_file, value_str) {
        Ok(_) => Ok(()),
        Err(_) => Err("Failed to write to blob file".to_string()),
    }
}

#[command]
fn delete_kv(namespace_id: String, keys: Vec<String>, state: State<AppState>) -> Result<(), String> {
    let path = match state.current_path.lock().unwrap().as_ref() {
        Some(path) => path.clone(),
        None => return Err("No folder selected".to_string()),
    };

    let kv_path = path.join(".wrangler").join("state").join("v3").join("kv");
    let namespace_path = kv_path.join(&namespace_id).join("blobs");

    // Find the sqlite database
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

    // Open the SQLite database
    let conn = match Connection::open(db_path.unwrap()) {
        Ok(conn) => conn,
        Err(_) => return Err("Failed to open SQLite database".to_string()),
    };

    // Start a transaction
    match conn.execute("BEGIN TRANSACTION", []) {
        Ok(_) => {},
        Err(_) => return Err("Failed to start transaction".to_string()),
    }

    for key in &keys {
        // Get the blob ID for the key
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

        // Delete the KV entry from the database
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

        // Delete the blob file
        let blob_file = namespace_path.join(&blob_id);
        if blob_file.exists() {
            if let Err(_) = fs::remove_file(&blob_file) {
                conn.execute("ROLLBACK", []).ok();
                return Err(format!("Failed to delete blob file for key: {}", key));
            }
        }
    }

    // Commit the transaction
    match conn.execute("COMMIT", []) {
        Ok(_) => Ok(()),
        Err(_) => {
            conn.execute("ROLLBACK", []).ok();
            Err("Failed to commit transaction".to_string())
        }
    }
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            current_path: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![select_folder, update_kv, delete_kv])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}