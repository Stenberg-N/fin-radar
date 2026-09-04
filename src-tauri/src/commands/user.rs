use serde::{Serialize};
use sqlx::{query_as, FromRow};
use tauri::State;
use argon2::{password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash}};
use log::{info, warn, error};

use crate::{AppState, commands::helpers::check_user_capabilities};
use super::helpers::{generate_recovery_key, validate_password, create_timestamp};
use crate::structs::session::SessionData;

/************************************************************************************************************************\

USER ACCOUNT COMMANDS | LOGIN, REGISTRATION, RECOVERY ETC.

\************************************************************************************************************************/

#[derive(FromRow, Clone)]
struct User {
    id: i64,
    name: String,
    password: String,
    requires_password_reset: bool,
}

#[derive(Serialize, Clone)]
pub struct SafeUser {
    pub id: i64,
    pub name: String,
    pub requires_password_reset: bool,
}

impl From<User> for SafeUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user.name,
            requires_password_reset: user.requires_password_reset
        }
    }
}

#[derive(FromRow)]
struct RecoveryKey {
    key_hash: String,
    is_used: bool,
}

#[tauri::command]
pub async fn create_user(
    state: State<'_, AppState>,
    name: String,
    password: String,
    confirm_password: String,
) -> Result<String, String> {
    if password != confirm_password {
        warn!("ACCOUNT CREATION FAILED ({}): Password mismatch", create_timestamp());
        return Err("Password mismatch".to_string());
    }
    if !validate_password(&password) {
        warn!("ACCOUNT CREATION FAILED ({}): Password requirements not being met", create_timestamp());
        return Err("Password requirements not met".to_string());
    }

    let state: &AppState = &*state;

    let existing_user = query_as::<_, User>(
        "SELECT * FROM users WHERE name = ?"
    )
    .bind(&name)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to check existing user at {}: {:#?}", create_timestamp(), e);
        "Database error".to_string()
    })?;

    if existing_user.is_some() {
        warn!("ACCOUNT CREATION FAILED ({}): Username already taken", create_timestamp());
        return Err("User with this name already exists".to_string());
    }

    let password_hash = state.argon2
        .hash_password(password.as_bytes())
        .map_err(|e| {
            error!("Hashing failed: {:#?}", e);
            "Failed to create user".to_string()
        })?;

    let recovery_key = generate_recovery_key();
    let key_hash = state.argon2
        .hash_password(recovery_key.as_bytes())
        .map_err(|e| {
            error!("Recovery key's hashing failed: {:#?}", e);
            "Failed to create user".to_string()
        })?;

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction to insert user and recovery key to database: {:#?}", e);
        "Database error".to_string()
    })?;

    let user_id: i64 = sqlx::query_scalar("INSERT INTO users (name, password) VALUES (?, ?) RETURNING id")
        .bind(&name)
        .bind(password_hash.to_string())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            error!("Database error when creating user: {:#?}", e);
            "Database error".to_string()
        })?;

    sqlx::query("INSERT INTO recovery_keys (user_id, key_hash) VALUES (?, ?)")
        .bind(user_id)
        .bind(key_hash.to_string())
        .execute(&mut *tx)
        .await.map_err(|e| {
            error!("Database error when inserting recovery key: {:#?}", e);
            "Database error".to_string()
        })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction to insert user and recovery key to database: {:#?}", e);
        "Database error".to_string()
    })?;

    info!("ACCOUNT CREATION SUCCESSFUL ({}): User '{}' created successfully", create_timestamp(), name);

    Ok(recovery_key)
}

