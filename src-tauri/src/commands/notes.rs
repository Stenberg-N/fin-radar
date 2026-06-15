use crate::AppState;
use super::helpers::create_timestamp;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, query_as, Row};
use tauri::State;
use log::{info, error};
use ammonia;
use std::collections::HashSet;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Note {
    pub id: i64,
    pub user_id: i64,
    pub tab_id: i64,
    pub order_id: i32,
    pub title: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Tab {
    pub id: i64,
    pub user_id: i64,
    pub order_id: i32,
    pub title: String,
    pub color: String,
}

#[derive(Serialize, FromRow)]
pub struct TabIdTitle {
    pub id: i64,
    pub title: String,
}

#[tauri::command]
pub async fn create_note (
    state: State<'_, AppState>,
    tab_id: i64,
    title: String,
    content: String,
) -> Result<Note, String> {
    let session = state.get_session().map_err(|e| {
        error!("Creating note failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    let max_order_id = sqlx::query_scalar::<_, i32>("SELECT MAX(order_id) FROM notes WHERE tab_id = ?")
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

    Ok(note)
}

#[tauri::command]
pub async fn get_notes (
    state: State<'_, AppState>,
    tab_id: i64,
) -> Result<Vec<Note>, String> {
    let session = state.get_session().map_err(|e| {
        error!("Fetching notes failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let notes = query_as::<_, Note>("SELECT * FROM notes WHERE user_id = ? AND tab_id = ? ORDER BY order_id ASC")
        .bind(session.user.id)
        .bind(tab_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch notes for user '{}': {:#?}", session.user.name, e);
            "Database error".to_string()
        })?;

    Ok(notes)
}

#[tauri::command]
pub async fn update_note (
    state: State<'_, AppState>,
    note_array: Vec<Note>,
) -> Result<Vec<Note>, String> {
    let session = state.get_session().map_err(|e| {
        error!("Updating note failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    let mut cleaner = ammonia::Builder::new();
    cleaner
        .add_tags(&["p", "span", "div", "h1", "h2", "h3"])
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

    Ok(updated_notes)
}

#[tauri::command]
pub async fn delete_note (
    state: State<'_, AppState>,
    note_id: i64,
) -> Result<Note, String> {
    let session = state.get_session().map_err(|e| {
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

    Ok(note)
}

#[tauri::command]
pub async fn create_tab (
    state: State<'_, AppState>,
    title: String,
) -> Result<Tab, String> {
    let session = state.get_session().map_err(|e| {
        error!("Creating tab failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to start transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    let max_order_id = sqlx::query_scalar::<_, i32>("SELECT MAX(order_id) FROM tabs WHERE user_id = ?")
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
pub async fn get_tabs (
    state: State<'_, AppState>,
) -> Result<Vec<Tab>, String> {
    let session = state.get_session().map_err(|e| {
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
pub async fn update_tab (
    state: State<'_, AppState>,
    tab_id: i64,
    title: String,
) -> Result<TabIdTitle, String> {
    let session = state.get_session().map_err(|e| {
        error!("Updating tab failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if title.trim().is_empty() {
        error!("User '{}' did not give a name for a tab", session.user.name);
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
pub async fn update_tab_color (
    state: State<'_, AppState>,
    tab_id: i64,
    color: String,
) -> Result<Tab, String> {
    let session = state.get_session().map_err(|e| {
        error!("Updating tab color failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

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
pub async fn delete_tab (
    state: State<'_, AppState>,
    tab_id: i64,
) -> Result<Tab, String> {
    let session = state.get_session().map_err(|e| {
        error!("Deleting tab failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

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