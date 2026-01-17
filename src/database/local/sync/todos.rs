use crate::auth::backend::{ANON_KEY, SUPABASE_URL};
use crate::utils::structs::{TodoEventLight, TodoListLight};
use dioxus::prelude::ServerFnError;
use sqlx::{Sqlite, Transaction};
use std::collections::HashSet;

pub async fn sync_todos(
    tx: &mut Transaction<'_, Sqlite>,
    token: &str,
) -> Result<(), ServerFnError> {
    let http_client = reqwest::Client::new();
    let bearer_token = format!("Bearer {}", token);
    //ToDo-Listen laden
    println!("Loading Todo Lists...");

    //Config & Resonse von http-Anfrage
    let url_lists = format!("{}/rest/v1/todo_lists?select=*", SUPABASE_URL);
    let response_lists = http_client
        .get(&url_lists)
        .header("apikey", ANON_KEY)
        .header("Authorization", &bearer_token)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Http Request TodoLists Error: {}", e)))?;
    if !response_lists.status().is_success() {
        let err = response_lists.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!(
            "Supabase Error TodoLists: {}",
            err
        )));
    }

    //Response in Json parsen
    let text_lists = response_lists
        .text()
        .await
        .map_err(|e| ServerFnError::new(format!("Text Error: {}", e)))?;

    //Json in Vec von ToDoLists parsen
    let lists: Vec<TodoListLight> = serde_json::from_str(&text_lists)
        .map_err(|e| ServerFnError::new(format!("JSON Parse TodoLists: {}", e)))?;

    //neues set aus Remote-DB Id's für Löschung von verwaisten Einträgen
    let mut remote_list_ids = HashSet::new();

    //über Vec mit Listen itterieren und in local DB (erst in Transactionswarteschlange, noch nicht direkt) speichern
    for l in lists {
        remote_list_ids.insert(l.id.clone());
        sqlx::query(
            r#"
            INSERT INTO todo_lists (
                id, name, type, description, owner_id, group_id, 
                due_datetime, priority, attachment, rrule, recurrence_until, 
                recurrence_id, attached_to_calendar_event, 
                created_at, created_by, last_mod
            ) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) 
            ON CONFLICT(id) DO UPDATE SET 
                name=excluded.name, 
                type=excluded.type, 
                description=excluded.description,
                owner_id=excluded.owner_id,
                group_id=excluded.group_id,
                due_datetime=excluded.due_datetime,
                priority=excluded.priority,
                attachment=excluded.attachment,
                rrule=excluded.rrule,
                recurrence_until=excluded.recurrence_until,
                recurrence_id=excluded.recurrence_id,
                attached_to_calendar_event=excluded.attached_to_calendar_event,
                created_at=excluded.created_at,
                created_by=excluded.created_by,
                last_mod=excluded.last_mod
            "#,
        )
        .bind(l.id)
        .bind(l.name)
        .bind(l.list_type)
        .bind(l.description)
        .bind(l.owner_id)
        .bind(l.group_id)
        .bind(l.due_datetime)
        .bind(l.priority)
        .bind(l.attachment)
        .bind(l.rrule)
        .bind(l.recurrence_until)
        .bind(l.recurrence_id)
        .bind(l.attached_to_calendar_event)
        .bind(l.created_at)
        .bind(l.created_by)
        .bind(l.last_mod)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Error TodoList: {}", e)))?;
    }

    // CleanUp: Listen die lokal noch da sind aber nicht mehr in remote -> löschen
    //Vec aus lokalen Listen anhand ID
    let local_list_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM todo_lists")
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Local List IDs: {}", e)))?;

    //Ist locale ID nicht in remote ID -> löschen
    for id in local_list_ids {
        if !remote_list_ids.contains(&id) {
            println!("Deleting orphan todo list: {}", id);
            sqlx::query("DELETE FROM todo_lists WHERE id = ?")
                .bind(id)
                .execute(&mut **tx)
                .await
                .ok();
        }
    }

    //ToDos laden
    println!("Loading Todo Events...");

    //Config & Resonse von http-Anfrage
    let url_events = format!("{}/rest/v1/todo_events?select=*", SUPABASE_URL);
    let response_events = http_client
        .get(&url_events)
        .header("apikey", ANON_KEY)
        .header("Authorization", &bearer_token)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Http Request TodoEvents Error: {}", e)))?;
    if !response_events.status().is_success() {
        let err = response_events.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!(
            "Supabase Error TodoEvents: {}",
            err
        )));
    }

    //Response in Json parsen
    let text_events = response_events
        .text()
        .await
        .map_err(|e| ServerFnError::new(format!("Text Error: {}", e)))?;

    //Json in Vec von ToDos parsen
    let events: Vec<TodoEventLight> = serde_json::from_str(&text_events)
        .map_err(|e| ServerFnError::new(format!("JSON Parse TodoEvents: {}", e)))?;

    //neues set aus Remote-DB Id's für Löschung von verwaisten Einträgen
    let mut remote_event_ids = HashSet::new();

    //über Vec mit ToDos itterieren und in local DB (erst in Transactionswarteschlange, noch nicht direkt) speichern
    for e in events {
        //id in Set aus remote IDs speichern
        remote_event_ids.insert(e.id.clone());
        sqlx::query(
            r#"
            INSERT INTO todo_events (
                id, todo_list_id, summary, description, completed, 
                due_datetime, priority, assigned_to_user, attachment, 
                rrule, recurrence_until, recurrence_id, 
                created_at, created_by, last_mod
            ) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) 
            ON CONFLICT(id) DO UPDATE SET 
                todo_list_id=excluded.todo_list_id, 
                summary=excluded.summary, 
                description=excluded.description,
                completed=excluded.completed,
                due_datetime=excluded.due_datetime,
                priority=excluded.priority,
                assigned_to_user=excluded.assigned_to_user,
                attachment=excluded.attachment,
                rrule=excluded.rrule,
                recurrence_until=excluded.recurrence_until,
                recurrence_id=excluded.recurrence_id,
                created_at=excluded.created_at,
                created_by=excluded.created_by,
                last_mod=excluded.last_mod
            "#,
        )
        .bind(e.id)
        .bind(e.todo_list_id)
        .bind(e.summary)
        .bind(e.description)
        .bind(e.completed)
        .bind(e.due_datetime)
        .bind(e.priority)
        .bind(e.assigned_to_user)
        .bind(e.attachment)
        .bind(e.rrule)
        .bind(e.recurrence_until)
        .bind(e.recurrence_id)
        .bind(e.created_at)
        .bind(e.created_by)
        .bind(e.last_mod)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Error TodoEvent: {}", e)))?;
    }

    // CleanUp: ToDos die lokal noch da sind aber nicht mehr in remote -> löschen
    //Vec aus lokalen ToDos anhand ID
    let local_event_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM todo_events")
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Local Event IDs: {}", e)))?;

    //Ist locale ID nicht in remote ID -> löschen
    for id in local_event_ids {
        if !remote_event_ids.contains(&id) {
            sqlx::query("DELETE FROM todo_events WHERE id = ?")
                .bind(id)
                .execute(&mut **tx)
                .await
                .ok();
        }
    }
    Ok(())
}
