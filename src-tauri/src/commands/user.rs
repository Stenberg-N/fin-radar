use serde::{Deserialize, Serialize};
use sqlx::{query_as, FromRow, SqlitePool};
use tauri::State;
use argon2::{Argon2, password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash}};
use time::{OffsetDateTime, macros::{format_description}};
use log::{info, warn, error};
use super::helpers::{generate_recovery_key, validate_password};

/************************************************************************************************************************\

USER ACCOUNT COMMANDS | LOGIN, REGISTRATION, RECOVERY ETC.

\************************************************************************************************************************/

#[derive(FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub password: String,
    pub requires_password_reset: bool,
}

#[derive(FromRow)]
struct RecoveryKey {
    key_hash: String,
    is_used: bool,
}

#[tauri::command]
pub async fn create_user (
    pool: State<'_, SqlitePool>,
    name: String,
    password: String,
    confirm_password: String,
) -> Result<String, String> {
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

    if Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_err() {
        error!("Failed to verify password hash");
        return Err("Failed to create user".to_string());
    }

    let recovery_key = generate_recovery_key();
    let key_hash = Argon2::default()
        .hash_password(recovery_key.as_bytes())
        .map_err(|e| {
            error!("Recovery key's hashing failed: {:#?}", e);
            "Failed to create user".to_string()
        })?
        .to_string();
    let parsed_key_hash = PasswordHash::new(&key_hash)
        .map_err(|e| {
            error!("Recovery key hash parsing failed: {:#?}", e);
            "Failed to create user".to_string()
        })?;

    if Argon2::default().verify_password(recovery_key.as_bytes(), &parsed_key_hash).is_err() {
        error!("Failed to verify recovery key hash");
        return Err("Failed to create user".to_string());
    }

    let mut tx = pool.begin().await.map_err(|e| {
        error!("Failed to begin transaction to insert user and recovery key to database: {:#?}", e);
        "Database error".to_string()
    })?;

    let user_id: i64 = sqlx::query_scalar("INSERT INTO users (name, password) VALUES (?, ?) RETURNING id")
        .bind(&name)
        .bind(&password_hash)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            error!("Database error when creating user: {:#?}", e);
            "Database error".to_string()
        })?;

    sqlx::query("INSERT INTO recovery_keys (user_id, key_hash) VALUES (?, ?)")
        .bind(user_id)
        .bind(&key_hash)
        .execute(&mut *tx)
        .await.map_err(|e| {
            error!("Database error when inserting recovery key: {:#?}", e);
            "Database error".to_string()
        })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction to insert user and recovery key to database: {:#?}", e);
        "Database error".to_string()
    })?;

    let timestamp = OffsetDateTime::now_local()
        .ok()
        .and_then(|dt| dt.format(&format_description!("[year]-[month]-[day]__at__[hour]H-[minute]M-[second]S")).ok())
        .unwrap_or("Unknown time".to_string());

    info!("User '{}' created successfully at {}", name, timestamp);

    Ok(recovery_key)
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
    current_password: Option<String>,
    new_password: String,
    confirm_new_password: String,
) -> Result<bool, String> {
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

    let parsed_hash = PasswordHash::new(&user.password).map_err(|e| {
        error!("Failed to parse hash from user's password: {:#?}", e);
        "Password update failed".to_string()
    })?;

    if let Some(ref current_password) = current_password {
        match Argon2::default().verify_password(current_password.as_bytes(), &parsed_hash) {
            Ok(_) => info!("PASSWORD CHANGE: User's '{}' given password matched the account's current password", name),
            Err(_) => {
                warn!("PASSWORD CHANGE FAILED: User's '{}' given password did not match with the account's current password!", name);
                return Err("Updating password failed".to_string());
            }
        }
    } else {
        info!("No current password provided. Assuming account recovery for user '{}'", name);
    }

    if Argon2::default().verify_password(new_password.as_bytes(), &parsed_hash).is_err() {
        error!("PASSWORD CHANGE FAILED: User '{}' attempted to reuse the current password", name);
        return Err("Password update failed".to_string());
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

    if Argon2::default().verify_password(new_password.as_bytes(), &new_parsed_hash).is_err() {
        error!("Failed to verify new password hash");
        return Err("Password update failed".to_string());
    }

    let mut tx = pool.begin().await.map_err(|e| {
        error!("Failed to begin transaction to update user's '{}' password: {:#?}", name, e);
        "Database error".to_string()
    })?;

    if user.requires_password_reset {
        sqlx::query("UPDATE recovery_keys SET is_used = 1 WHERE user_id = ?")
            .bind(&user.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to set recovery key to used in database: {:#?}", e);
                "An error occurred".to_string()
            })?;

        info!("ACCOUNT RECOVERY KEY USED: Account '{}' recovery key was used", name);
    }

    let requires_reset: bool = sqlx::query_scalar("UPDATE users SET password = ?, requires_password_reset = 0 WHERE id = ? RETURNING requires_password_reset")
        .bind(&new_password_hash)
        .bind(&id)
        .fetch_one(&mut *tx)
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

    Ok(requires_reset)
}

#[tauri::command]
pub async fn cancel_password_recovery (
    pool: State<'_, SqlitePool>,
    id: i64,
    name: String,
) -> Result<(), String> {
    sqlx::query("UPDATE users SET requires_password_reset = 0 WHERE id = ?")
        .bind(&id)
        .execute(&*pool)
        .await
        .map_err(|e| {
            error!("Failed to cancel recovery for user id: {}: {:#?}", id, e);
            "Database error".to_string()
        })?;

    let timestamp = OffsetDateTime::now_local()
        .ok()
        .and_then(|dt| dt.format(&format_description!("[year]-[month]-[day]__at__[hour]H-[minute]M-[second]S")).ok())
        .unwrap_or("Unknown time".to_string());

    info!("Account recovery cancelled for user '{}' at {}", name, timestamp);

    Ok(())
}

#[tauri::command]
pub async fn recover_password (
    pool: State<'_, SqlitePool>,
    name: String,
    recovery_key: String,
) -> Result<User, String> {
    if name.is_empty() || recovery_key.is_empty() {
        error!("ACCOUNT RECOVERY FAILED: An account was tried to be recovered but failed due to account name or recovery key missing");
        return Err("An error occurred".to_string());
    }

    let user = query_as::<_, User>("SELECT * FROM users WHERE name = ?")
        .bind(&name)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| {
            error!("Failed to find user from database: {:#?}", e);
            "An error occurred".to_string()
        })?;
    let user = user.ok_or("An error occurred")?;

    let key = query_as::<_, RecoveryKey>("SELECT key_hash, is_used FROM recovery_keys WHERE user_id = ?")
        .bind(&user.id)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch user's recovery key from database: {:#?}", e);
            "An error occurred".to_string()
        })?;
    let key = key.ok_or("An error occurred")?;

    if key.is_used {
        error!("ACCOUNT RECOVERY FAILED: Key already used for user '{}'", name);
        return Err("An error occurred".to_string());
    }

    let parsed_key_hash = PasswordHash::new(&key.key_hash)
        .map_err(|e| {
            error!("Failed to parse key's hash: {:#?}", e);
            "An error occurred".to_string()
        })?;

    match Argon2::default().verify_password(recovery_key.as_bytes(), &parsed_key_hash) {
        Ok(_) => {
            info!("ACCOUNT RECOVERY KEY MATCHED: The given key matched account's '{}' recovery key", name);

            let mut tx = pool.begin().await.map_err(|e| {
                error!("Failed to begin transaction to prepare user for password reset: {:#?}", e);
                "An error occurred".to_string()
            })?;

            let updated_user = query_as::<_, User>("UPDATE users SET requires_password_reset = 1 WHERE id = ? RETURNING *")
                .bind(&user.id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| {
                    error!("Failed to update user to require password reset: {:#?}", e);
                    "An error occurred".to_string()
                })?;

            tx.commit().await.map_err(|e| {
                error!("Failed to commit transaction to prepare user for password reset: {:#?}", e);
                "An error occurred".to_string()
            })?;

            let timestamp = OffsetDateTime::now_local()
                .ok()
                .and_then(|dt| dt.format(&format_description!("[year]-[month]-[day]__at__[hour]H-[minute]M-[second]S")).ok())
                .unwrap_or("Unknown time".to_string());

            info!("Account '{}' successfully set into password recovery mode at {}", name, timestamp);

            Ok(updated_user)
        },
        Err(_) => {
            warn!("ACCOUNT RECOVERY FAILED: The given key did not match the account's '{}' recovery key", name);
            return Err("An error occurred".to_string());
        }
    }
}