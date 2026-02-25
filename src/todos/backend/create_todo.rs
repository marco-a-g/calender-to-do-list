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

//Struct to fix cargo clippy warning and minimize arguments for functions frontend_input_to_todo
pub struct TodoFrontendInput {
    pub todo_list_id: String,
    pub summary: String,
    pub description: Option<String>,
    pub due_datetime: Option<String>,
    pub priority: Option<String>,
    pub rrule: Option<String>,
    pub recurrence_until: Option<String>,
    pub assigned_to_user: Option<String>,
}

//Wandelt den Input aus Frontend in ein valides ToDoEvent struct um für Typesafety
/// Transforms raw input into a `TodoEvent` object.
///
/// Acts as primary data validation regarding todos, to ensure typesafety during their creation process.
///
/// Manages the following parsing scenarios:
/// - **UUIDs:** Parses the associated list and assignee IDs. If the assignee ID fails to parse, logs a warning but allows task to remain unassigned (`None`). If no list ID is provided, it defaults to a `nil` UUID.
/// - **Dates:** Converts HTML input strings into UTC `DateTime` objects. Failures are logged and fall back to `None`.
/// - **Recurrence:** Parses string-based recurrence rules and  end dates into the nested `Recurrent` struct, must both must be present and valid for the recurrence to be applied.
///
/// # Arguments
///
/// * `todo_list_id` - The raw UUID (String) of the parent to-do list.
/// * `summary` - The title of the to-do task.
/// * `description` - An optional string containing task details.
/// * `due_datetime` - An optional HTML date input string for the task's deadline.
/// * `priority` - An optional string representing the task's priority level.
/// * `rrule` - An optional string defining the recurrance pattern.
/// * `recurrence_until` - An optional HTML date input string defining the end of recurrence.
/// * `assigned_to_user` - An optional UUID (String)of the user responsible for the task.
///
/// # Errors
///
/// Returns a boxed dynamic error if the required `todo_list_id` is provided but is an invalid UUID string that cannot be parsed.
pub fn frontend_input_to_todo(
    input_todo: TodoFrontendInput,
) -> Result<TodoEvent, Box<dyn std::error::Error>> {
    let TodoFrontendInput {
        todo_list_id,
        summary,
        description,
        due_datetime,
        priority,
        rrule,
        recurrence_until,
        assigned_to_user,
    } = input_todo;

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
/// Converts a `TodoEvent` into a `ToDoTransfer` payload.
///
/// Prepares the task data for JSON serialization and transfer to the remote database.
///
/// Extracts the values from the nested `recurrence` (converting the `Rrule` stuct into its string equivalent) and `recurrence_exception` (extracting `recurrence_id`,`overrides_datetime`, and `skipped`) structs.
/// Formats the `Priority` enum into standard lowercase string and converts a `nil` list UUID into a `None` value.
///
/// # Arguments
///
/// * `todo` - The `TodoEvent` object to be transformed.
///
/// # Errors
///
/// Returns `Result<..., Box<dyn Error>>` to maintain consistency with other data mapping operations, but currently does not throw errors.
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
/// Creates a new to-do in the remote database.
///
/// Manages final preparation of Transferobject and network transmission of a new todo, therefore it completes following steps:
/// 1. **List Resolution:** It fetches the local database lists and uses `map_id_to_shadow_list` to resolve unclear list assignments (ensuring todos without a specified todolist are correctly routed to the user's/group's "shadow list").
/// 2. **Auto-Assignment:** If the resolved list is private (has no `group_id`) or no user was explicitly assigned, it automatically assigns the task to the current user.
///
/// After modifying the payload, it creates the new todo by sending a `POST` request to the remote database.
///
/// Triggers `sync_local_to_remote_db()` after succesfull creation.
///
/// # Arguments
///
/// * `todo` - The `ToDoTransfer` containing the todos data.
///
/// # Errors
///
/// Returns a `ServerFnError` if user authentication fails, list fetching or shadow-list mapping fails or if the Supabase request fails or returns an error status.
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
/// Resolves an unclear identifier into a concrete to-do list UUID, handling "shadow lists".
///
/// "Shadow lists": The List per User/Group that inhabits all Todos, that are not assigned to a specific ToDoList. identifiable by their `name` field, which matches the UUID of the owning user/group.
///
/// Uses following logic to assign the final id:
/// 1. **No ID Provided (`None`):** Assumes the task is personal, searching for the user's shadow list.
/// 2. **Explicit List ID:** Checks if the provided ID matches an existing to-do list.
/// 3. **Group ID:** If the ID isn't a direct list ID, assumes it is a group ID and searches for the corresponding group's shadow list.
///
/// # Arguments
///
/// * `id_to_check` - An optional `Uuid` that could represent specific list, a group, or be empty (`None`).
/// * `user_uuid` - The `Uuid` of the current user, used to resolve personal shadow lists.
/// * `all_lists` - A slice containing all available `TodoListLight` to search against.
///
/// # Errors
///
/// Returns a boxed dynamic error if a matching real list or required shadow list cannot be found or if UUID string parsing fails.
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
