#![allow(dead_code)]
#![allow(unused_imports)]

use chrono::{DateTime, Datelike, Utc, Weekday};
use dioxus::prelude::*;
use reqwest::*;
use std::*;
use uuid::Uuid;

use crate::auth::backend::*;
use crate::{auth::backend::*, utils::structs::*};

// #[server]
pub async fn get_user_id_and_session_token() -> core::result::Result<(Uuid, String), ServerFnError>
{
    //Client holen und Auth checken
    // println!("get_user_zeug gestartet");
    let client = match get_client() {
        Ok(c) => c,
        Err(e) => return Err(ServerFnError::new(format!("get_client Error: {}", e))),
    };
    // println!("client erhalten");
    if !client.auth().is_authenticated() {
        return Err(ServerFnError::new(
            "get_user_id_and_session_token Error: User not authenticated.",
        ));
    }
    // println!("client authentifiziert");
    let user_id = client
        .current_user()
        .await
        .map_err(|e| ServerFnError::new(format!("get_user_id_and_session_token Error: {}", e)))?
        .unwrap()
        .id;
    //Token holen und zurückgeben
    let session = client
        .auth()
        .get_session()
        .map_err(|e| ServerFnError::new(format!("get_user_id_and_session_token Error: {}", e)))?;
    let token_str = session.access_token.clone();
    Ok((user_id, token_str))
}

pub async fn get_calendar_event_from_remote(
    id: Uuid,
) -> core::result::Result<CalendarEvent, ServerFnError> {
    let url_events = format!("{}/rest/v1/calendar_events?id=eq.{}", SUPABASE_URL, id);
    let response_event = get_elements_from_remote_by_url_string_unchecked(url_events).await?;
    let mut events = parse_response_string_to_calendar_events(response_event).await?;
    match events.pop() {
        None => {
            return Err(ServerFnError::new(format!(
                "get_calendar_event_from_remote Error: No element found"
            )));
        }
        Some(ev) => return Ok(ev),
    }
}

pub async fn get_calendar_events_by_recurrence_id(
    recurrence_id: Uuid,
) -> core::result::Result<Vec<CalendarEvent>, ServerFnError> {
    //get the data from the server
    let url_events = format!(
        "{}/rest/v1/calendar_events?recurrence_id=eq.{}",
        SUPABASE_URL, recurrence_id
    );
    let response_event_text = get_elements_from_remote_by_url_string_unchecked(url_events).await?;
    let cal_events = parse_response_string_to_calendar_events(response_event_text).await?;
    Ok(cal_events)
}

pub async fn get_calendar_events_ids_by_recurrence_id(
    rec_id: Uuid,
) -> core::result::Result<Vec<Uuid>, ServerFnError> {
    let url_events = format!(
        "{}/rest/v1/calendar_events?select=id&recurrence_id=eq.{}",
        SUPABASE_URL, rec_id
    );
    let response_ids_text = get_elements_from_remote_by_url_string_unchecked(url_events).await?;
    // Json in Vec von Ids parsen
    let ids: Vec<Uuid> = serde_json::from_str(&response_ids_text)
        .map_err(|e| ServerFnError::new(format!("JSON Parse Events: {}", e)))?;
    Ok(ids)
}

pub fn parse_calendar_event_to_light(event: CalendarEvent) -> CalendarEventLight {
    CalendarEventLight {
        id: event.id.to_string(),
        calendar_id: event.calendar_id.to_string(),
        summary: event.summary,
        description: event.description,
        from_date_time: event.from_date_time.to_string(),
        to_date_time: match event.to_date_time {
            Some(t) => Some(t.to_string()),
            None => None,
        },
        attachment: event.attachment,
        rrule: match event.recurrence.clone() {
            Some(r) => Some(r.rrule.to_string()),
            None => None,
        },
        recurrence_until: match event.recurrence {
            Some(r) => Some(r.recurrence_until.to_string()),
            None => None,
        },
        location: event.location,
        category: match event.categories {
            Some(c) => Some(c.join(", ")),
            None => None,
        },
        is_all_day: event.is_all_day,
        recurrence_id: match &event.recurrence_exception {
            Some(r) => Some(r.recurrence_id.to_string()),
            None => None,
        },
        overrides_datetime: match &event.recurrence_exception {
            Some(r) => match &r.overrides {
                Some(o) => Some(o.overrides_datetime.to_string()),
                None => None,
            },
            None => None,
        },
        skipped: match &event.recurrence_exception {
            Some(r) => match &r.overrides {
                Some(o) => o.skipped,
                None => false,
            },
            None => false,
        },
        created_at: event.created_at.to_string(),
        created_by: event.created_by.to_string(),
        last_mod: event.last_mod.to_string(),
    }
}

