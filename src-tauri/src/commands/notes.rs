use serde::{Deserialize, Serialize};
use sqlx::{FromRow, query_as, Row};
use tauri::State;
use log::{info, error, warn};
use ammonia;
use std::collections::{HashSet, HashMap};

use crate::{AppState, structs::session::SessionData};
use super::helpers::create_timestamp;
use crate::structs::cache::{CacheData, UpdateTask};

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct Note {
    pub id: i64,
    user_id: i64,
    pub tab_id: i64,
    order_id: u32,
    title: String,
    content: String,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Tab {
    id: i64,
    user_id: i64,
    order_id: u32,
    title: String,
    color: String,
}

#[derive(Serialize, FromRow)]
pub struct TabIdTitle {
    id: i64,
    title: String,
}

async fn fetch_and_cache_notes(
    state: &State<'_, AppState>,
    key: &str,
    user_id: i64,
    username: &str,
    tab_id: i64,
) -> Result<Vec<Note>, String> {
    let notes = query_as::<_, Note>("SELECT * FROM notes WHERE user_id = ? AND tab_id = ? ORDER BY order_id ASC")
        .bind(user_id)
        .bind(tab_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch notes for user '{}': {:#?}", username, e);
            "Database error".to_string()
        })?;

    if let Err(e) = state.session.cache.cache_results(key.to_string(), CacheData::from(notes.clone())) {
        error!("CACHE POISONED ({}): Failed to set notes to cache for user '{}': {:#?}", create_timestamp(), username, e);
    }

    Ok(notes)
}

#[tauri::command]
pub async fn create_note(
    state: State<'_, AppState>,
    tab_id: i64,
    title: String,
    content: String,
) -> Result<Note, String> {
    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Creating note failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if tab_id.le(&0) {
        error!("User '{}' tried creating a note with an invalid tab ID: '{}'", session.user.name, tab_id);
        return Err("An error occurred".to_string());
    }

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    let max_order_id = sqlx::query_scalar::<_, u32>("SELECT MAX(order_id) FROM notes WHERE tab_id = ?")
        .bind(&tab_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to fetch the highest current order ID for creating note: {:#?}", e);
            "Database error".to_string()
        })?
        .unwrap_or(0);

    let new_order_id = max_order_id + 1;

    let mut cleaner = ammonia::Builder::new();
    cleaner
        .add_tags(&["p", "span"])
        .add_tag_attributes("p", &["style"])
        .add_tag_attributes("span", &["style"])
        .filter_style_properties(
            HashSet::from(["font-size"])
        );

    let title = cleaner.clean(&title).to_string();
    let content = cleaner.clean(&content).to_string();

    let note = query_as::<_, Note>("INSERT INTO notes (user_id, tab_id, order_id, title, content) VALUES (?, ?, ?, ?, ?) RETURNING *")
        .bind(session.user.id)
        .bind(tab_id)
        .bind(new_order_id)
        .bind(title)
        .bind(content)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to create note for user '{}': {:#?}", session.user.name, e);
            "Database error".to_string()
        })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    info!("User '{}' successfully added a note at {}", session.user.name, create_timestamp());

    let key = format!("{}-{}-notes", session.user.id, tab_id);

    match state.session.cache.contains(&key) {
        Ok(true) => {
            if let Err(e) = state.session.cache.update_cache(&key, &HashMap::from([(note.id, note.clone())]), &UpdateTask::Update) {
                error!("CACHE POISONED ({}): Failed to add note to cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
            }
        },
        Ok(false) => {
            if let Err(e) = state.session.cache.cache_results(key, CacheData::from(Vec::from([note.clone()]))) {
                error!("CACHE POISONED ({}): Failed to add note to cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
            }
        },
        Err(e) => {
            error!("CACHE POISONED ({}): Failed to check cache for user '{}'. Refetching data: {:#?}", create_timestamp(), session.user.name, e);
            fetch_and_cache_notes(&state, &key, session.user.id, &session.user.name, tab_id).await?;
        }
    }

    Ok(note)
}

#[tauri::command]
pub async fn get_notes(
    state: State<'_, AppState>,
    tab_id: i64,
) -> Result<Vec<Note>, String> {
    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Fetching notes failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if tab_id.le(&0) {
        error!("NOTES FETCH FAILED ({}): User '{}' tried fetching notes with an invalid tab ID: '{}'", create_timestamp(), session.user.name, tab_id);
        return Err("An error occurred".to_string());
    }

    let key = format!("{}-{}-notes", session.user.id, tab_id);

    let mut notes = match state.session.cache.contains(&key) {
        Ok(true) => {
            match state.session.cache.get_notes(&key) {
                Ok(Some(notes)) => notes.values().cloned().collect(),
                Ok(None) => fetch_and_cache_notes(&state, &key, session.user.id, &session.user.name, tab_id).await?,
                Err(e) => {
                    error!("CACHE POISONED ({}): Failed to get notes from cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
                    fetch_and_cache_notes(&state, &key, session.user.id, &session.user.name, tab_id).await?
                }
            }
        },
        Ok(false) => fetch_and_cache_notes(&state, &key, session.user.id, &session.user.name, tab_id).await?,
        Err(e) => {
            error!("CACHE POISONED ({}): Failed to check cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
            fetch_and_cache_notes(&state, &key, session.user.id, &session.user.name, tab_id).await?
        }
    };

    notes.sort_by(|a, b| a.order_id.cmp(&b.order_id));
    Ok(notes)
}

#[tauri::command]
pub async fn update_note(
    state: State<'_, AppState>,
    note_array: Vec<Note>,
) -> Result<Vec<Note>, String> {
    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Updating note failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if note_array.is_empty() {
        warn!("NOTE UPDATE FAILED ({}): User '{}' provided no notes", create_timestamp(), session.user.name);
        return Err("An error occurred".to_string());
    }

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    let mut cleaner = ammonia::Builder::new();
    cleaner
        .add_tags(&["p", "span", "div", "h1", "h2", "h3", "label", "input", "ul", "li"])
        .add_tag_attributes("input", &["type"])
        .add_tag_attribute_values("input", "type", &["checkbox"])
        .add_tag_attributes("ul", &["data-type"])
        .add_tag_attribute_values("ul", "data-type", &["taskList"])
        .add_tag_attributes("li", &["data-checked", "data-type"])
        .add_tag_attribute_values("li", "data-type", &["taskItem"])
        .add_tag_attributes("p", &["style"])
        .add_tag_attributes("span", &["style"])
        .add_tag_attributes("div", &["style"])
        .add_tag_attributes("h1", &["style"])
        .add_tag_attributes("h2", &["style"])
        .add_tag_attributes("h3", &["style"])
        .filter_style_properties(
            HashSet::from(["text-align", "font-size", "background-color", "color"])
        );

    for note in &note_array {
        let title = cleaner.clean(&note.title).to_string();
        let content = cleaner.clean(&note.content).to_string();

        sqlx::query("UPDATE notes SET title = ?, content = ? WHERE id = ? AND user_id = ?")
            .bind(&title)
            .bind(content)
            .bind(note.id)
            .bind(session.user.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to update note '{}' for user '{}': {:#?}", title, session.user.name, e);
                "Database error".to_string()
            })?;
    }

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    let placeholders: Vec<_> = (0..note_array.len()).map(|_| "?").collect();
    let select_query = format!("SELECT * FROM notes WHERE user_id = ? AND id IN ({})", placeholders.join(", "));
    let mut select_query = sqlx::query(&select_query).bind(session.user.id);

    for note in &note_array {
        select_query = select_query.bind(&note.id);
    }

    let rows = select_query.fetch_all(&state.db).await.map_err(|e| {
        error!("Failed to fetch updated notes for user '{}': {:#?}", session.user.name, e);
        "Database error".to_string()
    })?;

    let updated_notes: Vec<Note> = rows
        .into_iter()
        .map(|row| Note {
            id: row.get("id"),
            user_id: row.get("user_id"),
            tab_id: row.get("tab_id"),
            order_id: row.get("order_id"),
            title: row.get("title"),
            content: row.get("content"),
        })
        .collect();

    let tab_id = match updated_notes.first().map(|n| n.tab_id) {
        Some(value) => value,
        None => {
            error!("CACHE ERROR ({}): No tab ID found when updating notes for user '{}'", create_timestamp(), session.user.name);
            return Err("Cache error".to_string());
        }
    };

    let key = format!("{}-{}-notes", session.user.id, tab_id);

    if let Err(e) = state.session.cache.update_cache(&key, &updated_notes.clone().into_iter().map(|n| (n.id, n)).collect(), &UpdateTask::Update) {
        error!("CACHE POISONED ({}): Failed to update note in cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
    }

    Ok(updated_notes)
}

#[tauri::command]
pub async fn delete_note(
    state: State<'_, AppState>,
    note_id: i64,
) -> Result<Note, String> {
    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Deleting note failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let note = query_as::<_, Note>("DELETE FROM notes WHERE id = ? AND user_id = ? RETURNING *")
        .bind(note_id)
        .bind(session.user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e|{
            error!("Failed to delete note {} from user '{}': {:#?}", note_id, session.user.name, e);
            "Database error".to_string()
        })?;

    info!("User '{}' successfully deleted a note at {}", session.user.name, create_timestamp());

    let key = format!("{}-{}-notes", session.user.id, note.tab_id);

    if let Err(e) = state.session.cache.update_cache(&key, &HashMap::from([(note.id, note.clone())]), &UpdateTask::Delete) {
        error!("CACHE POISONED ({}): Failed to delete note from cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
    }

    Ok(note)
}

#[tauri::command]
pub async fn create_tab(
    state: State<'_, AppState>,
    title: String,
) -> Result<Tab, String> {
    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Creating tab failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to start transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    let max_order_id = sqlx::query_scalar::<_, u32>("SELECT MAX(order_id) FROM tabs WHERE user_id = ?")
        .bind(session.user.id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to fetch the current highest order ID for creating tab: {:#?}", e);
            "Database error".to_string()
        })?
        .unwrap_or(0);

    let new_order_id = max_order_id + 1;

    let title = ammonia::clean(&title);

    let tab = query_as::<_, Tab>("INSERT INTO tabs (user_id, order_id, title) VALUES (?, ?, ?) RETURNING *")
        .bind(session.user.id)
        .bind(new_order_id)
        .bind(title)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to create tab for user '{}': {:#?}", session.user.name, e);
            "Database error".to_string()
        })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    info!("User '{}' successfully added a new tab at {}", session.user.name, create_timestamp());

    Ok(tab)
}

#[tauri::command]
pub async fn get_tabs(
    state: State<'_, AppState>,
) -> Result<Vec<Tab>, String> {
    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Fetching tabs failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let tabs = query_as::<_, Tab>("SELECT * FROM tabs WHERE user_id = ? ORDER BY order_id ASC")
        .bind(session.user.id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch tabs for user '{}': {:#?}", session.user.name, e);
            "Database error".to_string()
        })?;

    Ok(tabs)
}

#[tauri::command]
pub async fn update_tab(
    state: State<'_, AppState>,
    tab_id: i64,
    title: String,
) -> Result<TabIdTitle, String> {
    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Updating tab failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if tab_id.le(&0) {
        error!("TAB UPDATE FAILED ({}): User '{}' tried updating a tab with an invalid tab ID: '{}'", create_timestamp(), session.user.name, tab_id);
        return Err("An error occurred".to_string());
    }

    if title.trim().is_empty() {
        error!("TAB UPDATE FAILED ({}): User '{}' did not give a name for a tab", create_timestamp(), session.user.name);
        return Err("No name for tab".to_string());
    }

    let title = ammonia::clean(&title);

    let tab = query_as::<_, TabIdTitle>("UPDATE tabs SET title = ? WHERE user_id = ? AND id = ? RETURNING id, title")
        .bind(title)
        .bind(session.user.id)
        .bind(tab_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to update tab with ID {}, by user '{}': {:#?}", tab_id, session.user.name, e);
            "Database error".to_string()
        })?;

    Ok(tab)
}

#[tauri::command]
pub async fn update_tab_color(
    state: State<'_, AppState>,
    tab_id: i64,
    color: String,
) -> Result<Tab, String> {
    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Updating tab color failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if tab_id.le(&0) {
        error!("TAB UPDATE FAILED ({}): User '{}' tried updating a tab with an invalid tab ID: '{}'", create_timestamp(), session.user.name, tab_id);
        return Err("An error occurred".to_string());
    }

    if color.trim().is_empty() {
        error!("User '{}' provided no color for tab", session.user.name);
        return Err("No color provided".to_string());
    }

    let color = ammonia::clean(&color);

    let tab = query_as::<_, Tab>("UPDATE tabs SET color = ? WHERE user_id = ? AND id = ? RETURNING *")
        .bind(color)
        .bind(session.user.id)
        .bind(tab_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to update tab color for user '{}': {:#?}", session.user.name, e);
            "Database error".to_string()
        })?;

    Ok(tab)
}

#[tauri::command]
pub async fn delete_tab(
    state: State<'_, AppState>,
    tab_id: i64,
) -> Result<Tab, String> {
    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Deleting tab failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if tab_id.le(&0) {
        error!("TAB DELETION FAILED ({}): User '{}' tried deleting a tab with an invalid tab ID: '{}'", create_timestamp(), session.user.name, tab_id);
        return Err("An error occurred".to_string());
    }

    let tab = query_as::<_, Tab>("DELETE FROM tabs WHERE user_id = ? AND id = ? RETURNING *")
        .bind(session.user.id)
        .bind(tab_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to delete tab with ID '{}' for user '{}': {:#?}", tab_id, session.user.name, e);
            "Database error".to_string()
        })?;

    info!("User '{}' successfully deleted a tab at {}", session.user.name, create_timestamp());

    Ok(tab)
}