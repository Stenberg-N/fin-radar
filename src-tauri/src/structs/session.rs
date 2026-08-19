use tauri::{async_runtime, Emitter, AppHandle};
use std::sync::{Arc, Mutex};
use log::{error, warn, info};
use tokio_util::sync::CancellationToken;

use crate::{commands::{helpers::create_timestamp, user::SafeUser}, structs::cache::Cache};

#[derive(Clone)]
pub struct SessionData {
    pub user: SafeUser,
    created_at: time::OffsetDateTime,
    expires_in: u64,
}

impl SessionData {
    pub fn new(user: SafeUser) -> Self {
        Self {
            user,
            created_at: time::OffsetDateTime::now_utc(),
            expires_in: 3600,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = time::OffsetDateTime::now_utc();
        let secs = i64::try_from(self.expires_in).unwrap_or(3600);
        let expiry_time = self.created_at + time::Duration::seconds(secs);

        now > expiry_time
    }
}

pub struct Session {
    data: Mutex<Option<SessionData>>,
    expiry_token: Mutex<Option<CancellationToken>>,
    app_handle: AppHandle,
    pub cache: Cache,
}

impl Session {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            data: Mutex::new(None),
            expiry_token: Mutex::new(None),
            app_handle: app_handle,
            cache: Cache::new(),
        }
    }

    pub fn set_session(self: &Arc<Self>, session_data: SessionData) -> Result<(), String> {
        let expires_in = session_data.expires_in;
        let app_handle = self.app_handle.clone();

        match self.data.lock() {
            Ok(mut guard) => {
                *guard = Some(session_data.clone());
                match self.expiry_token.lock() {
                    Ok(mut token_guard) => {
                        if let Some(token) = token_guard.take() {
                            token.cancel();
                        }

                        let new_cancel_token = CancellationToken::new();
                        let cancel_token_cloned = new_cancel_token.clone();
                        *token_guard = Some(new_cancel_token);

                        let this = Arc::clone(self);

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

                                    if let Err(e) = this.clear_session() {
                                        error!("SESSION CLEAR FAILED ({}): Failed to clear session on expiry: {:#?}", create_timestamp(), e);
                                    }
                                    app_handle.emit("session-expired", ()).ok();
                                    info!("User '{}' was logged out at {} due to inactivity.", session_data.user.name, create_timestamp());
                                } => {
                                    if let Err(e) = this.cache.clear() {
                                        warn!("CACHE POISONED ({}): Cache was poisoned when clearing session: {:#?}", create_timestamp(), e);
                                    }
                                }
                                _ = cancel_token_cloned.cancelled() => {}
                            }
                        });
                        Ok(())
                    }
                    Err(e) => {
                        error!("SESSION EXPIRY TOKEN POISONED ({}): Failed to set new cancellation token. Clearing session and expiry token.", create_timestamp());
                        *guard = None;
                        self.expiry_token.clear_poison();
                        *e.into_inner() = None;
                        Err("Session expiry token poisoned".to_string())
                    }
                }
            }
            Err(e) => {
                error!("SESSION CLEAR FAILED ({}): Session poisoned. Clearing session.", create_timestamp());
                self.data.clear_poison();
                *e.into_inner() = None;
                Err("Session poisoned".to_string())
            }
        }
    }

    pub fn get_session(&self) -> Result<SessionData, String> {
        let mut guard = self.data.lock().map_err(|e| {
            error!("SESSION POISONED ({}): Failed to get session. Clearing session.", create_timestamp());
            self.data.clear_poison();
            *e.into_inner() = None;
            "Session poisoned".to_string()
        })?;

        match guard.as_ref() {
            Some(session_data) if !session_data.is_expired() => Ok(session_data.clone()),
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

    pub fn clear_session(&self) -> Result<(), String> {
        match self.data.lock() {
            Ok(mut guard) => {
                let was_active = guard.is_some();
                *guard = None;

                if !was_active {
                    return Ok(())
                }

                match self.expiry_token.lock() {
                    Ok(mut token_guard) => {
                        match token_guard.take() {
                            Some(token) => token.cancel(),
                            None => warn!("NO SESSION EXPIRY TOKEN SET ({})", create_timestamp()),
                        }
                        Ok(())
                        
                    }
                    Err(e) => {
                        warn!("SESSION EXPIRY TOKEN POISONED ({}): Session expiry token poisoned while clearing session! Clearing expiry token!", create_timestamp());
                        *e.into_inner() = None;
                        self.expiry_token.clear_poison();
                        Ok(())
                    }
                }
            }
            Err(e) => {
                error!("SESSION CLEAR FAILED ({}): Session poisoned. Clearing session.", create_timestamp());
                self.data.clear_poison();
                *e.into_inner() = None;
                Err("Session poisoned".to_string())
            }
        }
    }

    pub fn update_session(self: &Arc<Self>) -> Result<(), String> {
        let updated_session = match self.data.lock() {
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
            self.set_session(session)
        } else {
            Err("No session to update".to_string())
        }
    }
}