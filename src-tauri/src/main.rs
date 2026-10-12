// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![warn(unreachable_pub)]

use sqlx::SqlitePool;
use argon2::Argon2;
use tauri::{Manager, WebviewWindow, async_runtime, Emitter, Listener};
use tauri_plugin_log::{Target, TargetKind, RotationStrategy};
use std::{sync::{Arc, atomic::AtomicBool, atomic::Ordering::SeqCst}};
use log::{info, warn};

use crate::services::session::Session;

#[allow(unreachable_pub)]
mod commands;
mod database;
mod services;
mod models;
mod utils;

struct AppState {
    session: Arc<Session>,
    db: SqlitePool,
    argon2: Argon2<'static>,
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
            let pool = database::db::init_db_pool(app)?;

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
