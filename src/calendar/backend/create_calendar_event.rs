use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use dioxus::prelude::*;
use reqwest::*;
use serde::{Deserialize, Serialize};
use supabase::client::*;
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
    recurrence_id: Option<Uuid>,
    location: Option<String>,
    categories: Option<Vec<String>>,
    is_all_day: bool,
) -> core::result::Result<(), ServerFnError> {
    match check_input_sensibility(
        summary.clone(),
        calendar_id.clone(),
        from_date_time.clone(),
        to_date_time.clone(),
        recurrence.clone(),
        recurrence_id.clone(),
    ) {
        Ok(()) => {
            match create_calendar_event_unchecked(
                summary,
                description,
                calendar_id,
                from_date_time,
                to_date_time,
                attachment,
                recurrence,
                recurrence_id,
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
                "chek_input_sensibility error: {}",
                e
            )));
        }
    }
    let _ = sync_local_to_remote_db();
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
    recurrence_id: Option<Uuid>,
    location: Option<String>,
    categories: Option<Vec<String>>,
    is_all_day: bool,
) -> core::result::Result<StatusCode, ServerFnError> {
    // get the session token
    // println!("create_cal gestartet");
    let current_user = match get_user_id_and_session_token().await {
        Ok(c) => c,
        Err(e) => {
            return Err(ServerFnError::new(format!(
                "get_session_token Error: {}",
                e
            )));
        }
    };
    // println!("user_id und token erstellt");
    // fit data into a NewCalendarEvent for building the json
    let new_cal_event = NewCalendarEvent {
        summary: summary,
        description: description,
        calendar_id: calendar_id.into(),
        from_date_time: from_date_time.to_string(),
        to_date_time: match to_date_time {
            Some(t) => Some(t.to_string()),
            None => None,
        },
        attachment: attachment.into(),
        rrule: match &recurrence {
            Some(r) => Some(r.rrule.to_string().to_lowercase()),
            None => None,
        },
        recurrence_until: match &recurrence {
            Some(r) => Some(r.recurrence_until.to_string()),
            None => None,
        },
        recurrence_id: match recurrence_id {
            Some(r) => Some(r.to_string()),
            None => None,
        },
        location: location.into(),
        category: match categories {
            Some(c) => Some(c.join(", ")),
            None => None,
        },
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
    Ok(insert_event.status())
}

//Test:

// pub async fn test_create_cal_event() -> core::result::Result<(), ServerFnError> {
//     println!("Testfunktion gestartet");
//     let cal_id = match Uuid::parse_str("fdb5cf9c-0a19-416b-aa92-330a474e1529") {
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
//         "Testevent 9".to_string(),
//         None,
//         cal_id,
//         date,
//         None,
//         None,
//         None,
//         Some(recurrence_id),
//         Some("wo anders".to_string()),
//         None,
//         true,
//     )
//     .await;
//     println!("Testfunktion durchgelaufen");
//     Ok(())
// }
