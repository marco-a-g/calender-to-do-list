use crate::auth::backend::SUPABASE_URL;
use crate::auth::backend::*;
use crate::database::local::sync_local_db::sync_local_to_remote_db;
use crate::utils::date_handling::html_input_to_db;
use crate::utils::functions::get_user_id_and_session_token;
use crate::utils::structs::TodoListLight;
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use server_fn::error::ServerFnError;
use uuid::Uuid;

// Transferobjekt für Kommunikation an Supabase
#[derive(Debug, Deserialize, Serialize)]
struct UpdateTodoListTransfer {
    name: String,
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_datetime: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attachment: Option<String>,
    // Recurrence Felder immer None bei Listen
    /*     #[serde(skip_serializing_if = "Option::is_none")]
    rrule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recurrence_until: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recurrence_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    overrides_datetime: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")] */
    attached_to_calendar_event: Option<Uuid>,
}

// ToDoListLight in UpdateTodoListTransfer Objekt
fn light_list_into_update(
    light: TodoListLight,
) -> Result<UpdateTodoListTransfer, Box<dyn std::error::Error>> {
    // Due Date parsen
    let due_datetime_transfer = light
        .due_datetime
        .as_deref()
        .and_then(|s| html_input_to_db(s).unwrap_or(None));

    // Priority parsen
    let priority_transfer = light
        .priority
        .unwrap_or("normal".to_string())
        .to_lowercase();

    // Event ID parsen
    let event_id_transfer = match light.attached_to_calendar_event {
        Some(evt_id) => Uuid::parse_str(&evt_id).ok(),
        None => None,
    };

    let group_uuid = light
        .group_id
        .as_deref()
        .and_then(|g| Uuid::parse_str(g).ok());

    let owner_uuid = light
        .owner_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .and_then(|s| Uuid::parse_str(s).ok());

    // Type der Liste und Owner zuordnen
    let (type_transfer, owner_id_transfer, group_id_transfer) = if light.list_type == "private" {
        (Some("private".to_string()), owner_uuid, None)
    } else {
        (Some("group".to_string()), None, group_uuid)
    };

    Ok(UpdateTodoListTransfer {
        name: light.name,
        description: light.description,
        r#type: type_transfer,
        owner_id: owner_id_transfer,
        group_id: group_id_transfer,
        due_datetime: due_datetime_transfer,
        priority: Some(priority_transfer),
        attachment: light.attachment,
        // Recurrance doch noch nicht für listen
        /*rrule: None,
        recurrence_until: None,
        recurrence_id: None,
        overrides_datetime: None, */
        attached_to_calendar_event: event_id_transfer,
    })
}

// #[server]
pub async fn edit_todo_list(list: TodoListLight) -> Result<StatusCode, ServerFnError> {
    println!("Starting update_todo_list for '{}'", list.name);

    // ID holen für patch
    let list_id = Uuid::parse_str(&list.id)
        .map_err(|e| ServerFnError::new(format!("Invalid uuid in update_todo_list: {}", e)))?;

    let (_user_id_str, token) = match get_user_id_and_session_token().await {
        Ok(data) => data,
        Err(e) => {
            println!("Error: User not authenticated");
            return Err(e);
        }
    };

    // Mappen in UpdateTodoListTransfer
    let update_transfer = match light_list_into_update(list) {
        Ok(data) => data,
        Err(e) => {
            println!("Error mapping LightList into UpdateList: {}", e);
            return Err(ServerFnError::new(format!("Mapping Error: {}", e)));
        }
    };

    // Http config
    let url_update = format!("{}/rest/v1/todo_lists?id=eq.{}", SUPABASE_URL, list_id);
    let client = reqwest::Client::new();
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
                println!("Supabase respons error: {}", error_text);
                return Err(ServerFnError::new(format!(
                    "Supabase Error {}: {}",
                    status, error_text
                )));
            }
            println!("Updated ToDo-List in Supabase.");
            if let Err(e) = sync_local_to_remote_db().await {
                println!("Error on sync after update_todo_list: {:?}", e);
            }
            Ok(status)
        }
        Err(e) => Err(ServerFnError::new(format!("Network Error?: {}", e))),
    }
}
