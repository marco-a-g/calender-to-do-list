use chrono::{DateTime, Local, NaiveDate, ParseError, TimeZone, Utc};
use std::fmt;

#[derive(Debug)]
pub enum DateFormattingError {
    Parse(ParseError),
    InvalidFormat(String),
}

impl fmt::Display for DateFormattingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DateFormattingError::Parse(e) => write!(f, "Date Parse Error: {}", e),
            DateFormattingError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
        }
    }
}

impl std::error::Error for DateFormattingError {}

impl From<ParseError> for DateFormattingError {
    fn from(err: ParseError) -> DateFormattingError {
        DateFormattingError::Parse(err)
    }
}

//HTML Eingaben über Eingabemaske bsp. "2026-01-30" in "2026-01-30T00:00:00Z" (für Supabase) ISO Date → RFC 3339
pub fn html_input_to_db(date_str: &str) -> Result<Option<DateTime<Utc>>, DateFormattingError> {
    if date_str.trim().is_empty() {
        return Ok(None);
    }
    let clean = date_str.replace(" ", "T");
    if let Ok(dt) = DateTime::parse_from_rfc3339(&clean) {
        return Ok(Some(dt.with_timezone(&Utc)));
    }
    let naive_date = NaiveDate::parse_from_str(&clean, "%Y-%m-%d")?;
    let dt_utc = naive_date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| DateFormattingError::InvalidFormat("Zeitfehler".to_string()))?;

    Ok(Some(Utc.from_utc_datetime(&dt_utc)))
}

// RFC 3339 → Deutsches Datum, also: "2026-01-30T18:00:00+00" in "30.01.2026"
pub fn db_to_display_only_date(date_str: &Option<String>) -> Result<String, DateFormattingError> {
    match date_str {
        Some(s) if !s.is_empty() => {
            let clean_date = s.replace(" ", "T"); //Falls RFC 3339 mit " " statt "T"
            //Falls mit Uhrzeit
            if let Ok(dt) = DateTime::parse_from_rfc3339(&clean_date) {
                return Ok(dt.with_timezone(&Local).format("%d.%m.%Y").to_string());
            }
            //Falls ohne Uhrzeit
            let date = NaiveDate::parse_from_str(&clean_date, "%Y-%m-%d")?;
            Ok(date.format("%d.%m.%Y").to_string())
        }
        //Falls String leer oder None
        _ => Ok("".to_string()),
    }
}

// aus DB in HTML Input Value (Bei Edit Mode relevant)
pub fn db_to_html_input(iso_string: &Option<String>) -> Result<String, DateFormattingError> {
    match iso_string {
        Some(s) if !s.is_empty() => {
            let clean_s = s.replace(" ", "T");
            if let Ok(dt) = DateTime::parse_from_rfc3339(&clean_s) {
                return Ok(dt.format("%Y-%m-%d").to_string());
            }
            let date = NaiveDate::parse_from_str(&clean_s, "%Y-%m-%d")?;
            Ok(date.format("%Y-%m-%d").to_string())
        }
        _ => Ok(String::new()),
    }
}
