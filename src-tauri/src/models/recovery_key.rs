use sqlx::FromRow;

#[derive(FromRow)]
pub(crate) struct RecoveryKey {
    pub(crate) key_hash: String,
    pub(crate) is_used: bool,
}

impl std::fmt::Debug for RecoveryKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Recovery Key")
            .field("key_hash", &"[REDACTED]")
            .field("is_used", &self.is_used)
            .finish()
    }
}