use tauri::{async_runtime, Emitter, AppHandle};
use std::sync::Mutex;
use log::{error, warn};
use tokio_util::sync::CancellationToken;

use crate::commands::{helpers::create_timestamp, user::SafeUser};

#[derive(Clone)]
pub struct SessionData {
    pub user: SafeUser,
    created_at: time::OffsetDateTime,
    expires_in: u64,
}

impl SessionData {
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

pub struct Session {
    data: Mutex<Option<SessionData>>,
    expiry_token: Mutex<Option<CancellationToken>>,
    app_handle: AppHandle,
}

impl Session {
    pub fn new (app_handle: AppHandle) -> Self {
        Self {
            data: Mutex::new(None),
            expiry_token: Mutex::new(None),
            app_handle: app_handle,
        }
    }

    pub fn set_session (&self, session_data: SessionData) -> Result<(), String> {
        let expires_in = session_data.expires_in;
        let app_handle = self.app_handle.clone();

        match self.data.lock() {
            Ok(mut guard) => {
                *guard = Some(session_data);
                match self.expiry_token.lock() {
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

    pub fn get_session (&self) -> Result<SessionData, String> {
        let mut guard = self.data.lock().map_err(|_| {
            error!("SESSION POISONED ({})", create_timestamp());
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

    pub fn clear_session (&self) -> Result<(), String> {
        match self.data.lock() {
            Ok(mut guard) => {
                *guard = None;

                match self.expiry_token.lock() {
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
            match self.set_session(session) {
                Ok(_) => Ok(()),
                Err(e) => Err(e)
            }
        } else {
            Err("No session to update".to_string())
        }
    }
}