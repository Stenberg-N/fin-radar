use serde::{Deserialize, Serialize};
use sqlx::{query_as, FromRow, SqlitePool};
use tauri::State;
use argon2::{Argon2, password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash}};
use dirs::data_local_dir;
use std::fs::{copy, create_dir, read_dir};
use std::path::PathBuf;
use std::io::ErrorKind;
use time::{macros::format_description, OffsetDateTime};
use log::{info, warn, error, debug};

#[derive(FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub password: String,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i64,
    pub category: String,
    pub description: String,
    pub amount: f64,
    pub _type: String,
}

fn validate_password(pw: &str) -> bool {
    let has_min_length = pw.chars().count() >= 10;
    let no_spaces = !pw.chars().any(|c| c.is_whitespace());
    let has_numbers = pw.chars().any(|c| c.is_ascii_digit());
    let has_uppercase = pw.chars().any(|c| c.is_uppercase() && c.is_alphabetic());
    let has_lowercase = pw.chars().any(|c| c.is_lowercase() && c.is_alphabetic());
    let has_special_char = pw.chars().any(|c| r#"=_!@#$€£¤%^&*(){}[],.?'":|/\+-<>~§"#.contains(c));

    has_min_length && no_spaces && has_numbers && has_uppercase && has_lowercase && has_special_char
}

#[tauri::command]
pub async fn create_user (
    pool: State<'_, SqlitePool>,
    name: String,
    password: String,
    confirm_password: String,
) -> Result<(), String> {
    if password != confirm_password {
        error!("User creation failed due to password mismatch");
        return Err("Password mismatch".to_string());
    }
    if !validate_password(&password) {
        error!("User creation failed due to password requirements not being met");
        return Err("Password requirements not met".to_string());
    }

    let existing_user = query_as::<_, User>(
        "SELECT * FROM users WHERE name = ?"
    )
    .bind(&name)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| {
        error!("Failed to check existing user: {:#?}", e);
        "Database error".to_string()
    })?;

    if existing_user.is_some() {
        error!("User creation failed due to using an already taken username");
        return Err("User with this name already exists".to_string());
    }

    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes())
        .map_err(|e| {
            error!("Hashing failed: {:#?}", e);
            "Failed to create user".to_string()
        })?
        .to_string();
    let parsed_hash = PasswordHash::new(&password_hash)
        .map_err(|e| {
            error!("Hash parsing failed: {:#?}", e);
            "Failed to create user".to_string()
        })?;
    assert!(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok());

    let affected_rows = sqlx::query("INSERT INTO users (name, password) VALUES (?, ?)")
        .bind(&name)
        .bind(&password_hash)
        .execute(&*pool)
        .await
        .map_err(|e| {
            error!("Database error when creating user: {:#?}", e);
            "Database error".to_string()
        })?
        .rows_affected();

    if affected_rows == 0 {
        error!("Failed to create user");
        return Err("Failed to create user".to_string());
    }

    let timestamp = OffsetDateTime::now_local()
        .ok()
        .and_then(|dt| dt.format(&format_description!("[year]-[month]-[day]__at__[hour]H-[minute]M-[second]S")).ok())
        .unwrap_or("Unknown time".to_string());

    info!("User '{}' created successfully at {}", name, timestamp);

    Ok(())
}

#[tauri::command]
pub async fn login_user (
    pool: State<'_, SqlitePool>,
    name: String,
    password: String,
) -> Result<User, String> {
    info!("LOGIN ATTEMPT: initiated for user {}", name);

    let user = query_as::<_, User>(
        "SELECT * FROM users WHERE name = ?"
    )
    .bind(&name)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| {
        error!("Database error when fetching user '{}' {:#?}", name, e);
        "Database error".to_string()
    })?;

    let user = user.ok_or("Invalid login information")?;

    let parsed_hash = PasswordHash::new(&user.password)
        .map_err(|_| {
            "Invalid login information".to_string()
        })?;

    let timestamp = OffsetDateTime::now_local()
        .ok()
        .and_then(|dt| dt.format(&format_description!("[year]-[month]-[day]__at__[hour]H-[minute]M-[second]S")).ok())
        .unwrap_or("Unknown time".to_string());

    let argon2 = Argon2::default();
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => {
            info!("LOGIN SUCCESS: User '{}' successfully logged in at {}", name, timestamp);
            Ok(user)
        },
        Err(_) => {
            warn!("LOGIN FAILED: incorrect password for user '{}' at {}", name, timestamp);
            return Err("Invalid login information".to_string());
        }
    }
}

#[tauri::command]
pub async fn delete_user (
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| {
        error!("Failed to begin transaction to delete user with id: {}: {:#?}", id, e);
        "Database error".to_string()
    })?;

    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to delete user {:#?}: {:#?}", id, e);
            "Failed to delete user".to_string()
        })?
        .rows_affected();

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction to delete user with id: {}: {:#?}", id, e);
        "Database error".to_string()
    })?;

    info!("User deleted successfully: {}", id);

    Ok(())
}

