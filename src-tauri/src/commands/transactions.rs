use serde::{Deserialize, Serialize};
use sqlx::{query_as, FromRow, Row};
use tauri::State;
use std::collections::HashMap;
use time::{Date, macros::{format_description}};
use log::{info, error, warn};

use crate::{AppState, structs::session::SessionData};
use super::helpers::{valid_categories, valid_transaction_types, create_timestamp, validate_year_month, check_user_capabilities};
use crate::structs::cache::{CacheData, UpdateTask};

/************************************************************************************************************************\

TRANSACTIONS COMMANDS

\************************************************************************************************************************/

#[derive(FromRow, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub id: i64,
    user_id: i64,
    category: String,
    date: String,
    description: String,
    amount: f64,
    _type: String,
}

/// The `year_month` parameter accepts either:
/// - A 4-digit year (e.g., `"2024"`) to fetch all transactions in that year.
/// - A year-month string (e.g., `"2024-05"`) to fetch transactions for a specific month.
async fn fetch_and_cache_transactions(
    state: &AppState,
    year_month: &str,
    key: &str,
    user_id: i64,
    username: &str,
) -> Result<Vec<Transaction>, String> {
    let (query, bind_value) = if year_month.len() == 4 {
        (
            "SELECT * FROM transactions WHERE user_id = ? AND strftime('%Y', date) = ? ORDER BY date DESC",
            year_month.to_string()
        )
    } else if year_month.len() == 7 {
        (
            "SELECT * FROM transactions WHERE user_id = ? AND strftime('%Y-%m', date) = ? ORDER BY date DESC",
            year_month.to_string()
        )
    } else {
        return Err("An error occurred".to_string());
    };

    let txs = query_as::<_, Transaction>(query)
        .bind(user_id)
        .bind(bind_value)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch transactions for user '{}': {:#?}", username, e);
            "Database error".to_string()
        })?;

    if let Err(e) = state.session.cache.cache_results(key.to_string(), CacheData::from(txs.clone())) {
        error!("CACHE POISONED ({}): Failed to set transactions to cache for user '{}': {:#?}", create_timestamp(), username, e);
    }

    Ok(txs)
}

