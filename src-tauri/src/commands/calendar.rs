use tauri::State;
use sqlx::query_as;
use log::{warn, error, info};
use ammonia;
use time::{Date, macros::format_description};
use std::collections::HashMap;

use crate::{
    AppState,
    models::calendar::{
        CalendarEvent,
        CalendarEventForm,
        CalendarEventPayloadRef,
        CalendarEventPayload,
        CalendarEventTagRow,
        CalendarEventWithTag,
        CalendarTag,
        StoredCalendarEvent
    },
    services::{
        cache::{CacheData, UpdateTask},
        crypto::{KEY_LEN, decrypt, encrypt},
        session::SessionData,
    },
    utils::time::{create_timestamp, validate_year_month},
};

async fn fetch_and_cache_calendar_events(
    state: &AppState,
    year_month: &str,
    key: &str,
    user_id: i64,
    username: &str,
    dek: &[u8; KEY_LEN],
) -> Result<Vec<CalendarEvent>, String> {
    let stored_events = query_as::<_, StoredCalendarEvent>("SELECT * FROM calendar_events WHERE user_id = ? AND strftime('%Y-%m', isodate) = ?")
        .bind(user_id)
        .bind(year_month)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch calendar events for user '{}': {:#?}", username, e);
            "Database error".to_string()
        })?;

    let mut calendar_events = Vec::<CalendarEvent>::new();
    for se in stored_events {
        let blob = decrypt(dek, &se.payload, "calendar_events.payload", user_id, se.id)?;
        let payload: CalendarEventPayload = serde_json::from_slice(&blob).map_err(|e| {
            error!("Failed to parse event payload during fetch: {:#?}", e);
            "An error occurred".to_string()
        })?;

        let event = CalendarEvent {
            id: se.id,
            user_id: se.user_id,
            isodate: se.isodate,
            title: payload.title,
            description: payload.description,
            start_time: se.start_time,
            end_time: se.end_time,
        };

        calendar_events.push(event);
    }

    state.session.cache.cache_results(key.to_string(), CacheData::from(calendar_events.clone()), "Failed to set calendar events to cache during fetch");

    Ok(calendar_events)
}

#[tauri::command]
pub async fn add_calendar_event(
    state: State<'_, AppState>,
    form: CalendarEventForm,
) -> Result<CalendarEvent, String> {
    let state: &AppState = &state;

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

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to start transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    let new_event_id = sqlx::query_scalar::<_, i64>("INSERT INTO calendar_events (user_id, isodate, payload, start_time, end_time) VALUES (?, ?, ?, ?, ?) RETURNING id")
        .bind(session.user.id)
        .bind(&cleaned_date)
        .bind(Vec::<u8>::new())
        .bind(form.start_time)
        .bind(form.end_time)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to create calendar event for user '{}': {:#?}", session.user.name, e);
            "Database error".to_string()
        })?;

    let new_event = CalendarEvent {
        id: new_event_id,
        user_id: session.user.id,
        isodate: cleaned_date,
        title: cleaned_title,
        description: cleaned_description,
        start_time: form.start_time,
        end_time: form.end_time,
    };

    let dek = session.get_dek(&state.session)?;

    let payload = serde_json::to_vec(&CalendarEventPayloadRef { 
        title: &new_event.title,
        description: &new_event.description,
    }).map_err(|e| {
        error!("Failed to parse calendar event's title and description for payload: {:#?}", e);
        "An error occurred".to_string()
    })?;
    let blob = encrypt(&dek, &payload, "calendar_events.payload", session.user.id, new_event.id)?;

    sqlx::query("UPDATE calendar_events SET payload = ? WHERE id = ? AND user_id = ?")
        .bind(blob)
        .bind(new_event.id)
        .bind(session.user.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to update placeholder payload for calendar event: {:#?}", e);
            "Database error".to_string()
        })?;

    if !form.tags.is_empty() {
        let tag_ids: Vec<i64> = form.tags.iter().map(|t| t.id).collect();
        let values_part: Vec<&str> = (0..tag_ids.len()).map(|_| "(?, ?)").collect();
        let insert_query = format!("INSERT OR IGNORE INTO calendar_events_tags (event_id, tag_id) VALUES {}", values_part.join(", "));
        let mut insert_query = sqlx::query::<sqlx::Sqlite>(&insert_query);

        for id in &tag_ids {
            insert_query = insert_query.bind(new_event.id).bind(id);
        }

        insert_query.execute(&mut *tx).await.map_err(|e| {
            error!("Failed to add tags to event: {:#?}", e);
            "Database error".to_string()
        })?;
    }

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    info!("User '{}' added an event to calendar successfully at {}", session.user.name, create_timestamp());

    if let Some(value) = form.isodate.get(..7) {
        let year_month = value;
        let key = format!("{}-{}-calevents", session.user.id, year_month);

        match state.session.cache.contains(&key) {
            Ok(true) => {
                state.session.cache.update_cache(&key, &HashMap::from([(new_event.id, new_event.clone())]), &UpdateTask::Update, "Failed to add calendar event to cache during event creation");
            },
            Ok(false) => {},
            Err(e) => {
                error!("CACHE POISONED ({}): Failed to check cache for user '{}'. Refetching data: {:#?}", create_timestamp(), session.user.name, e);
                fetch_and_cache_calendar_events(state, year_month, &key, session.user.id, &session.user.name, &dek).await?;
            }
        }
    } else {
        error!("CACHING FAILED ({}): Failed to add calendar event to cache for user '{}': Calendar event date invalid", create_timestamp(), session.user.name);
    }

    Ok(new_event)
}

