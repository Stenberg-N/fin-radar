// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use sqlx::SqlitePool;
use argon2::Argon2;
use tauri::{App, Manager, WebviewWindow, async_runtime, Emitter, Listener};
use tauri_plugin_log::{Target, TargetKind, RotationStrategy};
use std::{fs, path::PathBuf, sync::{Arc, atomic::AtomicBool, atomic::Ordering::SeqCst}};
use log::{info, error, warn};

use crate::structs::session::Session;

mod commands;
mod db;
mod structs;

struct AppState {
    session: Arc<Session>,
    db: SqlitePool,
    argon2: Argon2<'static>,
}

fn init_db_pool(app: &App) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let base_dir: PathBuf = app.path().app_local_data_dir()?.into();
    let db_dir = base_dir.join("database");

    info!("Attempting to create database directory");
    fs::create_dir_all(&db_dir).map_err(|e| {
        error!("Failed to create database directory: {:#?}", e);
        e
    })?;
    info!("Database directory ready");

    let db_file = db_dir.join("data.db");
    let db_str = db_file.to_str().ok_or("Database path invalid")?;
    let db_path = format!("sqlite://{}?mode=rwc", db_str);

    let pool = async_runtime::block_on(db::init_db(&db_path)).map_err(|e| {
        error!("Failed to initialize database: {:#?}", e);
        e
    })?;

    Ok(pool)
}

fn setup_window_close_handler(window: &WebviewWindow, pool: &SqlitePool) {
    let is_closing = AtomicBool::new(false);
    let win = window.clone();
    let pool = pool.clone();

    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            if is_closing.swap(true, SeqCst) {
                return;
            }

            let pool = pool.clone();
            let win = win.clone();

            async_runtime::spawn(async move {
                if let Err(e) = sqlx::query("PRAGMA optimize; PRAGMA wal_checkpoint(TRUNCATE);")
                    .execute(&pool)
                    .await
                {
                    warn!("Error during database optimization: {:#?}", e);
                }
                pool.close().await;

                win.emit("app-closing", ()).ok();
            });
        }
    });

    let win = window.clone();
    window.listen("app-ready-to-close", move |_| {
        let _ = win.close();
        win.app_handle().exit(0);
    });
}

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
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            let pool = init_db_pool(app)?;

            if let Some(window) = app.get_webview_window("main") {
                setup_window_close_handler(&window, &pool);
            }

            let state = AppState {
                session: Arc::new(Session::new(app.app_handle().clone())),
                db: pool,
                argon2: Argon2::default(),
            };

            app.manage(state);
            info!("App setup complete");
            Ok(())
        })
        .invoke_handler(commands::all_handlers())
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
