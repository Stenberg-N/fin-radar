// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use sqlx::SqlitePool;
use argon2::Argon2;
use tauri::{App, Manager, WebviewWindow, async_runtime, Emitter, Listener};
use tauri_plugin_log::{Target, TargetKind, RotationStrategy};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use log::{info, error, warn};
use tokio_util::sync::CancellationToken;

use crate::commands::helpers::create_timestamp;
use crate::commands::user::SafeUser;

mod commands;
mod db;

#[derive(Clone)]
pub struct Session {
    user: SafeUser,
    created_at: time::OffsetDateTime,
    expires_in: u64,
}

impl Session {
    pub fn new (user: SafeUser) -> Self {
        Self {
            user,
            created_at: time::OffsetDateTime::now_utc(),
            expires_in: 3600,
        }
    }

    pub fn is_expired (&self) -> bool {
        let now = time::OffsetDateTime::now_utc();
        let secs = i64::try_from(self.expires_in).unwrap_or(3600);
        let expiry_time = self.created_at + time::Duration::seconds(secs);

        now > expiry_time
    }
}

pub struct AppState {
    session: Mutex<Option<Session>>,
    session_expiry_token: Mutex<Option<CancellationToken>>,
    db: SqlitePool,
    argon2: Argon2<'static>,
    app_handle: tauri::AppHandle,
}

impl AppState {
    pub fn set_session (&self, session: Session) -> Result<(), String> {
        let expires_in = session.expires_in;
        let app_handle = self.app_handle.clone();

        match self.session.lock() {
            Ok(mut guard) => {
                *guard = Some(session);
                match self.session_expiry_token.lock() {
                    Ok(mut token_guard) => {
                        if let Some(token) = token_guard.take() {
                            token.cancel();
                        }

                        let new_token = CancellationToken::new();
                        let cloned = new_token.clone();
                        *token_guard = Some(new_token);

                        async_runtime::spawn(async move {
                            tokio::select! {
                                _ = async {
                                    if expires_in > 300 {
                                        tokio::time::sleep(tokio::time::Duration::from_secs(expires_in - 300)).await;
                                        app_handle.emit("session-about-to-expire", ()).ok();
                                        tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
                                    } else {
                                        tokio::time::sleep(tokio::time::Duration::from_secs(expires_in)).await;
                                    }
                                    app_handle.emit("session-expired", ()).ok();
                                } => {}
                                _ = cloned.cancelled() => {}
                            }
                        });
                        Ok(())
                    }
                    Err(e) => {
                        error!("SESSION EXPIRY TOKEN POISONED ({}): Failed to set new cancellation token. Clearing session", create_timestamp());
                        *guard = None;
                        Err(e.to_string())
                    }
                }
            }
            Err(e) => Err(e.to_string())
        }
    }

    pub fn get_session (&self) -> Result<Session, String> {
        let mut guard = self.session.lock().map_err(|_| {
            error!("SESSION POISONED ({})", create_timestamp());
            "Session poisoned".to_string()
        })?;

        match guard.as_ref() {
            Some(session) if !session.is_expired() => Ok(session.clone()),
            Some(_) => {
                *guard = None;
                Err("Session expired".to_string())
            }
            None => {
                warn!("NO SESSION ({})", create_timestamp());
                Err("No session".to_string())
            }
        }
    }

    pub fn clear_session (&self) -> Result<(), String> {
        match self.session.lock() {
            Ok(mut guard) => {
                *guard = None;

                match self.session_expiry_token.lock() {
                    Ok(mut token_guard) => {
                        match token_guard.take() {
                            Some(token) => token.cancel(),
                            None => warn!("NO SESSION EXPIRY TOKEN SET ({})", create_timestamp()),
                        }
                        Ok(())
                        
                    }
                    Err(_) => {
                        warn!("SESSION EXPIRY TOKEN POISONED ({}): Session cleared but token could not be cancelled", create_timestamp());
                        Ok(())
                    }
                }
            }
            Err(e) => Err(e.to_string())
        }
    }

    pub fn update_session (&self) -> Result<(), String> {
        let updated_session = match self.session.lock() {
            Ok(mut guard) => {
                match guard.as_mut() {
                    Some(session) => {
                        session.expires_in = 3600;
                        session.created_at = time::OffsetDateTime::now_utc();
                        Some(session.clone())
                    }
                    None => {
                        warn!("UPDATE SESSION FAILED ({}): No active session", create_timestamp());
                        None
                    }
                }
            }
            Err(_) => {
                error!("UPADTE SESSION FAILED ({}): Session poisoned", create_timestamp());
                None
            }
        };

        if let Some(session) = updated_session {
            match self.set_session(session) {
                Ok(_) => Ok(()),
                Err(e) => Err(e)
            }
        } else {
            Err("No session to update".to_string())
        }
    }
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
                session: Mutex::new(None),
                session_expiry_token: Mutex::new(None),
                db: pool,
                argon2: Argon2::default(),
                app_handle: app.app_handle().clone(),
            };

            app.manage(state);
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
            commands::user::logout_user,
            commands::user::update_user_session,
            commands::others::backup_database,
            commands::others::reorder_array,
            commands::transactions::add_transaction,
            commands::transactions::get_transactions,
            commands::transactions::delete_transaction,
            commands::transactions::update_transaction,
            commands::transactions::get_year_transactions,
            commands::notes::create_note,
            commands::notes::get_notes,
            commands::notes::update_note,
            commands::notes::delete_note,
            commands::notes::create_tab,
            commands::notes::get_tabs,
            commands::notes::update_tab,
            commands::notes::delete_tab,
            commands::notes::update_tab_color,
            commands::timers::create_timer,
            commands::timers::get_timers,
            commands::timers::update_timer,
            commands::timers::delete_timer,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
