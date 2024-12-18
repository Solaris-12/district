use std::collections::HashMap;
use std::time::SystemTime;

use chrono::{DateTime, ParseError, Utc};

use crate::application::features::lang::get_translation;
use crate::logger::{LogLevel, Logger};
use crate::{log_e, log_x};

/// Parses a date string from RFC3339 format and converts it to a DateTime<Utc> object.
///
/// # Arguments
///
/// * `date_str` - A date string in RFC3339 format.
///
/// # Returns
///
/// * `Result<DateTime<Utc>, ParseError>` - A DateTime<Utc> object on success, or a ParseError on failure.
pub fn parse_rfc3339_to_utc(date_str: String) -> Result<DateTime<Utc>, ParseError> {
    match DateTime::parse_from_rfc3339(date_str.as_str()) {
        Ok(val) => Ok(val.with_timezone(&Utc)),
        Err(e) => {
            log_e!(format!("Invalid date format: {}", e));
            Err(e)
        }
    }
}

/// Parses a date string from RFC3339 format and converts it to a DateTime<Utc> object.
/// Returns None if the input is "NULL" or if parsing fails.
///
/// # Arguments
///
/// * `date_str` - A date string in RFC3339 format or "NULL".
///
/// # Returns
///
/// * `Option<DateTime<Utc>>` - A DateTime<Utc> object on success, None if the input is "NULL" or parsing fails.
pub fn parse_rfc3339_to_utc_or_none(date_str: Option<String>) -> Option<DateTime<Utc>> {
    match date_str {
        Some(date_str) if date_str != "NULL" => DateTime::parse_from_rfc3339(&date_str)
            .map(|dt| dt.with_timezone(&Utc))
            .ok(),
        _ => None,
    }
}

/// Generates a Discord-formatted timestamp string.
///
/// This function retrieves a translated timestamp string and replaces a placeholder with the current Unix timestamp.
///
/// # Returns
///
/// * `String` - A Discord-formatted timestamp string.
pub fn get_discord_timestamp(lang: Option<&HashMap<String, String>>) -> String {
    let msg = match lang {
        Some(lang_data) => match get_translation(&lang_data, "utils.timestamp") {
            Some(val) => val,
            None => String::from("<t:{t}:R>"),
        },
        None => String::from("<t:{t}:R>"),
    };
    msg.replace(
        "{t}",
        &SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string(),
    )
}
