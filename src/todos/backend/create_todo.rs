use crate::auth::backend::*;
use crate::database::local::sync_local_db::sync_local_to_remote_db;
use crate::utils::date_formatting::html_input_to_db;
use crate::utils::functions::get_user_id_and_session_token;
use crate::utils::structs::{Priority, Recurrent, Rrule, TodoEvent};
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
pub struct ToDoTransfer {
    summary: String,
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    // Sollte noch angepasst werden, sobald supabase trigger fertig mit ToDo ohne ToDoList
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

//Wandelt den Input aus Frontend in ein valides ToDoEvent struct um für Typesafety
pub fn frontend_input_to_todo(
    todo_list_id: String,
    summary: String,
    description: Option<String>,
    due_datetime: Option<String>,
    priority: Option<String>,
    rrule: Option<String>,
    recurrence_until: Option<String>,
    assigned_to_user: Option<String>,
) -> Result<TodoEvent, Box<dyn std::error::Error>> {
    println!("In frontend into todo func gibt vorher {:?}", rrule);

    //List id parsen
    let list_uuid = Uuid::parse_str(&todo_list_id)?;
    //ID Zugewiesener user parsen
    let assignee_uuid = assigned_to_user
        .filter(|s| !s.is_empty())
        .and_then(|s| Uuid::parse_str(&s).ok());
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
    let recurrence_settings = if let (Some(rule_str), Some(until_str)) = (rrule, recurrence_until) {
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

    let new_todo = TodoEvent {
        id: Uuid::new_v4(),
        summary,
        description: description.filter(|s| !s.is_empty()),
        to_do_list_id: list_uuid,
        completed: false,
        due_date_time: due_date,
        priority,
        assigned_to_user: assignee_uuid,
        attachment: None,
        recurrence: recurrence_settings,
        recurrence_exception: None, //Bei Create eh nicht vorhanden
        created_at: Utc::now(),
        created_by: Uuid::nil(),
        last_mod: Utc::now(),
    };
    println!(
        "In frontend into todo func gibt nachher {:?}",
        new_todo.recurrence
    );

    Ok(new_todo)
}

// ToDoEvent in ToDoTransfer Objekt wandeln
pub fn todo_event_into_to_do_transfer(
    todo: TodoEvent,
) -> Result<ToDoTransfer, Box<dyn std::error::Error>> {
    println!("In transfer func gibt vorher {:?}", todo);

    //rrule und until extrahieren wenn vorhanden
    let (rrule_transfer, until_transfer) = match todo.recurrence {
        Some(rec) => (
            Some(format!("{:?}", rec.rrule).to_lowercase()),
            Some(rec.recurrence_until),
        ),
        None => (None, None),
    };
    //Skipped und overrides DT handeln
    let (rec_id_transfer, overrides_dt_transfer, skipped_transfer) = match todo.recurrence_exception
    {
        Some(ex) => {
            let (dt, sk) = match ex.overrides {
                Some(ov) => (Some(ov.overrides_datetime), ov.skipped),
                None => (None, false),
            };
            (Some(ex.recurrence_id), dt, sk)
        }
        None => (None, None, false),
    };
    println!("In transfer func gibt nachher {:?}", rrule_transfer);
    let priority_string = format!("{:?}", todo.priority).to_lowercase();
    //Neues ToDoTransferObjekt damit erstellen
    Ok(ToDoTransfer {
        summary: todo.summary,
        description: todo.description,
        todo_list_id: Some(todo.to_do_list_id),
        completed: todo.completed,
        due_datetime: todo.due_date_time,
        priority: priority_string,
        assigned_to_user: todo.assigned_to_user,
        attachment: todo.attachment,
        rrule: rrule_transfer,
        recurrence_until: until_transfer,
        recurrence_id: rec_id_transfer,
        overrides_datetime: overrides_dt_transfer,
        skipped: skipped_transfer,
    })
}

//Zu erstellendes ToDo-Transferobjekt an Supabase senden
// #[server]
pub async fn create_todo_event(todo: ToDoTransfer) -> Result<StatusCode, ServerFnError> {
    println!("Startin create_todo function with: '{:#?}'", todo);

    let (_user_id_str, token) = match get_user_id_and_session_token().await {
        Ok(data) => data,
        Err(e) => {
            println!("Not Authed!");
            return Err(e);
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
        .json(&todo)
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
