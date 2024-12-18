use serde_json::Result;

use crate::logger::{LogLevel, Logger};
use crate::{log_e, log_x};

/// Parses a JSON array string into a Vec<String> and trims ' each string.
///
/// # Arguments
///
/// * `json_str` - A JSON string representing an array of strings.
///
/// # Returns
///
/// * `Result<Vec<String>, Box<dyn Error>>` - A vector of trimmed strings on success, or an error on failure.
pub fn parse_and_trim_json_strings(json_str: &str) -> Result<Vec<String>> {
    let array_result: Result<Vec<String>> = serde_json::from_str(json_str);
    match array_result {
        Ok(array) => Ok(array
            .into_iter()
            .map(|s| s.trim_matches('\'').to_string())
            .collect()),
        Err(e) => {
            log_e!(format!("Error parsing JSON: {}", e));
            Err(e)
        }
    }
}
