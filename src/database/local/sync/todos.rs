use crate::auth::backend::{ANON_KEY, SUPABASE_URL};
use crate::utils::structs::TodoEventLight;
use dioxus::prelude::*;
use server_fn::error::ServerFnError;
use sqlx::{Sqlite, Transaction};
use std::collections::HashSet;

/// Synchronizes to-do events from remote database to local database within the provided transaction-queue. Is called by `sync_local_to_remote_db()` function, where the transaction-queue is created.
///
/// Fetches all to-do events from Supabase through REST API GET request.
/// Insers new todos or updates existing ones based on UUID.
/// Deletes local to-do events that no longer exist in the remote database
///
/// ## Arguments
///
/// * `tx` - Reference to active SQLite transaction.
/// * `token` - Access token of the authenticated user.
///
/// ## Errors
///
/// Returns a `ServerFnError` if:
/// - The HTTP request to Supabase fails.
/// - Parseing into a Vec of TodoEventLight fails.
/// - Any Part of the SQL execution fails.
pub async fn sync_todos(
    tx: &mut Transaction<'_, Sqlite>,
    token: &str,
) -> Result<(), ServerFnError> {
    let http_client = reqwest::Client::new();
    let bearer_token = format!("Bearer {}", token);

    //ToDos laden
    println!("Loading Todo Events...");

    //Config & Response von http-Anfrage
    let url_todos = format!("{}/rest/v1/todo_events?select=*", SUPABASE_URL);
    let response_todos = http_client
        .get(&url_todos)
        .header("apikey", ANON_KEY)
        .header("Authorization", &bearer_token)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Http Request Todos Error: {}", e)))?;
    if !response_todos.status().is_success() {
        let err = response_todos.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!("Supabase Error Todos: {}", err)));
    }

    //Response in Json parsen
    let text_todos = response_todos
        .text()
        .await
        .map_err(|e| ServerFnError::new(format!("Text Error: {}", e)))?;

    //Json in Vec von ToDos parsen
    let todos: Vec<TodoEventLight> = serde_json::from_str(&text_todos)
        .map_err(|e| ServerFnError::new(format!("JSON Parse TodoEvents: {}", e)))?;

    //neues set aus Remote-DB Id's für Löschung von verwaisten Einträgen
    let mut remote_todos_ids = HashSet::new();

    //über Vec mit ToDos itterieren und in local DB (erst in Transactionswarteschlange, noch nicht direkt) speichern
    for todo in todos {
        //id in Set aus remote IDs speichern
        remote_todos_ids.insert(todo.id.clone());
        sqlx::query(
            r#"
            INSERT INTO todo_events (
                id, todo_list_id, summary, description, completed, 
                due_datetime, priority, assigned_to_user, attachment, 
                rrule, recurrence_until, recurrence_id, 
                created_at, created_by, last_mod, overrides_datetime, skipped
            ) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) 
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
                last_mod=excluded.last_mod,
                overrides_datetime=excluded.overrides_datetime,
                skipped=excluded.skipped
            "#,
        )
        .bind(todo.id)
        .bind(todo.todo_list_id)
        .bind(todo.summary)
        .bind(todo.description)
        .bind(todo.completed)
        .bind(todo.due_datetime)
        .bind(todo.priority)
        .bind(todo.assigned_to_user)
        .bind(todo.attachment)
        .bind(todo.rrule)
        .bind(todo.recurrence_until)
        .bind(todo.recurrence_id)
        .bind(todo.created_at)
        .bind(todo.created_by)
        .bind(todo.last_mod)
        .bind(todo.overrides_datetime)
        .bind(todo.skipped)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Error TodoEvent: {}", e)))?;
    }

    // CleanUp: ToDos die lokal noch da sind aber nicht mehr in remote -> löschen
    //Vec aus lokalen ToDos anhand ID
    let local_todos_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM todo_events")
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Local Event IDs: {}", e)))?;

    //Ist locale ID nicht in remote ID -> löschen
    for id in local_todos_ids {
        if !remote_todos_ids.contains(&id) {
            sqlx::query("DELETE FROM todo_events WHERE id = ?")
                .bind(id)
                .execute(&mut **tx)
                .await
                .ok();
        }
    }
    Ok(())
}
