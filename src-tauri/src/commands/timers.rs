use crate::AppState;
use super::helpers::create_timestamp;
use serde::{Deserialize, Serialize};
use sqlx::{query_as, FromRow, Row};
use tauri::State;
use log::{info, error};
use ammonia;

#[derive(FromRow, Serialize, Deserialize)]
pub struct Timer {
    pub id: i64,
    pub user_id: i64,
    pub order_id: i32,
    pub duration: i64,
    pub title: String,
    pub message: Option<String>,
}

#[tauri::command]
pub async fn create_timer (
    state: State<'_, AppState>,
    duration: i64,
    title: String,
    message: Option<String>,
) -> Result<Timer, String> {
    let session = state.get_session().map_err(|e| {
        error!("Adding timer failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let user_id = session.user_id;

    let username: String = sqlx::query_scalar("SELECT name FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to get user's name: {:#?}", e);
            "An error occurred".to_string()
        })?;

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    let max_order_id = sqlx::query_scalar::<_, i32>("SELECT MAX(order_id) FROM timers WHERE user_id = ?")
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to get highest order ID from timers: {:#?}", e);
            "Database error".to_string()
        })?
        .unwrap_or(0);
    let new_order_id = max_order_id + 1;

    let title = ammonia::clean(&title);
    let message = ammonia::clean(&message.unwrap_or(String::new()));
    let duration = duration.max(0);

    let timer = query_as::<_, Timer>("INSERT INTO timers (user_id, order_id, duration, title, message) VALUES (?, ?, ?, ?, ?) RETURNING *")
        .bind(user_id)
        .bind(new_order_id)
        .bind(duration)
        .bind(title)
        .bind(message)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to create timer: {:#?}", e);
            "Database error".to_string()
        })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    info!("Successfully created a timer for user '{}' at {}", username, create_timestamp());

    Ok(timer)
}

#[tauri::command]
pub async fn get_timers (
    state: State<'_, AppState>,
) -> Result<Vec<Timer>, String> {
    let session= state.get_session().map_err(|e| {
        error!("Fetching timers failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let user_id = session.user_id;

    let username: String = sqlx::query_scalar("SELECT name FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to get user's name: {:#?}", e);
            "An error occurred".to_string()
        })?;

    let timers = query_as::<_, Timer>("SELECT * FROM timers WHERE user_id = ? ORDER BY order_id ASC")
        .bind(user_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch timers for user '{}': {:#?}", username, e);
            "Database error".to_string()
        })?;

    Ok(timers)
}

#[tauri::command]
pub async fn update_timer (
    state: State<'_, AppState>,
    timer_array: Vec<Timer>,
) -> Result<Vec<Timer>, String> {
    let session= state.get_session().map_err(|e| {
        error!("Updating timer failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let user_id = session.user_id;

    let username: String = sqlx::query_scalar("SELECT name FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to get user's name: {:#?}", e);
            "An error occurred".to_string()
        })?;

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    for timer in &timer_array {
        let title = ammonia::clean(&timer.title);
        let message = ammonia::clean(&timer.message.clone().unwrap_or(String::new()));
        let duration = timer.duration.max(0);

        sqlx::query("UPDATE timers SET duration = ?, title = ?, message = ? WHERE id = ? AND user_id = ?")
            .bind(duration)
            .bind(title)
            .bind(message)
            .bind(timer.id)
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to update timer {} for user '{}': {:#?}", timer.id, username, e);
                "Database error".to_string()
            })?;
    }

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    let placeholders: Vec<_> = (0..timer_array.len()).map(|_| "?").collect();
    let select_query = format!("SELECT * FROM timers WHERE user_id = ? AND id IN ({})", placeholders.join(", "));
    let mut select_query = sqlx::query(&select_query).bind(user_id);

    for timer in &timer_array {
        select_query = select_query.bind(timer.id);
    }

    let rows = select_query.fetch_all(&state.db).await.map_err(|e| {
        error!("Failed to get updated timers for user '{}': {:#?}", username, e);
        "Database error".to_string()
    })?;

    let updated_timers: Vec<Timer> = rows
        .into_iter()
        .map(|row| Timer {
            id: row.get("id"),
            user_id: row.get("user_id"),
            order_id: row.get("order_id"),
            duration: row.get("duration"),
            title: row.get("title"),
            message: row.get("message"),
        })
        .collect();

    Ok(updated_timers)
}

#[tauri::command]
pub async fn delete_timer (
    state: State<'_, AppState>,
    timer_id: i64,
) -> Result<Timer, String> {
    let session= state.get_session().map_err(|e| {
        error!("Deleting timer failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let user_id = session.user_id;

    let username: String = sqlx::query_scalar("SELECT name FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to get user's name: {:#?}", e);
            "An error occurred".to_string()
        })?;

    let timer = query_as::<_, Timer>("DELETE FROM timers WHERE id = ? AND user_id = ? RETURNING *")
        .bind(timer_id)
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to delete timer for user '{}': {:#?}", username, e);
            "Database error".to_string()
        })?;

    info!("Successfully deleted timer by user '{}' at {}", username, create_timestamp());

    Ok(timer)
}