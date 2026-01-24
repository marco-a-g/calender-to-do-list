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

async fn delete_single_calendar_event_unchecked(
    event_id: Uuid,
) -> core::result::Result<StatusCode, ServerFnError> {
    let current_user = match get_user_id_and_session_token().await {
        Ok(c) => c,
        Err(e) => {
            return Err(ServerFnError::new(format!(
                "get_session_token Error: {}",
                e
            )));
        }
    };

    let url_events = format!(
        "{}/rest/v1/calendar_events?id=eq.{}",
        SUPABASE_URL, event_id
    );
    let delete_event = reqwest::Client::new()
        .delete(url_events)
        //.bearer_auth(current_user.1)
        .header("apikey", ANON_KEY)
        .header("Authorization", format!("Bearer {}", current_user.1))
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(delete_event.status())
}

// test

pub async fn test_delete() -> core::result::Result<(), ServerFnError> {
    println!("00");
    let id = Uuid::parse_str("c07ec80f-27f1-4366-8dbb-04518176665b").unwrap();
    println!("01");
    // let cal_id = Uuid::parse_str("fdb5cf9c-0a19-416b-aa92-330a474e1529").unwrap();
    // println!("03");
    // let create_at = Utc
    //     .with_ymd_and_hms(2026, 01, 23, 01, 0, 0)
    //     .unwrap()
    //     .to_utc();
    // // chrono::DateTime::parse_from_rfc3339("2026-01-23 00:00:00.63553+00")
    // //     .unwrap()
    // //     .into();
    // println!("04");
    // let created_by = Uuid::parse_str("03ef0d94-a65b-4e9a-ad74-012f32156444").unwrap();
    // println!("05");
    // let from_dt = Utc
    //     .with_ymd_and_hms(2027, 04, 08, 09, 10, 11)
    //     .unwrap()
    //     .to_utc();
    // // chrono::DateTime::parse_from_rfc3339("2027-04-08 09:10:11+00")
    // //     .unwrap()
    // //     .into();
    // println!("06");
    // let todat = Some(
    //     Utc.with_ymd_and_hms(2026, 01, 23, 20, 28, 26)
    //         .unwrap()
    //         .to_utc(),
    // );
    // // chrono::DateTime::parse_from_rfc3339("2026-01-23 20:28:26+00")
    // //     .unwrap()
    // //     .into(),
    // println!("07");
    // let runtil = Utc
    //     .with_ymd_and_hms(2026, 01, 23, 20, 28, 38)
    //     .unwrap()
    //     .to_utc();
    // // chrono::DateTime::parse_from_rfc3339("2026-01-23 20:28:38+00")
    // //     .unwrap()
    // //     .into();
    // println!("08");
    // let recid = Some(Uuid::parse_str("606e5574-f2bd-460b-888e-ac9bf9c7e817").unwrap());
    // println!("09");
    // let lamo = Utc
    //     .with_ymd_and_hms(2026, 01, 23, 0, 0, 0)
    //     .unwrap()
    //     .to_utc();
    // // chrono::DateTime::parse_from_rfc3339("2026-01-23 00:00:00.63553+00")
    // //     .unwrap()
    // //     .into();
    // println!("10");

    // let to_del = CalendarEvent {
    //     id: id,
    //     summary: "Testevent 8".to_string(),
    //     description: Some("egal".to_string()),
    //     calendar_id: cal_id,
    //     created_at: create_at,
    //     created_by: created_by,
    //     from_date_time: from_dt,
    //     to_date_time: todat,
    //     attachment: Some("blabla".to_string()),
    //     recurrence: Some(Recurrent {
    //         rrule: Rrule::Daily,
    //         recurrence_until: runtil,
    //     }),
    //     recurrence_id: recid,
    //     location: Some("wo anders".to_string()),
    //     categories: Some(vec!["texttext".to_string()]),
    //     is_all_day: true,
    //     last_mod: lamo,
    // };

    // match  {
    //     Ok(c) => c,
    //     Err(e) => {
    //         return Err(ServerFnError::new(format!("calendar_id Error: {}", e)));
    //     }
    // };
    let deletion = delete_single_calendar_event_unchecked(id).await?;
    println!("Deleted with status: {}", deletion);
    Ok(())
}
