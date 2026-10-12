use serde::{Deserialize, Serialize};
use sqlx::{FromRow};

#[allow(unreachable_pub)]
#[derive(FromRow, Serialize, Deserialize)]
pub struct Timer {
    pub(crate) id: i64,
    user_id: i64,
    order_id: u32,
    pub(crate) duration: i64,
    pub(crate) title: String,
    pub(crate) message: Option<String>,
}