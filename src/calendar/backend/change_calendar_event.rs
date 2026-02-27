use chrono::{Datelike, Days, NaiveTime, Utc};
use dioxus::prelude::*;
use reqwest::*;
use serde::{Deserialize, Serialize};
use server_fn::error::ServerFnError;

use crate::auth::backend::*;
use crate::calendar::backend::create_calendar_event::create_calendar_event;
use crate::calendar::backend::{delete_calendar_event::*, utils::check_input_sensibility};
use crate::database::local::sync_local_db::sync_local_to_remote_db;
use crate::utils::{functions::*, structs::*};

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

/// For changing a single instance of an recurrent event. To delete  an instance use delete_instance_of_recurrent_event().
// #[server]
pub async fn edit_instance_of_recurrent_event(
    instance: CalendarEvent,
) -> core::result::Result<(), ServerFnError> {
    //check validity of new version itself
    check_input_sensibility(
        instance.summary.clone(),
        instance.calendar_id,
        instance.from_date_time,
        instance.to_date_time,
        instance.recurrence,
        instance.recurrence_exception,
    )
    .await?;
    if let Some(rec_ex) = instance.recurrence_exception {
        if let Some(overr) = rec_ex.overrides {
            if overr.skipped {
                return delete_instance_of_recurrent_event(
                    rec_ex.recurrence_id,
                    overr.overrides_datetime,
                    None,
                    None,
                )
                .await;
            }
        }
    }
    create_calendar_event(
        instance.summary,
        instance.description,
        instance.calendar_id,
        instance.from_date_time,
        instance.to_date_time,
        instance.attachment,
        None,
        instance.recurrence_exception,
        instance.location,
        instance.categories,
        instance.is_all_day,
    )
    .await
}

