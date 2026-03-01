//! Helper functions for handling `CalendarEvent`s

use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use server_fn::error::ServerFnError;
use uuid::Uuid;

use crate::utils::functions::{check_overriding_recurrence, get_calendar_event_from_remote};
use crate::utils::structs::*;

/// Maximum length for the summary of a `CalendarEvent`
const SUMMARY_MAX_LENGTH: usize = 25;

/// Validating input for CalendarEvent
///
/// Only checks for sensibility of the input of the event itself, no checking of parent or child events.
///
/// ## Arguments
/// - `summary`- checked for `SUMMARY_MAX_LENGTH`
/// - `from_date_time`- beginning of the event
/// - `to_date_time`- end of the event
/// - `recurrence`- parameters for recurrent events
/// - `recurrence_exception`- used when the event is an instance of a recurrent event differing from the regular instances or to "delete" an instance within the recurrent event
///
/// ## Errors
/// Any error occuring will be handed on as a ServerFnError to fit the dioxus server function structure.
///
pub async fn check_input_sensibility(
    summary: String,
    // description: Option<String>,
    // _calendar_id: Uuid,
    from_date_time: DateTime<Utc>,
    to_date_time: Option<DateTime<Utc>>,
    // attachment: Option<String>,
    recurrence: Option<Recurrent>,
    recurrence_exception: Option<RecurrenceException>,
    // location: Option<String>,
    // categories: Option<Vec<String>>,
    // is_all_day: bool,
) -> Result<(), ServerFnError> {
    if summary.is_empty() {
        return Err(ServerFnError::new("Summary must not be empty".to_string()));
    }
    if summary.len() > SUMMARY_MAX_LENGTH {
        return Err(ServerFnError::new(format!(
            "Summary is too long. It must not exceed {}",
            SUMMARY_MAX_LENGTH
        )));
    }
    if let Some(end) = to_date_time {
        if end < from_date_time {
            return Err(ServerFnError::new(
                "The end of the event is before the beginning".to_string(),
            ));
        }
    }
    if let Some(rec) = recurrence {
        if rec.recurrence_until < from_date_time {
            return Err(ServerFnError::new(
                "The end of the recurrence is before the beginning of the event.".to_string(),
            ));
        }
        if recurrence_exception.is_some() {
            return Err(ServerFnError::new(
                "An event can only either be recurrent or a recurrence exception, not both.",
            ));
        }
    }
    if let Some(rec_ex) = recurrence_exception {
        // TODO: check if recurrence_id refers to an recurrent event.
        let parent = get_calendar_event_from_remote(rec_ex.recurrence_id).await?;
        if let Some(rec) = parent.recurrence {
            if let Some(over) = rec_ex.overrides {
                if !check_overriding_recurrence(
                    over.overrides_datetime,
                    parent.from_date_time,
                    rec.recurrence_until,
                    rec.rrule,
                ) {
                    return Err(ServerFnError::new(
                        "On this DateTime is no instance of the recurrent event to be overridden",
                    ));
                }
            } else {
                return Err(ServerFnError::new(
                    "There cannot be an RecurrenceException to a non recurrent event.",
                ));
            }
        }
    }
    core::result::Result::Ok(())
}

/// to check if an event was really deleted in supabase.
///
/// ## Arguments
/// - `id`- the `Uuid` of the event that is expected to have been deleted
/// - `status`- the status code that was returned on deletion.
///
/// ## Errors
/// Any error occuring will be handed on as a ServerFnError to fit the dioxus server function structure.
///
pub async fn check_deleted(id: Uuid, status: reqwest::StatusCode) -> Result<(), ServerFnError> {
    let sc = reqwest::StatusCode::from_u16(204)
        .map_err(|e| ServerFnError::new(format!("Delete Error: {}", e)))?;
    if status == sc {
        let hopefully_gone = get_calendar_event_from_remote(id).await;
        if hopefully_gone.is_ok() {
            return Err(ServerFnError::new(format!(
                "Deletion Error: Failed to delete the following element: {:?}",
                id
            )));
        }
    } else {
        return Err(ServerFnError::new(format!(
            "Deletion Error: unexpected Status: {}",
            status
        )));
    }
    Ok(())
}