#[tauri::command]
pub async fn change_password (
    pool: State<'_, SqlitePool>,
    id: i64,
    name: String,
    current_password: String,
    new_password: String,
    confirm_new_password: String,
) -> Result<(), String> {
    if new_password != confirm_new_password {
        error!("Password change for user '{}' failed due to new password and confirmation mismatching", name);
        return Err("Password mismatch".to_string());
    }

    let user = query_as::<_, User>(
        "SELECT * FROM users WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| {
        error!("Failed to get user '{}' from database: {:#?}", name, e);
        "Failed to get user from database".to_string()
    })?;

    let user = user.ok_or("Invalid user information")?;

    let parsed_hash = PasswordHash::new(&user.password).map_err(|_| {
        "Password update failed".to_string()
    })?;

    match Argon2::default().verify_password(current_password.as_bytes(), &parsed_hash) {
        Ok(_) => info!("PASSWORD CHANGE: User's '{}' given password matched the account's current password", name),
        Err(_) => {
            warn!("PASSWORD CHANGE FAILED: User's '{}' given password did not match with the account's current password!", name);
            return Err("Updating password failed".to_string());
        }
    }

    let new_password_hash = Argon2::default()
        .hash_password(new_password.as_bytes())
        .map_err(|e| {
            error!("Failed to create hash for new password: {:#?}", e);
            "Password update failed".to_string()
        })?
        .to_string();

    let new_parsed_hash = PasswordHash::new(&new_password_hash).map_err(|e| {
        error!("Failed to parse new password's hash: {:#?}", e);
        "Password update failed".to_string()
    })?;
    assert!(Argon2::default().verify_password(new_password.as_bytes(), &new_parsed_hash).is_ok());

    let mut tx = pool.begin().await.map_err(|e| {
        error!("Failed to begin transaction to update user's '{}' password: {:#?}", name, e);
        "Database error".to_string()
    })?;

    sqlx::query("UPDATE users SET password = ? WHERE id = ?")
        .bind(&new_password_hash)
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("PASSWORD UPDATE FAILED: Failed to update user's '{}' password into database: {:#?}", name, e);
            "Failed to update password".to_string()
        })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction to update user's '{}' password: {:#?}", name, e);
        "Database error".to_string()
    })?;

    let timestamp = OffsetDateTime::now_local()
        .ok()
        .and_then(|dt| dt.format(&format_description!("[year]-[month]-[day]__at__[hour]H-[minute]M-[second]S")).ok())
        .unwrap_or("Unknown time".to_string());

    info!("PASSWORD CHANGED: User '{}' changed their password successfully at {}", name, timestamp);

    Ok(())
}

#[tauri::command]
pub async fn backup_database () -> Result<(), String> {
    info!("Starting database backup");

    let local_data_dir: PathBuf = data_local_dir().ok_or("Failed to get Local data directory")?;
    let app_dir: PathBuf = local_data_dir.join("com.stenberg.finance-tracker");
    let database_dir: PathBuf = app_dir.join("database");
    let backup_dir: PathBuf = app_dir.join("backups");

    match create_dir(&backup_dir) {
        Ok(()) => info!("Backup directory created"),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            debug!("Backup directory already exists");
        }
        Err(e) => {
            error!("Failed to create backup directory: {:#?}", e);
            return Err(format!("Failed to create backup directory: {}", e));
        }
    }

    let now = match OffsetDateTime::now_local() {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to get local time: {:#?}", e);
            return Err(format!("Failed to get local time: {}", e));
        }
    };

    let timestamp = match now.format(&format_description!("[year]-[month]-[day]_T[hour]H-[minute]M-[second]S")) {
        Ok(timestamp) => timestamp,
        Err(e) => {
            error!("Time format error: {:#?}", e);
            return Err(format!("Time format error: {}", e));
        }
    };

    let backup_path: PathBuf = backup_dir.join(format!("database-backup_{}", timestamp));

    if let Err(e) = create_dir(&backup_path) {
        error!("Failed to create backup path {:#?}: {:#?}", backup_path, e);
        return Err(format!("Failed to create backup path: {}", e));
    }
    info!("Created backup path: {:#?}", backup_path);

    let entries = match read_dir(&database_dir) {
        Ok(entries) => entries,
        Err(e) => {
            error!("Failed to read database directory {:#?}: {:#?}", database_dir, e);
            return Err(format!("Failed to read database directory: {}", e));
        }
    };

    for entry in entries {
        let entry = entry.map_err(|e| {
            error!("Failed to read file: {:#?}", e);
            format!("Failed to read file: {}", e)
        })?;
        let src_path = entry.path();
        let dest_path = backup_path.join(entry.file_name());

        copy(&src_path, &dest_path).map_err(|e| {
            error!("Failed to copy {:#?} to {:#?}: {:#?}", src_path, dest_path, e);
            format!("Failed to copy file: {}", e)
        })?;
    }
    info!("Database backup completed successfully");

    Ok(())
}