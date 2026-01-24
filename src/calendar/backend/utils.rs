use chrono::{DateTime, Local, TimeZone, Utc};
use dioxus::prelude::ServerFnError;
use uuid::Uuid;

use crate::auth::backend::*;
use crate::utils::structs::*;

const summary_max_length: usize = 25;

pub fn check_input_sensibility(
    summary: String,
    // description: Option<String>,
    calendar_id: Uuid,
    from_date_time: DateTime<Utc>,
    to_date_time: Option<DateTime<Utc>>,
    // attachment: Option<String>,
    recurrence: Option<Recurrent>,
    recurrence_exception: Option<RecurrenceException>,
    // location: Option<String>,
    // categories: Option<Vec<String>>,
    // is_all_day: bool,
) -> Result<(), ServerFnError> {
    if summary.len() < 1 {
        return Err(ServerFnError::new("Summary must not be empty".to_string()));
    }
    if summary.len() > summary_max_length {
        return Err(ServerFnError::new(format!(
            "Summary is to long. It must not exceed {}",
            summary_max_length
        )));
    }
    // TODO: check if calendar_id is accessible for this user
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
        if let Some(rec_ex) = recurrence_exception {
            return Err(ServerFnError::new(
                "An event can only either be recurrent or a recurrence exception, not both.",
            ));
        }
    }
    if let Some(rec_ex) = recurrence_exception {
        // TODO: check if recurrence_id refers to an recurrent event.
    }
    core::result::Result::Ok(())
}