/// For making changes to a calendar event. Mind that for a recurrent event you may not change the beginning of the recurrent event and the frequency within one change.
// #[server]
pub async fn edit_calendar_event(
    new_version: CalendarEvent,
    keep_overridings: Option<bool>, // set this to true if recurrence exceptions that override dates that leave the range of the events recurrence, defaults to false
    keep_orphans: Option<bool>, // set this to true to keep recurrence exceptions as single elements if the event is turned to non-recurrent, defaults to false
) -> core::result::Result<(), ServerFnError> {
    //check validity of new version itself
    check_input_sensibility(
        new_version.summary.clone(),
        new_version.calendar_id,
        new_version.from_date_time,
        new_version.to_date_time,
        new_version.recurrence,
        new_version.recurrence_exception,
    )
    .await?;

    // get old version from the server to compare for changes that effect other elements because of recurrence
    let old_version = get_calendar_event_from_remote(new_version.id).await?;
    let mut to_be_del: Vec<CalendarEvent> = Vec::new();
    let mut to_non_override: Vec<CalendarEvent> = Vec::new();
    let mut to_be_orphaned: Vec<CalendarEvent> = Vec::new();

    // check, whether the recurrence is changed in a way that needs to check the exceptions
    if let (Some(new_recurrence), Some(old_recurrence)) =
        (new_version.recurrence, old_version.recurrence)
    {
        if new_recurrence.recurrence_until < old_recurrence.recurrence_until
            || new_version.from_date_time != old_version.from_date_time
            || new_recurrence.rrule != old_recurrence.rrule
        {
            //only allow changing either the rrule or from_date_time
            if new_recurrence.rrule != old_recurrence.rrule
                && new_version.from_date_time.date_naive()
                    != old_version.from_date_time.date_naive()
            {
                return Err(ServerFnError::new(
                    "Due to unknown expectations it is not possible to change the starting date and the recurrence rule in the same step.",
                ));
            }

            //get all changed instances
            let url_query = format!(
                "{}/rest/v1/calendar_events?recurrence_id=eq.{}",
                SUPABASE_URL, new_version.id,
            );
            let perhaps_need_change =
                get_elements_from_remote_by_url_string_unchecked(url_query).await?;
            let perhaps_need_change_vec =
                parse_response_string_to_calendar_events(perhaps_need_change).await?;

            // only recurrence_until is changed
            if new_version.from_date_time == old_version.from_date_time
                && new_recurrence.rrule == old_recurrence.rrule
            {
                for instance in perhaps_need_change_vec {
                    if let Some(rec_ex) = instance.recurrence_exception {
                        if let Some(over) = rec_ex.overrides {
                            if over.overrides_datetime > new_recurrence.recurrence_until {
                                if over.skipped {
                                    to_be_del.push(instance);
                                } else if keep_overridings == Some(true) {
                                    to_non_override.push(instance);
                                }
                            }
                        }
                    }
                }
            }
            // rrule and / or from_date_time is changed
            else {
                let mut to_be_shifted: Vec<CalendarEvent> = Vec::new();
                for instance in perhaps_need_change_vec {
                    if let Some(rec_ex) = instance.recurrence_exception.as_ref() {
                        if let Some(over) = rec_ex.overrides.as_ref() {
                            if over.overrides_datetime
                                <= new_version
                                    .from_date_time
                                    .with_time(NaiveTime::MIN)
                                    .unwrap()
                                || over.overrides_datetime > new_recurrence.recurrence_until
                            {
                                match (keep_overridings, over.skipped) {
                                    (Some(true), false) => to_non_override.push(instance),
                                    _ => to_be_del.push(instance),
                                }
                            } else {
                                to_be_shifted.push(instance);
                            }
                        }
                    }
                }
                for shifty in to_be_shifted {
                    let overr = shifty.recurrence_exception.unwrap().overrides.unwrap(); //can safely be unwraped for we checked before if it is an overriding exception
                    let mut odt = overr.overrides_datetime;
                    let mut from_dt = shifty.from_date_time;
                    let mut rec_ex: Option<RecurrenceException> = None;

                    //in case rrule did not change
                    if new_recurrence.rrule == old_recurrence.rrule {
                        match new_recurrence.rrule {
                            Rrule::Daily => {
                                // in case, the time was not changed in the exception it should also not be changed according to the recurrent event after the shift
                                if odt.time() == from_dt.time() {
                                    from_dt = from_dt
                                        .with_time(new_version.from_date_time.time())
                                        .unwrap();
                                }
                                //set odt to the new time
                                odt = odt.with_time(new_version.from_date_time.time()).unwrap();
                                rec_ex = Some(RecurrenceException {
                                    recurrence_id: new_version.id,
                                    overrides: Some(Overrides {
                                        overrides_datetime: odt,
                                        skipped: overr.skipped,
                                    }),
                                });
                            }
                            Rrule::Weekly => {
                                // in case, time and weekday were not changed in the exception they should also not be changed according to the recurrent event after the shift
                                if odt.time() == from_dt.time()
                                    && odt.weekday() == from_dt.weekday()
                                {
                                    from_dt = (from_dt
                                        - chrono::Duration::days(
                                            from_dt.weekday().num_days_from_monday().into(),
                                        )
                                        + chrono::Duration::days(
                                            new_version
                                                .from_date_time
                                                .weekday()
                                                .num_days_from_monday()
                                                .into(),
                                        ))
                                    .with_time(new_version.from_date_time.time())
                                    .unwrap();
                                }

                                //set odt to the new date and time
                                odt =
                                    (odt - chrono::Duration::days(
                                        from_dt.weekday().num_days_from_monday().into(),
                                    ) + chrono::Duration::days(
                                        new_version
                                            .from_date_time
                                            .weekday()
                                            .num_days_from_monday()
                                            .into(),
                                    ))
                                    .with_time(new_version.from_date_time.time())
                                    .unwrap();
                                rec_ex = Some(RecurrenceException {
                                    recurrence_id: new_version.id,
                                    overrides: Some(Overrides {
                                        overrides_datetime: odt,
                                        skipped: overr.skipped,
                                    }),
                                });
                            }
                            Rrule::OnWeekDays => {
                                // in case, the time was not changed in the exception it should also not be changed according to the recurrent event after the shift
                                if odt == from_dt {
                                    from_dt = from_dt
                                        .with_time(new_version.from_date_time.time())
                                        .unwrap();
                                }
                                //set odt to the new time
                                odt = odt.with_time(new_version.from_date_time.time()).unwrap();
                                rec_ex = Some(RecurrenceException {
                                    recurrence_id: new_version.id,
                                    overrides: Some(Overrides {
                                        overrides_datetime: odt,
                                        skipped: overr.skipped,
                                    }),
                                });
                            }
                            Rrule::Fortnight => {
                                // in case, time and weekday were not changed in the exception they should also not be changed according to the recurrent event after the shift
                                let time_dif_to_rec =
                                    (odt - new_version.from_date_time).num_days() % 14;
                                if odt.time() == from_dt.time()
                                    && (from_dt - odt).num_days().abs() % 14 == 0
                                {
                                    from_dt = from_dt
                                        .checked_add_days(Days::new(
                                            time_dif_to_rec.try_into().unwrap(),
                                        ))
                                        .unwrap()
                                        .with_time(new_version.from_date_time.time())
                                        .unwrap();
                                }
                                //set odt to the new date and time
                                odt = odt
                                    .checked_add_days(Days::new(
                                        time_dif_to_rec.try_into().unwrap(),
                                    ))
                                    .unwrap()
                                    .with_time(new_version.from_date_time.time())
                                    .unwrap();
                                rec_ex = Some(RecurrenceException {
                                    recurrence_id: new_version.id,
                                    overrides: Some(Overrides {
                                        overrides_datetime: odt,
                                        skipped: overr.skipped,
                                    }),
                                });
                            }
                            Rrule::MonthlyOnDate => {
                                // in case, time and weekday were not changed in the exception they should also not be changed according to the recurrent event after the shift
                                if odt.time() == from_dt.time() && from_dt.day() == odt.day() {
                                    from_dt = from_dt
                                        .with_day(new_version.from_date_time.day())
                                        .unwrap_or(
                                            //unwrap should work except it is the last day of the month and the current month is shorter. this cannot happen to december so the unwraps within should be without problem
                                            from_dt
                                                .with_day(
                                                    from_dt
                                                        .with_month(from_dt.month() + 1)
                                                        .unwrap()
                                                        .with_day(1)
                                                        .unwrap()
                                                        .checked_sub_days(Days::new(1))
                                                        .unwrap()
                                                        .day(),
                                                )
                                                .unwrap(),
                                        )
                                }
                                //set odt to the new date and time
                                odt = odt
                                    .with_day(new_version.from_date_time.day())
                                    .unwrap_or(
                                        //unwrap should work except it is the last day of the month and the current month is shorter. this cannot happen to december so the unwraps within should be without problem
                                        odt.with_day(
                                            odt.with_month(odt.month() + 1)
                                                .unwrap()
                                                .with_day(1)
                                                .unwrap()
                                                .checked_sub_days(Days::new(1))
                                                .unwrap()
                                                .day(),
                                        )
                                        .unwrap(),
                                    )
                                    .with_time(new_version.from_date_time.time())
                                    .unwrap();

                                rec_ex = Some(RecurrenceException {
                                    recurrence_id: new_version.id,
                                    overrides: Some(Overrides {
                                        overrides_datetime: odt,
                                        skipped: overr.skipped,
                                    }),
                                });
                            }
                            Rrule::MonthlyOnWeekday => {
                                let new_rrule_parts = (
                                    new_version.from_date_time.weekday().num_days_from_monday(),
                                    new_version.from_date_time.day() / 7,
                                );
                                let from_mon = from_dt
                                    .with_day(1)
                                    .unwrap()
                                    .weekday()
                                    .num_days_from_monday();
                                let mut new_day: u32 = 1;
                                if from_mon <= new_rrule_parts.0 {
                                    new_day = odt
                                        .with_day(1)
                                        .unwrap()
                                        .checked_add_days(Days::new(
                                            (new_rrule_parts.0 - from_mon) as u64,
                                        ))
                                        .unwrap()
                                        .day()
                                        + (7 * new_rrule_parts.1)
                                }
                                // in case, time and weekday were not changed in the exception they should also not be changed according to the recurrent event after the shift
                                if odt == from_dt {
                                    from_dt = from_dt
                                        .with_day(new_day)
                                        .unwrap_or(from_dt.with_day(new_day - 7).unwrap())
                                        .with_time(new_version.from_date_time.time())
                                        .unwrap();
                                }
                                //set odt to the new date and time
                                odt = odt
                                    .with_day(new_day)
                                    .unwrap_or(odt.with_day(new_day - 7).unwrap())
                                    .with_time(new_version.from_date_time.time())
                                    .unwrap();

                                rec_ex = Some(RecurrenceException {
                                    recurrence_id: new_version.id,
                                    overrides: Some(Overrides {
                                        overrides_datetime: odt,
                                        skipped: overr.skipped,
                                    }),
                                });
                            }
                            Rrule::Annual => {
                                // in case, time and weekday were not changed in the exception they should also not be changed according to the recurrent event after the shift
                                if odt == from_dt {
                                    from_dt = new_version
                                        .from_date_time
                                        .with_year(from_dt.year())
                                        .unwrap_or(
                                            //unwrap should work except from_date_time of the parent event is the 29.2.
                                            from_dt
                                                .with_month(2)
                                                .unwrap()
                                                .with_day(28)
                                                .unwrap()
                                                .with_time(new_version.from_date_time.time())
                                                .unwrap(),
                                        );
                                }
                                //set odt to the new date and time
                                odt = odt
                                    .with_day(new_version.from_date_time.day())
                                    .unwrap_or(
                                        //unwrap should work except it is the last day of the month and the current month is shorter. this cannot happen to december so the unwraps within should be without problem
                                        odt.with_day(
                                            odt.with_month(odt.month() + 1)
                                                .unwrap()
                                                .with_day(1)
                                                .unwrap()
                                                .checked_sub_days(Days::new(1))
                                                .unwrap()
                                                .day(),
                                        )
                                        .unwrap(),
                                    )
                                    .with_time(new_version.from_date_time.time())
                                    .unwrap();

                                rec_ex = Some(RecurrenceException {
                                    recurrence_id: new_version.id,
                                    overrides: Some(Overrides {
                                        overrides_datetime: odt,
                                        skipped: overr.skipped,
                                    }),
                                });
                            }
                        }
                    }
                    // if rrule was changed
                    else if check_overriding_recurrence(
                        odt,
                        new_version.from_date_time,
                        new_recurrence.recurrence_until,
                        new_recurrence.rrule,
                    ) {
                        // in case, the time was not changed in the exception it should also not be changed according to the recurrent event after the shift
                        if odt.time() == from_dt.time() {
                            from_dt = from_dt
                                .with_time(new_version.from_date_time.time())
                                .unwrap();
                        }
                        //set odt to the new time
                        odt = odt.with_time(new_version.from_date_time.time()).unwrap();
                        rec_ex = Some(RecurrenceException {
                            recurrence_id: new_version.id,
                            overrides: Some(Overrides {
                                overrides_datetime: odt,
                                skipped: overr.skipped,
                            }),
                        });
                    } else if let Some(true) = keep_overridings {
                        to_non_override.push(shifty.clone());
                    } else {
                        to_be_del.push(shifty.clone());
                    }
                    edit_calendar_event_unchecked(CalendarEvent {
                        id: shifty.id,
                        summary: shifty.summary,
                        description: shifty.description,
                        calendar_id: shifty.calendar_id,
                        created_at: shifty.created_at,
                        created_by: shifty.created_by,
                        from_date_time: from_dt,
                        to_date_time: shifty.to_date_time,
                        attachment: shifty.attachment,
                        recurrence: None,
                        recurrence_exception: rec_ex,
                        location: shifty.location,
                        categories: shifty.categories,
                        is_all_day: shifty.is_all_day,
                        last_mod: Utc::now(),
                    })
                    .await?;
                }
            }
        }
    }
    // in case, the element is switched to non-recurrent, check for depending events to handle them
    if old_version.recurrence.is_some() && new_version.recurrence.is_none() {
        let to_be_deleted_or_orphaned =
            get_calendar_events_by_recurrence_id(new_version.id).await?;
        for child in to_be_deleted_or_orphaned {
            if let Some(rec_ex) = &child.recurrence_exception {
                match (&rec_ex.overrides, keep_overridings, keep_orphans) {
                    (Some(over), Some(true), Some(true)) => {
                        if over.skipped {
                            to_be_del.push(child);
                        } else {
                            to_be_orphaned.push(child);
                        }
                    }
                    (Some(over), _, _) => to_be_del.push(child),
                    (None, _, Some(true)) => to_be_orphaned.push(child),
                    _ => to_be_del.push(child),
                }
            } else {
                return Err(ServerFnError::new(format!(
                    "Unexpected Error: The CalendarEvent {} was either parsed wrong or is in an invalid state",
                    child.id
                )));
            }
        }
    }
    for delete in to_be_del {
        delete_single_calendar_event(delete.id).await?;
    }
    for orphanise in to_be_orphaned {
        edit_calendar_event_unchecked(CalendarEvent {
            id: orphanise.id,
            summary: orphanise.summary,
            description: orphanise.description,
            calendar_id: orphanise.calendar_id,
            created_at: orphanise.created_at,
            created_by: orphanise.created_by,
            from_date_time: orphanise.from_date_time,
            to_date_time: orphanise.to_date_time,
            attachment: orphanise.attachment,
            recurrence: orphanise.recurrence,
            recurrence_exception: None,
            location: orphanise.location,
            categories: orphanise.categories,
            is_all_day: orphanise.is_all_day,
            last_mod: Utc::now(),
        })
        .await?;
    }
    for non_over in to_non_override {
        edit_calendar_event_unchecked(CalendarEvent {
            id: non_over.id,
            summary: non_over.summary,
            description: non_over.description,
            calendar_id: non_over.calendar_id,
            created_at: non_over.created_at,
            created_by: non_over.created_by,
            from_date_time: non_over.from_date_time,
            to_date_time: non_over.to_date_time,
            attachment: non_over.attachment,
            recurrence: non_over.recurrence,
            recurrence_exception: Some(RecurrenceException {
                recurrence_id: new_version.id,
                overrides: None,
            }),
            location: non_over.location,
            categories: non_over.categories,
            is_all_day: non_over.is_all_day,
            last_mod: Utc::now(),
        })
        .await?;
    }
    edit_calendar_event_unchecked(new_version).await?;
    sync_local_to_remote_db().await?;
    Ok(())
}