#[tauri::command]
pub async fn get_calendar_events(
    state: State<'_, AppState>,
    year_month: String,
) -> Result<Vec<CalendarEventWithTag>, String> {
    let state: &AppState = &state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Failed to fetch calendar events at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if year_month.is_empty() {
        error!("CALENDAR EVENT FETCH FAILED ({}): User '{}' provided an invalid YYYY-MM date", create_timestamp(), session.user.name);
        return Err("An error occurred".to_string());
    }

    let year_month = validate_year_month(&year_month, &session.user.name)?;
    let key = format!("{}-{}-calevents", session.user.id, year_month);
    let dek = session.get_dek(&state.session)?;

    let calendar_events = match state.session.cache.contains(&key) {
        Ok(true) => {
            match state.session.cache.get_calendar_events(&key) {
                Ok(Some(events)) => events.values().cloned().collect(),
                Ok(None) => fetch_and_cache_calendar_events(state, &year_month, &key, session.user.id, &session.user.name, &dek).await?,
                Err(e) => {
                    error!("CACHE POISONED ({}): Failed to get calendar events from cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
                    fetch_and_cache_calendar_events(state, &year_month, &key, session.user.id, &session.user.name, &dek).await?
                }
            }
        },
        Ok(false) => fetch_and_cache_calendar_events(state, &year_month, &key, session.user.id, &session.user.name, &dek).await?,
        Err(e) => {
            error!("CACHE POISONED ({}): Failed to check cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
            fetch_and_cache_calendar_events(state, &year_month, &key, session.user.id, &session.user.name, &dek).await?
        }
    };

    let event_ids: Vec<i64> = calendar_events.iter().map(|e| e.id).collect();
    let placeholders: Vec<&str> = (0..event_ids.len()).map(|_| "?").collect();
    let select_query = format!("SELECT cet.event_id, ct.* FROM calendar_events_tags cet JOIN calendar_tags ct ON ct.id = cet.tag_id WHERE cet.event_id IN ({})", placeholders.join(", "));
    let mut select_query = query_as::<_, CalendarEventTagRow>(&select_query);

    for id in event_ids {
        select_query = select_query.bind(id);
    }

    let rows = select_query.fetch_all(&state.db).await.map_err(|e| {
        error!("Failed to fetch calendar tags: {:#?}", e);
        "Database error".to_string()
    })?;

    let mut tags_map: HashMap<i64, Vec<CalendarTag>> = HashMap::new();

    for row in rows {
        let tag = CalendarTag {
            id: row.id,
            name: row.name,
            user_id: row.user_id,
        };

        tags_map.entry(row.event_id).or_default().push(tag);
    }

    let mut result: Vec<CalendarEventWithTag> = Vec::new();
    for event in calendar_events {
        let tags = tags_map.remove(&event.id).unwrap_or_default();
        result.push(CalendarEventWithTag { event, tags });
    }

    Ok(result)
}

#[tauri::command]
pub async fn delete_calendar_event(
    state: State<'_, AppState>,
    event: CalendarEvent,
) -> Result<(), String> {
    let state: &AppState = &state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Failed to delete calendar event at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let deleted_event = query_as::<_, StoredCalendarEvent>("DELETE FROM calendar_events WHERE id = ? AND user_id = ? RETURNING *")
        .bind(event.id)
        .bind(session.user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to delete calendar event for user '{}': {:#?}", session.user.name, e);
            "Database error".to_string()
        })?;

    let dek = session.get_dek(&state.session)?;
    let blob = decrypt(&dek, &deleted_event.payload, "calendar_events.payload", session.user.id, deleted_event.id)?;
    let payload: CalendarEventPayload = serde_json::from_slice(&blob).map_err(|e| {
        error!("Failed to parse event payload during event deletion: {:#?}", e);
        "An error occurred".to_string()
    })?;

    let event = CalendarEvent {
        id: deleted_event.id,
        user_id: session.user.id,
        isodate: deleted_event.isodate,
        title: payload.title,
        description: payload.description,
        start_time: deleted_event.start_time,
        end_time: deleted_event.end_time,
    };

    info!("CALENDAR EVENT DELETION SUCCESS ({}): User '{}' deleted a calendar event successfully", create_timestamp(), session.user.name);

    if let Some(value) = event.isodate.get(..7) {
        let year_month = value;
        let key = format!("{}-{}-calevents", session.user.id, year_month);

        state.session.cache.update_cache(&key, &HashMap::from([(event.id, event)]), &UpdateTask::Delete, "Failed to delete calendar event from cache");
    } else {
        error!("CACHE DELETION FAILED ({}): Calendar event date invalid for user '{}', event ID: '{}'", create_timestamp(), session.user.name, event.id);
    }

    Ok(())
}

#[tauri::command]
pub async fn update_calendar_event(
    state: State<'_, AppState>,
    form: CalendarEventForm,
    event_id: i64,
) -> Result<CalendarEvent, String> {
    let state: &AppState = &state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Failed to update calendar event at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if form.title.trim().is_empty() {
        error!("CALENDAR UPDATE FAILED ({}): User '{}' tried updating an event with an empty title", create_timestamp(), session.user.name);
        return Err("An error occurred".to_string());
    }

    let stored_event = query_as::<_, StoredCalendarEvent>("SELECT * FROM calendar_events WHERE id = ? AND user_id = ?")
        .bind(event_id)
        .bind(session.user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch event to update: {:#?}", e);
            "Database error".to_string()
        })?;

    let tags = query_as::<_, CalendarTag>("SELECT ct.* FROM calendar_events_tags cet JOIN calendar_tags ct ON ct.id = cet.tag_id WHERE cet.event_id = ?")
        .bind(stored_event.id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch tags for event to update: {:#?}", e);
            "Database error".to_string()
        })?;

    let dek = session.get_dek(&state.session)?;
    let blob = decrypt(&dek, &stored_event.payload, "calendar_events.payload", session.user.id, stored_event.id)?;
    let payload: CalendarEventPayload = serde_json::from_slice(&blob).map_err(|e| {
        error!("Failed to parse event payload during event update: {:#?}", e);
        "An error occurred".to_string()
    })?;

    if stored_event.isodate == form.isodate &&
    payload.title == form.title &&
    payload.description == form.description &&
    stored_event.start_time == form.start_time &&
    stored_event.end_time == form.end_time &&
    tags == form.tags {
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

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to start transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    let payload = serde_json::to_vec(&CalendarEventPayloadRef {
        title: &cleaned_title,
        description: &cleaned_description,
    }).map_err(|e| {
        error!("Failed to parse calendar event's title and description for payload: {:#?}", e);
        "An error occurred".to_string()
    })?;
    let blob = encrypt(&dek, &payload, "calendar_events.payload", session.user.id, stored_event.id)?;

    sqlx::query("UPDATE calendar_events SET isodate = ?, payload = ?, start_time = ?, end_time = ? WHERE id = ? AND user_id = ?")
        .bind(&cleaned_date)
        .bind(blob)
        .bind(start_time)
        .bind(end_time)
        .bind(stored_event.id)
        .bind(session.user.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to update calendar event: {:#?}", e);
            "Database error".to_string()
        })?;

    let updated_event = CalendarEvent {
        id: stored_event.id,
        user_id: session.user.id,
        isodate: cleaned_date,
        title: cleaned_title,
        description: cleaned_description,
        start_time,
        end_time,
    };

    if form.tags != tags {
        let tag_ids: Vec<i64> = form.tags.iter().map(|t| t.id).collect();
        let removed_tag_ids: Vec<i64> = tags.into_iter().filter(|t| !tag_ids.contains(&t.id)).map(|t| t.id).collect();

        if !tag_ids.is_empty() {
            let values_part: Vec<&str> = (0..tag_ids.len()).map(|_| "(?, ?)").collect();
            let insert_query = format!("INSERT OR IGNORE INTO calendar_events_tags (event_id, tag_id) VALUES {}", values_part.join(", "));
            let mut insert_query = sqlx::query::<sqlx::Sqlite>(&insert_query);

            for id in tag_ids {
                insert_query = insert_query.bind(updated_event.id).bind(id);
            }

            insert_query.execute(&mut *tx).await.map_err(|e| {
                error!("Failed to add tags to event: {:#?}", e);
                "Database error".to_string()
            })?;
        }

        if !removed_tag_ids.is_empty() {
            let placeholders: Vec<&str> = (0..removed_tag_ids.len()).map(|_| "?").collect();
            let delete_query = format!("DELETE FROM calendar_events_tags WHERE event_id = ? AND tag_id IN ({})", placeholders.join(", "));
            let mut delete_query = sqlx::query(&delete_query).bind(updated_event.id);

            for id in removed_tag_ids {
                delete_query = delete_query.bind(id);
            }

            delete_query.execute(&mut *tx).await.map_err(|e| {
                error!("Failed to delete tags from event during update: {:#?}", e);
                "Database error".to_string()
            })?;
        }
    }

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    info!("CALENDAR EVENT UPDATED SUCCESSFULLY ({}): User '{}' updated a calendar event", create_timestamp(), session.user.name);

    if let Some(value) = form.isodate.get(..7) {
        let year_month = value;
        let key = format!("{}-{}-calevents", session.user.id, year_month);

        state.session.cache.update_cache(&key, &HashMap::from([(updated_event.id, updated_event.clone())]), &UpdateTask::Update, "Failed to update calendar event in cache");
    } else {
        error!("CACHE UPDATE FAILED ({}): Calendar event date invalid for user '{}', event ID: '{}'", create_timestamp(), session.user.name, updated_event.id);
    }

    Ok(updated_event)
}

#[tauri::command]
pub async fn add_calendar_tag(
    state: State<'_, AppState>,
    name: String,
) -> Result<CalendarTag, String> {
    let state: &AppState = &state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Failed to add calendar tag at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if name.is_empty() {
        error!("ADDING CALENDAR TAG FAILED ({}): User '{}' provided no name for calendar tag", create_timestamp(), session.user.name);
        return Err("An error occurred".to_string());
    }

    let name = ammonia::clean(&name);

    let tag = query_as::<_, CalendarTag>("INSERT INTO calendar_tags (name, user_id) VALUES (?, ?) RETURNING *")
        .bind(name)
        .bind(session.user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to add calendar tag: {:#?}", e);
            "Database error".to_string()
        })?;

    info!("CALENDAR TAG CREATION SUCCESS ({}): User '{}' added a calendar tag successfully", create_timestamp(), session.user.name);

    Ok(tag)
}

#[tauri::command]
pub async fn get_calendar_tags(
    state: State<'_, AppState>,
) -> Result<Vec<CalendarTag>, String> {
    let state: &AppState = &state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Failed to fetch calendar tags at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let tags = query_as::<_, CalendarTag>("SELECT * FROM calendar_tags WHERE user_id = ?")
        .bind(session.user.id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch calendar tags: {:#?}", e);
            "Database error".to_string()
        })?;

    Ok(tags)
}

#[tauri::command]
pub async fn delete_calendar_tag(
    state: State<'_, AppState>,
    tag_id: i64,
) -> Result<i64, String> {
    let state: &AppState = &state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Failed to delete calendar tag at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let deleted_tag = sqlx::query_scalar::<_, i64>("DELETE FROM calendar_tags WHERE id = ? AND user_id = ? RETURNING id")
        .bind(tag_id)
        .bind(session.user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to delete tag '{}' for user '{}': {:#?}", tag_id, session.user.name, e);
            "Database error".to_string()
        })?;

    Ok(deleted_tag)
}