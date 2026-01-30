use crate::auth::backend::SUPABASE_URL;
use crate::auth::backend::*;
use crate::database::local::sync_local_db::sync_local_to_remote_db;
use crate::utils::functions::get_user_id_and_session_token;
use crate::utils::structs::{Priority, TodoEventLight};
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

//Transferobjekt dür Kommunikation an Supabase
#[derive(Debug, Deserialize, Serialize)]
struct ToDoTransfer {
    summary: String,
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    todo_list_id: Option<Uuid>,
    completed: bool,
    due_datetime: Option<DateTime<Utc>>,
    priority: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    assigned_to_user: Option<Uuid>,
    attachment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rrule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recurrence_until: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recurrence_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    overrides_datetime: Option<DateTime<Utc>>,
    skipped: bool,
}

//die hier nutzen an den anderen Stellen? //DAtum Parsen
fn parse_date_flexible(input: &str) -> Option<DateTime<Utc>> {
    if input.is_empty() {
        return None;
    }
    let clean = input.trim().replace(" ", "T");
    if let Ok(dt) = DateTime::parse_from_rfc3339(&clean) {
        return Some(dt.with_timezone(&Utc));
    }
    if clean.len() == 10 {
        let full_iso = format!("{}T00:00:00Z", clean);
        if let Ok(dt) = DateTime::parse_from_rfc3339(&full_iso) {
            return Some(dt.with_timezone(&Utc));
        }
    }
    None
}
//LightToDo in TransferTodo Objekt
fn light_todo_into_transfertodo(
    light: TodoEventLight,
) -> Result<ToDoTransfer, Box<dyn std::error::Error>> {
    //Wieso Box Error?

    let todo_list_id_transfer = Uuid::parse_str(&light.todo_list_id).ok();
    let due_datetime_transfer = if let Some(d) = light.due_datetime {
        parse_date_flexible(&d)
    } else {
        None
    };
    let priority_transfer = light
        .priority
        .unwrap_or("normal".to_string())
        .to_lowercase();
    let assigned_to_user_transfer = match light.assigned_to_user {
        Some(user_id) => Uuid::parse_str(&user_id).ok(),
        None => None,
    };
    let recurrence_until_transfer = if let Some(d) = light.recurrence_until {
        parse_date_flexible(&d)
    } else {
        None
    };
    let overrides_transfer = if let Some(d) = light.overrides_datetime {
        parse_date_flexible(&d)
    } else {
        None
    };
    let recurrence_id_transfer = if let Some(rid) = light.recurrence_id {
        Uuid::parse_str(&rid).ok()
    } else {
        None
    };
    //rest kann so genommen werden
    //Transferobjekt bauen
    Ok(ToDoTransfer {
        summary: light.summary,
        description: light.description,
        todo_list_id: todo_list_id_transfer,
        completed: light.completed,
        due_datetime: due_datetime_transfer,
        priority: priority_transfer,
        assigned_to_user: assigned_to_user_transfer,
        attachment: light.attachment,
        rrule: light.rrule.filter(|s| !s.is_empty()),
        recurrence_until: recurrence_until_transfer,
        recurrence_id: recurrence_id_transfer,
        overrides_datetime: overrides_transfer,
        skipped: light.skipped,
    })
}

// #[server]
pub async fn create_todo_event(todo: TodoEventLight) -> Result<StatusCode, ServerFnError> {
    println!("Startin create_todo function with: '{}'", todo.summary);

    let (_user_id_str, token) = match get_user_id_and_session_token().await {
        Ok(data) => data,
        Err(e) => {
            println!("Not Authed!");
            return Err(e);
        }
    };
    //ToDoLight aus Frontend in ToDoTransfer parsen
    let todo_transfer = match light_todo_into_transfertodo(todo) {
        Ok(todo_parsed) => todo_parsed,
        Err(e) => {
            println!("Error on parsing ToDoLightEvent in ToDoTransfer: {}", e);
            return Err(ServerFnError::new(format!("Error on map: {}", e)));
        }
    };

    //Http config
    let url_todos = format!("{}/rest/v1/todo_events", SUPABASE_URL);
    let client = reqwest::Client::new();
    let response_result = client
        .post(&url_todos)
        .bearer_auth(token)
        .header("apikey", ANON_KEY)
        .header("Content-Type", "application/json")
        .json(&todo_transfer)
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
            println!("Created ToDo in Supabase.");
            if let Err(e) = sync_local_to_remote_db().await {
                println!("Error on sync after create_todo: {:?}", e);
            }
            Ok(status)
        }
        Err(e) => Err(ServerFnError::new(format!("Network Error?: {}", e))),
    }
}
