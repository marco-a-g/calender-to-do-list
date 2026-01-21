use chrono::{DateTime, Local, NaiveDate, Utc};
use dioxus::prelude::*;
use reqwest::*;
use serde::{Deserialize, Serialize};
use supabase::client::*;
use uuid::Uuid;

use crate::auth::backend::{ANON_KEY, SUPABASE_URL, get_client};
use crate::utils::{functions::*, structs::*};

#[derive(Debug, Deserialize, Serialize)]
pub struct NewCalendarEvent {
    pub id: Option<Uuid>,
    pub summary: String,
    pub description: Option<String>,
    pub calendar_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub from_date_time: DateTime<Utc>,
    pub to_date_time: Option<DateTime<Utc>>,
    pub attachment: Option<String>, //the path, regularly the web address, of a (shared) folder
    pub recurrence: Option<Recurrent>, // see explanation at "Recurrent"
    pub recurrence_id: Option<Uuid>, // see explanation at "Recurrent"
    pub location: Option<String>,
    pub categories: Option<Vec<String>>, // used to add tags to the event
    pub is_all_day: bool,
    pub last_mod: DateTime<Utc>,
}

//#[server]
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
) -> core::result::Result<() /*Uuid*/, ServerFnError> {
    let current_user = match get_user_id_and_session_token().await {
        Ok(c) => c,
        Err(e) => {
            return Err(ServerFnError::new(format!(
                "get_session_token Error: {}",
                e
            )));
        }
    };
    let bearer_token = format!("Bearer {}", current_user.1);

    let new_cal_event = NewCalendarEvent {
        id: None,
        summary: summary,
        description: description,
        calendar_id: calendar_id,
        created_at: Utc::now(),
        created_by: current_user.0,
        from_date_time: from_date_time,
        to_date_time: to_date_time,
        attachment: attachment,
        recurrence: recurrence,
        recurrence_id: recurrence_id,
        location: location,
        categories: categories,
        is_all_day: is_all_day,
        last_mod: Utc::now(),
    };

    let url_events = format!("{}/rest/v1/calendar_events", SUPABASE_URL);
    let insert_event = reqwest::Client::new()
        .post(url_events)
        .header("apikey", ANON_KEY)
        .header("Content-Type", "application/json")
        .header("Authorization", &bearer_token)
        .json(&new_cal_event)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

//Test:

// pub async fn test_creat_cal_event() -> core::result::Result<() /*Uuid*/, ServerFnError> {
//     // let calendar_id = Uuid::parse_str("fdb5cf9c - 0a19 - 416b - aa92 - 330a474e1529")?;
//     let calendar_id = match Uuid::parse_str("fdb5cf9c - 0a19 - 416b - aa92 - 330a474e1529") {
//         Ok(c) => c,
//         Err(e) => {
//             return Err(ServerFnError::new(format!("calendar_id Error: {}", e)));
//         }
//     };
//     let created_by = match Uuid::parse_str("fdb5cf9c - 0a19 - 416b - aa92 - 330a474e1529") {
//         Ok(c) => c,
//         Err(e) => {
//             return Err(ServerFnError::new(format!("created_by Error: {}", e)));
//         }
//     };
//     create_calendar_event(
//         "Testevent".to_string(),
//         Some("Beschreibung".to_string()),
//         calendar_id,
//         created_by,
//         from_date_time,
//         to_date_time,
//         attachment,
//         recurrence,
//         recurrence_id,
//         location,
//         categories,
//         is_all_day,
//     )
//     .await
// }
