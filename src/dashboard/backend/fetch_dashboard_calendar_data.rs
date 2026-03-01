use crate::database::local::init_fetch::init_fetch_local_db::{
    fetch_calendar_events_lokal_db, fetch_calendars_lokal_db, fetch_groups_lokal_db,
};

use crate::calendar::backend::handle_recurrence_cal_events::expand_recurring_events;
use crate::utils::structs::CalendarEventLight;
use chrono::{DateTime, Datelike, Duration, Local};
use tokio::join;

/// Fetches, filters, and formats calendar events for the dashboard.
///
/// Retrieves all calendar events, calendars, and groups from the local database, expands recurring calendar events to generate "fake"" instances and filters them to include only events that are scheduled this week.
///
/// Events are mapped into a tuple of  `(event, group_name, group_color)` to provide the necessary metadata for rendering for the dashboard UI.
///
/// ## Errors
///
/// Returns a boxed dynamic error if the database queries fail or the recurrence expansion fails.
pub async fn fetch_calendar_dashboard_tuples()
-> Result<Vec<(CalendarEventLight, String, String)>, Box<dyn std::error::Error>> {
    //Daten holen
    let (events_res, calendars_res, groups_res) = join!(
        fetch_calendar_events_lokal_db(),
        fetch_calendars_lokal_db(),
        fetch_groups_lokal_db()
    );
    //extrahieren
    let pool_events = events_res.map_err(|e| e.to_string())?;
    let all_calendars = calendars_res.map_err(|e| e.to_string())?;
    let all_groups = groups_res.map_err(|e| e.to_string())?;

    let current_time = chrono::Utc::now();
    let (expanded_pool, _hidden_masters) =
        expand_recurring_events(pool_events, Some(current_time))?;

    // Datumsgrenzen für diese Woche
    let now = Local::now();
    let today = now.date_naive();
    let current_weekday_num = now.weekday().num_days_from_monday() as i64;
    let start_of_week = today - Duration::days(current_weekday_num);
    let end_of_week = start_of_week + Duration::days(6);

    //Events rausfiltern, die nicht in diese Woche gehören
    let filtered_pool: Vec<CalendarEventLight> = expanded_pool
        .into_iter()
        .filter(|evt| {
            if let Ok(date) = DateTime::parse_from_rfc3339(&evt.from_date_time) {
                let evt_date = date.with_timezone(&Local).date_naive();
                //liegt das fromdatetime des events innerhalb dieser Woche?
                return evt_date >= start_of_week && evt_date <= end_of_week;
            }
            false
        })
        .collect();

    // Für Dashboardansicht in Tupel aus Event, Gruppennamen und Farbe Mappen
    let result_tuples: Vec<(CalendarEventLight, String, String)> = filtered_pool
        .into_iter()
        .map(|evt| {
            let (group_name, group_color) =
                //Den dazugehörigen Calender finden
                if let Some(calendar) = all_calendars.iter().find(|c| c.id == evt.calendar_id) {
                    if let Some(gid) = &calendar.group_id {
                        if let Some(group) = all_groups.iter().find(|g| g.id == *gid) {
                            //Gruppennamen und Farbe extrahieren
                            (group.name.clone(), group.color.clone())
                        } else {
                            ("Unknown Group".to_string(), "#9ca3af".to_string()) //Sollte nicht passieren, wenn Einträge stimmen
                        }
                    } else {
                        ("Private".to_string(), "#9ca3af".to_string()) // wenn kein Eintrag in group_id -> ist es ein Privater Termin, Standardgrau
                    }
                } else {
                    ("Unknown Calendar".to_string(), "#9ca3af".to_string()) //Sollte nicht passieren, wenn Einträge stimmen
                };
            (evt, group_name, group_color)
        })
        .collect();
    Ok(result_tuples)
}
