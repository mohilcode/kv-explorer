#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app_state;
mod models;
mod handlers;

use app_state::AppState;
use handlers::local::{select_folder, update_kv, delete_kv};
use handlers::remote::{
    connect_cloudflare, get_remote_namespaces, get_remote_keys,
    get_remote_value, update_remote_kv, delete_remote_kv
};

fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
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