use crate::auth::backend::SUPABASE_URL;
use crate::auth::backend::*;
use crate::database::local::sync_local_db::sync_local_to_remote_db;
use crate::todos::backend::create_todo::ToDoTransfer;
use crate::utils::date_handling::html_input_to_db;
use crate::utils::functions::get_user_id_and_session_token;
use crate::utils::structs::TodoEventLight;
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

//Transferobjekt dür Kommunikation an Supabase
#[derive(Debug, Deserialize, Serialize)]
struct UpdateTodoTransfer {
    summary: String,
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    todo_list_id: Option<Uuid>,
    completed: bool,
    due_datetime: Option<DateTime<Utc>>,
    priority: String,
    assigned_to_user: Option<Uuid>,
    attachment: Option<String>,
    rrule: Option<String>,
    recurrence_until: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recurrence_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    overrides_datetime: Option<DateTime<Utc>>,
    skipped: bool,
}

// LightToDo in UpdateTodoTransfer Objekt
fn light_todo_into_update(
    light: TodoEventLight,
) -> Result<UpdateTodoTransfer, Box<dyn std::error::Error>> {
    let todo_list_id_transfer = match Uuid::parse_str(&light.todo_list_id) {
        Ok(uuid) => Some(uuid),
        Err(e) => {
            if !light.todo_list_id.is_empty() {
                eprintln!(
                    "Warnung: Invalid Todo List UUID '{}': {}",
                    light.todo_list_id, e
                );
            }
            None
        }
    };
    let due_datetime_transfer =
        light
            .due_datetime
            .as_deref()
            .and_then(|s| match html_input_to_db(s) {
                Ok(dt) => dt,
                Err(e) => {
                    eprintln!("Warnung: Invalid Due Date '{}': {}", s, e);
                    None
                }
            });

    let priority_transfer = light
        .priority
        .unwrap_or("normal".to_string())
        .to_lowercase();

    let assigned_to_user_transfer = match light.assigned_to_user {
        Some(user_id) => match Uuid::parse_str(&user_id) {
            Ok(uid) => Some(uid),
            Err(e) => {
                eprintln!("Warnung: Invalid Assigned User UUID '{}': {}", user_id, e);
                None
            }
        },
        None => None,
    };

    let recurrence_until_transfer =
        light
            .recurrence_until
            .as_deref()
            .and_then(|s| match html_input_to_db(s) {
                Ok(dt) => dt,
                Err(e) => {
                    eprintln!("Warnung: Invalid Recurrence Until Date '{}': {}", s, e);
                    None
                }
            });

    let overrides_transfer =
        light
            .overrides_datetime
            .as_deref()
            .and_then(|s| match html_input_to_db(s) {
                Ok(dt) => dt,
                Err(e) => {
                    eprintln!("Warnung: Invalid Overrides Date '{}': {}", s, e);
                    None
                }
            });

    let recurrence_id_transfer = if let Some(rid) = light.recurrence_id {
        match Uuid::parse_str(&rid) {
            Ok(uid) => Some(uid),
            Err(e) => {
                eprintln!("Warnung: Invalid Recurrence ID '{}': {}", rid, e);
                None
            }
        }
    } else {
        None
    };

    Ok(UpdateTodoTransfer {
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
pub async fn edit_todo_event(todo: TodoEventLight) -> Result<StatusCode, ServerFnError> {
    println!(
        "Starting edit_todo_event für '{}' (ID: {})",
        todo.summary, todo.id
    );

    let (_user_id_str, token) = match get_user_id_and_session_token().await {
        Ok(data) => data,
        Err(e) => {
            println!("Error: User not authenticated");
            return Err(e);
        }
    };

    let client = reqwest::Client::new();

    //Für Recurrance instanzen / nicht master -> Exception erstellen oder patchen
    if todo.recurrence_id.is_some() {
        let exception_id = todo.id.clone();

        //Ist es bereits eine Exception -> nur patchen, dafür Prüfung: hat ein overrides_datetime && ist die id dieser instanz ungleich der rec_id (also vom Master), da im "edit only this instanze modus" im Master hier als rec_id vorrübergehend zur Zuordnung die id selber genommen wird
        let is_different_from_mastr = todo.recurrence_id.as_deref() != Some(&exception_id);

        let is_existing_exception = todo.overrides_datetime.is_some() && is_different_from_mastr;

        if is_existing_exception {
            // Es ist eine echte DB-Exception -> PATCH Update
            let update_transfer = match light_todo_into_update(todo.clone()) {
                Ok(data) => data,
                Err(e) => return Err(ServerFnError::new(format!("Mapping Error: {}", e))),
            };
            let url_update = format!("{}/rest/v1/todo_events?id=eq.{}", SUPABASE_URL, todo.id);

            let response = client
                .patch(&url_update)
                .bearer_auth(token)
                .header("apikey", ANON_KEY)
                .header("Content-Type", "application/json")
                .json(&update_transfer)
                .send()
                .await;

            match response {
                Ok(res) => {
                    if !res.status().is_success() {
                        let error_text = res.text().await.unwrap_or_default();
                        return Err(ServerFnError::new(format!(
                            "Supabase Error: {}",
                            error_text
                        )));
                    }
                    println!("Existing exception patched via edit.");
                    if let Err(e) = sync_local_to_remote_db().await {
                        eprintln!("Sync error: {:?}", e);
                    }
                    return Ok(res.status());
                }
                Err(e) => return Err(ServerFnError::new(format!("Network Error: {}", e))),
            }
        } else {
            //Keine bestehende Exception -> neue Exception erstellen bei edit
            // Overrides_date holen
            let overrides_date = if let Some(s) = &todo.due_datetime {
                html_input_to_db(s).unwrap_or(None)
            } else {
                None
            };

            let overrides_date =
                overrides_date.ok_or(ServerFnError::new("Instance has no valid due date"))?;

            // Exceptioneintrag erstellen
            let exception = ToDoTransfer {
                summary: todo.summary.clone(),
                description: todo.description.clone(),
                todo_list_id: Uuid::parse_str(&todo.todo_list_id).ok(),
                completed: todo.completed,
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
                skipped: todo.skipped,
            };

            // Anfrage an Supabase mit neuem Exception event
            let url_create = format!("{}/rest/v1/todo_events", SUPABASE_URL);
            let response_create = client
                .post(&url_create)
                .bearer_auth(token)
                .header("apikey", ANON_KEY)
                .header("Content-Type", "application/json")
                .json(&exception)
                .send()
                .await;

            match response_create {
                Ok(res) => {
                    let status = res.status();
                    if !status.is_success() {
                        let error_text = res.text().await.unwrap_or_default();
                        eprintln!(
                            "Supabase Error (Create Exception): {} - {}",
                            status, error_text
                        );
                        return Err(ServerFnError::new(format!(
                            "Supabase Error on Exception Create: {}",
                            error_text
                        )));
                    }
                    println!("Exception created successfully via edit.");
                    if let Err(e) = sync_local_to_remote_db().await {
                        eprintln!("Sync error after edit_todo: {:?}", e);
                    }
                    return Ok(status);
                }
                Err(e) => {
                    eprintln!("Network Error bei Exception Create: {}", e);
                    return Err(ServerFnError::new(format!("Network Error: {}", e)));
                }
            }
        }
    }
    //Master oder nicht recurring todo -> Eintrag selbst Updaten, keine exception
    // ID Checken
    let todo_id = Uuid::parse_str(&todo.id)
        .map_err(|e| ServerFnError::new(format!("Invalid uuid in edit_todo: {}", e)))?;

    // Mappen in UpdateToDoTransfer
    let update_transfer = match light_todo_into_update(todo) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error: mapping LightTodo into UpdateTodo: {}", e);
            return Err(ServerFnError::new(format!("Mapping Error: {}", e)));
        }
    };

    //Http config
    let url_update = format!("{}/rest/v1/todo_events?id=eq.{}", SUPABASE_URL, todo_id);

    let response_result = client
        .patch(&url_update)
        .bearer_auth(token)
        .header("apikey", ANON_KEY)
        .header("Content-Type", "application/json")
        .json(&update_transfer)
        .send()
        .await;

    // Response check
    match response_result {
        Ok(response) => {
            let status = response.status();
            if !status.is_success() {
                let error_text = response.text().await.unwrap_or_default();
                eprintln!("Supabase respons error: {} - {}", status, error_text);
                return Err(ServerFnError::new(format!(
                    "Supabase Error {}: {}",
                    status, error_text
                )));
            }
            println!("Updated ToDo in Supabase.");
            if let Err(e) = sync_local_to_remote_db().await {
                eprintln!("Error on sync after edit_todo: {:?}", e);
            }
            Ok(status)
        }
        Err(e) => {
            eprintln!("Network Error bei Update: {}", e);
            Err(ServerFnError::new(format!("Network Error?: {}", e)))
        }
    }
}
