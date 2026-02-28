use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, ParseError, TimeZone, Utc, Weekday};
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
/// Converts raw HTML date input string into UTC `DateTime`.
///
/// Normalizes date strings from HTML frontend inputs by:
/// - Evaluating empty or whitespace-only strings as `None`.
/// - Replaces spaces with 'T' parsing as a full RFC 3339 datetime.
/// - Fallback to parsing a standard ISO 8601 calendar date (`YYYY-MM-DD`), appending 00:00:00 in the UTC timezone.
///
/// ## Arguments
///
/// * `date_str` - The raw string value submitted by frontend.
///
/// ## Errors
///
/// Returns a `DateFormattingError` if fails to parse as either a valid RFC 3339 datetime or a valid `YYYY-MM-DD` string.
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

// RFC 3339 → Deutsches Ausgabe-Datum, also: "2026-01-30T18:00:00+00" in "30.01.2026"
/// Converts an optional database datetime string into German date format (DD.MM.YYYY) for usage in Frontend.
///
/// Parses provided String as a full RFC 3339 datetime (using systems local timezone)
/// Fallback to parsing it as a naive ISO 8601 date (`YYYY-MM-DD`).
/// If input is `None` or empty string, returns an empty string rather than error.
///
/// ## Arguments
///
/// * `date_str` - AReference to an `Option<String>` containing raw database value.
///
/// ## Errors
///
/// Returns a `DateFormattingError` if parse as a valid RFC 3339 datetime or a naive `YYYY-MM-DD` date fails.
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
/// Converts a database datetime string into the `YYYY-MM-DD` format required by HTML date inputs.
///
/// Parses a stored RFC 3339 datetime or naive date string and formats it specifically for pre-filling `<input type="date">` fields.
/// Used for example when editing a rendered component that uses a datetime Element (e.g., inside the Edit Todo modal).
/// If input is empty or `None`, returns an empty string.
///
/// ## Arguments
///
/// * `iso_string` - A Reference to an `Option<String>` containing raw database datetime.
///
/// ## Errors
///
/// Returns a `DateFormattingError` if the string cannot be parsed as a valid RFC 3339 datetime or a naive `YYYY-MM-DD` string.
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

//Helper um nächste Datumsinstanzen nach rrule zu finden

//generiert das nächste Datum einer wiederholenden Instanz, wird innerhalb des recurrance Handlers für Todos und CalenderEvents genutzt
//current bezieht sich hierbei auf das aktuelle Datum im loop im recurrance_handler

/// Calculates the next occurrence of a recurring datetime based on a specific recurrence-rule.
///
/// Calculates the next date in a recurring series during the expansion loop of recurring To-Dos or Calendar-Events (`expand_recurring_todos` or `expand_recurring_events`).
/// "Edge-Cases" are handeled by subfunctions (e.g. last day of month in monthly reccuring events, or handling 29. of February in Leapyear), therefore  original start-date of reccurance is passed to those subfunctions.
///
/// ## Arguments
///
/// * `current` - Date of the current iteration in the recurrence loop.
/// * `rrule` - String slice representing recurrence rule.
/// * `start_date_of_rec` - The original starting datetime of the master event, handed over to subfunctions for Edge-Cases in next date calculation.
///
/// ## Errors
///
/// Returns boxed dynamic error if the provided `rrule` string does not match the supported recurrence Rule patterns.
pub fn calculate_next_date(
    current: DateTime<Utc>,
    rrule: &str,
    start_date_of_rec: DateTime<Utc>,
) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
    match rrule {
        "Daily" | "daily" => Ok(current + Duration::days(1)),
        "Weekly" | "weekly" => Ok(current + Duration::weeks(1)),
        "Fortnight" | "fortnight" => Ok(current + Duration::weeks(2)),
        "weekdays" => {
            //wenn Freitag oder Samstag auf Monatag,
            let next_date = match current.weekday() {
                Weekday::Fri => current + Duration::days(3),
                Weekday::Sat => current + Duration::days(2),
                _ => current + Duration::days(1),
            };
            Ok(next_date)
        }
        "monthly_on_date" => add_months_same_date(current, 1, start_date_of_rec.day()), //Start date mitgeben um Probleme um 31. des Monats zu handeln
        "monthly_on_weekday" => add_month_on_same_weekday(current),
        "Annual" | "annual" => add_months_same_date(current, 12, start_date_of_rec.day()), //Start date mitgeben um Probleme um 31. des Monats zu handeln
        _ => Err("No matching rrule".to_string().into()),
    }
}

