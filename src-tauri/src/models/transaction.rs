use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[allow(unreachable_pub)]
#[derive(FromRow, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub(crate) id: i64,
    pub(crate) user_id: i64,
    pub(crate) category: String,
    pub(crate) date: String,
    pub(crate) description: String,
    pub(crate) amount: f64,
    _type: String,
}