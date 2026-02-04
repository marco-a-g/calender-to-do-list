use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;
use strum::EnumString;
use uuid::Uuid;

/// Is currently limited do the frequency of recurrence. Building recurrent events is described at
/// struct "Recurrent".
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, EnumString)]
pub enum Rrule {
    #[strum(ascii_case_insensitive)]
    Daily,
    #[strum(ascii_case_insensitive)]
    Weekly,
    #[strum(ascii_case_insensitive)]
    Fortnight,
    #[strum(ascii_case_insensitive)]
    OnWeekDays,
    #[strum(serialize = "monthly_on_date")]
    MonthlyOnDate,
    #[strum(serialize = "monthly_on_weekday")]
    MonthlyOnWeekday,
    #[strum(ascii_case_insensitive)]
    Annual,
}
impl fmt::Display for Rrule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, EnumString)]
pub enum OwnerType {
    #[strum(ascii_case_insensitive)]
    Private,
    #[strum(ascii_case_insensitive)]
    Group,
}
impl fmt::Display for OwnerType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, EnumString)]
pub enum Role {
    #[strum(ascii_case_insensitive)]
    Owner,
    #[strum(ascii_case_insensitive)]
    Admin,
    #[strum(ascii_case_insensitive)]
    Member,
    #[strum(ascii_case_insensitive)]
    Guest,
}
impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, EnumString)]
pub enum Priority {
    #[strum(ascii_case_insensitive)]
    Low,
    #[strum(ascii_case_insensitive)]
    Normal,
    #[strum(ascii_case_insensitive)]
    High,
    #[strum(ascii_case_insensitive)]
    Top,
}
impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Used for building an exception of an recurrent element.
/// If RecurrenceException is not None, recurrence id must refer to the id of an recurrent
/// element of the same type as this element (e.g. CalendarEvents) marking this element an
/// exception to the recurrent element.
/// Overrides shows wether this exception replaces an regular element (see Overrides) or is an
/// additional element to the recurrent element (None).
/// An element that is used as an recurrence exception must not be recurrent itself.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct RecurrenceException {
    pub recurrence_id: Uuid,
    pub overrides: Option<Overrides>,
}
impl fmt::Display for RecurrenceException {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.overrides {
            Some(o) => write!(f, "recurrence_id: {}, overrides: {}", self.recurrence_id, o),
            None => write!(
                f,
                "recurrende_id: {}, overrides: None (overrides_datetime: None, skipped: false)",
                self.recurrence_id
            ),
        }
    }
}

/// Describes which instance of the recurrent element should be overridden.
/// overrides_datetime must match from_date_time of the instance that shall be replaced.
/// skipped is used when the overridden instance is not replaced but simply skipped.
/// If skipped is set to true, the RecurrenceException will not be displayed and the only value
/// of it that is used is overrides_datetime.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Overrides {
    pub overrides_datetime: DateTime<Utc>,
    pub skipped: bool,
}
impl fmt::Display for Overrides {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "overrides_datetime: {}, skipped: {}",
            self.overrides_datetime, self.skipped
        )
    }
}

/// Used to describe a recurrent event.
/// rrule is currently limited to the frequency of the recurrence.
/// In case, there should be an irregularity within a recurrent event or the recurrence is skipped,
/// construct a different event that shows the irregularity (as explained at RecurrenceException)
/// and attach it to the recurrent event by setting the recurrence_id of the irregular event to the
/// id of the recurrent event.
/// This way you can also build recurrent events with odd recurrencies.
/// A recurrent event itself must not be marked as an RecurrenceException.
/// Example: You want an event that takes place every wednesday at 5 and every friday at 8.
/// Build a recurrent event at wednesday at 5, rrule = Weekly.
/// Build a second recurrent event at friday at 7, rrule = Weekly, recurrence_id = id of the first
/// event.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Recurrent {
    pub rrule: Rrule,
    pub recurrence_until: DateTime<Utc>,
}
impl fmt::Display for Recurrent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "rrule: {}, recurrence_until: {}",
            self.rrule, self.recurrence_until
        )
    }
}

