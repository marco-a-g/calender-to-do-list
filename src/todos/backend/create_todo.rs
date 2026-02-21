use crate::auth::backend::*;
use crate::database::local::init_fetch::init_fetch_local_db::fetch_todo_lists_lokal_db;
use crate::database::local::sync_local_db::sync_local_to_remote_db;
use crate::utils::date_handling::html_input_to_db;
use crate::utils::functions::get_user_id_and_session_token;
use crate::utils::structs::TodoListLight;
use crate::utils::structs::{Priority, Recurrent, Rrule, TodoEvent};
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use server_fn::error::ServerFnError;
use std::str::FromStr;
use uuid::Uuid;

//Transferobjekt dür Kommunikation an Supabase
//Exkludiert Felder die von Supabase gesetzt werden
//LLM: #[serde(skip_serializing_if = "Option::is_none")] setzt bei .json None auf leer statt auf None, damit defaults in Suabase greifen, verhindert dass NULL als NULL in Supabase gespeichert wird wenn leeres Feld gewollt wird
#[derive(Debug, Deserialize, Serialize)]
pub struct ToDoTransfer {
    pub summary: String,
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub todo_list_id: Option<Uuid>,
    pub completed: bool,
    pub due_datetime: Option<DateTime<Utc>>,
    pub priority: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to_user: Option<Uuid>,
    pub attachment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rrule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence_until: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overrides_datetime: Option<DateTime<Utc>>,
    pub skipped: bool,
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
    //List id parsen
    let list_uuid = if todo_list_id.is_empty() {
        Uuid::nil()
    } else {
        Uuid::parse_str(&todo_list_id)?
    };

