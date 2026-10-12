use serde::Serialize;
use sqlx::{FromRow};

#[derive(FromRow, Clone)]
pub(crate) struct User {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) password: String,
    pub(crate) requires_password_reset: bool,
    created_at: String,
    pub(crate) last_password_change: String,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("password_hash", &"[REDACTED]")
            .field("requires_password_reset", &self.requires_password_reset)
            .field("created_at", &self.created_at)
            .field("last_password_change", &self.last_password_change)
            .finish()
    }
}

#[allow(unreachable_pub)]
#[derive(Serialize, Clone)]
pub struct SafeUser {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) requires_password_reset: bool,
    created_at: String,
    last_password_change: String,
}

impl From<User> for SafeUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user.name,
            requires_password_reset: user.requires_password_reset,
            created_at: user.created_at,
            last_password_change: user.last_password_change,
        }
    }
}