pub fn parse_calendar_event_light_to_calendar_event(
    ev_light: CalendarEventLight,
) -> core::result::Result<CalendarEvent, ServerFnError> {
    let id = Uuid::parse_str(&ev_light.id)
        .map_err(|e| ServerFnError::new(format!("Parsing Error: {}", e)))?;
    let calendar_id = Uuid::parse_str(&ev_light.calendar_id)
        .map_err(|e| ServerFnError::new(format!("Parsing Error: {}", e)))?;
    let created_at = ev_light
        .created_at
        .parse::<DateTime<chrono::FixedOffset>>()
        .map_err(|e| ServerFnError::new(format!("Parsing Error: {}", e)))?
        .with_timezone(&Utc);
    let created_by = Uuid::parse_str(&ev_light.created_by)
        .map_err(|e| ServerFnError::new(format!("Parsing Error: {}", e)))?;
    let from_date_time = ev_light
        .from_date_time
        .parse::<DateTime<chrono::FixedOffset>>()
        .map_err(|e| ServerFnError::new(format!("Parsing Error: {}", e)))?
        .with_timezone(&Utc);
    let to_date_time = match ev_light.to_date_time {
        Some(tdt) => Some(
            tdt.parse::<chrono::DateTime<Utc>>()
                .map_err(|e| ServerFnError::new(format!("Parsing Error: {}", e)))?,
        ),
        None => None,
    };
    let recurrence = match (ev_light.rrule, ev_light.recurrence_until) {
        (None, None) => None,
        (Some(rr), Some(ru)) => Some(Recurrent {
            rrule: rr
                .parse::<Rrule>()
                .map_err(|e| ServerFnError::new(format!("Parsing Error: {}", e)))?,
            recurrence_until: ru
                .parse::<chrono::DateTime<Utc>>()
                .map_err(|e| ServerFnError::new(format!("Parsing Error: {}", e)))?,
        }),

        _ => {
            return Err(ServerFnError::new(format!(
                "Parse CalendarEvent Logic Error: Recurrence needs rrule and recurrence_until"
            )));
        }
    };
    let overr = match ev_light.overrides_datetime {
        Some(or) => Some(Overrides {
            overrides_datetime: or
                .parse::<chrono::DateTime<Utc>>()
                .map_err(|e| ServerFnError::new(format!("Parsing Error: {}", e)))?,
            skipped: ev_light.skipped,
        }),
        None => None,
    };
    let recurrence_exception: Option<RecurrenceException> = match (ev_light.recurrence_id, overr) {
        (Some(ri), or) => Some(RecurrenceException {
            recurrence_id: ri
                .parse::<Uuid>()
                .map_err(|e| ServerFnError::new(format!("Parsing Error: {}", e)))?,
            overrides: or,
        }),
        _ => None,
    };
    match (&recurrence, &recurrence_exception) {
        (Some(_), Some(_)) => {
            return Err(ServerFnError::new(format!(
                "Parse CalendarEvent Logic Error: An Event must not be recurrent and a recurrence exception!"
            )));
        }
        (_, _) => {}
    }
    let categories: Option<Vec<String>> = match ev_light.category {
        None => None,
        Some(cat) => Some(cat.split(',').map(|c| c.trim().to_string()).collect()),
    };
    let last_mod: DateTime<Utc> = ev_light
        .last_mod
        .parse::<chrono::DateTime<Utc>>()
        .map_err(|e| ServerFnError::new(format!("Parsing Error: {}", e)))?;
    let cal_event = CalendarEvent {
        id,
        summary: ev_light.summary,
        description: ev_light.description,
        calendar_id,
        created_at,
        created_by,
        from_date_time,
        to_date_time,
        attachment: ev_light.attachment,
        recurrence,
        recurrence_exception,
        location: ev_light.location,
        categories,
        is_all_day: ev_light.is_all_day,
        last_mod,
    };
    Ok(cal_event)
}

