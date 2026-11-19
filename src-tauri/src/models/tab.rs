use serde::{Deserialize, Serialize};
use sqlx::{FromRow};

#[allow(unreachable_pub)]
#[derive(Serialize, Deserialize, FromRow)]
pub struct Tab {
    id: i64,
    user_id: i64,
    order_id: u32,
    title: String,
    color: String,
}

#[derive(Deserialize)]
pub(crate) struct TabUpdate {
    pub(crate) id: i64,
    pub(crate) title: Option<String>,
    pub(crate) color: Option<String>,
}