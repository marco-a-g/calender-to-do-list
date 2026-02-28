//! Functions for deleting calendar events.

use chrono::{DateTime, Datelike, Days, Months, Utc};
use reqwest::*;
use server_fn::error::ServerFnError;
use uuid::Uuid;

use crate::auth::backend::*;
use crate::calendar::backend::change_calendar_event::*;
use crate::calendar::backend::create_calendar_event::*;
use crate::calendar::backend::utils::check_deleted;
use crate::database::local::sync_local_db::sync_local_to_remote_db;
use crate::utils::date_handling::calculate_next_date;
use crate::utils::{functions::*, structs::*};

/// Deletes an Instance of an recurrent event. In case this is the last regular instance of the recurrent event the whole event will be deleted.
/// In this case pass_on_to will be used as new parent event (and therefore set to recurrent if it is not) for remaining instances.
/// If pass_on_to is None, will be turned into single CalendarEvents (if keep_orphans == true) or they will be deleted (otherwise).
// #[server]
#[allow(clippy::diverging_sub_expression)] 
pub async fn delete_instance_of_recurrent_event(
    recurrent_event_id: Uuid,
    instance_date: DateTime<Utc>,
    pass_on_to: Option<CalendarEvent>,
    keep_orphans: Option<bool>,
) -> core::result::Result<(), ServerFnError> {
    let mut cur_instance_date = instance_date;

    loop {
        let rec_event = get_calendar_event_from_remote(recurrent_event_id).await?;

        let url_events = format!(
            "{}/rest/v1/calendar_events?recurrence_id=eq.{}&overrides_datetime=eq.{}",
            SUPABASE_URL, recurrent_event_id, cur_instance_date
        );
        let response_event_text =
            get_elements_from_remote_by_url_string_unchecked(url_events).await?;
        let mut exception_event =
            parse_response_string_to_calendar_events(response_event_text).await?;

        // check if this is the only regular instance and handle orphans
        if rec_event.from_date_time == rec_event.recurrence.unwrap().recurrence_until {
            let exceptions = get_calendar_events_by_recurrence_id(recurrent_event_id).await?;
            if !exceptions.is_empty() {
                if let Some(p_o_t) = pass_on_to {
                    if p_o_t.recurrence.is_none() {
                        edit_calendar_event(
                            CalendarEvent {
                                id: p_o_t.id,
                                summary: p_o_t.summary,
                                description: p_o_t.description,
                                calendar_id: p_o_t.calendar_id,
                                created_at: p_o_t.created_at,
                                created_by: p_o_t.created_by,
                                from_date_time: p_o_t.from_date_time,
                                to_date_time: p_o_t.to_date_time,
                                attachment: p_o_t.attachment,
                                recurrence: Some(Recurrent {
                                    rrule: Rrule::Daily,
                                    recurrence_until: p_o_t.from_date_time,
                                }),
                                recurrence_exception: None,
                                location: p_o_t.location,
                                categories: p_o_t.categories,
                                is_all_day: p_o_t.is_all_day,
                                last_mod: Utc::now(),
                            },
                            None,
                            None,
                        )
                        .await?;
                    }
                    for adopted in exceptions {
                        if adopted.recurrence_exception.unwrap().overrides.is_some() {
                            delete_single_calendar_event(adopted.id).await?;
                        } else {
                            edit_calendar_event_unchecked(CalendarEvent {
                                id: adopted.id,
                                summary: adopted.summary,
                                description: adopted.description,
                                calendar_id: adopted.calendar_id,
                                created_at: adopted.created_at,
                                created_by: adopted.created_by,
                                from_date_time: adopted.from_date_time,
                                to_date_time: adopted.to_date_time,
                                attachment: adopted.attachment,
                                recurrence: None,
                                recurrence_exception: Some(RecurrenceException {
                                    recurrence_id: p_o_t.id,
                                    overrides: None,
                                }),
                                location: adopted.location,
                                categories: adopted.categories,
                                is_all_day: adopted.is_all_day,
                                last_mod: Utc::now(),
                            })
                            .await?;
                        }
                    }
                } else if let Some(true) = keep_orphans {
                    for adopted in exceptions {
                        if adopted.recurrence_exception.unwrap().overrides.is_some() {
                            delete_single_calendar_event_unchecked(adopted.id).await?;
                        } else {
                            edit_calendar_event_unchecked(CalendarEvent {
                                id: adopted.id,
                                summary: adopted.summary,
                                description: adopted.description,
                                calendar_id: adopted.calendar_id,
                                created_at: adopted.created_at,
                                created_by: adopted.created_by,
                                from_date_time: adopted.from_date_time,
                                to_date_time: adopted.to_date_time,
                                attachment: adopted.attachment,
                                recurrence: None,
                                recurrence_exception: None,
                                location: adopted.location,
                                categories: adopted.categories,
                                is_all_day: adopted.is_all_day,
                                last_mod: Utc::now(),
                            })
                            .await?;
                        }
                    }
                } else {
                    for kill in exceptions {
                        delete_single_calendar_event(kill.id).await?;
                    }
                }
            }
            let status = delete_single_calendar_event_unchecked(recurrent_event_id).await?;
            // check wether deletion was successful
            check_deleted(recurrent_event_id, status).await?;
            sync_local_to_remote_db().await?;
            return Ok(());
        }
        if cur_instance_date == rec_event.from_date_time {
            if !exception_event.is_empty() {
                let excep = exception_event.pop().unwrap();
                delete_single_calendar_event(excep.id).await?;
            }
            let next_date = calculate_next_date(
                rec_event.from_date_time,
                &rec_event.recurrence.unwrap().rrule.to_string(),
                rec_event.from_date_time,
            )
            .map_err(|e| ServerFnError::new(e.to_string()))?;

            let mut end = rec_event.to_date_time;
            if let Some(to_dt) = rec_event.to_date_time {
                end = Some(
                    calculate_next_date(
                        to_dt,
                        &rec_event.recurrence.unwrap().rrule.to_string(),
                        to_dt,
                    )
                    .map_err(|e| ServerFnError::new(e.to_string()))?,
                );
            }

            edit_calendar_event(
                CalendarEvent {
                    id: recurrent_event_id,
                    summary: rec_event.summary,
                    description: rec_event.description,
                    calendar_id: rec_event.calendar_id,
                    created_at: rec_event.created_at,
                    created_by: rec_event.created_by,
                    from_date_time: next_date,
                    to_date_time: end,
                    attachment: rec_event.attachment,
                    recurrence: rec_event.recurrence,
                    recurrence_exception: None,
                    location: rec_event.location,
                    categories: rec_event.categories,
                    is_all_day: rec_event.is_all_day,
                    last_mod: Utc::now(),
                },
                None,
                None,
            )
            .await?;

            // in case the next instance was already deleted before, meaning it is overridden and skipped, it should be deleted completely
            let url = format!(
                "{}/rest/v1/calendar_events?recurrence_id=eq.{}&overrides_datetime=eq.{}&skipped=eq.true",
                SUPABASE_URL, recurrent_event_id, next_date
            );
            let response_event_text = get_elements_from_remote_by_url_string_unchecked(url).await?;
            let skippy = parse_response_string_to_calendar_events(response_event_text).await?;
            if skippy.is_empty() {
                break;
            } else {
                cur_instance_date = next_date;
            }
        } else if cur_instance_date == rec_event.recurrence.unwrap().recurrence_until {
            if !exception_event.is_empty() {
                let excep = exception_event.pop().unwrap();
                delete_single_calendar_event(excep.id).await?;
            }
            let weekday_number_old = (
                rec_event
                    .recurrence
                    .unwrap()
                    .recurrence_until
                    .weekday()
                    .num_days_from_monday(),
                rec_event.recurrence.unwrap().recurrence_until.day() / 7,
            );
            let first_last_month = rec_event
                .recurrence
                .unwrap()
                .recurrence_until
                .checked_sub_months(Months::new(1))
                .unwrap()
                .with_day(1)
                .unwrap();
            #[allow(unused)]
            let date_before = match rec_event.recurrence.unwrap().rrule {
                Rrule::Daily => rec_event
                    .recurrence
                    .unwrap()
                    .recurrence_until
                    .checked_sub_days(Days::new(1))
                    .unwrap_or(return Err(ServerFnError::new("Rrule Error: Finding the previous element was not possible due to chrono missmatch"))),
                Rrule::Weekly => rec_event
                    .recurrence
                    .unwrap()
                    .recurrence_until
                    .checked_sub_days(Days::new(7))
                    .unwrap_or(return Err(ServerFnError::new("Rrule Error: Finding the previous element was not possible due to chrono missmatch"))),
                Rrule::Fortnight => rec_event
                    .recurrence
                    .unwrap()
                    .recurrence_until
                    .checked_sub_days(Days::new(14))
                    .unwrap_or(return Err(ServerFnError::new("Rrule Error: Finding the previous element was not possible due to chrono missmatch"))),
                Rrule::MonthlyOnDate => rec_event
                    .recurrence
                    .unwrap()
                    .recurrence_until
                    .checked_sub_months(Months::new(1))
                    .unwrap_or(return Err(ServerFnError::new("Rrule Error: Finding the previous element was not possible due to chrono missmatch"))),
                Rrule::Annual => rec_event
                    .recurrence
                    .unwrap()
                    .recurrence_until
                    .checked_sub_months(Months::new(12))
                    .unwrap_or(return Err(ServerFnError::new("Rrule Error: Finding the previous element was not possible due to chrono missmatch"))),
                Rrule::OnWeekDays => {
                    if rec_event
                        .recurrence
                        .unwrap()
                        .recurrence_until
                        .weekday()
                        .num_days_from_monday()
                        == 0
                    {
                        rec_event
                            .recurrence
                            .unwrap()
                            .recurrence_until
                            .checked_sub_days(Days::new(3))
                            .unwrap_or(return Err(ServerFnError::new("Rrule Error: Finding the previous element was not possible due to chrono missmatch")))
                    } else if rec_event
                        .recurrence
                        .unwrap()
                        .recurrence_until
                        .weekday()
                        .num_days_from_monday()
                        == 6
                    {
                        rec_event
                            .recurrence
                            .unwrap()
                            .recurrence_until
                            .checked_sub_days(Days::new(2))
                            .unwrap_or(return Err(ServerFnError::new("Rrule Error: Finding the previous element was not possible due to chrono missmatch")))
                    } else {
                        rec_event
                            .recurrence
                            .unwrap()
                            .recurrence_until
                            .checked_sub_days(Days::new(1))
                            .unwrap_or(return Err(ServerFnError::new("Rrule Error: Finding the previous element was not possible due to chrono missmatch")))
                    }
                }
                Rrule::MonthlyOnWeekday => {
                    if weekday_number_old.0 >= first_last_month.weekday().num_days_from_monday() {
                        first_last_month
                            .checked_add_days(Days::new(
                                ((7 * weekday_number_old.1)
                                    + (weekday_number_old.0
                                        - first_last_month.weekday().num_days_from_monday()))
                                    as u64,
                            ))
                            .unwrap_or(
                                first_last_month
                                    .checked_add_days(Days::new(
                                        ((7 * weekday_number_old.1)
                                            + (weekday_number_old.0
                                                - first_last_month
                                                    .weekday()
                                                    .num_days_from_monday())
                                            - 7) as u64,
                                    ))
                                    .unwrap_or(return Err(ServerFnError::new("Rrule Error: Finding the previous element was not possible due to chrono missmatch"))),
                            )
                    } else {
                        first_last_month
                            .checked_add_days(Days::new(
                                ((7 * weekday_number_old.1)
                                    - (first_last_month.weekday().num_days_from_monday()
                                        - weekday_number_old.0))
                                    as u64,
                            ))
                            .unwrap_or(
                                first_last_month
                                    .checked_add_days(Days::new(
                                        ((7 * weekday_number_old.1)
                                            - (first_last_month.weekday().num_days_from_monday()
                                                - weekday_number_old.0)
                                            - 7) as u64,
                                    ))
                                    .unwrap_or(return Err(ServerFnError::new("Rrule Error: Finding the previous element was not possible due to chrono missmatch"))),
                            )
                    }
                }
            };
            edit_calendar_event(
                CalendarEvent {
                    id: recurrent_event_id,
                    summary: rec_event.summary,
                    description: rec_event.description,
                    calendar_id: rec_event.calendar_id,
                    created_at: rec_event.created_at,
                    created_by: rec_event.created_by,
                    from_date_time: rec_event.from_date_time,
                    to_date_time: rec_event.to_date_time,
                    attachment: rec_event.attachment,
                    recurrence: Some(Recurrent {
                        rrule: rec_event.recurrence.unwrap().rrule,
                        recurrence_until: date_before,
                    }),
                    recurrence_exception: None,
                    location: rec_event.location,
                    categories: rec_event.categories,
                    is_all_day: rec_event.is_all_day,
                    last_mod: Utc::now(),
                },
                None,
                None,
            )
            .await?;

            // in case the instance before the current was already deleted before, meaning it is overridden and skipped, it should be deleted completely
            let url = format!(
                "{}/rest/v1/calendar_events?recurrence_id=eq.{}&overrides_datetime=eq.{}&skipped=eq.true",
                SUPABASE_URL, recurrent_event_id, date_before
            );
            let response_event_text = get_elements_from_remote_by_url_string_unchecked(url).await?;
            let skippy = parse_response_string_to_calendar_events(response_event_text).await?;
            if skippy.is_empty() {
                break;
            }
        } else if !exception_event.is_empty() {
            let excep = exception_event.pop().unwrap();
            edit_calendar_event_unchecked(CalendarEvent {
                id: excep.id,
                summary: excep.summary,
                description: excep.description,
                calendar_id: excep.calendar_id,
                created_at: excep.created_at,
                created_by: excep.created_by,
                from_date_time: excep
                    .recurrence_exception
                    .unwrap()
                    .overrides
                    .unwrap()
                    .overrides_datetime,
                to_date_time: excep.to_date_time,
                attachment: excep.attachment,
                recurrence: None,
                recurrence_exception: Some(RecurrenceException {
                    recurrence_id: excep.recurrence_exception.unwrap().recurrence_id,
                    overrides: Some(Overrides {
                        overrides_datetime: excep
                            .recurrence_exception
                            .unwrap()
                            .overrides
                            .unwrap()
                            .overrides_datetime,
                        skipped: true,
                    }),
                }),
                location: excep.location,
                categories: excep.categories,
                is_all_day: excep.is_all_day,
                last_mod: Utc::now(),
            })
            .await?;
            return Ok(());
        } else {
            return create_calendar_event(
                "nothing".to_string(),
                None,
                rec_event.calendar_id,
                cur_instance_date,
                None,
                None,
                None,
                Some(RecurrenceException {
                    recurrence_id: rec_event.id,
                    overrides: Some(Overrides {
                        overrides_datetime: cur_instance_date,
                        skipped: true,
                    }),
                }),
                None,
                None,
                false,
            )
            .await;
        }
    }
    Ok(())
}