///used for changing a calendar event, that is a non-recurrent event (before) and not an recurrence exception
// #[server]
pub async fn edit_single_calendar_event(
    new_version: CalendarEvent,
) -> core::result::Result<(), ServerFnError> {
    //check wether it really was a single calendar event
    let old_version = get_calendar_event_from_remote(new_version.id).await?;
    if old_version.recurrence.is_some() {
        return Err(ServerFnError::new(
            "Mismatching Event: The CalendarEvent to be altered is not a single event.",
        ));
    };
    if old_version.recurrence_exception.is_some() {
        return Err(ServerFnError::new(
            "Mismatching Event: The CalendarEvent to be altered is instance of a recurrent event.",
        ));
    };

    //check validity of new version itself
    check_input_sensibility(
        new_version.summary.clone(),
        new_version.calendar_id,
        new_version.from_date_time,
        new_version.to_date_time,
        new_version.recurrence,
        new_version.recurrence_exception,
    )
    .await?;

    let stat = edit_calendar_event_unchecked(new_version.clone()).await?;
    let uploaded = get_calendar_event_from_remote(new_version.id).await?;
    if new_version.description != uploaded.description
        || new_version.from_date_time != uploaded.from_date_time
        || new_version.to_date_time != uploaded.to_date_time
        || new_version.attachment != uploaded.attachment
        || new_version.recurrence != uploaded.recurrence
        || new_version.location != uploaded.location
        || new_version.categories != uploaded.categories
        || new_version.is_all_day != uploaded.is_all_day
        || new_version.recurrence_exception != uploaded.recurrence_exception
    {
        return Err(ServerFnError::new(
            "Changing CalendarEvent went wrong. Unexpected Error.",
        ));
    };
    sync_local_to_remote_db().await?;
    Ok(())
}

