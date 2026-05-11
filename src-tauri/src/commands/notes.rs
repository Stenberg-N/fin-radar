use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool, query_as};
use tauri::State;
use log::{info, warn, error};
use ammonia;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Note {
    pub id: i64,
    pub user_id: i64,
    pub tab_id: i32,
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
}

#[tauri::command]
pub async fn create_note (
    pool: State<'_, SqlitePool>,
    user_id: i64,
    username: String,
    tab_id: i32,
    title: String,
    content: String,
) -> Result<Note, String> {
    let mut tx = pool.begin().await.map_err(|e| {
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

    let title = ammonia::clean(&title);
    let content = ammonia::clean(&content);

    let note = query_as::<_, Note>("INSERT INTO notes (user_id, tab_id, order_id, title, content) VALUES (?, ?, ?, ?, ?) RETURNING *")
        .bind(user_id)
        .bind(&tab_id)
        .bind(&new_order_id)
        .bind(&title)
        .bind(&content)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to create note for user '{}': {:#?}", username, e);
            "Database error".to_string()
        })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    info!("User '{}' successfully added a note", username);

    Ok(note)
}

#[tauri::command]
pub async fn get_notes (
    pool: State<'_, SqlitePool>,
    user_id: i64,
    username: String,
    tab_id: i32,
) -> Result<Vec<Note>, String> {
    let notes = query_as::<_, Note>("SELECT * FROM notes WHERE user_id = ? AND tab_id = ?")
        .bind(user_id)
        .bind(&tab_id)
        .fetch_all(&*pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch notes for user '{}': {:#?}", username, e);
            "Database error".to_string()
        })?;

    Ok(notes)
}

#[tauri::command]
pub async fn create_tab (
    pool: State<'_, SqlitePool>,
    user_id: i64,
    username: String,
    title: String,
) -> Result<Tab, String> {
    let mut tx = pool.begin().await.map_err(|e| {
        error!("Failed to start transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    let max_order_id = sqlx::query_scalar::<_, i32>("SELECT MAX(order_id) FROM tabs WHERE user_id = ?")
        .bind(user_id)
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
        .bind(user_id)
        .bind(new_order_id)
        .bind(&title)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to create tab for user '{}': {:#?}", username, e);
            "Database error".to_string()
        })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    Ok(tab)
}

#[tauri::command]
pub async fn get_tabs (
    pool: State<'_, SqlitePool>,
    user_id: i64,
    username: String,
) -> Result<Vec<Tab>, String> {
    let tabs = query_as::<_, Tab>("SELECT * FROM tabs WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(&*pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch tabs for user '{}': {:#?}", username, e);
            "Database error".to_string()
        })?;

    Ok(tabs)
}