use sqlx::query_as;
use tauri::State;
use argon2::{password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash}};
use log::{info, warn, error};
use rand::{distr::Alphanumeric, distr::SampleString, rng};

use crate::{
    AppState, models::{
        recovery_key::RecoveryKey, user::{SafeUser, User}
    }, services::{
        crypto::{KdfParams, SALT_LEN, WRAPPED_DEK_LEN, derive_kek, generate_dek, generate_salt, unwrap_dek, wrap_dek}, session::{ExpiresIn, SessionData}
    }, utils::time::create_timestamp,
};

/************************************************************************************************************************\

USER ACCOUNT COMMANDS | LOGIN, REGISTRATION, RECOVERY ETC.

\************************************************************************************************************************/

fn validate_password(pw: &str) -> bool {
    let has_min_length = pw.chars().count() >= 10;
    let no_spaces = !pw.chars().any(|c| c.is_whitespace());
    let has_numbers = pw.chars().any(|c| c.is_ascii_digit());
    let has_uppercase = pw.chars().any(|c| c.is_uppercase() && c.is_alphabetic());
    let has_lowercase = pw.chars().any(|c| c.is_lowercase() && c.is_alphabetic());
    let has_special_char = pw.chars().any(|c| r#"=_!@#$€£¤%^&*(){}[],.?'":|/\+-<>~§"#.contains(c));

    has_min_length && no_spaces && has_numbers && has_uppercase && has_lowercase && has_special_char
}

fn generate_recovery_key() -> String {
    Alphanumeric.sample_string(&mut rng(), 48)
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

    let state: &AppState = &state;

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

    let params = KdfParams::DEFAULT;
    let argon2 = params.argon2().map_err(|e| {
        error!("Argon2 initialization failed during account creation: {:#?}", e);
        "An error occurred".to_string()
    })?;

    let password_hash = argon2
        .hash_password(password.as_bytes())
        .map_err(|e| {
            error!("Hashing failed: {:#?}", e);
            "Failed to create user".to_string()
        })?;

    let recovery_key = generate_recovery_key();
    let key_hash = argon2
        .hash_password(recovery_key.as_bytes())
        .map_err(|e| {
            error!("Recovery key's hashing failed: {:#?}", e);
            "Failed to create user".to_string()
        })?;

    let dek = generate_dek();

    let recovery_kek_salt = generate_salt();
    let recovery_kek = derive_kek(recovery_key.as_bytes(), &recovery_kek_salt, &params)?;
    let wrapped_dek_recovery = wrap_dek(&recovery_kek, &dek)?;

    let kek_salt = generate_salt();
    let kek = derive_kek(password.as_bytes(), &kek_salt, &params)?;
    let wrapped_dek = wrap_dek(&kek, &dek)?;

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
        .await
        .map_err(|e| {
            error!("Database error when inserting recovery key: {:#?}", e);
            "Database error".to_string()
        })?;

    sqlx::query("INSERT INTO user_crypto (user_id, wrapped_dek, wrapped_dek_recovery, kek_salt, recovery_kek_salt, kdf_params) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(user_id)
        .bind(wrapped_dek.as_slice())
        .bind(wrapped_dek_recovery.as_slice())
        .bind(kek_salt.as_slice())
        .bind(recovery_kek_salt.as_slice())
        .bind(params.to_bytes().as_slice())
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("Database error during crypto insertion: {:#?}", e);
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

    let state: &AppState = &state;

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
        .map_err(|_| "Invalid login information".to_string())?;

    let p = sqlx::query_scalar::<_, Vec<u8>>("SELECT kdf_params FROM user_crypto WHERE user_id = ?")
        .bind(user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch KDF params for user '{}': {:#?}", user.name, e);
            "Database error".to_string()
        })?;
    let params = KdfParams::from_bytes(p.try_into().map_err(|_| "KDF params corrupt")?);
    let argon2 = params.argon2().map_err(|e| {
        error!("Argon2 initialization failed during login: {:#?}", e);
        "An error occurred".to_string()
    })?;

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => {
            let (kek_salt, wrapped_dek) = query_as::<_, (Vec<u8>, Vec<u8>)>("SELECT kek_salt, wrapped_dek FROM user_crypto WHERE user_id = ?")
                .bind(user.id)
                .fetch_one(&state.db)
                .await
                .map_err(|e| {
                    error!("Failed to get KEK salt and DEK: {:#?}", e);
                    "Database error".to_string()
                })?;
            let (parsed_salt, parsed_dek): ([u8; SALT_LEN], [u8; WRAPPED_DEK_LEN]) = (
                kek_salt.try_into().map_err(|_| {
                    error!("KEK salt is corrupt");
                    "An error occurred".to_string()
                })?,
                wrapped_dek.try_into().map_err(|_| {
                    error!("DEK is corrupt");
                    "An error occurred".to_string()
                })?,
            );

            let kek = derive_kek(password.as_bytes(), &parsed_salt, &params)?;
            let dek = unwrap_dek(&kek, &parsed_dek)?;

            let safe_user = SafeUser::from(user);
            state.session.set_session(SessionData::new(safe_user.clone(), Some(dek))).map_err(|e| {
                error!("LOGIN FAILED ({}): Failed to set session for user '{}': {:#?}", create_timestamp(), name, e);
                "An error occurred".to_string()
            })?;

            info!("LOGIN SUCCESS ({}): User '{}' successfully logged in", create_timestamp(), name);

            Ok(safe_user)
        }
        Err(_) => {
            warn!("LOGIN FAILED ({}): Incorrect password for user '{}'", create_timestamp(), name);
            Err("Invalid login information".to_string())
        }
    }
}

#[tauri::command]
pub async fn logout_user(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let state: &AppState = &state;

    state.session.clear_session(false).map_err(|e| {
        error!("LOGOUT FAILED ({}): Failed to clear session: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    Ok(())
}

#[tauri::command]
pub async fn update_user_session(
    state: State<'_, AppState>,
    expires_in: ExpiresIn,
) -> Result<(), String> {
    let state: &AppState = &state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("SESSION UPDATE FAILED ({}): Failed to get session: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    state.session.update_session(expires_in).map_err(|e| {
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
    let state: &AppState = &state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("ACCOUNT DELETION FAILED ({}): {:#?}", create_timestamp(), e);
        "Failed to delete user".to_string()
    })?;

    info!("ACCOUNT DELETION ({}): Initiated for user '{}'", create_timestamp(), session.user.name);

    let user_password: String = sqlx::query_scalar("SELECT password FROM users WHERE id = ?")
        .bind(session.user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to get user's '{}' password: {:#?}", session.user.name, e);
            "Failed to delete user".to_string()
        })?;

    let params = sqlx::query_scalar::<_, Vec<u8>>("SELECT kdf_params FROM user_crypto WHERE user_id = ?")
        .bind(session.user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to get user's '{}' KDF params: {:#?}", session.user.name, e);
            "Database error".to_string()
        })?;
    let argon2 = KdfParams::from_bytes(params.try_into().map_err(|_| "Corrupt KDF params")?).argon2().map_err(|e| {
        error!("Argon2 initialization failed during account deletion: {:#?}", e);
        "An error occurred".to_string()
    })?;

    let parsed_password_hash = PasswordHash::new(&user_password)
        .map_err(|e| {
            error!("Failed to parse user's '{}' password: {:#?}", session.user.name, e);
            "Failed to delete user".to_string()
        })?;

    match argon2.verify_password(password.as_bytes(), &parsed_password_hash) {
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

            state.session.clear_session(false).map_err(|e| {
                error!("SESSION CLEAR FAILED ({}): Failed to clear session on user deletion: {:#?}", create_timestamp(), e);
                "An error occurred".to_string()
            })?;

            info!("ACCOUNT DELETION SUCCESSFUL ({}): User '{}' with ID: '{}' deleted successfully", create_timestamp(), session.user.name, session.user.id);

            Ok(())
        },
        Err(_) => {
            warn!("ACCOUNT DELETION FAILED ({}): Incorrect password for user '{}'", create_timestamp(), session.user.name);
            Err("Failed to delete user".to_string())
        }
    }
}