/// deletes an (recurrent) event and turns all changed instances into single events
// #[server]
#[allow(unused)]
pub async fn delete_calendar_event_without_changed_instances(
    event_id: Uuid,
) -> core::result::Result<(), ServerFnError> {
    // check wether event is recurrent
    if get_calendar_event_from_remote(event_id)
        .await?
        .recurrence
        .is_some()
    {
        let children = get_calendar_events_by_recurrence_id(event_id).await?;
        let mut orphanage: Vec<(Uuid, StatusCode)> = Vec::new();
        let mut to_be_deleted: Vec<Uuid> = Vec::new();
        to_be_deleted.push(event_id);
        //check instances for skipped for deletion or orphaning
        for child in children {
            if let Some(excep) = child.recurrence_exception {
                if let Some(overr) = excep.overrides {
                    if overr.skipped {
                        to_be_deleted.push(child.id);
                    } else {
                        let orphan = CalendarEvent {
                            id: child.id,
                            summary: child.summary,
                            description: child.description,
                            calendar_id: child.calendar_id,
                            created_at: child.created_at,
                            created_by: child.created_by,
                            from_date_time: child.from_date_time,
                            to_date_time: child.to_date_time,
                            attachment: child.attachment,
                            recurrence: child.recurrence,
                            recurrence_exception: None,
                            location: child.location,
                            categories: child.categories,
                            is_all_day: child.is_all_day,
                            last_mod: Utc::now(),
                        };
                        let orphaned = edit_calendar_event_unchecked(orphan).await?;
                        orphanage.push((child.id, orphaned));
                    }
                }
            }
        }
        let mut not_orphaned: Vec<(Uuid, StatusCode, ServerFnError)> = Vec::new();
        //check orphanage worked
        for orphan in orphanage {
            if get_calendar_event_from_remote(orphan.0)
                .await?
                .recurrence_exception
                .is_some()
            {
                not_orphaned.push((
                    orphan.0,
                    orphan.1,
                    ServerFnError::new("orphaning did not work"),
                ))
            }
        }
        if !not_orphaned.is_empty() {
            return Err(ServerFnError::new(format!(
                "delete_calendar_event_without_changed Error: {:?}",
                not_orphaned
            )));
        }

        // delete event and skipped instances
        let mut deleted: Vec<(Uuid, StatusCode)> = Vec::new();
        for id in to_be_deleted {
            let stat = delete_single_calendar_event_unchecked(id).await?;
            deleted.push((id, stat));
        }
        //check if elemnts were really deleted
        let mut failed_to_delete: Vec<(Uuid, StatusCode, ServerFnError)> = Vec::new();
        for hopefully_gone in deleted {
            if let Err(e) = check_deleted(hopefully_gone.0, hopefully_gone.1).await {
                failed_to_delete.push((hopefully_gone.0, hopefully_gone.1, e))
            }
        }
        if !failed_to_delete.is_empty() {
            return Err(ServerFnError::new(format!(
                "Failed to delete the following elements (id, StatusCode, Error): {:?}",
                failed_to_delete
            )));
        }
        sync_local_to_remote_db().await?;
        Ok(())
    }
    //element non-recurrent
    else {
        delete_single_calendar_event(event_id).await
    }
}

///used to delete an (recurrent or non recurrent) calendar_event completely with all instances.
// #[server]
pub async fn delete_calendar_event_with_all_instances(
    event_id: Uuid,
) -> core::result::Result<(), ServerFnError> {
    // check wether event is recurrent and delete element and instances
    if get_calendar_event_from_remote(event_id)
        .await?
        .recurrence
        .is_some()
    {
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
            if let Err(e) = check_deleted(hopefully_gone.0, hopefully_gone.1).await {
                failed_to_delete.push((hopefully_gone.0, hopefully_gone.1, e))
            }
        }
        if !failed_to_delete.is_empty() {
            return Err(ServerFnError::new(format!(
                "Failed to delete the following elements (id, StatusCode, Error): {:?}",
                failed_to_delete
            )));
        }
        sync_local_to_remote_db().await?;
        Ok(())
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
    if remote_event.recurrence.is_some() {
        return Err(ServerFnError::new(format!(
            "delete_single_calendar_event Error: CalendarEvent with id: {:?} is recurrent",
            event_id
        )));
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
