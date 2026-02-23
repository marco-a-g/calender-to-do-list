use crate::utils::structs::TodoEventLight;
use crate::utils::{date_handling::calculate_next_date, structs::CalendarEventLight};
use chrono::{DateTime, Timelike, Utc};
use std::collections::HashMap;
use uuid::Uuid;

pub fn expand_recurring_events(events: Vec<CalendarEventLight>) -> Result<Vec<CalendarEventLight>> {
    let mut result = Vec::new();
}
