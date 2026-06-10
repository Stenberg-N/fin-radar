use crate::AppState;
use serde::{Deserialize, Serialize};
use sqlx::{query_as, FromRow, Row};
use tauri::State;
use time::{Date, macros::{format_description}};
use log::{info, error};
use super::helpers::{valid_categories, valid_transaction_types, create_timestamp};

/************************************************************************************************************************\

TRANSACTIONS COMMANDS

\************************************************************************************************************************/

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

#[tauri::command]
pub async fn add_transaction (
    state: State<'_, AppState>,
    category: String,
    date: String,
    description: String,
    amount: f64,
    _type: String,
) -> Result<Transaction, String> {
    let session = state.get_session().map_err(|e| {
        error!("Adding transaction failed at {} due to: {:#?}", create_timestamp(), e);
        "Adding transaction failed".to_string()
    })?;

    if !valid_categories().contains(category.as_str()) {
        error!("User '{}' tried adding a transaction with an invalid category: {}", session.user.name, category);
        return Err("Adding transaction failed".to_string());
    }

    match Date::parse(date.as_str(), &format_description!("[year]-[month]-[day]")) {
        Ok(_) => info!("Transaction date valid"),
        Err(e) => {
            error!("Transaction date '{}' is invalid: {:#?}", date, e);
            return Err("Adding transaction failed".to_string());
        }
    }

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
        .bind(date)
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

    Ok(transaction)
}

#[tauri::command]
pub async fn get_transactions (
    state: State<'_, AppState>,
    year_month: String,
) -> Result<Vec<Transaction>, String> {
    let session = state.get_session().map_err(|e| {
        error!("Fetching transactions failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let date_parts: Vec<&str> = year_month.split("-").collect();

    if date_parts.len() != 2 {
        error!("User '{}' provided an invalid date (YYYY-MM) format", session.user.name);
        return Err("An error occurred".to_string());
    }

    let year = match date_parts[0].parse::<u16>() {
        Ok(year) => year,
        Err(_) => {
            error!("User '{}' provided a date with an invalid year", session.user.name);
            return Err("An error occurred".to_string());
        }
    };

    let month = match date_parts[1] {
        "01" | "02" | "03" | "04" | "05" | "06" | "07" | "08" | "09" | "10" | "11" | "12" => date_parts[1],
        _ => {
            error!("User '{}' provided a date with an invalid month", session.user.name);
            return Err("An error occurred".to_string());
        }
    };

    let transactions = query_as::<_, Transaction>("SELECT * FROM transactions WHERE user_id = ? AND strftime('%Y-%m', date) = ? ORDER BY date DESC")
        .bind(session.user.id)
        .bind(format!("{}-{}", year, month))
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch transactions for user '{}': {:#?}", session.user.name, e);
            "Database error".to_string()
        })?;

    Ok(transactions)
}

#[tauri::command]
pub async fn get_year_transactions (
    state: State<'_, AppState>,
    year: String,
) -> Result<Vec<Transaction>, String> {
    let session = state.get_session().map_err(|e| {
        error!("Fetching transactions failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let year = match year.parse::<u16>() {
        Ok(year) => year,
        Err(_) => {
            error!("User '{}' provided an invalid year", session.user.name);
            return Err("An error occurred".to_string());
        }
    };

    let transactions = query_as::<_, Transaction>("SELECT * FROM transactions WHERE user_id = ? AND strftime('%Y', date) = ? ORDER BY date DESC")
        .bind(session.user.id)
        .bind(format!("{}", year))
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch yearly transactions for user '{}': {:#?}", session.user.name, e);
            "Database error".to_string()
        })?;

    Ok(transactions)
}

#[tauri::command]
pub async fn delete_transaction (
    state: State<'_, AppState>,
    ids: Vec<i64>,
) -> Result<Vec<Transaction>, String> {
    let session = state.get_session().map_err(|e| {
        error!("Deleting transaction failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if ids.is_empty() {
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

    Ok(deleted_transactions)
}

#[tauri::command]
pub async fn update_transaction (
    state: State<'_, AppState>,
    transactions: Vec<Transaction>,
) -> Result<Vec<Transaction>, String> {
    let session = state.get_session().map_err(|e| {
        error!("Updating transaction failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if transactions.is_empty() {
        return Err("No transactions provided".to_string());
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

        match Date::parse(transaction.date.as_str(), &format_description!("[year]-[month]-[day]")) {
            Ok(_) => info!("Updated transaction's date valid"),
            Err(_) => {
                error!("Transaction date '{}' is invalid", transaction.date);
                return Err("An error occurred".to_string());
            }
        }

        if !valid_categories().contains(transaction.category.as_str()) {
            error!("User '{}' tried updating a transaction with an invalid category: {}", session.user.name, transaction.category);
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
            .bind(t_type)
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

    Ok(updated_transactions)
}