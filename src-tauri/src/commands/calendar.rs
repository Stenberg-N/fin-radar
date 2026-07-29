use tauri::State;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow};
use log::{warn, error, info};
use ammonia;
use time::{Date, macros::format_description};

use crate::{AppState, commands::helpers::{create_timestamp, validate_year_month}};

#[derive(FromRow, Serialize)]
pub struct CalendarEvent {
    pub id: i64,
    pub user_id: i64,
    pub isodate: String,
    pub title: String,
    pub description: Option<String>,
    pub start_time: Option<u32>,
    pub end_time: Option<u32>,
}

#[derive(Deserialize)]
pub struct CalendarEventForm {
    isodate: String,
    title: String,
    description: Option<String>,
    start_time: Option<u32>,
    end_time: Option<u32>,
}

#[tauri::command]
pub async fn add_calendar_event (
    state: State<'_, AppState>,
    form: CalendarEventForm,
) -> Result<CalendarEvent, String> {
    let session = state.session.get_session().map_err(|e| {
        error!("Adding calendar event failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if form.isodate.is_empty() || form.title.is_empty() {
        warn!("ADDING CALENDAR EVENT FAILED ({}): User '{}' tried adding a calendar event with missing date or title", create_timestamp(), session.user.name);
        return Err("An error occurred".to_string());
    }

    Date::parse(form.isodate.as_str(), &format_description!("[year]-[month]-[day]")).map_err(|e| {
        error!("Calendar event date '{}' is invalid: {:#?}", form.isodate, e);
        "An error occurred".to_string()
    })?;

    let cleaner = ammonia::Builder::new();
    let cleaned_date = cleaner.clean(&form.isodate).to_string();
    let cleaned_description = form.description.map(|description| cleaner.clean(&description).to_string());
    let cleaned_title = cleaner.clean(&form.title).to_string();

    let new_event = sqlx::query_as::<_, CalendarEvent>("INSERT INTO calendar_events (user_id, isodate, title, description, start_time, end_time) VALUES (?, ?, ?, ?, ?, ?) RETURNING *")
        .bind(session.user.id)
        .bind(cleaned_date)
        .bind(cleaned_title)
        .bind(cleaned_description)
        .bind(form.start_time)
        .bind(form.end_time)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to create calendar event for user '{}': {:#?}", session.user.name, e);
            "Database error".to_string()
        })?;

    info!("User '{}' added an event to calendar successfully at {}", session.user.name, create_timestamp());

    Ok(new_event)
}

#[tauri::command]
pub async fn get_calendar_events (
    state: State<'_, AppState>,
    year_month: String,
) -> Result<Vec<CalendarEvent>, String> {
    let session = state.session.get_session().map_err(|e| {
        error!("Adding calendar event failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if year_month.is_empty() {
        error!("CALENDAR EVENT FETCH FAILED ({}): User '{}' provided an invalid YYYY-MM date", create_timestamp(), session.user.name);
        return Err("An error occurred".to_string());
    }

    let year_month = validate_year_month(&year_month, &session.user.name).map_err(|e| e )?;

    let calendar_events = sqlx::query_as::<_, CalendarEvent>("SELECT * FROM calendar_events WHERE user_id = ? AND strftime('%Y-%m', isodate) = ?")
        .bind(session.user.id)
        .bind(year_month)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch calendar events for user '{}': {:#?}", session.user.name, e);
            "Database error".to_string()
        })?;

    Ok(calendar_events)
}