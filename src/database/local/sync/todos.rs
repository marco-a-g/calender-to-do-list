#![allow(unused_variables)]

use crate::utils::structs::{TodoEventLight, TodoListLight};
use dioxus::prelude::ServerFnError;
use sqlx::{Sqlite, Transaction};
use std::collections::HashSet;
use supabase::Client;

pub async fn sync_todos(
    client: &Client,
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), ServerFnError> {
    // To-Do Listen laden
    println!("Loading To-Do Lists");
    let lists_json = client
        .database()
        .from("todo_lists")
        .select("*")
        .execute()
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Todo Lists Error: {}", e)))?;

    //To-Do Listen in Vec parsen
    let lists: Vec<TodoListLight> = serde_json::from_value(serde_json::Value::Array(lists_json))
        .map_err(|e| ServerFnError::new(format!("JSON Parse Todo Lists: {}", e)))?;

    //temporäres set mit den keys der remote Listen
    let mut remote_list_ids = HashSet::new();

    //über Vec mit To-Do Listen itterieren und in local db (erst in tx, noch nicht direkt speichern -> in Änderungsqueue) speichern
    for l in lists {
        remote_list_ids.insert(l.id.clone());
        sqlx::query(
            r#"
            INSERT INTO todo_lists (
                id, name, type, description, owner_id, group_id, 
                created_by, created_at, due_datetime, priority, 
                rrule, recurrence_id, recurrence_until, attached_to_calendar_event, last_mod
            ) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) 
            ON CONFLICT(id) DO UPDATE SET 
                name=excluded.name, type=excluded.type, description=excluded.description, 
                owner_id=excluded.owner_id, group_id=excluded.group_id,
                created_by=excluded.created_by, created_at=excluded.created_at,
                due_datetime=excluded.due_datetime, priority=excluded.priority,
                rrule=excluded.rrule, recurrence_id=excluded.recurrence_id,
                recurrence_until=excluded.recurrence_until,
                attached_to_calendar_event=excluded.attached_to_calendar_event,
                last_mod=excluded.last_mod
            "#,
        )
        .bind(l.id)
        .bind(l.name)
        .bind(l.list_type)
        .bind(l.description)
        .bind(l.owner_id)
        .bind(l.group_id)
        .bind(l.created_by)
        .bind(l.created_at)
        .bind(l.due_datetime)
        .bind(l.priority)
        .bind(l.rrule)
        .bind(l.recurrence_id)
        .bind(l.recurrence_until)
        .bind(l.attached_to_calendar_event)
        .bind(l.last_mod)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Error TodoList: {}", e)))?;
    }

    // Cleanup: listen die local sind aber nicht remote -> löschen
    let local_list_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM todo_lists")
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Local List IDs: {}", e)))?;
    for local_id in local_list_ids {
        if !remote_list_ids.contains(&local_id) {
            println!("Deleting orphan list: {}", local_id);
            sqlx::query("DELETE FROM todo_lists WHERE id = ?")
                .bind(local_id)
                .execute(&mut **tx)
                .await
                .ok();
        }
    }

    //ToDos Laden
    println!("Loading To-Do's...");

    let todo_json = client
        .database()
        .from("todo_events")
        .select("*")
        // .r#in(...) entfernt
        .execute()
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Todo Items Error: {}", e)))?;

    //To-Do's in Vec parsen
    let todos: Vec<TodoEventLight> = serde_json::from_value(serde_json::Value::Array(todo_json))
        .map_err(|e| ServerFnError::new(format!("JSON Parse Todo Items: {}", e)))?;

    //temporäres set mit den keys der remote ToDo's
    let mut remote_todo_ids = HashSet::new();

    //über Vec mit To-Do's itterieren und in local db (erst in tx, noch nicht direkt speichern -> in Änderungsqueue) speichern
    for t in todos {
        remote_todo_ids.insert(t.id.clone());
        sqlx::query(
            r#"
            INSERT INTO todo_events (
                id, todo_list_id, summary, description, completed, 
                due_datetime, priority, attachment, 
                rrule, recurrence_id, recurrence_until, 
                created_by, created_at, last_mod, assigned_to_user
            ) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET 
                summary=excluded.summary, description=excluded.description, 
                completed=excluded.completed, due_datetime=excluded.due_datetime, 
                priority=excluded.priority, attachment=excluded.attachment,
                rrule=excluded.rrule, recurrence_id=excluded.recurrence_id,
                recurrence_until=excluded.recurrence_until,
                created_by=excluded.created_by, created_at=excluded.created_at,
                last_mod=excluded.last_mod, assigned_to_user=excluded.assigned_to_user
        "#,
        )
        .bind(t.id)
        .bind(t.todo_list_id)
        .bind(t.summary)
        .bind(t.description)
        .bind(t.completed)
        .bind(t.due_datetime)
        .bind(t.priority)
        .bind(t.attachment)
        .bind(t.rrule)
        .bind(t.recurrence_id)
        .bind(t.recurrence_until)
        .bind(t.created_by)
        .bind(t.created_at)
        .bind(t.last_mod)
        .bind(t.assigned_to_user)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Error TodoItem: {}", e)))?;
    }
    // Cleanup: set aus lokalen todo ids erstellen
    let local_todo_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM todo_events")
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Local Todo IDs: {}", e)))?;

    //Cleanup: locale Todo id nicht in remote ToDo ids -> löschen
    let local_todo_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM todo_events")
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Local Todo IDs: {}", e)))?;

    for local_id in local_todo_ids {
        if !remote_todo_ids.contains(&local_id) {
            println!("Deleting orphan todo: {}", local_id);
            sqlx::query("DELETE FROM todo_events WHERE id = ?")
                .bind(local_id)
                .execute(&mut **tx)
                .await
                .ok();
        }
    }

    Ok(())
}
