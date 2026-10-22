use serde::{Deserialize, Serialize};
use sqlx::{FromRow};

#[allow(unreachable_pub)]
#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct Note {
    pub(crate) id: i64,
    pub(crate) user_id: i64,
    pub(crate) tab_id: i64,
    pub(crate) order_id: u32,
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) bg_color: String,
}

#[allow(unreachable_pub)]
#[derive(Deserialize, Clone)]
pub struct NoteUpdate {
    pub(crate) id: i64,
    pub(crate) title: Option<String>,
    pub(crate) content: Option<String>,
    pub(crate) bg_color: Option<String>,
}