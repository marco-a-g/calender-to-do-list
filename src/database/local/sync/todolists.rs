use crate::auth::backend::{ANON_KEY, SUPABASE_URL};
use crate::utils::structs::TodoListLight;
use dioxus::prelude::*;
use server_fn::error::ServerFnError;
use sqlx::{Sqlite, Transaction};
use std::collections::HashSet;

/// Synchronizes to-do-lists from remote database to local database within the provided transaction-queue. Is called by `sync_local_to_remote_db()` function, where the transaction-queue is created.
///
/// Fetches all to-do-lists from Supabase through REST API GET request.
/// Insers new to-do-lists or updates existing ones based on UUID.
/// Deletes local to-do-lists events that no longer exist in the remote database
///
/// # Arguments
///
/// * `tx` - Reference to active SQLite transaction.
/// * `token` - Access token of the authenticated user.
///
/// # Errors
///
/// Returns a `ServerFnError` if:
/// - The HTTP request to Supabase fails.
/// - Parseing into a Vec of TodoListLight fails.
/// - Any Part of the SQL execution fails.
pub async fn sync_todolists(
    tx: &mut Transaction<'_, Sqlite>,
    token: &str,
) -> Result<(), ServerFnError> {
    let http_client = reqwest::Client::new();
    let bearer_token = format!("Bearer {}", token);
    //ToDo-Listen laden
    println!("Loading Todo Lists...");

    //Config & Response von http-Anfrage
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
                created_at, created_by, last_mod, overrides_datetime, skipped
            ) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) 
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
                last_mod=excluded.last_mod,
                overrides_datetime=excluded.overrides_datetime,
                skipped=excluded.skipped
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
        .bind(l.overrides_datetime)
        .bind(l.skipped)
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
    Ok(())
}
