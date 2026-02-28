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
struct UpdateTodoStatus {
    completed: bool,
}
#[derive(Serialize)]
struct UpdateMasterDate {
    due_datetime: DateTime<Utc>,
}

// #[server]
/// Marks a to-do event as completed in the remote database.
///
/// Manages completion process to handle different scenarios.
/// Depending on the task, it routes the network request into one of three operatinos:
///
/// 1. **Master of Recurring Tasks series:** Calculates the next scheduled recurrence date and creates (`POST`) an exception for the current date marked as completed, and then moves (`PATCH`) the master task's `due_datetime` forward to the next scheduled occurrence.
/// 2. **Recurring Instances:** If the instance is already an exception, it updates (`PATCH`) it to marked completed. If it is not already an Exception, it creates (`POST`) a new completed exception for that date tied to the master task.
/// 3. **Standard Tasks:** Updates (`PATCH`) the task's `completed` status to `true`.
///
/// Triggers `sync_local_to_remote_db()` after succesfull completion.
///
/// ## Arguments
///
/// * `todo` - The `TodoEventLight` instance representing the task to be marked as complete.
///
/// ## Errors
///
/// Returns a `ServerFnError` if user authentication fails, if datetime calculations fail or if the Supabase request fails or returns an error status.
pub async fn complete_todo_event(todo: TodoEventLight) -> Result<StatusCode, ServerFnError> {
    println!("Starting complete_todo_event for: '{}'", todo.summary);
    let client = reqwest::Client::new();
    let (_user_id_str, token) = match get_user_id_and_session_token().await {
        Ok(data) => data,
        Err(e) => {
            println!("Error: User not authenticated");
            return Err(e);
        }
    };

    //Wenn Master instanz -> Exception Instant erstellen
    if let Some(rrule_str) = &todo.rrule {
        if !rrule_str.is_empty() {
            // nächstes Datum der recurrance holen
            let current_due = if let Some(s) = &todo.due_datetime {
                html_input_to_db(s).unwrap_or(None)
            } else {
                None
            };
            // wenn nächstes Datum nicht berechnet werden konnte Fehler werfen
            let current_due =
                current_due.ok_or(ServerFnError::new("Master has no valid due date"))?;

            // Startdatum der Wiederholung für calculate_next_date setzen, einfach due_date nehmen
            let start_date_rec = current_due;

            //Nächstes Datum berechnen damit man dieses als neues start datum im master setzen kann
            let next_due = calculate_next_date(current_due, rrule_str, start_date_rec)
                .map_err(|e| ServerFnError::new(format!("Error on calc next date: {}", e)))?;

            // Exception instanz mit completed=true erstellen, damit History eintrag existiert
            let history_entry = ToDoTransfer {
                summary: todo.summary.clone(),
                description: todo.description.clone(),
                todo_list_id: Uuid::parse_str(&todo.todo_list_id).ok(),
                completed: true,
                due_datetime: Some(current_due),
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
                recurrence_id: Uuid::parse_str(&todo.id).ok(),
                overrides_datetime: Some(current_due),
                skipped: false,
            };

            // Anfrage an Supabase mit neuem Exception event
            let url_create = format!("{}/rest/v1/todo_events", SUPABASE_URL);
            let response_history = client
                .post(&url_create)
                .bearer_auth(token.clone())
                .header("apikey", ANON_KEY)
                .header("Content-Type", "application/json")
                .json(&history_entry)
                .send()
                .await;

            match response_history {
                Ok(res) => {
                    if !res.status().is_success() {
                        let error_text = res.text().await.unwrap_or_default();
                        println!(
                            "Fehler beim Erstellen der History/Exception: {}",
                            error_text
                        );
                        return Err(ServerFnError::new(format!(
                            "History Creation Failed: {}",
                            error_text
                        )));
                    }
                }
                Err(e) => {
                    println!("Netzwerkfehler beim History-Eintrag: {}", e);
                    return Err(ServerFnError::new(format!("Network Error History: {}", e)));
                }
            }

            // Due Date für master auf späteres setzen
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
                        println!("Supabase respons error: {}", error_text);
                        return Err(ServerFnError::new(format!(
                            "Supabase Error {}: {}",
                            status, error_text
                        )));
                    }
                    println!("Completed ToDo in Supabase.");
                    if let Err(e) = sync_local_to_remote_db().await {
                        println!("Error on sync after complete_todo: {:?}", e);
                    }
                    return Ok(status);
                }
                Err(e) => return Err(ServerFnError::new(format!("Network Error?: {}", e))),
            }
        }
    }

    //Für Recurrance instanzen / nicht master
    if todo.recurrence_id.is_some()
        && (todo.rrule.is_none() || todo.rrule.as_ref().unwrap().is_empty())
    {
        //Copilot PR-Review Anpassung -> weiterer Check ob zu completende Instanz bereits eine Exception ist -> dann nur Patch, keine neue exception
        let is_existing_exception = todo.overrides_datetime.is_some();

        if is_existing_exception {
            let url_update = format!("{}/rest/v1/todo_events?id=eq.{}", SUPABASE_URL, todo.id);
            let payload = UpdateTodoStatus { completed: true };
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
                    println!("Existing exception patched to completed.");
                    if let Err(e) = sync_local_to_remote_db().await {
                        println!("Sync error: {:?}", e);
                    }
                    return Ok(res.status());
                }
                Err(e) => return Err(ServerFnError::new(format!("Network Error: {}", e))),
            }
        } else {
            //wenn es sich nicht um bereits bestehende Exception einer Recurring instanz handelt, dann exception createn
            // Overrides_date holen anhand der ausgewählten wiederholenden Instanz
            let overrides_date = if let Some(s) = &todo.due_datetime {
                html_input_to_db(s).unwrap_or(None)
            } else {
                None
            };

            // wenn nächstes Datum nicht berechnet werden konnte Fehler werfen
            let overrides_date =
                overrides_date.ok_or(ServerFnError::new("Exception has no valid due date"))?;

            //Exception erstellen mit overrides datetime und true
            let exception_entry = ToDoTransfer {
                summary: todo.summary.clone(),
                description: todo.description.clone(),
                todo_list_id: Uuid::parse_str(&todo.todo_list_id).ok(),
                completed: true,
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
                skipped: false,
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
                    if !res.status().is_success() {
                        let error_text = res.text().await.unwrap_or_default();
                        println!("Supabase Error creating exception: {}", error_text);
                        return Err(ServerFnError::new(format!(
                            "Supabase Error: {}",
                            error_text
                        )));
                    }

                    println!("Exception created.");
                    if let Err(e) = sync_local_to_remote_db().await {
                        println!("Sync error: {:?}", e);
                    }
                    return Ok(res.status());
                }
                Err(e) => return Err(ServerFnError::new(format!("Network Error: {}", e))),
            }
        }
    }

    //--Nicht Recurrend todos completen -> nur completed setzen
    let url_update = format!("{}/rest/v1/todo_events?id=eq.{}", SUPABASE_URL, todo.id);
    let payload = UpdateTodoStatus { completed: true };
    let response_result = client
        //patch auf nicht recurring event
        .patch(&url_update)
        .bearer_auth(token)
        .header("apikey", ANON_KEY)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await;
    match response_result {
        Ok(response) => {
            let status = response.status();
            if !status.is_success() {
                let error_text = response.text().await.unwrap_or_default();
                println!("Supabase response error: {}", error_text);
                return Err(ServerFnError::new(format!(
                    "Supabase Error {}: {}",
                    status, error_text
                )));
            }
            println!("Completed ToDo in Supabase.");
            if let Err(e) = sync_local_to_remote_db().await {
                println!("Error on sync after complete_todo: {:?}", e);
            }
            Ok(status)
        }
        Err(e) => Err(ServerFnError::new(format!("Network Error?: {}", e))),
    }
}
