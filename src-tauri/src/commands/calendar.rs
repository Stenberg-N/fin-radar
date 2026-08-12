use tauri::State;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow};
use log::{warn, error, info};
use ammonia;
use time::{Date, macros::format_description};

use crate::{AppState, commands::helpers::{create_timestamp, validate_year_month}, structs::session::SessionData, structs::cache::CacheData};

#[derive(FromRow, Serialize, Deserialize, Clone)]
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

async fn fetch_and_cache_calendar_events(
    state: &State<'_, AppState>,
    year_month: &str,
    key: &str,
    user_id: i64,
    username: &str,
) -> Result<Vec<CalendarEvent>, String> {
    let calendar_events = sqlx::query_as::<_, CalendarEvent>("SELECT * FROM calendar_events WHERE user_id = ? AND strftime('%Y-%m', isodate) = ?")
        .bind(user_id)
        .bind(year_month)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch calendar events for user '{}': {:#?}", username, e);
            "Database error".to_string()
        })?;

    if let Err(e) = state.cache.cache_results(key.to_string(), CacheData::from(calendar_events.clone())) {
        error!("CACHE POISONED ({}): Failed to set calendar events to cache for user '{}': {:#?}", create_timestamp(), username, e);
    }

    Ok(calendar_events)
}

#[tauri::command]
pub async fn add_calendar_event(
    state: State<'_, AppState>,
    form: CalendarEventForm,
) -> Result<CalendarEvent, String> {
    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Adding calendar event failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if form.isodate.trim().is_empty() || form.title.trim().is_empty() {
        warn!("ADDING CALENDAR EVENT FAILED ({}): User '{}' tried adding a calendar event with missing date or title", create_timestamp(), session.user.name);
        return Err("An error occurred".to_string());
    }

    Date::parse(form.isodate.as_str(), &format_description!("[year]-[month]-[day]")).map_err(|e| {
        error!("ADDING CALENDAR EVENT FAILED ({}): User '{}' provided an invalid date '{}' is invalid: {:#?}", create_timestamp(), session.user.name, form.isodate, e);
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
pub async fn get_calendar_events(
    state: State<'_, AppState>,
    year_month: String,
) -> Result<Vec<CalendarEvent>, String> {
    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Failed to fetch calendar events at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if year_month.is_empty() {
        error!("CALENDAR EVENT FETCH FAILED ({}): User '{}' provided an invalid YYYY-MM date", create_timestamp(), session.user.name);
        return Err("An error occurred".to_string());
    }

    let year_month = validate_year_month(&year_month, &session.user.name).map_err(|e| e )?;
    let key = format!("{}-{}-calevents", session.user.id, year_month);

    let calendar_events = match state.cache.contains(&key) {
        Ok(true) => {
            match state.cache.get_calendar_events(&key) {
                Ok(Some(events)) => events.values().cloned().collect(),
                Ok(None) => fetch_and_cache_calendar_events(&state, &year_month, &key, session.user.id, &session.user.name).await?,
                Err(e) => {
                    error!("CACHE POISONED ({}): Failed to get calendar events from cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
                    fetch_and_cache_calendar_events(&state, &year_month, &key, session.user.id, &session.user.name).await?
                }
            }
        },
        Ok(false) => fetch_and_cache_calendar_events(&state, &year_month, &key, session.user.id, &session.user.name).await?,
        Err(e) => {
            error!("CACHE POISONED ({}): Failed to check cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
            fetch_and_cache_calendar_events(&state, &year_month, &key, session.user.id, &session.user.name).await?
        }
    };

    Ok(calendar_events)
}

#[tauri::command]
pub async fn delete_calendar_event(
    state: State<'_, AppState>,
    event: CalendarEvent,
) -> Result<CalendarEvent, String> {
    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Failed to delete calendar event at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let deleted_event = sqlx::query_as::<_, CalendarEvent>("DELETE FROM calendar_events WHERE user_id = ? AND id = ? RETURNING *")
        .bind(session.user.id)
        .bind(event.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to delete calendar event for user '{}': {:#?}", session.user.name, e);
            "Database error".to_string()
        })?;

    info!("CALENDAR EVENT DELETION SUCCESS ({}): User '{}' deleted a calendar event successfully", create_timestamp(), session.user.name);

    Ok(deleted_event)
}

#[tauri::command]
pub async fn update_calendar_event(
    state: State<'_, AppState>,
    form: CalendarEventForm,
    event: CalendarEvent,
) -> Result<CalendarEvent, String> {
    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Failed to update calendar event at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if event.isodate == form.isodate && event.title == form.title && event.description == form.description && event.start_time == form.start_time && event.end_time == form.end_time {
        error!("CALENDAR UPDATE FAILED ({}): User '{}' made no changes to the event", create_timestamp(), session.user.name);
        return Err("An error occurred".to_string());
    }

    if form.isodate.trim().is_empty() || form.title.trim().is_empty() {
        error!("CALENDAR UPDATE FAILED ({}): User '{}' tried updating an event with an empty date or title", create_timestamp(), session.user.name);
        return Err("An error occurred".to_string());
    }

    Date::parse(form.isodate.as_str(), &format_description!("[year]-[month]-[day]")).map_err(|e| {
        error!("CALENDAR UPDATE FAILED ({}): User '{}' provided an invalid date: {:#?}", create_timestamp(), session.user.name, e);
        "An error occurred".to_string()
    })?;

    let cleaner = ammonia::Builder::new();
    let cleaned_title = cleaner.clean(&form.title).to_string();
    let cleaned_description = form.description.map(|description| cleaner.clean(&description).to_string());
    let cleaned_date = cleaner.clean(&form.isodate).to_string();
    let start_time = form.start_time.map(|value| value.max(0));
    let end_time = form.end_time.map(|value| value.max(0));

    let updated_event = sqlx::query_as::<_, CalendarEvent>(
        "UPDATE calendar_events SET isodate = ?, title = ?, description = ?, start_time = ?, end_time = ? WHERE id = ? AND user_id = ? RETURNING *"
    )
        .bind(cleaned_date)
        .bind(cleaned_title)
        .bind(cleaned_description)
        .bind(start_time)
        .bind(end_time)
        .bind(event.id)
        .bind(session.user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to update calendar event: {:#?}", e);
            "Database error".to_string()
        })?;

    info!("CALENDAR EVENT UPDATED SUCCESSFULLY ({}): User '{}' updated a calendar event", create_timestamp(), session.user.name);

    Ok(updated_event)
}