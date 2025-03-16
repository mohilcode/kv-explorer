use reqwest::{Client, header};
use serde_json::Value;
use tauri::{command, State};

use crate::app_state::AppState;
use crate::models::kv::{KVEntry, KVNamespace, RemoteConnection};
use crate::models::cloudflare::{CloudflareListResponse, CloudflareNamespace, CloudflareKey};

#[command]
pub async fn connect_cloudflare(account_id: String, api_token: String, state: State<'_, AppState>) -> Result<(), String> {
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

    let mut connections = state.remote_connections.lock().unwrap();
    connections.push(RemoteConnection {
        account_id,
        api_token,
    });

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

    let mut entries = Vec::new();
    for (index, key) in response_data.result.iter().enumerate() {
        entries.push(KVEntry {
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