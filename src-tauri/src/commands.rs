use serde::{Deserialize, Serialize};
use sqlx::{query_as, FromRow, SqlitePool, Row};
use tauri::State;
use argon2::{Argon2, password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash}};
use dirs::data_local_dir;
use std::{collections::HashSet, fs::{copy, create_dir, read_dir}};
use std::path::PathBuf;
use std::io::ErrorKind;
use time::{Date, OffsetDateTime, macros::{format_description}};
use log::{info, warn, error, debug};
use rand::{distr::Alphanumeric, distr::SampleString, rng};

#[derive(FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub password: String,
    pub requires_password_reset: bool,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i64,
    pub user_id: i64,
    pub category: String,
    pub date: String,
    pub description: String,
    pub amount: f64,
    pub _type: String,
}

#[derive(FromRow)]
struct RecoveryKey {
    key_hash: String,
    is_used: bool,
}

/*
**********************************************************************************************************************************

HELPER FUNCTIONS

**********************************************************************************************************************************
*/

fn validate_password(pw: &str) -> bool {
    let has_min_length = pw.chars().count() >= 10;
    let no_spaces = !pw.chars().any(|c| c.is_whitespace());
    let has_numbers = pw.chars().any(|c| c.is_ascii_digit());
    let has_uppercase = pw.chars().any(|c| c.is_uppercase() && c.is_alphabetic());
    let has_lowercase = pw.chars().any(|c| c.is_lowercase() && c.is_alphabetic());
    let has_special_char = pw.chars().any(|c| r#"=_!@#$€£¤%^&*(){}[],.?'":|/\+-<>~§"#.contains(c));

    has_min_length && no_spaces && has_numbers && has_uppercase && has_lowercase && has_special_char
}

fn generate_recovery_key () -> String {
    Alphanumeric.sample_string(&mut rng(), 48)
}

fn valid_categories () -> HashSet<&'static str> {
    vec![
        "rent", "taxes", "groceries", "utilities", "transportation", "travel", "entertainment", "healthcare",
        "insurance", "subscription", "education", "other", "salary", "freelance", "investments",
    ]
    .into_iter()
    .collect()
}

fn valid_transaction_types () -> HashSet<&'static str> {
    vec!["income", "expense"]
    .into_iter()
    .collect()
}

/*
**********************************************************************************************************************************

ACCOUNT CREATION, LOGIN, AUTHENTICATION

**********************************************************************************************************************************
*/

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
    assert!(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok());

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
    assert!(Argon2::default().verify_password(recovery_key.as_bytes(), &parsed_key_hash).is_ok());

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

    if Argon2::default().verify_password(new_password.as_bytes(), &parsed_hash).is_ok() {
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
    assert!(Argon2::default().verify_password(new_password.as_bytes(), &new_parsed_hash).is_ok());

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

/*
**********************************************************************************************************************************

TRANSACTION COMMANDS

**********************************************************************************************************************************
*/

#[tauri::command]
pub async fn add_transaction (
    pool: State<'_, SqlitePool>,
    user_id: i64,
    category: String,
    date: String,
    description: String,
    amount: f64,
    _type: String,
    name: String,
) -> Result<Transaction, String> {
    if !valid_categories().contains(category.as_str()) {
        error!("User '{}' tried adding a transaction with an invalid category: {}", name, category);
        return Err("Adding transaction failed".to_string());
    }

    match Date::parse(date.as_str(), &format_description!("[day]-[month]-[year]")) {
        Ok(_) => info!("Transaction date valid"),
        Err(e) => {
            error!("Transaction date '{}' is invalid: {:#?}", date, e);
            return Err("Adding transaction failed".to_string());
        }
    }

    if !valid_transaction_types().contains(_type.as_str()) {
        error!("User '{}' tried adding a transaction with an invalid type: {}", name, _type);
        return Err("Adding transaction failed".to_string());
    }

    let description = ammonia::clean(&description);

    let transaction = query_as::<_, Transaction>("INSERT INTO transactions (user_id, category, date, description, amount, _type) VALUES (?, ?, ?, ?, ?, ?) RETURNING *")
        .bind(user_id)
        .bind(&category)
        .bind(&date)
        .bind(&description)
        .bind(&amount)
        .bind(&_type)
        .fetch_one(&*pool)
        .await
        .map_err(|e| {
            error!("Failed to add transaction to database by user '{}': {:#?}", name, e);
            "Database error".to_string()
        })?;

    info!("Transaction added successfully by user '{}'", name);

    Ok(transaction)
}

#[tauri::command]
pub async fn get_transactions (
    pool: State<'_, SqlitePool>,
    user_id: i64,
    name: String,
) -> Result<Vec<Transaction>, String> {
    let transactions = query_as::<_, Transaction>("SELECT * FROM transactions WHERE user_id = ?")
        .bind(&user_id)
        .fetch_all(&*pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch transactions for user '{}': {:#?}", name, e);
            "Database error".to_string()
        })?;

    Ok(transactions)
}

#[tauri::command]
pub async fn delete_transaction (
    pool: State<'_, SqlitePool>,
    user_id: i64,
    ids: Vec<i64>,
    name: String,
) -> Result<Vec<Transaction>, String> {
    if ids.is_empty() {
        return Err("No transactions provided".to_string());
    }

    let mut tx = pool.begin().await.map_err(|e| {
        error!("Failed to begin transaction to delete the selected transactions: {:#?}", e);
        "An error occurred".to_string()
    })?;

    let placeholders: Vec<_> = (0..ids.len()).map(|_| "?").collect();
    let select_query = format!("SELECT * FROM transactions WHERE user_id = ? AND id IN ({})", placeholders.join(", "));

    let mut select_query = sqlx::query(&select_query).bind(user_id);
    for id in &ids {
        select_query = select_query.bind(id);
    }
    let rows = select_query.fetch_all(&mut *tx).await.map_err(|e| {
        error!("Failed to fetch transactions for deletion: {:#?}", e);
        "An error occurred".to_string()
    })?;

    let deleted_transactions: Vec<Transaction> = rows
        .into_iter()
        .map(|row| Transaction {
            id: row.get("id"),
            user_id: row.get("user_id"),
            category: row.get("category"),
            date: row.get("date"),
            description: row.get("description"),
            amount: row.get("amount"),
            _type: row.get("_type"),
        })
        .collect();

    let delete_query = format!("DELETE FROM transactions WHERE user_id = ? AND id IN ({})", placeholders.join(", "));

    let mut delete_query = sqlx::query(&delete_query).bind(user_id);
    for id in ids {
        delete_query = delete_query.bind(id);
    }

    let result = delete_query.execute(&mut *tx).await.map_err(|e| {
        error!("Failed to delete transactions: {:#?}", e);
        "An error occurred".to_string()
    })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:#?}", e);
        "An error occurred".to_string()
    })?;

    let rows_deleted = result.rows_affected();
    info!("User '{}' successfully deleted {} transactions", name, rows_deleted);

    Ok(deleted_transactions)
}

