// src-tauri/src/handlers/remote.rs
use reqwest::{Client, header};
use serde_json::Value;
use tauri::{command, State};

use crate::app_state::AppState;
use crate::models::kv::{KVEntry, KVNamespace, RemoteConnection};
use crate::models::cloudflare::{CloudflareListResponse, CloudflareNamespace, CloudflareKey};

#[command]
pub async fn connect_cloudflare(account_id: String, api_token: String, state: State<'_, AppState>) -> Result<(), String> {
    // Validate connection first
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

    // Save connection to database
    {
        let db = state.db.lock().unwrap();
        if let Err(e) = db.save_remote_connection(&account_id, &api_token) {
            return Err(format!("Failed to save connection: {}", e));
        }
    }

    // Update connection in state
    {
        let mut connections = state.remote_connections.lock().unwrap();
        // Check if connection already exists
        if !connections.iter().any(|c| c.account_id == account_id) {
            connections.push(RemoteConnection {
                account_id,
                api_token,
            });
        }
    }

    Ok(())
}

#[command]
pub async fn get_remote_namespaces(state: State<'_, AppState>) -> Result<Vec<KVNamespace>, String> {
    let connections = state.remote_connections.lock().unwrap().clone();
    if connections.is_empty() {
        return Ok(vec![]);
    }

    let mut all_namespaces = Vec::new();

    for connection in connections {
        let client = Client::new();

        let namespaces_url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces",
            connection.account_id
        );

        let namespaces_response = client
            .get(&namespaces_url)
            .header(header::AUTHORIZATION, format!("Bearer {}", connection.api_token))
            .send()
            .await
            .map_err(|e| format!("API request failed: {}", e))?;

        if !namespaces_response.status().is_success() {
            return Err(format!("API request failed with status: {}", namespaces_response.status()));
        }

        let namespaces_data: CloudflareListResponse<CloudflareNamespace> = namespaces_response
            .json()
            .await
            .map_err(|e| format!("Failed to parse API response: {}", e))?;

        if !namespaces_data.success {
            let error_msg = namespaces_data.errors
                .iter()
                .map(|e| format!("{}: {}", e.code, e.message))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!("API request failed: {}", error_msg));
        }

        // Update timestamp
        {
            let db = state.db.lock().unwrap();
            if let Err(e) = db.update_connection_timestamp(&connection.account_id) {
                eprintln!("Failed to update connection timestamp: {}", e);
            }
        }

        let analytics_url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/analytics/stored?dimensions=namespaceId&metrics=storedKeys",
            connection.account_id
        );

        let analytics_response = client
            .get(&analytics_url)
            .header(header::AUTHORIZATION, format!("Bearer {}", connection.api_token))
            .send()
            .await;

        let mut namespace_counts = std::collections::HashMap::new();

        if let Ok(response) = analytics_response {
            if response.status().is_success() {
                if let Ok(analytics_json) = response.json::<serde_json::Value>().await {
                    if let Some(data) = analytics_json["result"]["data"].as_array() {
                        for item in data {
                            if let (Some(dimensions), Some(metrics)) = (item["dimensions"].as_array(), item["metrics"].as_array()) {
                                if let Some(namespace_id) = dimensions.get(0).and_then(|d| d.as_str()) {
                                    if let Some(metric_row) = metrics.get(0).and_then(|m| m.as_array()) {
                                        if let Some(stored_keys) = metric_row.get(0).and_then(|k| k.as_u64()) {
                                            namespace_counts.insert(namespace_id.to_string(), stored_keys as usize);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if namespace_counts.is_empty() {
            for namespace in &namespaces_data.result {
                let keys_url = format!(
                    "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/keys?limit=1",
                    connection.account_id, namespace.id
                );

                let keys_response = client
                    .get(&keys_url)
                    .header(header::AUTHORIZATION, format!("Bearer {}", connection.api_token))
                    .send()
                    .await;

                if let Ok(response) = keys_response {
                    if response.status().is_success() {
                        if let Ok(keys_json) = response.json::<serde_json::Value>().await {
                            let total_count = keys_json["result_info"]["total_count"]
                                .as_u64()
                                .unwrap_or(0) as usize;

                            namespace_counts.insert(namespace.id.clone(), total_count);
                        }
                    }
                }
            }
        }

        for namespace in namespaces_data.result {
            let count = namespace_counts.get(&namespace.id).copied();

            all_namespaces.push(KVNamespace {
                id: namespace.id.clone(),
                name: namespace.title,
                entries: vec![], // Empty entries, use count field instead
                r#type: "remote".to_string(),
                account_id: Some(connection.account_id.clone()),
                folder_id: None,
                count: count,
            });
        }
    }

    Ok(all_namespaces)
}

// src-tauri/src/handlers/remote.rs (continued)
#[command]
pub async fn get_remote_keys(account_id: String, namespace_id: String, state: State<'_, AppState>) -> Result<Vec<KVEntry>, String> {
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

    // Update timestamp
    {
        let db = state.db.lock().unwrap();
        if let Err(e) = db.update_connection_timestamp(&account_id) {
            eprintln!("Failed to update connection timestamp: {}", e);
        }
    }

    let mut entries = Vec::new();
    for (index, key) in response_data.result.iter().enumerate() {
        entries.push(KVEntry {
            id: format!("{}-{}", namespace_id, index),
            key: key.name.clone(),
            blob_id: format!("remote-{}", index),
            expiration: key.expiration,
            metadata: key.metadata.as_ref().map(|m| m.to_string()),
            value: None,
        });
    }

    Ok(entries)
}

#[command]
pub async fn get_remote_value(account_id: String, namespace_id: String, key_name: String, state: State<'_, AppState>) -> Result<Value, String> {
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

    // Update timestamp
    {
        let db = state.db.lock().unwrap();
        if let Err(e) = db.update_connection_timestamp(&account_id) {
            eprintln!("Failed to update connection timestamp: {}", e);
        }
    }

    let text = response.text().await
        .map_err(|e| format!("Failed to get response text: {}", e))?;

    match serde_json::from_str::<Value>(&text) {
        Ok(value) => Ok(value),
        Err(_) => Ok(Value::String(text)),
    }
}

#[command]
pub async fn update_remote_kv(account_id: String, namespace_id: String, key_name: String, value: String, state: State<'_, AppState>) -> Result<(), String> {
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

    // Update timestamp
    {
        let db = state.db.lock().unwrap();
        if let Err(e) = db.update_connection_timestamp(&account_id) {
            eprintln!("Failed to update connection timestamp: {}", e);
        }
    }

    Ok(())
}

#[command]
pub async fn delete_remote_kv(account_id: String, namespace_id: String, keys: Vec<String>, state: State<'_, AppState>) -> Result<(), String> {
    let connections = state.remote_connections.lock().unwrap().clone();
    let connection = connections
        .iter()
        .find(|c| c.account_id == account_id)
        .ok_or_else(|| "Connection not found".to_string())?;

    let client = Client::new();

    // Update timestamp
    {
        let db = state.db.lock().unwrap();
        if let Err(e) = db.update_connection_timestamp(&account_id) {
            eprintln!("Failed to update connection timestamp: {}", e);
        }
    }

    if keys.len() == 1 {
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

#[command]
pub async fn disconnect_cloudflare(state: State<'_, AppState>) -> Result<(), String> {
    // Remove from database
    {
        let db = state.db.lock().unwrap();
        if let Err(e) = db.remove_all_connections() {
            return Err(format!("Failed to remove connections from database: {}", e));
        }
    }

    // Clear connections from state
    {
        let mut connections = state.remote_connections.lock().unwrap();
        connections.clear();
    }

    Ok(())
}