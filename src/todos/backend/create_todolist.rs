use crate::auth::backend::*;
use crate::database::local::sync_local_db::sync_local_to_remote_db;
use crate::utils::date_handling::html_input_to_db;
use crate::utils::functions::get_user_id_and_session_token;
use crate::utils::structs::{OwnedBy, OwnerType, Priority, /* Recurrent, Rrule, */ ToDoList};
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

//Transferobjekt dür Kommunikation an Supabase
//Exkludiert Felder die von Supabase gesetzt werden
//LLM: #[serde(skip_serializing_if = "Option::is_none")] setzt bei .json None auf leer statt auf None, damit defaults in Suabase greifen, verhindert dass NULL als NULL in Supabase gespeichert wird wenn leeres Feld gewollt wird
#[derive(Debug, Deserialize, Serialize)]
pub struct ToDoListTransfer {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_datetime: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    //Recurrance bei Listen noch nicht
    /*    pub rrule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence_until: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overrides_datetime: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")] */
    pub attached_to_calendar_event: Option<Uuid>,
}

//Wandelt den Input aus Frontend in ein valides ToDoList struct um für Typesafety
pub fn frontend_input_to_todo_list(
    name: String,
    description: Option<String>,
    group_id: String,
    current_user: String,
    due_datetime: Option<String>,
    priority: Option<String>,
    //rrule: Option<String>,
    //recurrence_until: Option<String>,
    attatched_to_cal_evt: Option<String>,
) -> Result<ToDoList, Box<dyn std::error::Error>> {
    //Due Date parsen
    let due_date = due_datetime
        .as_deref()
        .and_then(|s| html_input_to_db(s).unwrap_or(None));
    //Priority parsen
    let priority = priority
        .as_deref()
        .and_then(|s| Priority::from_str(s).ok())
        .unwrap_or(Priority::Normal);
    //RRUle (Rule und until) parsen, Skipped und overrides bei create irrelevant
    /* let recurrence_settings = if let (Some(rule_str), Some(until_str)) = (rrule, recurrence_until) {
        if !rule_str.is_empty() && !until_str.is_empty() {
            let parsed_rule = Rrule::from_str(&rule_str).ok();
            let parsed_until = html_input_to_db(&until_str).unwrap_or(None);
            if let (Some(r), Some(u)) = (parsed_rule, parsed_until) {
                Some(Recurrent {
                    rrule: r,
                    recurrence_until: u,
                })
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    */
    //rec bei todolisten doch nicht, daher erstmal auf None
    let recurrence_settings = None;

    //zugehörige Event id parsen wenn vorhanden
    let evt_id = attatched_to_cal_evt
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());
    //Gruppen id parsen
    let group_uuid_opt = Uuid::parse_str(&group_id).ok();
    //User ID Parsen
    let user_id = Uuid::parse_str(&current_user)?;
    //Owner id nach List type parsen
    let owned_by = match group_uuid_opt {
        Some(gid) => OwnedBy {
            owner_type: OwnerType::Group,
            owner_id: gid,
        },
        None => OwnedBy {
            owner_type: OwnerType::Private,
            owner_id: user_id,
        },
    };
    //neue Liste zusammenstellen
    let new_list = ToDoList {
        id: Uuid::new_v4(),
        name: name,
        description: description.filter(|s| !s.is_empty()),
        owned_by: owned_by,
        due_date_time: due_date,
        priority: priority,
        attachment: None,
        recurrence: recurrence_settings,
        recurrence_exception: None,
        created_at: Utc::now(),
        created_by: Uuid::nil(),
        last_mod: Utc::now(),
        attached_to_calendar_event: evt_id,
    };
    Ok(new_list)
}

// ToDoEvent in ToDoTransfer Objekt wandeln
pub fn todo_list_into_todo_list_transfer(
    todo_list: ToDoList,
) -> Result<ToDoListTransfer, Box<dyn std::error::Error>> {
    //Listen type und owner und dementsprechende id extrahieren
    let (list_type_transfer, list_owner_transfer, list_group_transfer) =
        match todo_list.owned_by.owner_type {
            OwnerType::Private => (
                Some("private".to_string()),
                Some(todo_list.owned_by.owner_id),
                None,
            ),
            OwnerType::Group => (
                Some("group".to_string()),
                None,
                Some(todo_list.owned_by.owner_id),
            ),
        };

    //Recurrance für Listen erstmal nicht
    //rrule und until extrahieren wenn vorhanden
    /*
    let (rrule_transfer, until_transfer) = match todo_list.recurrence {
        Some(rec) => (
            Some(format!("{:?}", rec.rrule).to_lowercase()),
            Some(rec.recurrence_until),
        ),
        None => (None, None),
    };
    */

    //Neues ToDoListTransfer erstellen
    Ok(ToDoListTransfer {
        name: todo_list.name,
        r#type: list_type_transfer,
        description: todo_list.description,
        owner_id: list_owner_transfer,
        group_id: list_group_transfer,
        due_datetime: todo_list.due_date_time,
        priority: Some(format!("{:?}", todo_list.priority).to_lowercase()),
        attachment: todo_list.attachment,
        /*rrule: rrule_transfer,
        recurrence_until: until_transfer,
        recurrence_id: None,
        overrides_datetime: None, */
        attached_to_calendar_event: todo_list.attached_to_calendar_event,
    })
}

//Zu erstellendes ToDo-List-Transferobjekt an Supabase senden
// #[server]
pub async fn create_todo_list(todo_list: ToDoListTransfer) -> Result<StatusCode, ServerFnError> {
    println!("Startin create_todo_list function with: '{:#?}'", todo_list);

    let (_user_id_str, token) = match get_user_id_and_session_token().await {
        Ok(data) => data,
        Err(e) => {
            println!("Not Authed!");
            return Err(e);
        }
    };
    //Http config
    let url_todo_lists = format!("{}/rest/v1/todo_lists", SUPABASE_URL);
    let client = reqwest::Client::new();
    let response_result = client
        .post(&url_todo_lists)
        .bearer_auth(token)
        .header("apikey", ANON_KEY)
        .header("Content-Type", "application/json")
        .json(&todo_list)
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