#[tauri::command]
pub async fn update_transaction (
    pool: State<'_, SqlitePool>,
    user_id: i64,
    transactions: Vec<Transaction>,
    name: String,
) -> Result<Vec<Transaction>, String> {
    if transactions.is_empty() {
        return Err("No transactions provided".to_string());
    }

    let mut tx = pool.begin().await.map_err(|e| {
        error!("Failed to begin transaction to update user transactions: {:#?}", e);
        "An error occurred".to_string()
    })?;

    for transaction in &transactions {
        if transaction.user_id != user_id {
            error!("User's '{}' ID did not match some transactions' user ID", name);
            return Err("An error occurred".to_string());
        }

        match Date::parse(transaction.date.as_str(), &format_description!("[day]-[month]-[year]")) {
            Ok(_) => info!("Updated transaction's date valid"),
            Err(_) => {
                error!("Transaction date '{}' is invalid", transaction.date);
                return Err("An error occurred".to_string());
            }
        }

        if !valid_categories().contains(transaction.category.as_str()) {
            error!("User '{}' tried updating a transaction with an invalid category: {}", name, transaction.category);
            return Err("An error occurred".to_string());
        }

        let t_type = if 
        ["rent", "taxes", "groceries", "utilities", "transportation", "travel", "entertainment", "healthcare",
        "insurance", "subscription", "education", "other"].contains(&transaction.category.as_str()) {
            "expense"
        } else if ["salary", "freelance", "investments"].contains(&transaction.category.as_str()) {
            "income"
        } else {
            return Err("An error occurred".to_string());
        };

        let description = ammonia::clean(&transaction.description);

        sqlx::query("UPDATE transactions SET category = ?, date = ?, description = ?, amount = ?, _type = ? WHERE id = ? AND user_id = ?")
            .bind(&transaction.category)
            .bind(&transaction.date)
            .bind(&description)
            .bind(&transaction.amount)
            .bind(t_type)
            .bind(transaction.id)
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to update transaction {}: {:#?}", transaction.id, e);
                "An error occurred".to_string()
            })?;
    }

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:#?}", e);
        "An error occurred".to_string()
    })?;

    let placeholders: Vec<_> = (0..transactions.len()).map(|_| "?").collect();
    let select_query = format!("SELECT * FROM transactions WHERE user_id = ? AND id IN ({})", placeholders.join(", "));

    let mut select_query = sqlx::query(&select_query).bind(user_id);
    for transaction in &transactions {
        select_query = select_query.bind(&transaction.id);
    }

    let rows = select_query.fetch_all(&*pool).await.map_err(|e| {
        error!("Failed to fetch updated transactions: {:#?}", e);
        "An error occurred".to_string()
    })?;
    let updated_transactions: Vec<Transaction> = rows
        .into_iter()
        .map(|row| Transaction {
            id: row.get("id"),
            user_id: row.get("user_id"),
            category: row.get("category"),
            date: row.get("date"),
            description: row.get("description"),
            amount: row.get("amount"),
            _type: row.get("_type"),
        })
        .collect();

    info!("User '{}' updated {} transactions successfully", name, updated_transactions.len());

    Ok(updated_transactions)
}

/*
**********************************************************************************************************************************

OTHER COMMANDS

**********************************************************************************************************************************
*/

#[tauri::command]
pub async fn backup_database () -> Result<(), String> {
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