#[tauri::command]
pub async fn login_user(
    state: State<'_, AppState>,
    name: String,
    password: String,
) -> Result<SafeUser, String> {
    info!("LOGIN ATTEMPT ({}): Initiated for user {}", create_timestamp(), name);

    let state: &AppState = &*state;

    let user = query_as::<_, User>("SELECT * FROM users WHERE name = ?")
        .bind(&name)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            error!("Database error when fetching user '{}' {:#?}", name, e);
            "Database error".to_string()
        })?
        .ok_or_else(|| {
            warn!("LOGIN FAILED ({}): User '{}' does not exist", create_timestamp(), name);
            "Invalid login information".to_string()
        })?;

    let parsed_hash = PasswordHash::new(&user.password)
        .map_err(|_| {
            "Invalid login information".to_string()
        })?;

    match state.argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => {
            let safe_user = SafeUser::from(user);
            state.session.set_session(SessionData::new(safe_user.clone())).map_err(|e| {
                error!("LOGIN FAILED ({}): Failed to set session for user '{}': {:#?}", create_timestamp(), name, e);
                "An error occurred".to_string()
            })?;

            info!("LOGIN SUCCESS ({}): User '{}' successfully logged in", create_timestamp(), name);

            Ok(safe_user)
        }
        Err(_) => {
            warn!("LOGIN FAILED ({}): Incorrect password for user '{}'", create_timestamp(), name);
            return Err("Invalid login information".to_string());
        }
    }
}

#[tauri::command]
pub async fn logout_user(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let state: &AppState = &*state;

    state.session.clear_session().map_err(|e| {
        error!("LOGOUT FAILED ({}): Failed to clear session: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    Ok(())
}

#[tauri::command]
pub async fn update_user_session(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let state: &AppState = &*state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("SESSION UPDATE FAILED ({}): Failed to get session: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    state.session.update_session().map_err(|e| {
        error!("SESSION UPDATE FAILED ({}): Failed to update session for user '{}': {:#?}", create_timestamp(), session.user.name, e);
        "An error occurred".to_string()
    })?;

    Ok(())
}

#[tauri::command]
pub async fn delete_user(
    state: State<'_, AppState>,
    password: String,
) -> Result<(), String> {
    let state: &AppState = &*state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("ACCOUNT DELETION FAILED ({}): {:#?}", create_timestamp(), e);
        "Failed to delete user".to_string()
    })?;

    info!("ACCOUNT DELETION ({}): Initiated for user '{}'", create_timestamp(), session.user.name);

    check_user_capabilities(&session.user, "delete_user")?;

    let user_password: String = sqlx::query_scalar("SELECT password FROM users WHERE id = ?")
        .bind(session.user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to get user's '{}' password: {:#?}", session.user.name, e);
            "Failed to delete user".to_string()
        })?;

    let parsed_password_hash = PasswordHash::new(&user_password)
        .map_err(|e| {
            error!("Failed to parse user's '{}' password: {:#?}", session.user.name, e);
            "Failed to delete user".to_string()
        })?;

    match state.argon2.verify_password(password.as_bytes(), &parsed_password_hash) {
        Ok(_) => {
            let mut tx = state.db.begin().await.map_err(|e| {
                error!("Failed to begin transaction to delete user with ID: {}: {:#?}", session.user.id, e);
                "Database error".to_string()
            })?;

            let result = sqlx::query("DELETE FROM users WHERE id = ?")
                .bind(session.user.id)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    error!("Failed to delete user with ID: '{}': {:#?}", session.user.id, e);
                    "Failed to delete user".to_string()
                })?;
            if result.rows_affected() == 0 {
                warn!("ACCOUNT DELETION FAILED ({}): User '{}' with ID: '{}' could not be found", create_timestamp(), session.user.name, session.user.id);
                return Err("Failed to delete user".to_string());
            }

            tx.commit().await.map_err(|e| {
                error!("Failed to commit transaction to delete user with ID: '{}': {:#?}", session.user.id, e);
                "Database error".to_string()
            })?;

            state.session.clear_session().map_err(|e| {
                error!("SESSION CLEAR FAILED ({}): Failed to clear session on user deletion: {:#?}", create_timestamp(), e);
                "An error occurred".to_string()
            })?;

            info!("ACCOUNT DELETION SUCCESSFUL ({}): User '{}' with ID: '{}' deleted successfully", create_timestamp(), session.user.name, session.user.id);

            return Ok(())
        },
        Err(_) => {
            warn!("ACCOUNT DELETION FAILED ({}): Incorrect password for user '{}'", create_timestamp(), session.user.name);
            return Err("Failed to delete user".to_string());
        }
    }
}

