use serde::{Deserialize, Serialize};
use sqlx::{FromRow};

#[allow(unreachable_pub)]
#[derive(FromRow, Serialize, Deserialize, Clone)]
pub struct CalendarEvent {
    pub(crate) id: i64,
    user_id: i64,
    pub(crate) isodate: String,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) start_time: Option<u32>,
    pub(crate) end_time: Option<u32>,
}

#[derive(Deserialize)]
pub(crate) struct CalendarEventForm {
    pub(crate) isodate: String,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) start_time: Option<u32>,
    pub(crate) end_time: Option<u32>,
    pub(crate) tags: Vec<CalendarTag>,
}

#[allow(unreachable_pub)]
#[derive(Serialize, Deserialize, FromRow, PartialEq)]
pub struct CalendarTag {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) user_id: i64,
}

#[derive(FromRow)]
pub(crate) struct CalendarEventTagRow {
    pub(crate) event_id: i64,
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) user_id: i64,
}

#[allow(unreachable_pub)]
#[derive(Serialize)]
pub struct CalendarEventWithTag {
    pub(crate) event: CalendarEvent,
    pub(crate) tags: Vec<CalendarTag>,
}