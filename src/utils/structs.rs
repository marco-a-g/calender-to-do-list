use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Is currently limited do the frequency of recurrence. Building recurrent events is described at
/// struct "Recurrent".

#[derive(Debug, Deserialize, Serialize)]
pub enum Rrule {
    Daily,
    Weekly,
    Fortnight,
    OnWeekDays,
    Monthly,
    Annual,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum OwnerType {
    Private,
    Group,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Role {
    Owner,
    Admin,
    Member,
    Guest,
}
#[derive(Debug, Deserialize, Serialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Top,
}

/// Used to describe a recurrent event.
/// rrule is currently limited to the frequency of the recurrence.
/// In case, there should be an irregularity within a recurrent event, construct a different event
/// that shows the irregularity and attach it to the recurrent event by setting the recurrence_id
/// of the irregular event to the id of the recurrent event.
/// This way you can also build recurrent events with odd recurrencies.
/// Example: You want an event that takes place every wednesday at 5 and every friday at 8.
/// Build a recurrent event at wednesday at 5, rrule = Weekly.
/// Build a second recurrent event at friday at 7, rrule = Weekly, recurrence_id = id of the first
/// event.
#[derive(Debug, Deserialize, Serialize)]
pub struct Recurrent {
    pub rrule: Rrule,
    pub recurrence_until: DateTime<Utc>,
}

/// Used to describe whether the element belongs to a user or a group and to wich user or group.
#[derive(Debug, Deserialize, Serialize)]
pub struct OwnedBy {
    pub owner_type: OwnerType,
    pub owner_id: Uuid,
}

/// Used to describe the members of a group. Membership is defined within a group, not within a user.
#[derive(Debug, Deserialize, Serialize)]
pub struct GroupMemberOf {
    pub id: Uuid, //id used in the database table "group_members"
    pub user_id: Uuid,
    pub name: String,
    pub role: Role,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Profile {
    pub id: Uuid,
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub members: Vec<GroupMemberOf>,
}

/// A calendar must either belong to a user or to a group.
#[derive(Debug, Deserialize, Serialize)]
pub struct Calendar {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owned_by: OwnedBy,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub last_mod: DateTime<Utc>,
}

///
#[derive(Debug, Deserialize, Serialize)]
pub struct CalendarEvent {
    pub id: Uuid,
    pub summary: String,
    pub description: Option<String>,
    pub calendar_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid, //must be a users Uuid
    pub from_date_time: DateTime<Utc>,
    pub to_date_time: Option<DateTime<Utc>>,
    pub attachment: Option<String>, //the path, regularly the web address, of a (shared) folder
    pub recurrence: Option<Recurrent>, // see explanation at "Recurrent"
    pub recurrence_id: Option<Uuid>, // see explanation at "Recurrent"
    pub location: Option<String>,
    pub categories: Option<Vec<String>>, // used to add tags to the event
    pub is_all_day: bool,
    pub last_mod: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToDoList {
    pub id: Uuid,
    pub name: String,
    pub owned_by: OwnedBy,
    pub description: Option<String>,
    pub due_date_time: Option<DateTime<Utc>>,
    pub priority: Priority,
    pub attachment: Option<String>, //the path, regularly the web address, of a (shared) folder
    pub recurrence: Option<Recurrent>, // see explanation at "Recurrent"
    pub recurrence_id: Option<Uuid>, // see explanation at "Recurrent"
    pub attached_to_calendar_event: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub last_mod: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TodoEvent {
    pub id: Uuid,
    pub summary: String,
    pub description: Option<String>,
    pub to_do_list_id: Uuid,
    pub completed: bool,
    pub due_date_time: Option<DateTime<Utc>>,
    pub priority: Priority,
    pub assigned_to_user: Option<Uuid>,
    pub attachment: Option<String>, //the path, regularly the web address, of a (shared) folder
    pub recurrence: Option<Recurrent>, // see explanation at "Recurrent"
    pub recurrence_id: Option<Uuid>, // see explanation at "Recurrent"
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub last_mod: DateTime<Utc>,
}

/// Structs for the communication between the databases
/// The following structs (named "...Light")are only used to synchronise the local SQL-Light
/// database with the remote database.
/// Should not be used in the front end to avoid type problems!
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileLight {
    pub id: String,
    pub username: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupLight {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupMemberLight {
    pub id: String,
    pub user_id: String,
    pub group_id: String,
    pub role: String,
    pub joined_at: String,
}

/// A calendar is either
/// - belonging to a user
///     then list_type must be set to "private" and an owner_id must be provided.
/// - or to a group
///     then list_type must be set to "group" and a group_id must be provided.
/// There must only be one, either owner_id or group_id.
#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarLight {
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
pub struct CalendarEventLight {
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

/// a TodoList is either belonging to a user, then list_type must be set to "private" and a
/// owner_id must be provided or to a group, then list_type must be set to "group" and a group_id
/// must be provided. There must only be one, either owner_id or group_id.
#[derive(Debug, Serialize, Deserialize)]
pub struct TodoListLight {
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
pub struct TodoEventLight {
    pub id: String,
    pub todo_list_id: String,
    pub summary: String,
    pub description: Option<String>,
    pub completed: bool,
    pub due_datetime: Option<String>,
    pub priority: Option<String>,
    pub assigned_to_user: Option<String>,
    pub attachment: Option<String>,
    pub rrule: Option<String>,
    pub recurrence_until: Option<String>,
    pub recurrence_id: Option<String>,
    pub created_at: String,
    pub created_by: String,
    pub last_mod: String,
}
