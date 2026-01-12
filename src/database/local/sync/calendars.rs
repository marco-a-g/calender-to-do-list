#![allow(unused_variables)]

use crate::utils::structs::{CalendarEventLight, CalendarLight};
use dioxus::prelude::ServerFnError;
use sqlx::{Sqlite, Transaction};
use std::collections::HashSet;
use supabase::Client;

pub async fn sync_calendars_and_events(
    client: &Client,
    tx: &mut Transaction<'_, Sqlite>,
    user_id: &str,
    user_group_ids: &Vec<String>,
) -> Result<(), ServerFnError> {
    // Kalender laden
    println!("Loading Calendars...");
    let calendars_as_json = client
        .database()
        .from("calendars")
        .select("*")
        .execute()
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Calendars Error: {}", e)))?;

    //Kalender in Vec parsen
    let calendars: Vec<CalendarLight> =
        serde_json::from_value(serde_json::Value::Array(calendars_as_json))
            .map_err(|e| ServerFnError::new(format!("JSON Parse Calendars: {}", e)))?;

    //temporäres set mit den validen keys der Kalender -> für später bei ToDos und Events
    let mut valid_calendar_ids = HashSet::new();
    //temporäres set mit den keys der remote Kalender
    let mut remote_calendar_ids = HashSet::new();

    //über Vec mit Kalendern itterieren und in local db (erst in tx, noch nicht direkt speichern -> in Änderungsqueue) speichern
    for c in calendars {
        valid_calendar_ids.insert(c.id.clone());
        remote_calendar_ids.insert(c.id.clone());
        sqlx::query(
            r#"
            INSERT INTO calendars (
                id, name, type, description, owner_id, group_id, 
                created_by, created_at, last_mod
            ) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) 
            ON CONFLICT(id) DO UPDATE SET 
                name=excluded.name, 
                type=excluded.type, 
                description=excluded.description, 
                owner_id=excluded.owner_id, 
                group_id=excluded.group_id, 
                created_by=excluded.created_by,
                created_at=excluded.created_at,
                last_mod=excluded.last_mod
            "#,
        )
        .bind(c.id)
        .bind(c.name)
        .bind(c.calendar_type) // Verwendet das umbenannte Feld aus structs.rs
        .bind(c.description)
        .bind(c.owner_id)
        .bind(c.group_id)
        .bind(c.created_by)
        .bind(c.created_at)
        .bind(c.last_mod)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Error Calendar: {}", e)))?;
    }

    // Cleanup: Kalender die user nicht betreffen entfernen
    //set aus localen ids erstellen
    let local_calendar_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM calendars")
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Local Cal IDs: {}", e)))?;

    //sind local ids nicht in remote_ids -> löschen
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
    println!("Loading Events...");

    //Set in einen Vektor um, damit Supabase ihn als Filter benutzen kann
    let valid_calender_ids_vec: Vec<&str> = valid_calendar_ids.iter().map(|s| s.as_str()).collect();

    //Request nur starten, wenn überhaupt Kalender existieren
    let event_as_json: Vec<serde_json::Value> = if valid_calender_ids_vec.is_empty() {
        vec![]
    } else {
        client
            .database()
            .from("calendar_events")
            .select("*")
            .r#in("calendar_id", &valid_calender_ids_vec)
            .execute()
            .await
            .map_err(|e| ServerFnError::new(format!("Fetch Events Error: {}", e)))?
    };

    //Events in Vec parsen
    let events: Vec<CalendarEventLight> =
        serde_json::from_value(serde_json::Value::Array(event_as_json))
            .map_err(|e| ServerFnError::new(format!("JSON Parse Events: {}", e)))?;

    //temporäres set mit den keys der remote Events
    let mut remote_event_ids = HashSet::new();

    //über Vec mit Events itterieren und in local db (erst in tx, noch nicht direkt speichern -> in Änderungsqueue) speichern
    for e in events {
        remote_event_ids.insert(e.id.clone());
        sqlx::query(
            r#"
            INSERT INTO calendar_events (
                id, calendar_id, summary, description, 
                from_date_time, to_date_time, is_all_day, 
                location, category, attachment,
                rrule, recurrence_id, recurrence_until,
                created_by, created_at, last_mod
            ) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET 
                summary=excluded.summary, 
                description=excluded.description, 
                from_date_time=excluded.from_date_time, 
                to_date_time=excluded.to_date_time, 
                is_all_day=excluded.is_all_day,
                location=excluded.location,
                category=excluded.category,
                attachment=excluded.attachment,
                rrule=excluded.rrule, 
                recurrence_id=excluded.recurrence_id,
                recurrence_until=excluded.recurrence_until,
                created_by=excluded.created_by,
                created_at=excluded.created_at,
                last_mod=excluded.last_mod
        "#,
        )
        .bind(e.id)
        .bind(e.calendar_id)
        .bind(e.summary)
        .bind(e.description)
        .bind(e.from_date_time)
        .bind(e.to_date_time)
        .bind(e.is_all_day)
        .bind(e.location)
        .bind(e.category)
        .bind(e.attachment)
        .bind(e.rrule)
        .bind(e.recurrence_id)
        .bind(e.recurrence_until)
        .bind(e.created_by)
        .bind(e.created_at)
        .bind(e.last_mod)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Error Event: {}", e)))?;
    }

    // Cleanup: local Event ids laden
    let local_event_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM calendar_events")
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Local Event IDs: {}", e)))?;
    // Cleanup: local Event ids nicht in remote event ids -> löschen
    for local_id in local_event_ids {
        if !remote_event_ids.contains(&local_id) {
            println!("Deleting orphan event: {}", local_id);
            sqlx::query("DELETE FROM calendar_events WHERE id = ?")
                .bind(local_id)
                .execute(&mut **tx)
                .await
                .ok();
        }
    }

    Ok(())
}
