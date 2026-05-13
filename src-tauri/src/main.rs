// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, async_runtime, Emitter};
use tauri_plugin_log::{Target, TargetKind, RotationStrategy};
use std::fs;
use std::path::PathBuf;
use std::io::ErrorKind;
use std::sync::{Arc, Mutex};
use log::{info, error, debug, warn};

mod commands;
mod db;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::new()
            .targets([
                Target::new(TargetKind::LogDir {
                    file_name: Some("logs".to_string()),
                }),
                Target::new(TargetKind::Stdout),
            ])
            .max_file_size(500_000)
            .rotation_strategy(RotationStrategy::KeepAll)
            .level(log::LevelFilter::Info)
            .build())
        .setup(|app| {
            let base_dir: PathBuf = app.path().app_local_data_dir()?.into();
            let db_dir = base_dir.join("database");

            info!("Attempting to create database directory");
            if let Err(e) = fs::create_dir_all(&db_dir) {
                if e.kind() != ErrorKind::AlreadyExists {
                    error!("Failed to create database directory: {:#?}", e);
                }
            } else {
                debug!("Database directory already exists");
            }
            info!("Database directory ready");

            let db_file = db_dir.join("data.db");
            let db_path = format!("sqlite://{}?mode=rwc", db_file.to_string_lossy());

            let pool = async_runtime::block_on(db::init_db(&db_path)).map_err(|e| {
                error!("Failed to initialize database: {:#?}", e);
                format!("Failed to initialize database: {}", e)
            })?;

            let pool_cleanup = pool.clone();

            async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(900));

                loop {
                    interval.tick().await;
                    let _ = sqlx::query("PRAGMA optimize;")
                        .execute(&pool_cleanup)
                        .await
                        .map_err(|e| {
                            warn!("Ran into an error during periodic database optimization: {:#?}", e);
                            "Database error".to_string()
                        });
                }
            });

            let is_closing = Arc::new(Mutex::new(false));

            if let Some(window) = app.get_webview_window("main") {
                let win = window.clone();
                let is_closing = is_closing.clone();
                
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();

                        let mut closing = is_closing.lock().unwrap();
                        if *closing {
                            return;
                        }
                        *closing = true;
                        drop(closing);

                        let _ = win.emit("app-closing", ());

                        let _ = win.close();
                        win.app_handle().exit(0);
                    }
                });
            }

            app.manage(pool);

            info!("App setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::user::create_user,
            commands::user::login_user,
            commands::user::delete_user,
            commands::user::change_password,
            commands::user::recover_password,
            commands::user::cancel_password_recovery,
            commands::others::backup_database,
            commands::transactions::add_transaction,
            commands::transactions::get_transactions,
            commands::transactions::delete_transaction,
            commands::transactions::update_transaction,
            commands::transactions::get_year_transactions,
            commands::notes::create_note,
            commands::notes::get_notes,
            commands::notes::create_tab,
            commands::notes::get_tabs,
            commands::notes::update_tab,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
