use crate::auth::backend::{ANON_KEY, SUPABASE_URL};
use crate::utils::structs::{CalendarEventLight, CalendarLight};
use dioxus::prelude::ServerFnError;
use sqlx::{Sqlite, Transaction};
use std::collections::HashSet;

pub async fn sync_calendars_and_events(
    tx: &mut Transaction<'_, Sqlite>,
    token: &str,
) -> Result<(), ServerFnError> {
    let http_client = reqwest::Client::new();
    let bearer_token = format!("Bearer {}", token);

    // Kalender laden
    println!("Loading Calendars...");

    //Config & Resonse von http-Anfrage
    let url_calendars = format!("{}/rest/v1/calendars?select=*", SUPABASE_URL);
    let response = http_client
        .get(&url_calendars)
        .header("apikey", ANON_KEY)
        .header("Authorization", &bearer_token)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Http Request Calendars Error: {}", e)))?;
    if !response.status().is_success() {
        let err = response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!(
            "Supabase Error Calendars: {}",
            err
        )));
    }

    //Response in Json parsen
    let response_text = response
        .text()
        .await
        .map_err(|e| ServerFnError::new(format!("Text Error: {}", e)))?;

    //Json in Vec von Calendars parsen
    let calendars: Vec<CalendarLight> = serde_json::from_str(&response_text)
        .map_err(|e| ServerFnError::new(format!("JSON Parse Calendars: {}", e)))?;

    //neues set aus Remote-DB Id's für Löschung von verwaisten Einträgen
    let mut remote_calendar_ids = HashSet::new();

    for c in calendars {
        //id in Set aus remote IDs speichern
        remote_calendar_ids.insert(c.id.clone());
        sqlx::query(
            r#"
            INSERT INTO calendars (
                id, name, type, description, owner_id, group_id, 
                created_at, created_by, last_mod
            ) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) 
            ON CONFLICT(id) DO UPDATE SET 
                name=excluded.name, 
                type=excluded.type, 
                description=excluded.description, 
                owner_id=excluded.owner_id,
                group_id=excluded.group_id,
                created_at=excluded.created_at,
                created_by=excluded.created_by,
                last_mod=excluded.last_mod
            "#,
        )
        .bind(c.id)
        .bind(c.name)
        .bind(c.calendar_type) // Rust Struct: calendar_type -> DB: type
        .bind(c.description)
        .bind(c.owner_id)
        .bind(c.group_id)
        .bind(c.created_at)
        .bind(c.created_by)
        .bind(c.last_mod)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Error Calendar: {}", e)))?;
    }

    // Cleanup Calendars
    //Vec aus lokalen Kalendern anhand ID
    let local_calendar_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM calendars")
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Local Cal IDs: {}", e)))?;

    //Ist locale ID nicht in remote ID -> löschen
    for local_id in local_calendar_ids {
        if !remote_calendar_ids.contains(&local_id) {
            println!("Deleting orphan calendar: {}", local_id);
            sqlx::query("DELETE FROM calendars WHERE id = ?")
                .bind(local_id)
                .execute(&mut **tx)
                .await
                .ok();
        }
    }

    //Events laden
    println!("Loading Calendar Events...");

    //Config & Resonse von http-Anfrage
    let url_events = format!("{}/rest/v1/calendar_events?select=*", SUPABASE_URL);
    let response_events = http_client
        .get(&url_events)
        .header("apikey", ANON_KEY)
        .header("Authorization", &bearer_token)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Http Request Events Error: {}", e)))?;
    if !response_events.status().is_success() {
        let err = response_events.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!(
            "Supabase Error Events: {}",
            err
        )));
    }

    //Response in Json parsen
    let response_events_text = response_events
        .text()
        .await
        .map_err(|e| ServerFnError::new(format!("Text Error: {}", e)))?;

    //Json in Vec von Events parsen
    let events: Vec<CalendarEventLight> = serde_json::from_str(&response_events_text)
        .map_err(|e| ServerFnError::new(format!("JSON Parse Events: {}", e)))?;

    //neues set aus Remote-DB Id's für Löschung von verwaisten Einträgen
    let mut remote_event_ids = HashSet::new();

    for e in events {
        //id in Set aus remote IDs speichern
        remote_event_ids.insert(e.id.clone());
        sqlx::query(
            r#"
            INSERT INTO calendar_events (
                id, calendar_id, summary, description, 
                from_date_time, to_date_time, attachment, rrule, 
                recurrence_until, location, category, is_all_day, 
                recurrence_id, created_at, created_by, last_mod
            ) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) 
            ON CONFLICT(id) DO UPDATE SET 
                calendar_id=excluded.calendar_id, 
                summary=excluded.summary, 
                description=excluded.description,
                from_date_time=excluded.from_date_time, 
                to_date_time=excluded.to_date_time, 
                attachment=excluded.attachment,
                rrule=excluded.rrule,
                recurrence_until=excluded.recurrence_until,
                location=excluded.location,
                category=excluded.category,
                is_all_day=excluded.is_all_day,
                recurrence_id=excluded.recurrence_id,
                created_at=excluded.created_at,
                created_by=excluded.created_by,
                last_mod=excluded.last_mod
            "#,
        )
        .bind(e.id)
        .bind(e.calendar_id)
        .bind(e.summary)
        .bind(e.description)
        .bind(e.from_date_time)
        .bind(e.to_date_time)
        .bind(e.attachment)
        .bind(e.rrule)
        .bind(e.recurrence_until)
        .bind(e.location)
        .bind(e.category)
        .bind(e.is_all_day)
        .bind(e.recurrence_id)
        .bind(e.created_at)
        .bind(e.created_by)
        .bind(e.last_mod)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Error Event: {}", e)))?;
    }

    // Cleanup Events
    //Vec aus lokalen Events anhand ID
    let local_event_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM calendar_events")
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Local Event IDs: {}", e)))?;

    //Ist locale ID nicht in remote ID -> löschen
    for ev_id in local_event_ids {
        if !remote_event_ids.contains(&ev_id) {
            sqlx::query("DELETE FROM calendar_events WHERE id = ?")
                .bind(ev_id)
                .execute(&mut **tx)
                .await
                .ok();
        }
    }

    Ok(())
}
