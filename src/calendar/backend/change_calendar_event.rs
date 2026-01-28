use std::num::NonZeroI64;

use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use dioxus::prelude::*;
use reqwest::*;
use serde::{Deserialize, Serialize};
use serde_json::error::Category;
use supabase::client::*;
use uuid::Uuid;

use crate::auth::backend::*;
use crate::calendar::backend::utils::check_input_sensibility;
use crate::database::local::sync_local_db::sync_local_to_remote_db;
use crate::utils::{functions::*, structs::*};

///

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CalendarEventUp {
    // pub calendar_id: String,
    pub summary: String,
    pub description: Option<String>,
    pub from_date_time: String,
    pub to_date_time: Option<String>,
    pub attachment: Option<String>,
    pub rrule: Option<String>,
    pub recurrence_until: Option<String>,
    pub location: Option<String>,
    pub category: Option<String>,
    pub is_all_day: String,
    pub recurrence_id: Option<String>,
    pub overrides_datetime: Option<String>,
    pub skipped: String,
}

// // #[server]
// pub async fn change_calendar_event(
//     //check validity of new version itself
//     new_version: CalendarEvent,
// ) -> core::result::Result<(), ServerFnError> {
//     match check_input_sensibility(
//         new_version.summary,
//         new_version.calendar_id,
//         new_version.from_date_time,
//         new_version.to_date_time,
//         new_version.recurrence,
//         new_version.recurrence_exception,
//     ) {
//         Err(e) => {
//             return Err(ServerFnError::new(format!(
//                 "change_calendar_event Error: {}",
//                 e
//             )));
//         }
//         Ok(_) => {}
//     };

//     // get old version from the server to compare for changes that effect other elements because of recurrence
//     let old_version =
//     Ok(())
// }

// #[server]
pub async fn change_calendar_event_unchecked(
    changed_event: CalendarEvent,
) -> core::result::Result<StatusCode, ServerFnError> {
    // get the session token
    let current_user = get_user_id_and_session_token().await?;
    // fit data into a NewCalendarEvent for building the json
    let new_cal_event = CalendarEventUp {
        summary: changed_event.summary,
        description: changed_event.description,
        from_date_time: changed_event.from_date_time.to_string(),
        to_date_time: match changed_event.to_date_time {
            Some(t) => Some(t.to_string()),
            None => None,
        },
        attachment: changed_event.attachment.into(),
        rrule: match &changed_event.recurrence {
            Some(r) => Some(r.rrule.to_string().to_lowercase()),
            None => None,
        },
        recurrence_until: match &changed_event.recurrence {
            Some(r) => Some(r.recurrence_until.to_string()),
            None => None,
        },
        recurrence_id: match &changed_event.recurrence_exception {
            Some(r) => Some(r.recurrence_id.to_string()),
            None => None,
        },
        overrides_datetime: match &changed_event.recurrence_exception {
            Some(r) => match &r.overrides {
                Some(o) => Some(o.overrides_datetime.to_string()),
                None => None,
            },
            None => None,
        },
        skipped: match &changed_event.recurrence_exception {
            Some(r) => match &r.overrides {
                Some(o) => o.skipped.to_string(),
                None => false.to_string(),
            },
            None => false.to_string(),
        },
        location: changed_event.location.into(),
        category: match changed_event.categories {
            Some(c) => Some(c.join(", ")),
            None => None,
        },
        is_all_day: changed_event.is_all_day.to_string(),
    };

    let url_events = format!("{}/rest/v1/calendar_events", SUPABASE_URL);
    let insert_event = reqwest::Client::new()
        .patch(url_events)
        .query(&[("id", format!("eq.{}", changed_event.id))])
        //.bearer_auth(current_user.1)
        .header("apikey", ANON_KEY)
        .header("Authorization", format!("Bearer {}", current_user.1))
        .header("Content-Type", "application/json")
        .json(&new_cal_event)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(insert_event.status())
}

// //Test:

// pub async fn test_change_cal_event() -> core::result::Result<(), ServerFnError> {
//     println!("Testfunktion gestartet");
//     let id = match Uuid::parse_str("1ba43313-181c-4c6a-ab98-3d808dc02fa9") {
//         Ok(c) => c,
//         Err(e) => {
//             return Err(ServerFnError::new(format!("calendar_id Error: {}", e)));
//         }
//     };
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
//     let created_at = Utc.with_ymd_and_hms(2027, 4, 8, 9, 10, 11).unwrap();
//     let date = Utc.with_ymd_and_hms(2026, 4, 8, 9, 10, 11).unwrap(); // `2014-07-08T09:10:11Z`
//     println!("vor xyz");
//     let xyz = change_calendar_event_unchecked(CalendarEvent {
//         id,
//         summary: "Testevent 42".to_string(),
//         description: Some("was changed once again".to_string()),
//         calendar_id: cal_id,
//         created_at,
//         created_by: id,
//         from_date_time: date,
//         to_date_time: None,
//         attachment: Some("anderes anhängsel".to_string()),
//         recurrence: None,
//         recurrence_exception: None,
//         location: Some("wo anders".to_string()),
//         categories: Some(vec!["Hallo".to_string(), "Welt!".to_string()]),
//         is_all_day: false,
//         last_mod: date,
//     })
//     .await;
//     println!("Testfunktion durchgelaufen mit {}", xyz.unwrap());
//     Ok(())
// }
