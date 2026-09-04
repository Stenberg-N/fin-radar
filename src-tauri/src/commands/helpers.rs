use std::{collections::HashSet};
use rand::{distr::Alphanumeric, distr::SampleString, rng};
use time::{OffsetDateTime, macros::{format_description}};
use log::error;

use super::user::SafeUser;

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

pub fn generate_recovery_key() -> String {
    Alphanumeric.sample_string(&mut rng(), 48)
}

/// Returns a HashSet of acceptable transaction categories. Can be chained with the `.contains` method.
pub fn valid_categories() -> HashSet<&'static str> {
    HashSet::from([
        "rent", "taxes", "groceries", "utilities", "transportation", "travel", "entertainment", "healthcare",
        "insurance", "subscription", "education", "other", "salary", "freelance", "investments",
    ])
}

pub fn valid_transaction_types() -> HashSet<&'static str> {
    HashSet::from(["income", "expense"])
}

pub fn create_timestamp() -> String {
    OffsetDateTime::now_local()
        .ok()
        .and_then(|dt| dt.format(&format_description!("[year]-[month]-[day] | [hour]:[minute]:[second]")).ok())
        .unwrap_or("Failed to create timestamp".to_string())
}

pub fn validate_year_month(year_month: &str, username: &str) -> Result<String, String> {
    let date_parts: Vec<&str> = year_month.split("-").collect();

    if date_parts.len() != 2 ||
        date_parts[0].len() != 4 ||
        !date_parts[0].chars().all(|c| c.is_ascii_digit()) ||
        date_parts[1].len() != 2 ||
        !date_parts[1].chars().all(|c| c.is_ascii_digit())
    {
        error!("User '{}' provided an invalid date (YYYY-MM) format", username);
        return Err("An error occurred".to_string());
    }

    let year = match date_parts[0].parse::<u16>() {
        Ok(year) => year,
        Err(_) => {
            error!("User '{}' provided a date with an invalid year", username);
            return Err("An error occurred".to_string());
        }
    };

    let month = match date_parts[1] {
        "01" | "02" | "03" | "04" | "05" | "06" | "07" | "08" | "09" | "10" | "11" | "12" => date_parts[1],
        _ => {
            error!("User '{}' provided a date with an invalid month", username);
            return Err("An error occurred".to_string());
        }
    };

    Ok(format!("{}-{}", year, month))
}

/// Checks if the user is in recovery mode and returns an Err if so.
/// Use to restrict access to something if in recovery mode.
pub fn check_user_capabilities(user: &SafeUser, function_name: &str) -> Result<(), String> {
    if user.requires_password_reset {
        error!("CAPABILITY MISMATCH ({}): User '{}' tried running a function they have no access to: {}", create_timestamp(), user.name, function_name);
        return Err("An error occurred".to_string());
    }

    Ok(())
}