// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use sqlx::SqlitePool;
use argon2::Argon2;
use tauri::{App, Manager, WebviewWindow, async_runtime, Emitter, Listener};
use tauri_plugin_log::{Target, TargetKind, RotationStrategy};
use std::{fs, path::PathBuf, sync::{Arc, Mutex}};
use log::{info, error, warn};

use crate::structs::{cache::Cache, session::Session};

mod commands;
mod db;
mod structs;

pub struct AppState {
    session: Session,
    db: SqlitePool,
    argon2: Argon2<'static>,
    cache: Cache,
}

fn init_db_pool (app: &App) -> Result<SqlitePool, Box<dyn std::error::Error>> {
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

fn spawn_db_optimizer (pool: SqlitePool) {
    async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(900));

        loop {
            interval.tick().await;
            if let Err(e) = sqlx::query("PRAGMA optimize;").execute(&pool).await {
                warn!("Ran into an error during periodic database optimization: {:#?}", e);
            }
        }
    });
}

fn setup_window_close_handler (window: &WebviewWindow) {
    let is_closing = Arc::new(Mutex::new(false));

    let win = window.clone();
    let is_closing_clone = is_closing.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();

            let mut closing = is_closing_clone.lock().unwrap();
            if *closing {
                return;
            }
            *closing = true;
            drop(closing);

            let _ = win.emit("app-closing", ());
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
            spawn_db_optimizer(pool.clone());

            if let Some(window) = app.get_webview_window("main") {
                setup_window_close_handler(&window);
            }

            let state = AppState {
                session: Session::new(app.app_handle().clone()),
                db: pool,
                argon2: Argon2::default(),
                cache: Cache::new(),
            };

            app.manage(state);
            info!("App setup complete");
            Ok(())
        })
        .invoke_handler(commands::all_handlers())
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
