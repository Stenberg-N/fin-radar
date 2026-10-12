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

#[allow(unreachable_pub)]
#[derive(Serialize, FromRow)]
pub struct TabIdTitle {
    id: i64,
    title: String,
}