/// used to get elements from the remote database. The string must lead to the supabase database table including the query.
/// it returns the .text element of the json as a string.
// #[server]
pub async fn get_elements_from_remote_by_url_string_unchecked(
    url_query: String,
) -> core::result::Result<String, ServerFnError> {
    let current_user = get_user_id_and_session_token().await?;
    //get data from remote
    let response_event = reqwest::Client::new()
        .get(&url_query)
        .header("apikey", ANON_KEY)
        .header("Authorization", format!("Bearer {}", current_user.1))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Http Request Error: {}", e)))?;
    if !response_event.status().is_success() {
        let err = response_event.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!(
            "Get From Supabase Error: {}",
            err
        )));
    }
    // pars response text into json string for simple parsing to struct
    let response_event_text = response_event
        .text()
        .await
        .map_err(|e| ServerFnError::new(format!("Text Error: {}", e)))?;
    Ok(response_event_text)
}

/// for parsing an string formated answer of an supabase query into an calendar_event.
// #[server]
pub async fn parse_response_string_to_calendar_events(
    response_event_text: String,
) -> core::result::Result<Vec<CalendarEvent>, ServerFnError> {
    let light_events: Vec<CalendarEventLight> = serde_json::from_str(&response_event_text)
        .map_err(|e| ServerFnError::new(format!("JSON Parse Events: {}", e)))?;
    let mut cal_events: Vec<CalendarEvent> = Vec::new();
    for light_ev in light_events {
        cal_events.push(parse_calendar_event_light_to_calendar_event(light_ev)?);
    }
    Ok(cal_events)
}

/// check for overriding recurrence exception if overrides_datetime is valid according to the recurrent element.
pub fn check_overriding_recurrence(
    child_overrides_dt: DateTime<Utc>,
    parent_from_dt: DateTime<Utc>,
    parent_recurrence_until: DateTime<Utc>,
    rrule: Rrule,
) -> bool {
    if parent_from_dt.time() != child_overrides_dt.time()
        || parent_from_dt >= child_overrides_dt
        || child_overrides_dt >= parent_recurrence_until
    {
        return false;
    }
    match rrule {
        Rrule::Daily => return true,
        Rrule::Weekly => {
            return (child_overrides_dt
                .signed_duration_since(parent_from_dt)
                .num_days()
                % 7)
                == 0;
        }
        Rrule::Fortnight => {
            return (child_overrides_dt
                .signed_duration_since(parent_from_dt)
                .num_days()
                % 14)
                == 0;
        }
        Rrule::Annual => {
            return (child_overrides_dt.day() == parent_from_dt.day()
                && child_overrides_dt.month() == parent_from_dt.month());
        }
        Rrule::MonthlyOnDate => return (child_overrides_dt.day() == parent_from_dt.day()),
        Rrule::MonthlyOnWeekday => {
            if (child_overrides_dt.day() <= 7 && parent_from_dt.day() <= 7)
                || (child_overrides_dt.day() > 7
                    && child_overrides_dt.day() <= 14
                    && parent_from_dt.day() > 7
                    && parent_from_dt.day() <= 14)
                || (child_overrides_dt.day() > 14
                    && child_overrides_dt.day() <= 21
                    && parent_from_dt.day() > 14
                    && parent_from_dt.day() <= 21)
                || (child_overrides_dt.day() > 21
                    && child_overrides_dt.day() <= 28
                    && parent_from_dt.day() > 21
                    && parent_from_dt.day() <= 28)
            {
                return (child_overrides_dt.weekday() == parent_from_dt.weekday());
            } else {
                false
            }
        }
        Rrule::OnWeekDays => return child_overrides_dt.weekday().num_days_from_monday() <= 5,
    }
}