/// Used to describe whether the element belongs to a user or a group and to wich user or group.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct OwnedBy {
    pub owner_type: OwnerType,
    pub owner_id: Uuid,
}
impl fmt::Display for OwnedBy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "owner_type: {}, owner_id: {}",
            self.owner_type, self.owner_id
        )
    }
}

/// Used to describe the members of a group. Membership is defined within a group, not within a user.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct GroupMemberOf {
    pub id: Uuid, //id used in the database table "group_members"
    pub user_id: Uuid,
    pub name: String,
    pub role: Role,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Profile {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub members: Vec<GroupMemberOf>,
}

/// A calendar must either belong to a user or to a group.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
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
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
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
    pub recurrence_exception: Option<RecurrenceException>, // if not None, this event is not stand-alone but an exception of an recurrent event
    pub location: Option<String>,
    pub categories: Option<Vec<String>>, // used to add tags to the event
    pub is_all_day: bool,
    pub last_mod: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ToDoList {
    pub id: Uuid,
    pub name: String,
    pub owned_by: OwnedBy,
    pub description: Option<String>,
    pub due_date_time: Option<DateTime<Utc>>,
    pub priority: Priority,
    pub attachment: Option<String>, //the path, regularly the web address, of a (shared) folder
    pub recurrence: Option<Recurrent>, // see explanation at "Recurrent"
    pub recurrence_exception: Option<RecurrenceException>, // if not None, this event is not stand-alone but an exception of an recurrent event
    pub attached_to_calendar_event: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub last_mod: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
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
    pub recurrence_exception: Option<RecurrenceException>, // if not None, this event is not stand-alone but an exception of an recurrent event
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub last_mod: DateTime<Utc>,
}

/// Structs for the communication between the databases
/// The following structs (named "...Light")are only used to synchronise the local SQL-Light
/// database with the remote database.
/// Should not be used in the front end to avoid type problems!
#[derive(Debug, Serialize, Deserialize, Clone, FromRow, PartialEq)]
pub struct ProfileLight {
    pub id: String,
    pub username: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, PartialEq)]
pub struct GroupLight {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, PartialEq)]
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
#[derive(Debug, Serialize, Deserialize, Clone, FromRow, PartialEq)]
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

/// For use of recurrence_id see RecurrenceException.
/// recurrence_id must be None for recurrent events.
/// overrides_datetime must be None if recurrence_id is None. (See Overrides)
/// skipped must not be true if overrides_datetime is None. (See Overrides)
#[derive(Debug, Serialize, Deserialize, Clone, FromRow, PartialEq)]
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
    pub overrides_datetime: Option<String>,
    pub skipped: bool,
    pub created_at: String,
    pub created_by: String,
    pub last_mod: String,
}

/// a TodoList is either belonging to a user, then list_type must be set to "private" and a
/// owner_id must be provided or to a group, then list_type must be set to "group" and a group_id
/// must be provided. There must only be one, either owner_id or group_id.
/// For use of recurrence_id see RecurrenceException.
/// recurrence_id must be None for recurrent events.
/// overrides_datetime must be None if recurrence_id is None. (See Overrides)
/// skipped must not be true if overrides_datetime is None. (See Overrides)
#[derive(Debug, Serialize, Deserialize, Clone, FromRow, PartialEq)]
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
    pub overrides_datetime: Option<String>,
    pub skipped: bool,
    pub attached_to_calendar_event: Option<String>,
    pub created_at: String,
    pub created_by: String,
    pub last_mod: String,
}

/// For use of recurrence_id see RecurrenceException.
/// recurrence_id must be None for recurrent events.
/// overrides_datetime must be None if recurrence_id is None. (See Overrides)
/// skipped must not be true if overrides_datetime is None. (See Overrides)
#[derive(Debug, Serialize, Deserialize, Clone, FromRow, PartialEq)]
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
    pub overrides_datetime: Option<String>,
    pub skipped: bool,
    pub created_at: String,
    pub created_by: String,
    pub last_mod: String,
}
