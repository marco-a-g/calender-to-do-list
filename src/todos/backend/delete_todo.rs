use crate::auth::backend::SUPABASE_URL;
use crate::auth::backend::*;
use crate::database::local::sync_local_db::sync_local_to_remote_db;
use crate::todos::backend::create_todo::ToDoTransfer;
use crate::utils::date_handling::calculate_next_date;
use crate::utils::date_handling::html_input_to_db;
use crate::utils::functions::get_user_id_and_session_token;
use crate::utils::structs::TodoEventLight;
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use reqwest::StatusCode;
use serde::Serialize;
use server_fn::error::ServerFnError;
use uuid::Uuid;
// LLM: Lieber Payloads reduzieren so:
#[derive(Serialize)]
struct UpdateMasterDate {
    due_datetime: DateTime<Utc>,
}
#[derive(Serialize)]
struct UpdateSkippedOfExistingExceptions {
    skipped: bool,
}

// #[server]
/// Deletes a to-do event or manages skipped exceptions for recurring tasks in Supabase.
///
/// Manages deletion process to handle different scenarios.
/// Depending on the task, it routes the network request into one of three operatinos:
///
/// 1. **Master of Recurring Tasks series:** Calculates the next scheduled recurrence date and sends `PATCH` request to shift the master's `due_datetime` forward, without deleting the Master itself.
/// 2. **Recurring Instances:** Creates an Exception-event(`POST`) when deleting a recurring instance or updates (`PATCH`) an already existing exception on that specific date, marking it with`skipped: true`.
/// 3. **Standard Tasks:** Sends a `DELETE` request to delete the task from the remote database.
///
/// Triggers `sync_local_to_remote_db()` after succesfull deletion/skip.
///
/// ## Arguments
///
/// * `todo` - The `TodoEventLight` instance representing the task to be deleted or skipped.
///
/// ## Errors
///
/// Returns a `ServerFnError` if user authentication fails, if datetime calculations fail or if the Supabase request fails or returns an error status.
pub async fn delete_todo_event(todo: TodoEventLight) -> Result<StatusCode, ServerFnError> {
    println!("Starting delete_todo_event for: '{}'", todo.summary);

    let client = reqwest::Client::new();
    let (_user_id_str, token) = match get_user_id_and_session_token().await {
        Ok(data) => data,
        Err(e) => {
            println!("Error: User not authenticated");
            return Err(e);
        }
    };

    //Wenn Master instanz -> startdatum  (:= due_datetime) auf nächstes setzen
    if let Some(rrule_str) = &todo.rrule {
        if !rrule_str.is_empty() {
            // Nächstes Datum der recurrance holen
            // Due-Datum des Masters aus HTML-Input parsen
            let current_due = if let Some(s) = &todo.due_datetime {
                html_input_to_db(s).unwrap_or(None)
            } else {
                None
            };

            let current_due =
                current_due.ok_or(ServerFnError::new("Master has no valid due date"))?;

            // Startdatum der Wiederholung für calculate_next_date setzen, einfach due_date nehmen
            let start_date_rec = current_due;

            // Nächstes Datum berechnen damit man dieses als neues start datum im master setzen kann
            let next_due = calculate_next_date(current_due, rrule_str, start_date_rec)
                .map_err(|e| ServerFnError::new(format!("Error on calc next date: {}", e)))?;

            // Due Date für master auf das der nächsten Instanz setzen
            let url_update = format!("{}/rest/v1/todo_events?id=eq.{}", SUPABASE_URL, todo.id);
            let payload_update = UpdateMasterDate {
                due_datetime: next_due,
            };
            //Anfrage an Supbase mit geänderten DueDate im Master
            let response_result = client
                .patch(&url_update)
                .bearer_auth(token)
                .header("apikey", ANON_KEY)
                .header("Content-Type", "application/json")
                .json(&payload_update)
                .send()
                .await;

            // Response check
            match response_result {
                Ok(response) => {
                    let status = response.status();
                    if !status.is_success() {
                        let error_text = response.text().await.unwrap_or_default();
                        println!("Supabase error moving master: {}", error_text);
                        return Err(ServerFnError::new(format!("Supabase Error: {}", status)));
                    }
                    println!("Master moved to next date (current skipped).");
                    if let Err(e) = sync_local_to_remote_db().await {
                        println!("Sync error: {:?}", e);
                    }
                    return Ok(status);
                }
                Err(e) => return Err(ServerFnError::new(format!("Network Error: {}", e))),
            }
        }
    }

    //Für Recurrance instanzen / nicht master
    if todo.recurrence_id.is_some()
        && (todo.rrule.is_none() || todo.rrule.as_ref().unwrap().is_empty())
    {
        //Copilot PR-Review Anpassung -> weiterer Check ob zu deletende Instanz bereits eine Exception ist -> dann nur Patch, keine neue exception
        let is_existing_exception = todo.overrides_datetime.is_some();

        if is_existing_exception {
            let url_update = format!("{}/rest/v1/todo_events?id=eq.{}", SUPABASE_URL, todo.id);
            let payload = UpdateSkippedOfExistingExceptions { skipped: true };
            //patch auf existierendere Exception
            let response_result = client
                .patch(&url_update)
                .bearer_auth(token)
                .header("apikey", ANON_KEY)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await;

            match response_result {
                Ok(res) => {
                    if !res.status().is_success() {
                        let error_text = res.text().await.unwrap_or_default();
                        println!("Supabase Error patching exception: {}", error_text);
                        return Err(ServerFnError::new(format!(
                            "Supabase Error: {}",
                            error_text
                        )));
                    }
                    println!("Existing exception patched to skipped.");
                    if let Err(e) = sync_local_to_remote_db().await {
                        println!("Sync error: {:?}", e);
                    }
                    return Ok(res.status());
                }
                Err(e) => return Err(ServerFnError::new(format!("Network Error: {}", e))),
            }
        } else {
            // Overrides_date holen anhand der ausgewählten wiederholenden Instanz
            let overrides_date = if let Some(s) = &todo.due_datetime {
                html_input_to_db(s).unwrap_or(None)
            } else {
                None
            };
            // wenn nächstes Datum nicht berechnet werden konnte Fehler werfen
            let overrides_date =
                overrides_date.ok_or(ServerFnError::new("Exception has no valid due date"))?;

            // Exception erstellen mit overrides datetime skipped=true
            let exception_entry = ToDoTransfer {
                summary: todo.summary.clone(),
                description: todo.description.clone(),
                todo_list_id: Uuid::parse_str(&todo.todo_list_id).ok(),
                completed: false,
                due_datetime: Some(overrides_date),
                priority: todo
                    .priority
                    .clone()
                    .unwrap_or("normal".to_string())
                    .to_lowercase(),
                assigned_to_user: todo
                    .assigned_to_user
                    .as_deref()
                    .and_then(|u| Uuid::parse_str(u).ok()),
                attachment: todo.attachment.clone(),
                rrule: None,
                recurrence_until: None,
                recurrence_id: todo
                    .recurrence_id
                    .as_deref()
                    .and_then(|id| Uuid::parse_str(id).ok()),
                overrides_datetime: Some(overrides_date),
                skipped: true,
            };

            // Anfrage an Supabase mit neuem Exception event
            let url_create = format!("{}/rest/v1/todo_events", SUPABASE_URL);
            let response_create = client
                .post(&url_create)
                .bearer_auth(token)
                .header("apikey", ANON_KEY)
                .header("Content-Type", "application/json")
                .json(&exception_entry)
                .send()
                .await;

            // Response check
            match response_create {
                Ok(res) => {
                    let status = res.status();
                    if !status.is_success() {
                        let error = res.text().await.unwrap_or_default();
                        return Err(ServerFnError::new(format!("Supabase Error: {}", error)));
                    }
                    println!("Skipped-Exception created.");
                    if let Err(e) = sync_local_to_remote_db().await {
                        println!("Sync error: {:?}", e);
                    }
                    return Ok(status);
                }
                Err(e) => return Err(ServerFnError::new(format!("Network Error: {}", e))),
            }
        }
    }
    //--Nicht Recurrend todos completen -> nur eintrag löschen
    let url_delete = format!("{}/rest/v1/todo_events?id=eq.{}", SUPABASE_URL, todo.id);
    let response_result = client
        .delete(&url_delete)
        .bearer_auth(token)
        .header("apikey", ANON_KEY)
        .header("Content-Type", "application/json")
        .send()
        .await;

    // Response check
    match response_result {
        Ok(response) => {
            let status = response.status();
            if !status.is_success() {
                let error_text = response.text().await.unwrap_or_default();
                println!("Supabase respons error: {}", error_text);
                return Err(ServerFnError::new(format!(
                    "Supabase Error {}: {}",
                    status, error_text
                )));
            }
            println!("Deleted ToDo in Supabase.");
            if let Err(e) = sync_local_to_remote_db().await {
                println!("Error on sync after delete_todo: {:?}", e);
            }
            Ok(status)
        }
        Err(e) => Err(ServerFnError::new(format!("Network Error?: {}", e))),
    }
}