#[tauri::command]
pub async fn change_password(
    state: State<'_, AppState>,
    current_password: Option<String>,
    new_password: String,
    confirm_new_password: String,
) -> Result<(), String> {
    let state: &AppState = &*state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("PASSWORD CHANGE FAILED ({}): {:#?}", create_timestamp(), e);
        "Password update failed".to_string()
    })?;

    if new_password != confirm_new_password {
        warn!("PASSWORD CHANGE FAILED ({}): For user '{}' due to new and confirmation passwords being mismatched", create_timestamp(), session.user.name);
        return Err("Password mismatch".to_string());
    }
    if !validate_password(&new_password) {
        warn!("PASSWORD CHANGE FAILED ({}): For user '{}' due to password requirements not being met", create_timestamp(), session.user.name);
        return Err("Password requirements not met".to_string());
    }

    let user = query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(session.user.id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to get user '{}' from database: {:#?}", session.user.name, e);
            "Failed to get user from database".to_string()
        })?
        .ok_or_else(|| {
            warn!("PASSWORD CHANGE FAILED ({}): Could not find user '{}' with ID: '{}'", create_timestamp(), session.user.name, session.user.id);
            "Invalid user information".to_string()
        })?;

    let parsed_hash = PasswordHash::new(&user.password).map_err(|e| {
        error!("Failed to parse hash from user's password: {:#?}", e);
        "Password update failed".to_string()
    })?;

    if let Some(ref current_password) = current_password {
        if user.requires_password_reset {
            error!("PASSWORD CHANGE FAILED ({}): User '{}' tried changing their password using their current password while in recovery mode", create_timestamp(), session.user.name);
            return Err("Updating password failed".to_string());
        }

        match state.argon2.verify_password(current_password.as_bytes(), &parsed_hash) {
            Ok(_) => info!("PASSWORD CHANGE ({}): User's '{}' given password matched the account's current password", create_timestamp(), user.name),
            Err(_) => {
                warn!("PASSWORD CHANGE FAILED ({}): User's '{}' given password did not match with the account's current password", create_timestamp(), user.name);
                return Err("Updating password failed".to_string());
            }
        }
    } else {
        if !user.requires_password_reset {
            warn!("PASSWORD CHANGE FAILED ({}): No current password provided for non-recovery user '{}'", create_timestamp(), user.name);
            return Err("Updating password failed".to_string());
        }
        info!("PASSWORD CHANGE ({}): No current password provided. Assuming account recovery for user '{}'", create_timestamp(), user.name);
    }

    if state.argon2.verify_password(new_password.as_bytes(), &parsed_hash).is_ok() {
        warn!("PASSWORD CHANGE FAILED ({}): User '{}' attempted to reuse their current password", create_timestamp(), user.name);
        return Err("Password update failed".to_string());
    }

    let new_password_hash = state.argon2
        .hash_password(new_password.as_bytes())
        .map_err(|e| {
            error!("Failed to create hash for new password: {:#?}", e);
            "Password update failed".to_string()
        })?;

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction to update user's '{}' password: {:#?}", user.name, e);
        "Database error".to_string()
    })?;

    if user.requires_password_reset {
        sqlx::query("UPDATE recovery_keys SET is_used = 1 WHERE user_id = ?")
            .bind(user.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to set recovery key to used in database: {:#?}", e);
                "An error occurred".to_string()
            })?;

        info!("ACCOUNT RECOVERY KEY USED ({}): Account '{}' recovery key was used", create_timestamp(), user.name);
    }

    let updated_user = sqlx::query_as::<_, User>("UPDATE users SET password = ?, requires_password_reset = 0 WHERE id = ? RETURNING *")
        .bind(new_password_hash.to_string())
        .bind(user.id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            error!("PASSWORD UPDATE FAILED ({}): Failed to update user's '{}' password into database: {:#?}", create_timestamp(), user.name, e);
            "Failed to update password".to_string()
        })?;

    if user.requires_password_reset {
        state.session.update_user_in_session(SafeUser::from(updated_user.clone())).map_err(|e| {
            error!("UPDATE USER IN SESSION FAILED ({}): Failed to set updated user '{}' into session: {:#?}", create_timestamp(), updated_user.name, e);
            "Failed to update password".to_string()
        })?;
    }

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction to update user's '{}' password: {:#?}", updated_user.name, e);
        "Database error".to_string()
    })?;

    info!("PASSWORD CHANGE SUCCESSFUL ({}): User '{}' changed their password successfully", create_timestamp(), updated_user.name);

    Ok(())
}

