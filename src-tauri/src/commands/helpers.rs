use crate::AppState;
use tauri::State;
use std::{collections::HashSet};
use rand::{distr::Alphanumeric, distr::SampleString, rng};
use time::{OffsetDateTime, macros::{format_description}};
use log::warn;

/************************************************************************************************************************\

HELPER FUNCTIONS

\************************************************************************************************************************/

pub fn validate_password(pw: &str) -> bool {
    let has_min_length = pw.chars().count() >= 10;
    let no_spaces = !pw.chars().any(|c| c.is_whitespace());
    let has_numbers = pw.chars().any(|c| c.is_ascii_digit());
    let has_uppercase = pw.chars().any(|c| c.is_uppercase() && c.is_alphabetic());
    let has_lowercase = pw.chars().any(|c| c.is_lowercase() && c.is_alphabetic());
    let has_special_char = pw.chars().any(|c| r#"=_!@#$€£¤%^&*(){}[],.?'":|/\+-<>~§"#.contains(c));

    has_min_length && no_spaces && has_numbers && has_uppercase && has_lowercase && has_special_char
}

pub fn generate_recovery_key () -> String {
    Alphanumeric.sample_string(&mut rng(), 48)
}

pub fn valid_categories () -> HashSet<&'static str> {
    HashSet::from([
        "rent", "taxes", "groceries", "utilities", "transportation", "travel", "entertainment", "healthcare",
        "insurance", "subscription", "education", "other", "salary", "freelance", "investments",
    ])
}

pub fn valid_transaction_types () -> HashSet<&'static str> {
    HashSet::from(["income", "expense"])
}

pub fn create_timestamp () -> String {
    OffsetDateTime::now_local()
        .ok()
        .and_then(|dt| dt.format(&format_description!("[year]-[month]-[day] | [hour]:[minute]:[second]")).ok())
        .unwrap_or("Failed to create timestamp".to_string())
}

pub fn set_session_id (state: &State<'_, AppState>, session_id: Option<i64> ) {
    let mut guard = state.session.lock().unwrap_or_else(|poisoned| {
        warn!("SESSION ID POISONED ({})", create_timestamp());
        poisoned.into_inner()
    });
    *guard = session_id;
}

pub fn get_session_id (state: &State<'_, AppState>) -> Option<i64> {
    match state.session.lock() {
        Ok(guard) => *guard,
        Err(poisoned) => *poisoned.into_inner(),
    }
}