    //ID Zugewiesener user parsen
    let assignee_uuid =
        assigned_to_user
            .filter(|s| !s.is_empty())
            .and_then(|s| match Uuid::parse_str(&s) {
                Ok(uuid) => Some(uuid),
                Err(e) => {
                    eprintln!("Warnung: Invalid Assignee UUID '{}': {}", s, e);
                    None
                }
            });
    //Due Date parsen
    let due_date = due_datetime
        .as_deref()
        .and_then(|s| match html_input_to_db(s) {
            Ok(dt) => dt,
            Err(e) => {
                eprintln!("Warnung: Due Date Parse Error für '{}': {}", s, e);
                None
            }
        });
    //Priority parsen
    let priority = priority
        .as_deref()
        .and_then(|s| Priority::from_str(s).ok())
        .unwrap_or(Priority::Normal);
    //RRUle (Rule und until) parsen, Skipped und overrides bei create irrelevant
    let recurrence_settings = if let (Some(rule_str), Some(until_str)) = (rrule, recurrence_until) {
        if !rule_str.is_empty() && !until_str.is_empty() {
            let parsed_rule = match Rrule::from_str(&rule_str) {
                Ok(r) => Some(r),
                Err(e) => {
                    eprintln!("Rrule parsing error für '{}': {}", rule_str, e);
                    None
                }
            };
            let parsed_until = match html_input_to_db(&until_str) {
                Ok(dt) => dt,
                Err(e) => {
                    eprintln!(
                        "Warnung: Recurrence Until Parse Error für '{}': {}",
                        until_str, e
                    );
                    None
                }
            };

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

    Ok(new_todo)
}

// ToDoEvent in ToDoTransfer Objekt wandeln
pub fn todo_event_into_to_do_transfer(
    todo: TodoEvent,
) -> Result<ToDoTransfer, Box<dyn std::error::Error>> {
    // rrule und until extrahieren wenn vorhanden
    let (rrule_transfer, until_transfer) = match todo.recurrence {
        Some(rec) => {
            let rrule_str = match rec.rrule {
                Rrule::Daily => "daily",
                Rrule::Weekly => "weekly",
                Rrule::Fortnight => "fortnight",
                Rrule::OnWeekDays => "weekdays",
                Rrule::MonthlyOnDate => "monthly_on_date",
                Rrule::MonthlyOnWeekday => "monthly_on_weekday",
                Rrule::Annual => "annual",
            };
            (Some(rrule_str.to_string()), Some(rec.recurrence_until))
        }
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
    let priority_string = format!("{:?}", todo.priority).to_lowercase();

    let final_list_id = if todo.to_do_list_id.is_nil() {
        None
    } else {
        Some(todo.to_do_list_id)
    };

    //Neues ToDoTransferObjekt damit erstellen
    Ok(ToDoTransfer {
        summary: todo.summary,
        description: todo.description,
        todo_list_id: final_list_id,
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
pub async fn create_todo_event(mut todo: ToDoTransfer) -> Result<StatusCode, ServerFnError> {
    println!("Startin create_todo function with: '{:#?}'", todo);
    let (user_id_str, token) = match get_user_id_and_session_token().await {
        Ok(data) => data,
        Err(e) => {
            println!("Not Authed!");
            return Err(e);
        }
    };
    //bestehende Listen für ShadowList Abgleich holen
    let all_lists: Vec<TodoListLight> = fetch_todo_lists_lokal_db().await.map_err(|e| {
        eprintln!("Fehler beim Abrufen der lokalen Listen: {}", e);
        e
    })?;
    //checkt für ShadowListen ID bzw. Richtige Gruppen ID
    let final_list_id =
        map_id_to_shadow_list(todo.todo_list_id, user_id_str, &all_lists).map_err(|e| {
            eprintln!("Mapping Error (ShadowList check): {}", e);
            ServerFnError::new(format!("List Mapping Error on ShadowList check: {}", e))
        })?;

    // Setzt überprüfte id
    todo.todo_list_id = Some(final_list_id);

    //Private ToDos dem User selbst zuweisen
    if todo.assigned_to_user.is_none() {
        //Keine manuelle Zuweisung -> Suche nach der Liste mit der nun erarbeiteten id -> wenn der group id none ist -> private von diesem Nuter
        let is_private_list = all_lists
            .iter()
            .find(|l| l.id == final_list_id.to_string())
            .map(|l| l.group_id.is_none()) // Privat wenn group_id == None
            .unwrap_or(false); // Falls Liste nicht gefunden (sollte nicht passieren), sicherheitshalber false
        //wenn also private liste -> den User selbst als assigned user einstellen
        if is_private_list {
            todo.assigned_to_user = Some(user_id_str);
        }
    }

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
                eprintln!("Supabase respons error: {}", error_text);
                return Err(ServerFnError::new(format!(
                    "Supabase Error {}: {}",
                    status, error_text
                )));
            }
            println!("Created ToDo in Supabase.");
            if let Err(e) = sync_local_to_remote_db().await {
                eprintln!("Error on sync after create_todo: {:?}", e);
            }
            Ok(status)
        }
        Err(e) => {
            eprintln!("Network Error bei create_todo: {}", e);
            Err(ServerFnError::new(format!("Network Error?: {}", e)))
        }
    }
}

// Checkt ob die übergebene id eine Gruppen ID ist -> ShadowListe und gibt dann die richtige ID aus
fn map_id_to_shadow_list(
    id_to_check: Option<Uuid>,
    user_uuid: Uuid,
    all_lists: &[TodoListLight],
) -> Result<Uuid, Box<dyn std::error::Error>> {
    // ist übergebene id "" -> gehört ShadowList des Users -> suche anhand User ID die Liste mit entsprechenden Namen
    if id_to_check.is_none() {
        let user_id_str = user_uuid.to_string();
        //itteriert über alle Listen Einträge und gibt die ID der Liste aus dessen Namen == UserID
        let list = all_lists
            .iter()
            .find(|l| l.name == user_id_str)
            .ok_or_else(|| -> Box<dyn std::error::Error> {
                "No matching List found in mapping Shadow check".into()
            })?;
        let uuid = Uuid::parse_str(&list.id)?;
        return Ok(uuid);
    }

    let id_to_search = id_to_check.unwrap().to_string();

    // Ist eine echte Listen ID -> gib diese aus; itteriert über alle listen und checkt id für gleichheit
    if let Some(_list) = all_lists.iter().find(|l| l.id == id_to_search) {
        return Ok(id_to_check.unwrap());
    }

    // Übergebene ID ist eine Gruppen ID -> suche ShadowListe der Gruppe und gib diese aus
    if let Some(shadow_list) = all_lists.iter().find(|l| l.name == id_to_search) {
        let uuid = Uuid::parse_str(&shadow_list.id)?; //
        return Ok(uuid);
    }

    // Sollte keine passende Liste existieren
    Err(format!("Could not find List-ID '{}' .", id_to_search).into())
}