#[tauri::command]
pub async fn add_transaction(
    state: State<'_, AppState>,
    category: String,
    date: String,
    description: String,
    amount: f64,
    _type: String,
) -> Result<Transaction, String> {
    let state: &AppState = &*state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Adding transaction failed at {} due to: {:#?}", create_timestamp(), e);
        "Adding transaction failed".to_string()
    })?;

    check_user_capabilities(&session.user, "add_transaction")?;

    if !valid_categories().contains(category.as_str()) {
        error!("User '{}' tried adding a transaction with an invalid category: {}", session.user.name, category);
        return Err("Adding transaction failed".to_string());
    }

    Date::parse(date.as_str(), &format_description!("[year]-[month]-[day]")).map_err(|e| {
        error!("Transaction date '{}' is invalid: {:#?}", date, e);
        "Adding transaction failed".to_string()
    })?;

    if !valid_transaction_types().contains(_type.as_str()) {
        error!("User '{}' tried adding a transaction with an invalid type: {}", session.user.name, _type);
        return Err("Adding transaction failed".to_string());
    }

    if amount <= 0.00 {
        error!("User '{}' tried adding a transaction with zero or negative amount", session.user.name);
        return Err("Adding transaction failed".to_string())
    }

    let description = ammonia::clean(&description);

    let transaction = query_as::<_, Transaction>("INSERT INTO transactions (user_id, category, date, description, amount, _type) VALUES (?, ?, ?, ?, ?, ?) RETURNING *")
        .bind(session.user.id)
        .bind(category)
        .bind(&date)
        .bind(description)
        .bind(amount)
        .bind(_type)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to add transaction to database by user '{}': {:#?}", session.user.name, e);
            "Database error".to_string()
        })?;

    info!("Transaction added successfully by user '{}'", session.user.name);

    if let Some(value) = date.get(..7) {
        let year_month = value;
        let key = format!("{}-{}-txs", session.user.id, year_month);

        match state.session.cache.contains(&key) {
            Ok(true) => {
                if let Err(e) = state.session.cache.update_cache(&key, &HashMap::from([(transaction.id, transaction.clone())]), &UpdateTask::Update) {
                    error!("CACHE POISONED ({}): Failed to add transaction to cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
                }
            },
            Ok(false) => {
                if let Err(e) = state.session.cache.cache_results(key, CacheData::from(Vec::from([transaction.clone()]))) {
                    error!("CACHE POISONED ({}): Failed to add transaction to cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
                }
            },
            Err(e) => {
                error!("CACHE POISONED ({}): Failed to check cache for user '{}'. Refetching data: {:#?}", create_timestamp(), session.user.name, e);
                fetch_and_cache_transactions(state, &year_month, &key, session.user.id, &session.user.name).await?;
            }
        }
    } else {
        error!("CACHING FAILED ({}): Failed to add transaction to cache for user '{}': Invalid date", create_timestamp(), session.user.name);
    }

    Ok(transaction)
}

#[tauri::command]
pub async fn get_transactions(
    state: State<'_, AppState>,
    year_month: String,
) -> Result<Vec<Transaction>, String> {
    let state: &AppState = &*state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Fetching transactions failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    check_user_capabilities(&session.user, "get_transactions")?;

    let year_month = validate_year_month(&year_month, &session.user.name).map_err(|e| {
        e
    })?;

    let key = format!("{}-{}-txs", session.user.id, year_month);

    let mut transactions = match state.session.cache.contains(&key) {
        Ok(true) => {
            match state.session.cache.get_transactions(&key) {
                Ok(Some(txs)) => txs.values().cloned().collect(),
                Ok(None) => fetch_and_cache_transactions(state, &year_month, &key, session.user.id, &session.user.name).await?,
                Err(e) => {
                    error!("CACHE POISONED ({}): Failed to get transactions from cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
                    fetch_and_cache_transactions(state, &year_month, &key, session.user.id, &session.user.name).await?
                }
            }
        },
        Ok(false) => fetch_and_cache_transactions(state, &year_month, &key, session.user.id, &session.user.name).await?,
        Err(e) => {
            error!("CACHE POISONED ({}): Failed to check cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
            fetch_and_cache_transactions(state, &year_month, &key, session.user.id, &session.user.name).await?
        }
    };

    transactions.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(transactions)
}

#[tauri::command]
pub async fn get_year_transactions(
    state: State<'_, AppState>,
    year: String,
) -> Result<Vec<Transaction>, String> {
    let state: &AppState = &*state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Fetching transactions failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    check_user_capabilities(&session.user, "get_year_transactions")?;

    if year.len() != 4 || !year.chars().all(|c| c.is_ascii_digit()) {
        error!("User '{}' provided an invalid year", session.user.name);
        return Err("An error occurred".to_string());
    }

    let key = format!("{}-{}-txs", session.user.id, year);

    let mut transactions = match state.session.cache.contains(&key) {
        Ok(true) => {
            match state.session.cache.get_transactions(&key) {
                Ok(Some(txs)) => txs.values().cloned().collect(),
                Ok(None) => fetch_and_cache_transactions(state, &year, &key, session.user.id, &session.user.name).await?,
                Err(e) => {
                    error!("CACHE POISONED ({}): Failed to get yearly transactions from cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
                    fetch_and_cache_transactions(state, &year, &key, session.user.id, &session.user.name).await?
                }
            }
        },
        Ok(false) => fetch_and_cache_transactions(state, &year, &key, session.user.id, &session.user.name).await?,
        Err(e) => {
            error!("CACHE POISONED ({}): Failed to check cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
            fetch_and_cache_transactions(state, &year, &key, session.user.id, &session.user.name).await?
        }
    };

    transactions.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(transactions)
}

#[tauri::command]
pub async fn delete_transaction(
    state: State<'_, AppState>,
    ids: Vec<i64>,
    year_month: String,
) -> Result<Vec<Transaction>, String> {
    let state: &AppState = &*state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Deleting transaction failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    check_user_capabilities(&session.user, "delete_transaction")?;

    if ids.is_empty() {
        warn!("Transactions sent for deletion at {} by user '{}' were empty", create_timestamp(), session.user.name);
        return Err("No transactions provided".to_string());
    }

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction to delete the selected transactions: {:#?}", e);
        "An error occurred".to_string()
    })?;

    let placeholders: Vec<_> = (0..ids.len()).map(|_| "?").collect();
    let select_query = format!("SELECT * FROM transactions WHERE user_id = ? AND id IN ({})", placeholders.join(", "));

    let mut select_query = sqlx::query(&select_query).bind(session.user.id);
    for id in ids {
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

    let mut delete_query = sqlx::query(&delete_query).bind(session.user.id);
    for transaction in &deleted_transactions {
        delete_query = delete_query.bind(transaction.id);
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
    info!("User '{}' successfully deleted {} transactions at {}", session.user.name, rows_deleted, create_timestamp());

    if let Ok(year_month) = validate_year_month(&year_month, &session.user.name) {
        let key = format!("{}-{}-txs", session.user.id, year_month);

        if let Err(e) = state.session.cache.update_cache(&key, &deleted_transactions.clone().into_iter().map(|t| (t.id, t)).collect(), &UpdateTask::Delete) {
            error!("CACHE POISONED ({}): Failed to delete transactions from cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
        }
    }

    Ok(deleted_transactions)
}

#[tauri::command]
pub async fn update_transaction(
    state: State<'_, AppState>,
    transactions: Vec<Transaction>,
    year_month: String,
) -> Result<Vec<Transaction>, String> {
    let state: &AppState = &*state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Updating transaction failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    check_user_capabilities(&session.user, "update_transaction")?;

    if transactions.is_empty() {
        warn!("TRANSACTION UPDATE FAILED ({}): User '{}' provided no transactions", create_timestamp(), session.user.name);
        return Err("An error occurred".to_string());
    }

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction to update user transactions: {:#?}", e);
        "An error occurred".to_string()
    })?;

    for transaction in &transactions {
        if transaction.user_id != session.user.id {
            error!("User's '{}' ID did not match some transactions' user ID", session.user.name);
            return Err("An error occurred".to_string());
        }

        Date::parse(transaction.date.as_str(), &format_description!("[year]-[month]-[day]")).map_err(|e| {
            error!("Transaction date '{}' is invalid: {:#?}", transaction.date, e);
            "An error occurred".to_string()
        })?;

        if !valid_categories().contains(transaction.category.as_str()) {
            error!("User '{}' tried updating a transaction with an invalid category: {}", session.user.name, transaction.category);
            return Err("An error occurred".to_string());
        }

        let transaction_type = if 
        ["rent", "taxes", "groceries", "utilities", "transportation", "travel", "entertainment", "healthcare",
        "insurance", "subscription", "education", "other"].contains(&transaction.category.as_str()) {
            "expense"
        } else if ["salary", "freelance", "investments"].contains(&transaction.category.as_str()) {
            "income"
        } else {
            return Err("An error occurred".to_string());
        };

        if transaction.amount <= 0.00 {
            error!("User '{}' tried giving a transaction zero or negative amount", session.user.name);
            return Err("An error occurred".to_string())
        }

        let description = ammonia::clean(&transaction.description);

        sqlx::query("UPDATE transactions SET category = ?, date = ?, description = ?, amount = ?, _type = ? WHERE id = ? AND user_id = ?")
            .bind(&transaction.category)
            .bind(&transaction.date)
            .bind(description)
            .bind(transaction.amount)
            .bind(transaction_type)
            .bind(transaction.id)
            .bind(session.user.id)
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

    let mut select_query = sqlx::query(&select_query).bind(session.user.id);
    for transaction in &transactions {
        select_query = select_query.bind(&transaction.id);
    }

    let rows = select_query.fetch_all(&state.db).await.map_err(|e| {
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

    info!("User '{}' updated {} transactions successfully", session.user.name, updated_transactions.len());
    
    if let Ok(year_month) = validate_year_month(&year_month, &session.user.name) {
        let key = format!("{}-{}-txs", session.user.id, year_month);

        if let Err(e) = state.session.cache.update_cache(&key, &updated_transactions.clone().into_iter().map(|t| (t.id, t)).collect(), &UpdateTask::Update) {
            error!("CACHE POISONED ({}): Failed to update transactions to cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
        }
    }

    Ok(updated_transactions)
}