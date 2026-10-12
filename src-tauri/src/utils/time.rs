use time::{OffsetDateTime, macros::{format_description}};
use log::error;

pub(crate) fn create_timestamp() -> String {
    match OffsetDateTime::now_local() {
        Ok(dt) => {
            if let Some(dt) = dt.format(&format_description!("[year]-[month]-[day] [hour]:[minute]:[second]")).ok() {
                dt
            } else {
                time::OffsetDateTime::now_utc().to_string()
            }
        },
        Err(_) => {
            if let Some(dt) = time::OffsetDateTime::now_utc().format(&format_description!("[year]-[month]-[day] [hour]:[minute]:[second]")).ok() {
                dt
            } else {
                time::OffsetDateTime::now_utc().to_string()
            }
        }
    }
}

pub(crate) fn validate_year_month(year_month: &str, username: &str) -> Result<String, String> {
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