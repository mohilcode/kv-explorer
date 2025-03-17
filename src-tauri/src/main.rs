#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app_state;
mod models;
mod handlers;
mod persistence;

use app_state::AppState;
use tauri::Manager;
use handlers::local::{add_folder, remove_folder, load_folder, get_folders, update_kv, delete_kv};
use handlers::remote::{
    connect_cloudflare, get_remote_namespaces, get_remote_keys,
    get_remote_value, update_remote_kv, delete_remote_kv, disconnect_cloudflare
};

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle();
            let app_data_dir = app_handle.path_resolver().app_data_dir().unwrap_or_default();

            std::fs::create_dir_all(&app_data_dir).ok();

            #[cfg(debug_assertions)]
            {
                let db_path = app_data_dir.join("kv_explorer.db");
                if db_path.exists() {
                    println!("Resetting database for development mode");
                    std::fs::remove_file(db_path).ok();
                }
            }

            app.manage(AppState::new(app_data_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_folders,
            add_folder,
            remove_folder,
            load_folder,
            update_kv,
            delete_kv,
            connect_cloudflare,
            disconnect_cloudflare,
            get_remote_namespaces,
            get_remote_keys,
            get_remote_value,
            update_remote_kv,
            delete_remote_kv,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}