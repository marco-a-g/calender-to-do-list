use chrono::{DateTime, Local, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use supabase::*;
use uuid::Uuid;

/// Structs for the communication between the databases
/// The following structs are only used to synchronise the local database with the remote database.
/// Should not be used in the front end to avoid type problems!
#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub crated_by: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupMember {
    pub id: String,
    pub user_id: String,
    pub group_id: String,
    pub role: String,
    joined_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Calendar {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub calendar_type: String,
    pub description: Option<String>,
    pub owner_id: Option<String>,
    pub group_id: Option<String>,
    pub created_at: String,
    pub created_by: String,
    pub last_mod: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub calendar_id: String,
    pub summary: String,
    pub description: Option<String>,
    pub from_date_time: String,
    pub to_date_time: Option<String>,
    pub attachment: Option<String>,
    pub rrule: Option<String>,
    pub recurrence_until: Option<String>,
    pub location: Option<String>,
    pub category: Option<String>,
    pub is_all_day: bool,
    pub recurrence_id: Option<String>,
    pub created_at: String,
    pub created_by: String,
    pub last_mod: String,
}

/// a todo-list is either belonging to a user, then list_type must be set to "private" and a
/// owner_id must be provided or to a group, then list_type must be set to "group" and a group_id
/// must be provided. There must only be one, either owner_id or group_id.
#[derive(Debug, Serialize, Deserialize)]
pub struct TodoList {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub list_type: String,
    pub description: Option<String>,
    pub owner_id: Option<String>,
    pub group_id: Option<String>,
    pub due_datetime: Option<String>,
    pub priority: Option<String>,
    pub attachment: Option<String>,
    pub rrule: Option<String>,
    pub recurrence_until: Option<String>,
    pub recurrence_id: Option<String>,
    pub attached_to_calendar_event: Option<String>,
    pub created_at: String,
    pub created_by: String,
    pub last_mod: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodoEvent {
    pub id: String,
    pub todo_list_id: String,
    pub summary: String,
    pub description: Option<String>,
    pub completed: bool,
    pub due_datetime: Option<String>,
    pub priority: Option<String>,
    pub attachment: Option<String>,
    pub rrule: Option<String>,
    pub recurrence_until: Option<String>,
    pub recurrence_id: Option<String>,
    pub created_at: String,
    pub created_by: String,
    pub last_mod: String,
}