// #[server]
pub async fn edit_calendar_event_unchecked(
    changed_event: CalendarEvent,
) -> core::result::Result<StatusCode, ServerFnError> {
    // get the session token
    let current_user = get_user_id_and_session_token().await?;
    // fit data into a NewCalendarEvent for building the json
    let new_cal_event = CalendarEventUp {
        summary: changed_event.summary,
        description: changed_event.description,
        from_date_time: changed_event.from_date_time.to_string(),
        to_date_time: changed_event.to_date_time.map(|t| t.to_string()),
        attachment: changed_event.attachment,
        rrule: changed_event
            .recurrence
            .as_ref()
            .map(|r| r.rrule.to_string().to_lowercase()),
        recurrence_until: changed_event
            .recurrence
            .as_ref()
            .map(|r| r.recurrence_until.to_string()),
        recurrence_id: changed_event
            .recurrence_exception
            .as_ref()
            .map(|r| r.recurrence_id.to_string()),
        overrides_datetime: match &changed_event.recurrence_exception {
            Some(r) => r
                .overrides
                .as_ref()
                .map(|o| o.overrides_datetime.to_string()),
            None => None,
        },
        skipped: match &changed_event.recurrence_exception {
            Some(r) => match &r.overrides {
                Some(o) => o.skipped.to_string(),
                None => false.to_string(),
            },
            None => false.to_string(),
        },
        location: changed_event.location,
        category: changed_event.categories.map(|c| c.join(", ")),
        is_all_day: changed_event.is_all_day.to_string(),
    };

    let url_events = format!("{}/rest/v1/calendar_events", SUPABASE_URL);
    let insert_event = reqwest::Client::new()
        .patch(url_events)
        .query(&[("id", format!("eq.{}", changed_event.id))])
        .header("apikey", ANON_KEY)
        .header("Authorization", format!("Bearer {}", current_user.1))
        .header("Content-Type", "application/json")
        .json(&new_cal_event)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let status = insert_event.status();
    if !status.is_success() {
        return Err(ServerFnError::new(format!(
            "Editing went wrong: insert failed with status code: {}",
            status
        )));
    }
    Ok(status)
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
