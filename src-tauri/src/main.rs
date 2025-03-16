#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::{command, State};
use serde_json::Value;
use reqwest::{Client, header};

struct AppState {
    current_path: Mutex<Option<PathBuf>>,
    remote_connections: Mutex<Vec<RemoteConnection>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RemoteConnection {
    account_id: String,
    api_token: String,
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
    #[serde(default)]
    name: String,
    #[serde(default = "default_namespace_type")]
    r#type: String,
}

fn default_namespace_type() -> String {
    "local".to_string()
}

#[derive(Serialize, Deserialize, Debug)]
struct CloudflareNamespace {
    id: String,
    title: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CloudflareListResponse<T> {
    success: bool,
    errors: Vec<CloudflareError>,
    messages: Vec<String>,
    result: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CloudflareError {
    code: i32,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CloudflareKey {
    name: String,
    expiration: Option<i64>,
    metadata: Option<Value>,
}

// Local file system KV functions
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
            id: namespace_id.clone(),
            name: namespace_id.to_uppercase(),
            entries,
            r#type: "local".to_string(),
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

// Cloudflare API functions
#[command]
async fn connect_cloudflare(account_id: String, api_token: String, state: State<'_, AppState>) -> Result<(), String> {
    // Validate token by making a test API call
    let client = Client::new();
    let url = format!("https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces", account_id);

    let response = client
        .get(&url)
        .header(header::AUTHORIZATION, format!("Bearer {}", api_token))
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("API authentication failed with status: {}", status));
    }

    // Store connection info in state
    let mut connections = state.remote_connections.lock().unwrap();
    connections.push(RemoteConnection {
        account_id,
        api_token,
    });

    Ok(())
}

#[command]
async fn get_remote_namespaces(state: State<'_, AppState>) -> Result<Vec<KVNamespace>, String> {
    let connections = state.remote_connections.lock().unwrap().clone();
    if connections.is_empty() {
        return Ok(vec![]);
    }

    let mut all_namespaces = Vec::new();

    for connection in connections {
        let client = Client::new();
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces",
            connection.account_id
        );

        let response = client
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", connection.api_token))
            .send()
            .await
            .map_err(|e| format!("API request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()));
        }

        let response_data: CloudflareListResponse<CloudflareNamespace> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse API response: {}", e))?;

        if !response_data.success {
            let error_msg = response_data.errors
                .iter()
                .map(|e| format!("{}: {}", e.code, e.message))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!("API request failed: {}", error_msg));
        }

        // Convert CloudflareNamespace to KVNamespace
        for namespace in response_data.result {
            all_namespaces.push(KVNamespace {
                id: namespace.id,
                name: namespace.title,
                entries: vec![],
                r#type: "remote".to_string(),
            });
        }
    }

    Ok(all_namespaces)
}

#[command]
async fn get_remote_keys(account_id: String, namespace_id: String, state: State<'_, AppState>) -> Result<Vec<KVEntry>, String> {
    let connections = state.remote_connections.lock().unwrap().clone();
    let connection = connections
        .iter()
        .find(|c| c.account_id == account_id)
        .ok_or_else(|| "Connection not found".to_string())?;

    let client = Client::new();
    let url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/keys",
        account_id, namespace_id
    );

    let response = client
        .get(&url)
        .header(header::AUTHORIZATION, format!("Bearer {}", connection.api_token))
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()));
    }

    let response_data: CloudflareListResponse<CloudflareKey> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    if !response_data.success {
        let error_msg = response_data.errors
            .iter()
            .map(|e| format!("{}: {}", e.code, e.message))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!("API request failed: {}", error_msg));
    }

    // Convert CloudflareKey to KVEntry
    let mut entries = Vec::new();
    for (index, key) in response_data.result.iter().enumerate() {
        entries.push(KVEntry {
            key: key.name.clone(),
            blob_id: format!("remote-{}", index),
            expiration: key.expiration,
            metadata: key.metadata.as_ref().map(|m| m.to_string()),
            value: None, // Values are fetched separately
        });
    }

    Ok(entries)
}

#[command]
async fn get_remote_value(account_id: String, namespace_id: String, key_name: String, state: State<'_, AppState>) -> Result<Value, String> {
    let connections = state.remote_connections.lock().unwrap().clone();
    let connection = connections
        .iter()
        .find(|c| c.account_id == account_id)
        .ok_or_else(|| "Connection not found".to_string())?;

    let client = Client::new();
    let url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
        account_id, namespace_id, key_name
    );

    let response = client
        .get(&url)
        .header(header::AUTHORIZATION, format!("Bearer {}", connection.api_token))
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()));
    }

    // Get the text content first
    let text = response.text().await
        .map_err(|e| format!("Failed to get response text: {}", e))?;

    // Try to parse as JSON, fallback to returning as a string value
    match serde_json::from_str::<Value>(&text) {
        Ok(value) => Ok(value),
        Err(_) => Ok(Value::String(text)),
    }
}

#[command]
async fn update_remote_kv(account_id: String, namespace_id: String, key_name: String, value: String, state: State<'_, AppState>) -> Result<(), String> {
    let connections = state.remote_connections.lock().unwrap().clone();
    let connection = connections
        .iter()
        .find(|c| c.account_id == account_id)
        .ok_or_else(|| "Connection not found".to_string())?;

    let client = Client::new();
    let url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
        account_id, namespace_id, key_name
    );

    // Try to parse value as JSON to validate it
    let _: Value = serde_json::from_str(&value)
        .map_err(|_| "Invalid JSON value".to_string())?;

    let response = client
        .put(&url)
        .header(header::AUTHORIZATION, format!("Bearer {}", connection.api_token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(value)
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()));
    }

    Ok(())
}

#[command]
async fn delete_remote_kv(account_id: String, namespace_id: String, keys: Vec<String>, state: State<'_, AppState>) -> Result<(), String> {
    let connections = state.remote_connections.lock().unwrap().clone();
    let connection = connections
        .iter()
        .find(|c| c.account_id == account_id)
        .ok_or_else(|| "Connection not found".to_string())?;

    let client = Client::new();

    if keys.len() == 1 {
        // Delete single key
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
            account_id, namespace_id, keys[0]
        );

        let response = client
            .delete(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", connection.api_token))
            .send()
            .await
            .map_err(|e| format!("API request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()));
        }
    } else {
        // Bulk delete keys
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/bulk/delete",
            account_id, namespace_id
        );

        let response = client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", connection.api_token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&keys)
            .send()
            .await
            .map_err(|e| format!("API request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()));
        }
    }

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            current_path: Mutex::new(None),
            remote_connections: Mutex::new(Vec::new()),
        })
        .invoke_handler(tauri::generate_handler![
            select_folder,
            update_kv,
            delete_kv,
            connect_cloudflare,
            get_remote_namespaces,
            get_remote_keys,
            get_remote_value,
            update_remote_kv,
            delete_remote_kv,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}