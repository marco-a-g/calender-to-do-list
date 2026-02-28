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

//Helper um nächste Datumsinstanzen nach rrule zu finden

//generiert das nächste Datum einer wiederholenden Instanz, wird innerhalb des recurrance Handlers für Todos und CalenderEvents genutzt
//current bezieht sich hierbei auf das aktuelle Datum im loop im recurrance_handler
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
        _ => Err(format!("No matching rrule").into()),
    }
}

//für monatlich wiederholende Todos auf den selben Tag
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
