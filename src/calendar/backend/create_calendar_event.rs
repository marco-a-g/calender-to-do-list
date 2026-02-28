use chrono::{DateTime, Utc};
use reqwest::*;
use serde::{Deserialize, Serialize};
use server_fn::error::ServerFnError;
use uuid::Uuid;

use crate::auth::backend::*;
use crate::calendar::backend::utils::check_input_sensibility;
use crate::database::local::sync_local_db::sync_local_to_remote_db;
use crate::utils::{functions::*, structs::*};

#[derive(Debug, Deserialize, Serialize)]
struct NewCalendarEvent {
    summary: String,
    description: Option<String>,
    calendar_id: String,
    from_date_time: String,
    to_date_time: Option<String>,
    attachment: Option<String>,
    rrule: Option<String>,
    recurrence_until: Option<String>,
    recurrence_id: Option<String>,
    overrides_datetime: Option<String>,
    skipped: String,
    location: Option<String>,
    category: Option<String>,
    is_all_day: String,
}

// #[server]
pub async fn create_calendar_event(
    summary: String,
    description: Option<String>,
    calendar_id: Uuid,
    from_date_time: DateTime<Utc>,
    to_date_time: Option<DateTime<Utc>>,
    attachment: Option<String>,
    recurrence: Option<Recurrent>,
    recurrence_exception: Option<RecurrenceException>,
    location: Option<String>,
    categories: Option<Vec<String>>,
    is_all_day: bool,
) -> core::result::Result<(), ServerFnError> {
    match check_input_sensibility(
        summary.clone(),
        calendar_id,
        from_date_time,
        to_date_time,
        recurrence,
        recurrence_exception,
    )
    .await
    {
        Ok(()) => {
            match create_calendar_event_unchecked(
                summary,
                description,
                calendar_id,
                from_date_time,
                to_date_time,
                attachment,
                recurrence,
                recurrence_exception,
                location,
                categories,
                is_all_day,
            )
            .await
            {
                Err(e) => {
                    return Err(ServerFnError::new(format!(
                        "create_calendar_event_unchecked error: {}",
                        e
                    )));
                }
                Ok(s) => println!("Calendar Event Created. Status: {}", s),
            }
        }
        Err(e) => {
            return Err(ServerFnError::new(format!(
                "check_input_sensibility Error: {}",
                e
            )));
        }
    }
    sync_local_to_remote_db().await?;
    Ok(())
}

// #[server]
pub async fn create_calendar_event_unchecked(
    summary: String,
    description: Option<String>,
    calendar_id: Uuid,
    from_date_time: DateTime<Utc>,
    to_date_time: Option<DateTime<Utc>>,
    attachment: Option<String>,
    recurrence: Option<Recurrent>,
    recurrence_exception: Option<RecurrenceException>,
    location: Option<String>,
    categories: Option<Vec<String>>,
    is_all_day: bool,
) -> core::result::Result<StatusCode, ServerFnError> {
    // get the session token
    // println!("create_cal gestartet");
    let current_user = get_user_id_and_session_token().await?;
    // println!("user_id und token erstellt");
    // fit data into a NewCalendarEvent for building the json
    let new_cal_event = NewCalendarEvent {
        summary,
        description,
        calendar_id: calendar_id.to_string(),
        from_date_time: from_date_time.to_string(),
        to_date_time: to_date_time.map(|t| t.to_string()),
        attachment,
        rrule: recurrence
            .as_ref()
            .map(|r| r.rrule.to_string().to_lowercase()),
        recurrence_until: recurrence.as_ref().map(|r| r.recurrence_until.to_string()),
        recurrence_id: recurrence_exception
            .as_ref()
            .map(|r| r.recurrence_id.to_string()),
        overrides_datetime: match &recurrence_exception {
            Some(r) => r
                .overrides
                .as_ref()
                .map(|o| o.overrides_datetime.to_string()),
            None => None,
        },
        skipped: match &recurrence_exception {
            Some(r) => match &r.overrides {
                Some(o) => o.skipped.to_string(),
                None => false.to_string(),
            },
            None => false.to_string(),
        },
        location,
        category: categories.map(|c| c.join(", ")),
        is_all_day: is_all_day.to_string(),
    };
    // println!("new_cal_event erstellt");
    // Insert into database
    let url_events = format!("{}/rest/v1/calendar_events", SUPABASE_URL);
    let insert_event = reqwest::Client::new()
        .post(url_events)
        .bearer_auth(current_user.1)
        .header("apikey", ANON_KEY)
        .header("Content-Type", "application/json")
        .json(&new_cal_event)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    // println!("insert event durchgelaufen");

    if !insert_event.status().is_success() {
        println!(
            "Statuscode: {}\nText: {:?}",
            insert_event.status(),
            insert_event.text().await
        );
        return Err(ServerFnError::new(
            "Create calendar-event request not successful",
        ));
    }
    Ok(insert_event.status())
}

//Test:

// pub async fn test_create_cal_event() -> core::result::Result<(), ServerFnError> {
//     println!("Testfunktion gestartet");
//     let cal_id = match Uuid::parse_str("2e301e01-2d6a-4262-bf49-bc1000b2d57a") {
//         Ok(c) => c,
//         Err(e) => {
//             return Err(ServerFnError::new(format!("calendar_id Error: {}", e)));
//         }
//     };
//     let recurrence_id = match Uuid::parse_str("606e5574-f2bd-460b-888e-ac9bf9c7e817") {
//         Ok(c) => c,
//         Err(e) => {
//             return Err(ServerFnError::new(format!("calendar_id Error: {}", e)));
//         }
//     };
//     let date = Utc.with_ymd_and_hms(2027, 4, 8, 9, 10, 11).unwrap(); // `2014-07-08T09:10:11Z`

//     println!("vor xyz");
//     let xyz = create_calendar_event(
//         "Testevent 27".to_string(),
//         Some("to be deleted".to_string()),
//         cal_id,
//         date,
//         None,
//         None,
//         None,
//         None,
//         Some("wo anders".to_string()),
//         None,
//         true,
//     )
//     .await;
//     println!("Testfunktion durchgelaufen");
//     Ok(())
// }