#[tauri::command]
pub async fn change_password(
    state: State<'_, AppState>,
    new_password: String,
    confirm_new_password: String,
    current_password: Option<String>,
    name: Option<String>,
    recovery_key: Option<String>,
) -> Result<String, String> {
    if new_password != confirm_new_password {
        warn!("PASSWORD CHANGE FAILED ({}): Password mismatch", create_timestamp());
        return Err("Password mismatch".to_string());
    }

    if !validate_password(&new_password) {
        warn!("PASSWORD CHANGE FAILED ({}): Password requirements not being met", create_timestamp());
        return Err("Password requirements not met".to_string());
    }

    let state: &AppState = &state;
    let now = create_timestamp();

    if let (Some(ref name), Some(ref recovery_key)) = (name, recovery_key) {
        let user = query_as::<_, User>("SELECT * FROM users WHERE name = ?")
            .bind(name)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| {
                error!("Failed to fetch user: {:#?}", e);
                "Database error".to_string()
            })?
            .ok_or_else(|| "No user found".to_string())?;

        let key = query_as::<_, RecoveryKey>("SELECT is_used, key_hash FROM recovery_keys WHERE user_id = ?")
            .bind(user.id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| {
                error!("Failed to fetch user's '{}' recovery key: {:#?}", user.name, e);
                "Database error".to_string()
            })?;

        if key.is_used {
            error!("User's '{}' recovery key is already used", user.name);
            return Err("An error occurred".to_string());
        }

        let key_hash = PasswordHash::new(&key.key_hash).map_err(|e| {
            error!("Failed to parse key hash: {:#?}", e);
            "An error occurred".to_string()
        })?;

        let p = sqlx::query_scalar::<_, Vec<u8>>("SELECT kdf_params FROM user_crypto WHERE user_id = ?")
            .bind(user.id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| {
                error!("Failed to fetch KDF params for user '{}': {:#?}", user.name, e);
                "Database error".to_string()
            })?;
        let params = KdfParams::from_bytes(p.try_into().map_err(|_| "KDF params corrupt")?);
        let argon2 = params.argon2().map_err(|e| {
            error!("Argon2 initialization failed during password change: {:#?}", e);
            "An error occurred".to_string()
        })?;

        let current_pw_hash = PasswordHash::new(&user.password).map_err(|e| {
            error!("Failed to parse user password hash: {:#?}", e);
            "An error occurred".to_string()
        })?;

        if argon2.verify_password(new_password.as_bytes(), &current_pw_hash).is_ok() {
            warn!("PASSWORD CHANGE FAILED ({}): User '{}' attempted to reuse their current password", create_timestamp(), user.name);
            return Err("Password update failed".to_string());
        }

        match argon2.verify_password(recovery_key.as_bytes(), &key_hash) {
            Ok(_) => {
                let mut tx = state.db.begin().await.map_err(|e| {
                    error!("Failed to begin transaction during password change: {:#?}", e);
                    "Database error".to_string()
                })?;

                let (salt, dek) = query_as::<_, (Vec<u8>, Vec<u8>)>("SELECT recovery_kek_salt, wrapped_dek_recovery FROM user_crypto WHERE user_id = ?")
                    .bind(user.id)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|e| {
                        error!("Failed to fetch user crypto: {:#?}", e);
                        "Database error".to_string()
                    })?;
                let (parsed_salt, parsed_dek): ([u8; SALT_LEN], [u8; WRAPPED_DEK_LEN]) = (
                    salt.try_into().map_err(|_| "Recovery KEK salt corrupt".to_string())?,
                    dek.try_into().map_err(|_| "Recovery DEK corrupt".to_string())?,
                );

                let recovery_kek = derive_kek(recovery_key.as_bytes(), &parsed_salt, &params)?;
                let dek = unwrap_dek(&recovery_kek, &parsed_dek)?;

                let new_salt = generate_salt();
                let new_kek = derive_kek(new_password.as_bytes(), &new_salt, &params)?;
                let new_wrapped_dek = wrap_dek(&new_kek, &dek)?;

                let new_password_hash = argon2.hash_password(new_password.as_bytes()).map_err(|e| {
                    error!("Failed to hash new password: {:#?}", e);
                    "An error occurred".to_string()
                })?;

                sqlx::query("UPDATE user_crypto SET wrapped_dek = ?, kek_salt = ? WHERE user_id = ?")
                    .bind(new_wrapped_dek.as_slice())
                    .bind(new_salt.as_slice())
                    .bind(user.id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| {
                        error!("Failed to update DEK: {:#?}", e);
                        "Database error".to_string()
                    })?;

                let updated_user = query_as::<_, User>("UPDATE users SET password = ?, last_password_change = ? WHERE id = ? RETURNING *")
                    .bind(new_password_hash.to_string())
                    .bind(now)
                    .bind(user.id)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|e| {
                        error!("Failed to update user password: {:#?}", e);
                        "Database error".to_string()
                    })?;

                sqlx::query("UPDATE recovery_keys SET is_used = 1 WHERE user_id = ?")
                    .bind(user.id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| {
                        error!("Failed to set recovery key to used: {:#?}", e);
                        "Database error".to_string()
                    })?;

                tx.commit().await.map_err(|e| {
                    error!("Failed to commit transaction during password change: {:#?}", e);
                    "database error".to_string()
                })?;

                info!("PASSWORD CHANGE SUCCESSFUL ({}): User '{}' changed their password successfully", create_timestamp(), updated_user.name);

                Ok(updated_user.last_password_change)
            },
            Err(_) => {
                error!("PASSWORD CHANGE FAILED ({}): Recovery key mismatch", create_timestamp());
                return Err("An error occurred".to_string());
            }
        }
    } else if let Some(ref current_password) = current_password {
        let session: SessionData = state.session.get_session().map_err(|e| {
            error!("PASSWORD CHANGE FAILED ({}): {:#?}", create_timestamp(), e);
            "An error occurred".to_string()
        })?;

        let user = query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(session.user.id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| {
                error!("Failed to fetch user: {:#?}", e);
                "Database error".to_string()
            })?
            .ok_or_else(|| "An error occurred".to_string())?;

        let p = sqlx::query_scalar::<_, Vec<u8>>("SELECT kdf_params FROM user_crypto WHERE user_id = ?")
            .bind(user.id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| {
                error!("Failed to fetch KDF params for user '{}': {:#?}", user.name, e);
                "Database error".to_string()
            })?;
        let params = KdfParams::from_bytes(p.try_into().map_err(|_| "KDF params corrupt")?);
        let argon2 = params.argon2().map_err(|e| {
            error!("Argon2 initialization failed during password change: {:#?}", e);
            "An error occurred".to_string()
        })?;

        let current_pw_hash = PasswordHash::new(&user.password).map_err(|e| {
            error!("Failed to parse user password hash: {:#?}", e);
            "An error occurred".to_string()
        })?;

        if argon2.verify_password(new_password.as_bytes(), &current_pw_hash).is_ok() {
            warn!("PASSWORD CHANGE FAILED ({}): User '{}' attempted to reuse their current password", create_timestamp(), user.name);
            return Err("Password update failed".to_string());
        }

        match argon2.verify_password(current_password.as_bytes(), &current_pw_hash) {
            Ok(_) => {
                let mut tx = state.db.begin().await.map_err(|e| {
                    error!("Failed to begin transaction during password change: {:#?}", e);
                    "Database error".to_string()
                })?;

                let (salt, dek) = query_as::<_, (Vec<u8>, Vec<u8>)>("SELECT kek_salt, wrapped_dek FROM user_crypto WHERE user_id = ?")
                    .bind(user.id)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|e| {
                        error!("Failed to fetch DEK and salt: {:#?}", e);
                        "Database error".to_string()
                    })?;
                let (parsed_salt, parsed_dek): ([u8; SALT_LEN], [u8; WRAPPED_DEK_LEN]) = (
                    salt.try_into().map_err(|_| "KEK salt corrupt".to_string())?,
                    dek.try_into().map_err(|_| "DEK corrupt".to_string())?, 
                );

                let kek = derive_kek(current_password.as_bytes(), &parsed_salt, &params)?;
                let dek = unwrap_dek(&kek, &parsed_dek)?;

                let new_salt = generate_salt();
                let new_kek = derive_kek(new_password.as_bytes(), &new_salt, &params)?;
                let new_wrapped_dek = wrap_dek(&new_kek, &dek)?;

                let new_password_hash = argon2.hash_password(new_password.as_bytes()).map_err(|e| {
                    error!("Failed to hash new password: {:#?}", e);
                    "An error occurred".to_string()
                })?;

                sqlx::query("UPDATE user_crypto SET kek_salt = ?, wrapped_dek = ? WHERE user_id = ?")
                    .bind(new_salt.as_slice())
                    .bind(new_wrapped_dek.as_slice())
                    .bind(user.id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| {
                        error!("Failed to update KEK salt and DEK: {:#?}", e);
                        "Database error".to_string()
                    })?;

                let updated_user = query_as::<_, User>("UPDATE users SET password = ?, last_password_change = ? WHERE id = ? RETURNING *")
                    .bind(new_password_hash.to_string())
                    .bind(now)
                    .bind(user.id)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|e| {
                        error!("Failed to update user password: {:#?}", e);
                        "Database error".to_string()
                    })?;

                state.session.update_user_in_session(SafeUser::from(updated_user.clone())).map_err(|e| {
                    error!("UPDATE USER IN SESSION FAILED ({}): Failed to set updated user '{}' into session: {:#?}", create_timestamp(), updated_user.name, e);
                    "Failed to update password".to_string()
                })?;

                tx.commit().await.map_err(|e| {
                    error!("Failed to commit transaction during password change: {:#?}", e);
                    "database error".to_string()
                })?;

                info!("PASSWORD CHANGE SUCCESSFUL ({}): User '{}' changed their password successfully", create_timestamp(), updated_user.name);

                Ok(updated_user.last_password_change)
            },
            Err(_) => {
                error!("PASSWORD CHANGE FAILED ({}): Current password invalid", create_timestamp());
                return Err("An error occurred".to_string());
            }
        }
    } else {
        error!("None of the required fields were provided for password change");
        return Err("An error occurred".to_string());
    }
}

