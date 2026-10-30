use serde::Deserialize;
use tauri::{async_runtime, Emitter, AppHandle};
use std::sync::{Arc, Mutex, MutexGuard};
use log::{error, warn, info};
use tokio_util::sync::CancellationToken;

use crate::{utils::time::create_timestamp, models::user::SafeUser, services::cache::Cache};

#[derive(Deserialize)]
pub(crate) enum ExpiresIn {
    Hours1,
    Hours4,
    Hours8,
    Hours12,
}

impl ExpiresIn {
    pub(crate) const fn as_secs(&self) -> u64 {
        match self {
            Self::Hours1 => 3600,
            Self::Hours4 => 4 * 3600,
            Self::Hours8 => 8 * 3600,
            Self::Hours12 => 12 * 3600,
        }
    }
}

#[allow(unreachable_pub)]
pub struct SessionData {
    pub(crate) user: SafeUser,
    created_at: time::OffsetDateTime,
    expires_in: u64,
    expiry_token: Option<CancellationToken>,
}

impl SessionData {
    pub(crate) fn new(user: SafeUser) -> Self {
        Self {
            user,
            created_at: time::OffsetDateTime::now_utc(),
            expires_in: 3600,
            expiry_token: None
        }
    }

    pub(crate) fn snapshot(&self) -> Self {
        Self {
            user: self.user.clone(),
            created_at: self.created_at,
            expires_in: self.expires_in,
            expiry_token: None
        }
    }

    pub(crate) fn is_expired(&self) -> bool {
        let Ok(secs) = i64::try_from(self.expires_in) else { return true };
        match self.created_at.checked_add(time::Duration::seconds(secs)) {
            Some(expiry) => time::OffsetDateTime::now_utc() > expiry,
            None => true,
        }
    }

    /// Checks if the user is in recovery mode and returns an Err if so.
    /// Use to restrict access to something if in recovery mode.
    pub(crate) fn check_user_capabilities(&self, function_name: &str) -> Result<(), String> {
        match self.user.requires_password_reset {
            true => {
                error!("CAPABILITY MISMATCH ({}): User '{}' tried running a function they have no access to: {}", create_timestamp(), self.user.name, function_name);
                Err("An error occurred".to_string())
            },
            false => Ok(())
        }
    }
}

pub(crate) struct Session {
    data: Mutex<Option<SessionData>>,
    app_handle: AppHandle,
    pub(crate) cache: Cache,
}

impl Session {
    fn lock(&self) -> Result<MutexGuard<'_, Option<SessionData>>, String> {
        self.data.lock().map_err(|e| {
            self.data.clear_poison();
            let mut data = e.into_inner();
            if let Some(token) = data.take().and_then(|s| s.expiry_token) {
                token.cancel();
            }
            drop(data);
            error!("SESSION POISONED ({}): Clearing session", create_timestamp());
            if let Err(e) = self.clear_session(true) {
                error!("SESSION CLEAR FAILED ({}): Failed to clear session: {:#?}", create_timestamp(), e);
            }

            "Session poisoned".to_string()
        })
    }

    fn clear_slot(&self, slot: &mut Option<SessionData>) {
        if let Some(token) = slot.take().and_then(|s| s.expiry_token) {
            token.cancel();
        }
        self.cache.clear("Cache was poisoned when clearing session");
    }

    fn expire_session(&self, token: &CancellationToken) -> Result<bool, String> {
        let mut guard = self.lock()?;
        if token.is_cancelled() {
            return Ok(false)
        }
        self.clear_slot(&mut guard);
        Ok(true)
    }

    fn install(self: &Arc<Self>, slot: &mut Option<SessionData>, mut data: SessionData) {
        if let Some(token) = slot.take().and_then(|s| s.expiry_token) { token.cancel(); }
        if let Some(token) = data.expiry_token { token.cancel(); }

        let new_cancel_token = CancellationToken::new();
        let task_token = new_cancel_token.clone();
        data.expiry_token = Some(new_cancel_token);
        
        let expires_in = data.expires_in;
        let app_handle = self.app_handle.clone();
        let name = data.user.name.clone();
        let this = Arc::clone(self);
        *slot = Some(data);

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

                    match this.expire_session(&task_token) {
                        Ok(true) => {
                            app_handle.emit("session-expired", ()).ok();
                            info!("User '{}' was logged out at {} due to inactivity.", name, create_timestamp());
                        },
                        Ok(false) => {},
                        Err(e) => {
                            error!("SESSION CLEAR FAILED ({}): Failed to clear session on expiry: {:#?}", create_timestamp(), e);
                        }
                    }
                } => {}
                _ = task_token.cancelled() => {}
            }
        });
    }

    pub(crate) fn new(app_handle: AppHandle) -> Self {
        Self {
            data: Mutex::new(None),
            app_handle,
            cache: Cache::new(),
        }
    }

    pub(crate) fn set_session(self: &Arc<Self>, session_data: SessionData) -> Result<(), String> {
        let mut guard = self.lock()?;
        self.install(&mut guard, session_data);
        Ok(())
    }

    pub(crate) fn get_session(&self) -> Result<SessionData, String> {
        let expired = {
            let guard = self.lock()?;
            match guard.as_ref() {
                Some(s) if !s.is_expired() => return Ok(s.snapshot()),
                Some(_) => true,
                None => false,
            }
        };

        if expired {
            if let Err(e) = self.clear_session(true) {
                error!("SESSION CLEAR FAILED ({}): Failed to clear session: {:#?}", create_timestamp(), e);
            }
            return Err("Session expired".to_string());
        }
        warn!("NO SESSION ({})", create_timestamp());
        Err("No session".to_string())
    }

    pub(crate) fn clear_session(&self, logout_user_in_frontend: bool) -> Result<(), String> {
        {
            let mut guard = self.lock()?;
            self.clear_slot(&mut guard);
        }

        if logout_user_in_frontend {
            self.app_handle.emit("session-cleared", ()).ok();
        }

        Ok(())
    }

    pub(crate) fn update_session(self: &Arc<Self>, expires_in: ExpiresIn) -> Result<(), String> {
        let expires_in = expires_in.as_secs();

        let mut guard = self.lock()?;
        let Some(mut session) = guard.take() else {
            warn!("UPDATE SESSION FAILED ({}): No active session", create_timestamp());
            return Err("No session to update".to_string());
        };

        session.expires_in = expires_in;
        session.created_at = time::OffsetDateTime::now_utc();
        self.install(&mut guard, session);
        Ok(())
    }

    pub(crate) fn update_user_in_session(self: &Arc<Self>, user: SafeUser) -> Result<(), String> {
        let mut guard = self.lock()?;
        match guard.as_mut() {
            Some(session) => {
                session.user = user.to_owned();
                Ok(())
            },
            None => {
                warn!("UPDATE USER IN SESSION FAILED ({}): No active session", create_timestamp());
                Err("No session to update".to_string())
            }
        }
    }
}