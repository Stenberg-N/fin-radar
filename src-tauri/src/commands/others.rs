use crate::AppState;
use super::helpers::create_timestamp;
use tauri::State;
use dirs::data_local_dir;
use std::fs::{copy, create_dir, read_dir};
use std::path::PathBuf;
use std::io::ErrorKind;
use time::{OffsetDateTime, macros::{format_description}};
use log::{info, error, debug};

#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArrayOption {
    Notes,
    Tabs,
    Timers
}

/************************************************************************************************************************\

OTHER "MISCELLANEOUS" COMMANDS

\************************************************************************************************************************/

#[tauri::command]
pub async fn backup_database (
    state: State<'_, AppState>,
) -> Result<(), String> {
    let session = state.get_session().map_err(|e| {
        error!("DATABASE BACKUP FAILED ({}): Could not get session: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    info!("Starting database backup");

    let local_data_dir: PathBuf = data_local_dir().ok_or("Failed to get Local data directory")?;
    let app_dir: PathBuf = local_data_dir.join("com.stenberg.fin-radar");
    let database_dir: PathBuf = app_dir.join("database");
    let backup_dir: PathBuf = app_dir.join("backups");

    match create_dir(&backup_dir) {
        Ok(()) => info!("Backup directory created"),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            debug!("Backup directory already exists");
        }
        Err(e) => {
            error!("Failed to create backup directory: {:#?}", e);
            return Err("Failed to create backup directory".to_string());
        }
    }

    let now = match OffsetDateTime::now_local() {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to get local time: {:#?}", e);
            return Err("Failed to get local time".to_string());
        }
    };

    let timestamp = match now.format(&format_description!("[year]-[month]-[day]_[hour]h-[minute]m-[second]s")) {
        Ok(timestamp) => timestamp,
        Err(e) => {
            error!("Time format error: {:#?}", e);
            return Err("Time format error".to_string());
        }
    };

    let backup_path: PathBuf = backup_dir.join(format!("database-backup_{}", timestamp));

    if let Err(e) = create_dir(&backup_path) {
        error!("Failed to create backup path {:#?}: {:#?}", backup_path, e);
        return Err("Failed to create backup path".to_string());
    }
    info!("Created backup path: {:#?}", backup_path);

    let entries = match read_dir(&database_dir) {
        Ok(entries) => entries,
        Err(e) => {
            error!("Failed to read database directory {:#?}: {:#?}", database_dir, e);
            return Err("Failed to read database directory".to_string());
        }
    };

    for entry in entries {
        let entry = entry.map_err(|e| {
            error!("Failed to read file: {:#?}", e);
            "Failed to read file".to_string()
        })?;
        let src_path = entry.path();
        let dest_path = backup_path.join(entry.file_name());

        copy(&src_path, &dest_path).map_err(|e| {
            error!("Failed to copy {:#?} to {:#?}: {:#?}", src_path, dest_path, e);
            "Failed to copy file".to_string()
        })?;
    }
    info!("DATABASE BACKUP COMPLETED ({}): Successful by user '{}'", create_timestamp(), session.user.name);

    Ok(())
}

#[tauri::command]
pub async fn reorder_array (
    state: State<'_, AppState>,
    array: Vec<i64>,
    array_type: ArrayOption,
) -> Result<(), String> {
    let session = state.get_session().map_err(|e| {
        error!("Reordering array failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if array.is_empty() {
        error!("Reordering failed due to array being empty");
        return Err("Reordering failed".to_string());
    }

    let table = match array_type {
        ArrayOption::Notes => "notes",
        ArrayOption::Tabs => "tabs",
        ArrayOption::Timers => "timers",
    };

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {:#?}", e);
        "Reordering failer".to_string()
    })?;

    for (index, &id) in array.iter().enumerate() {
        let order_id = (index + 1) as i32;
        let query = format!("UPDATE {} SET order_id = ? WHERE id = ? AND user_id = ?", table);

        sqlx::query(&query)
            .bind(order_id)
            .bind(id)
            .bind(session.user.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to update {}' order IDs for user '{}': {:#?}", table, session.user.name, e);
                "Reordering failed".to_string()
            })?;
    }

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:#?}", e);
        "Reordering failed".to_string()
    })?;

    Ok(())
}