//für monatlich wiederholende Todos auf den selben Tag
/// Adds a specified number of months to a date, preserving the day of the month if possible.
///
/// Handles "Edge-Cases" of varying month lengths.
/// If original date is a day that does not exist in target month (e.g. from January 31st to February), the resulting date is moved to the last valid day of target month.
///
/// To prevent errors during mutation, the day is temporarily set to the 1st before applying the new year and month.
///
/// ## Arguments
///
/// * `date` - Base datetime to manipulate.
/// * `months_to_add` - The number of months to jump forward.
/// * `preferered_day_on_exception` - The target day of the month (usually the day of the master event).
///
/// ## Errors
///
/// Returns a boxed dynamic error if date manipulation results are out-of-bounds (e.g., invalid year, month, or day calculation).
fn add_months_same_date(
    date: DateTime<Utc>,
    months_to_add: u32,
    preferered_day_on_exception: u32,
) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
    let year = date.year() + (date.month() as i32 + months_to_add as i32 - 1) / 12;
    let month = (date.month() as i32 + months_to_add as i32 - 1) % 12 + 1;

    //falls wiederholendes todo in nächsten monat fallen würde letzten Tag des monats suchen (bsp. Wdh. Event am 31. März soll dann 30.April nicht 1. Mai) ...
    let day_raw = handle_last_day_of_month(year, month as u32)
        .ok_or_else(|| "Date conversion invalid in fn add_months_same_date".to_string())?;

    //...und das kleinere der beiden nehmen
    let day = std::cmp::min(preferered_day_on_exception, day_raw);
    //println!("Fehler bei {} {} {}", year, month, day);
    date.with_day(1) //erst Tag auf eins setzen sonst wirft es manchmal fehler
        .ok_or("Error resetting day to 1")?
        .with_year(year)
        .ok_or("Invalid year in fn add_months_same_date")?
        .with_month(month as u32)
        .ok_or("Invalid month in fn add_months_same_date")?
        .with_day(day)
        .ok_or("Invalid day in fn add_months_same_date")
        .map_err(|e| e.into())
}

//gibt den letzten Tag des Monats aus, bzw. Anzahl an Tage in dem Monat
/// Calculates the number of days/the last day of a given month.
///
/// Calculates the last day of the month by instantiating the first day of the following month and stepping backward by one day.
///
/// ## Arguments
///
/// * `year` - The full calendar year.
/// * `month` - The month.
///
/// # Returns
///
/// Returns an `Option<u32>` representing the last day of the requested month or `None` if input month is out of bounds.
fn handle_last_day_of_month(year: i32, month: u32) -> Option<u32> {
    if month == 12 {
        // im Dezember -> ersten Tag des neuen Jahres nehmen und davon dann vorgänger
        NaiveDate::from_ymd_opt(year + 1, 1, 1)?
            .pred_opt() // Gibt Vorgängerdatum
            .map(|d| d.day())
    } else {
        // alle anderen Monate
        NaiveDate::from_ymd_opt(year, month + 1, 1)? //erster Tag des Nächsten Monats
            .pred_opt() //Davon Vorgängerdatum
            .map(|d| d.day())
    }
}

/// Calculates the same `n-th` occurrence of a weekday in the following month.
///
/// E.g. a recurring event is scheduled for the "3rd Friday" of the current month.
/// Calculates the exact date of the "3rd Friday" in the next month.
///
/// If current month has a 5th occurrence of a weekday, but the next month only has 4 occurrences, result is automatically set to the last available occurrence of that month.
///
/// ## Arguments
///
/// * `date` - The base datetime..
///
/// ## Errors
///
/// Returns a boxed dynamic error if any datetime mutations result in an invalid or out-of-bounds date.
fn add_month_on_same_weekday(
    date: DateTime<Utc>,
) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
    let weekday = date.weekday();
    let day_date = date.day();
    //der wie vielte dieses Wochentages ist es im Monat (3ter Freitag im Monat)
    let nth_weekday_of_month = (day_date - 1) / 7 + 1;

    let (next_year, next_month) = if date.month() == 12 {
        // im Dezember
        (date.year() + 1 /*Jahr+1*/, 1 /*Januar*/)
    } else {
        //sonst nur monat+1
        (date.year(), date.month() + 1)
    };

    let mut date_result = date
        .with_year(next_year)
        .ok_or("Invalid year in fn add_month_on_same_weekday")?
        .with_month(next_month)
        .ok_or("Invalid month in fn add_month_on_same_weekday")?
        .with_day(1)
        .ok_or("Invalid day in fn add_month_on_same_weekday")?;

    //Sucht den ersten Wochentag des Monats, der gleich des Todo DueDate-Tages ist
    while date_result.weekday() != weekday {
        date_result += Duration::days(1);
    }
    //springt auf n-ten Wochentag des Monats vor
    date_result += Duration::weeks((nth_weekday_of_month - 1) as i64);

    //Wenn nächster passender Wochentag erst im übernächsten monat eine woche zurück gehen, letzten passenden Wochentag nehmen
    if date_result.month() != next_month {
        date_result -= Duration::weeks(1);
    }

    Ok(date_result) // Gibt das berechnete Datum zurück
}
