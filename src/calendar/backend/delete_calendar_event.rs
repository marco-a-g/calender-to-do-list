use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use dioxus::prelude::*;
use reqwest::*;
use serde::{Deserialize, Serialize};
use supabase::client::*;
use uuid::Uuid;

use crate::auth::backend::*;
use crate::calendar::backend::utils::{check_deleted, check_input_sensibility};
use crate::database::local::sync_local_db::sync_local_to_remote_db;
use crate::utils::{functions::*, structs::*};

// #[server]
pub async fn delete_calendar_event_without_sub_events(
    event_id: Uuid,
) -> core::result::Result<(), ServerFnError> {
    // TODO: check, if element is recurrent
    // if so, delete recurrence id of these elements with event_id as recurrence_id
    Ok(())
}

///used to delete an (recurrent or non recurrent) calendar_event completely with all instances.
// #[server]
pub async fn delete_calendar_event_with_sub_events(
    event_id: Uuid,
) -> core::result::Result<(), ServerFnError> {
    // check wether event is recurrent and delete element and instances
    if let Some(parent_recurrent) = get_calendar_event_from_remote(event_id).await?.recurrence {
        let mut children = get_calendar_events_ids_by_recurrence_id(event_id).await?;
        children.push(event_id);
        let mut deleted: Vec<(Uuid, StatusCode)> = Vec::new();
        for id in children {
            let stat = delete_single_calendar_event_unchecked(id).await?;
            deleted.push((id, stat));
        }
        //check if elemnts were really deleted
        let mut failed_to_delete: Vec<(Uuid, StatusCode, ServerFnError)> = Vec::new();
        for hopefully_gone in deleted {
            match check_deleted(hopefully_gone.0, hopefully_gone.1).await {
                Err(e) => failed_to_delete.push((hopefully_gone.0, hopefully_gone.1, e)),
                Ok(()) => {}
            }
        }
        if failed_to_delete.len() != 0 {
            return Err(ServerFnError::new(format!(
                "Failed to delete the following elements (id, StatusCode, Error): {:?}",
                failed_to_delete
            )));
        }
        sync_local_to_remote_db().await?;
        return Ok(());
    }
    //element non-recurrent
    else {
        delete_single_calendar_event(event_id).await
    }
}

/// to delete a non-recurrent element. Will return an Error if the element is recurrent.
// #[server]
pub async fn delete_single_calendar_event(
    event_id: Uuid,
) -> core::result::Result<(), ServerFnError> {
    // check, if element is not recurrent
    let remote_event = get_calendar_event_from_remote(event_id).await?;
    match remote_event.recurrence {
        Some(_) => {
            return Err(ServerFnError::new(format!(
                "delete_single_calendar_event Error: CalendarEvent with id: {:?} is recurrent",
                event_id
            )));
        }
        None => {}
    }
    // delete element
    let delete = delete_single_calendar_event_unchecked(event_id).await?;

    // check wether deletion was successful
    check_deleted(event_id, delete).await?;

    // sync
    sync_local_to_remote_db().await?;
    Ok(())
}

//returns 204 No Content even when delete is successful so an additional approval is necessary.
// #[server]
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

// pub async fn test_delete() -> core::result::Result<(), ServerFnError> {
//     println!("00");
//     let id = Uuid::parse_str("08b6bbfd-1519-420d-bef6-f23d9146894d").unwrap();
//     println!("01");
//     let deletion = delete_single_calendar_event(id).await?;
//     //println!("Deleted with status: {}", deletion);
//     Ok(())
// }