#[tauri::command]
pub async fn query_is_recovery_key_used(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let state: &AppState = &state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("ACCOUNT RECOVERY KEY QUERY ({}): Could not fetch the status of recovery key: {:#?}", create_timestamp(), e);
        "An error occurred".to_string() 
    })?;

    let recovery_key_status = sqlx::query_scalar::<_, bool>("SELECT is_used FROM recovery_keys WHERE user_id = ?")
        .bind(session.user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch the status of recovery key: {:#?}", e);
            "An error occurred".to_string()
        })?;

    Ok(recovery_key_status)
}

#[tauri::command]
pub async fn update_username(
    state: State<'_, AppState>,
    new_username: String,
) -> Result<String, String> {
    let state: &AppState = &state;

    let session = state.session.get_session().map_err(|e| {
        error!("USERNAME UPDATE FAILED ({}): Could not update username due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let cleaned_name = ammonia::clean(&new_username);

    let existing_user = sqlx::query_scalar::<_, String>("SELECT name FROM users WHERE name = ?")
        .bind(&cleaned_name)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch existing user: {:#?}", e);
            "An error occurred".to_string()
        })?;

    if existing_user.is_some() {
        warn!("USERNAME UPDATE FAILED ({}): Username already taken", create_timestamp());
        return Err("An error occurred".to_string())
    }

    if session.user.name == cleaned_name {
        warn!("USERNAME UPDATE FAILED ({}): User '{}' used their current name", create_timestamp(), session.user.name);
        return Err("An error occurred".to_string())
    }

    let updated_user = query_as::<_, User>("UPDATE users SET name = ? WHERE id = ? RETURNING *")
        .bind(cleaned_name)
        .bind(session.user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to update new username: {:#?}", e);
            "An error occurred".to_string()
        })?;

    state.session.update_user_in_session(SafeUser::from(updated_user.clone())).map_err(|e| {
        error!("UPDATE USER IN SESSION FAILED ({}): Failed to set updated user '{}' into session: {:#?}", create_timestamp(), updated_user.name, e);
        "An error occurred".to_string()
    })?;

    info!("USERNAME UPDATE SUCCESSFUL ({}): User '{}' updated their username to '{}'", create_timestamp(), session.user.name, updated_user.name);
    
    Ok(updated_user.name)
}