#[tauri::command]
pub async fn cancel_password_recovery(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let state: &AppState = &*state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("ACCOUNT RECOVERY CANCELLATION FAILED ({}): {:#?}", create_timestamp(), e);
        "Failed to cancel recovery".to_string()
    })?;

    sqlx::query("UPDATE users SET requires_password_reset = 0 WHERE id = ?")
        .bind(session.user.id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to cancel recovery for user ID: '{}': {:#?}", session.user.id, e);
            "Database error".to_string()
        })?;

    state.session.clear_session().map_err(|e| {
        error!("SESSION CLEAR FAILED ({}): Failed to clear session on user deletion: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    info!("ACCOUNT RECOVERY CANCELLED ({}): Account recovery successfully cancelled for user '{}'", create_timestamp(), session.user.name);

    Ok(())
}

#[tauri::command]
pub async fn recover_password(
    state: State<'_, AppState>,
    name: String,
    recovery_key: String,
) -> Result<SafeUser, String> {
    if name.trim().is_empty() || recovery_key.trim().is_empty() {
        warn!("ACCOUNT RECOVERY FAILED ({}): Missing name or recovery key", create_timestamp());
        return Err("An error occurred".to_string());
    }

    let state: &AppState = &*state;

    let user = query_as::<_, User>("SELECT * FROM users WHERE name = ?")
        .bind(&name)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to find user from database: {:#?}", e);
            "An error occurred".to_string()
        })?
        .ok_or_else(|| {
            warn!("ACCOUNT RECOVERY FAILED ({}): User '{}' could not be found from database", create_timestamp(), name);
            "An error occurred".to_string()
        })?;

    let key = query_as::<_, RecoveryKey>("SELECT key_hash, is_used FROM recovery_keys WHERE user_id = ?")
        .bind(user.id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch user's recovery key from database: {:#?}", e);
            "An error occurred".to_string()
        })?
        .ok_or_else(|| {
            warn!("ACCOUNT RECOVERY FAILED ({}): Recovery key for user '{}' could not be found", create_timestamp(), name);
            "An error occurred".to_string()
        })?;

    if key.is_used {
        warn!("ACCOUNT RECOVERY FAILED ({}): Key already used for user '{}'", create_timestamp(), name);
        return Err("An error occurred".to_string());
    }

    let parsed_key_hash = PasswordHash::new(&key.key_hash)
        .map_err(|e| {
            error!("Failed to parse key's hash: {:#?}", e);
            "An error occurred".to_string()
        })?;

    match state.argon2.verify_password(recovery_key.as_bytes(), &parsed_key_hash) {
        Ok(_) => {
            info!("ACCOUNT RECOVERY KEY MATCHED ({}): The given key matched account's '{}' recovery key", create_timestamp(), name);

            if user.requires_password_reset {
                info!("ACCOUNT RECOVERY ({}): User '{}' already in account recovery mode. Skipping updating reset state.", create_timestamp(), name);

                let safe_user = SafeUser::from(user);
                state.session.set_session(SessionData::new(safe_user.clone())).map_err(|e| {
                    error!("ACCOUNT RECOVERY FAILED ({}): Failed to set session for user '{}': {:#?}", create_timestamp(), name, e);
                    "An error occurred".to_string()
                })?;

                Ok(safe_user)
            } else {
                let mut tx = state.db.begin().await.map_err(|e| {
                    error!("Failed to begin transaction to prepare user for password reset: {:#?}", e);
                    "An error occurred".to_string()
                })?;

                let updated_user = query_as::<_, User>("UPDATE users SET requires_password_reset = 1 WHERE id = ? RETURNING *")
                    .bind(user.id)
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

                info!("ACCOUNT RECOVERY ({}): Account '{}' successfully set into password recovery mode", create_timestamp(), name);
                
                let safe_user = SafeUser::from(updated_user);
                state.session.set_session(SessionData::new(safe_user.clone())).map_err(|e| {
                    error!("ACCOUNT RECOVERY FAILED ({}): Failed to set session for user '{}': {:#?}", create_timestamp(), name, e);
                    "An error occurred".to_string()
                })?;

                Ok(safe_user)
            }
        },
        Err(_) => {
            warn!("ACCOUNT RECOVERY FAILED ({}): Given key did not match user's '{}' key", create_timestamp(), name);
            return Err("An error occurred".to_string());
        }
    }
}