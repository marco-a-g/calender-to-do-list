use crate::database::local::init_fetch::init_fetch_local_db::{
    fetch_calendar_events_lokal_db, fetch_calendars_lokal_db, fetch_groups_lokal_db,
};

//expanden von calevts muss noch erstellt werden
//use crate::calendar::backend::handle_recurrence_calendar::expand_recurring_calendar_events;
use crate::utils::structs::CalendarEventLight;
use chrono::{DateTime, Datelike, Days, Duration, Local};
use tokio::join;

pub async fn fetch_calendar_dashboard_tuples()
-> Result<Vec<(CalendarEventLight, String, String)>, Box<dyn std::error::Error>> {
    //Daten holen
    let (events_res, calendars_res, groups_res) = join!(
        fetch_calendar_events_lokal_db(),
        fetch_calendars_lokal_db(),
        fetch_groups_lokal_db()
    );
    //extrahieren
    let pool_events = events_res?;
    let all_calendars = calendars_res?;
    let all_groups = groups_res?;

    //Recurrring events entpacken
    //richtig: let expanded_pool = expand_recurring_calendar_events(pool_events)?;
    //jetzt noch mock:
    let expanded_pool = expand_recurring_calendar_events(pool_events)?;

    // Datumsgrenzen für diese Woche
    let now = Local::now();
    let today = now.date_naive();
    let current_weekday_num = now.weekday().num_days_from_monday() as i64;
    let start_of_week = today - Duration::days(current_weekday_num);
    let end_of_week = start_of_week + Duration::days(6);

    //Events rausfiltern, die nicht in diese Woche gehören
    let mut filtered_pool: Vec<CalendarEventLight> = expanded_pool
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

//muss noch richtig erstellt werden jetzt gibt es einfach nur den mock Vecot aus
pub fn expand_recurring_calendar_events(
    mut evts: Vec<CalendarEventLight>,
) -> Result<Vec<CalendarEventLight>, Box<dyn std::error::Error>> {
    let mut mock_events = generate_mock_calendar_events();
    evts.append(&mut mock_events);
    Ok(evts)
}

//MOCK EINTRÄGE, da gerade noch keine CalenderEinträge in RemoteDB
use uuid::Uuid;
pub fn generate_mock_calendar_events() -> Vec<CalendarEventLight> {
    let today = Local::now().date_naive();
    let tomorrow = today + Duration::days(1);
    let yesterday = today - Duration::days(1);
    let future_day = today + Duration::days(10);
    vec![
        CalendarEventLight {
            id: Uuid::new_v4().to_string(),
            calendar_id: "f616fb30-b1e1-41c4-ab01-06d53ecf1a91".to_string(),
            summary: "Festival".to_string(),
            description: Some("Einfach mal entspannen".to_string()),
            from_date_time: format!("{}T00:00:00Z", yesterday.format("%Y-%m-%d")),
            to_date_time: Some(format!("{}T23:59:59Z", today.format("%Y-%m-%d"))),
            attachment: None,
            rrule: None,
            recurrence_until: None,
            location: None,
            category: None,
            is_all_day: true,
            recurrence_id: None,
            overrides_datetime: None,
            skipped: false,
            created_at: "2026-02-15T10:00:00Z".to_string(),
            created_by: "7b3c6d5c-9154-4b4d-885a-75f2661fa44d".to_string(),
            last_mod: "2026-02-15T10:00:00Z".to_string(),
        },
        CalendarEventLight {
            id: Uuid::new_v4().to_string(),
            calendar_id: "78e0df1a-9c10-4dee-a601-98b7b9ba83a4".to_string(),
            summary: "Weekly Sync".to_string(),
            description: Some("Besprechung der nächsten Milestones".to_string()),
            from_date_time: format!("{}T13:00:00Z", yesterday.format("%Y-%m-%d")),
            to_date_time: Some(format!("{}T14:30:00Z", yesterday.format("%Y-%m-%d"))),
            attachment: None,
            rrule: None,
            recurrence_until: None,
            location: Some("Discord".to_string()),
            category: None,
            is_all_day: false,
            recurrence_id: None,
            overrides_datetime: None,
            skipped: false,
            created_at: "2026-02-15T10:00:00Z".to_string(),
            created_by: "7b3c6d5c-9154-4b4d-885a-75f2661fa44d".to_string(),
            last_mod: "2026-02-15T10:00:00Z".to_string(),
        },
        CalendarEventLight {
            id: Uuid::new_v4().to_string(),
            calendar_id: "bfe5101d-2ee9-4a47-864c-a5d924766e25".to_string(),
            summary: "Vorlesung: Memory Management".to_string(),
            description: None,
            from_date_time: format!("{}T09:15:00Z", today.format("%Y-%m-%d")),
            to_date_time: Some(format!("{}T10:45:00Z", today.format("%Y-%m-%d"))),
            attachment: None,
            rrule: None,
            recurrence_until: None,
            location: Some("Hörsaal 1".to_string()),
            category: None,
            is_all_day: false,
            recurrence_id: None,
            overrides_datetime: None,
            skipped: false,
            created_at: "2026-02-15T10:00:00Z".to_string(),
            created_by: "7b3c6d5c-9154-4b4d-885a-75f2661fa44d".to_string(),
            last_mod: "2026-02-15T10:00:00Z".to_string(),
        },
        CalendarEventLight {
            id: Uuid::new_v4().to_string(),
            calendar_id: "c14bbc82-e2b9-4b93-84f4-6f67579ff6fe".to_string(),
            summary: "Tutorium Vorbereitung".to_string(),
            description: None,
            from_date_time: format!("{}T08:00:00Z", tomorrow.format("%Y-%m-%d")),
            to_date_time: Some(format!("{}T09:00:00Z", tomorrow.format("%Y-%m-%d"))),
            attachment: None,
            rrule: None,
            recurrence_until: None,
            location: None,
            category: None,
            is_all_day: false,
            recurrence_id: None,
            overrides_datetime: None,
            skipped: false,
            created_at: "2026-02-15T10:00:00Z".to_string(),
            created_by: "7b3c6d5c-9154-4b4d-885a-75f2661fa44d".to_string(),
            last_mod: "2026-02-15T10:00:00Z".to_string(),
        },
        CalendarEventLight {
            id: Uuid::new_v4().to_string(),
            calendar_id: "f616fb30-b1e1-41c4-ab01-06d53ecf1a91".to_string(),
            summary: "Zahnarzt".to_string(),
            description: Some("Kontrolle".to_string()),
            // Heute 16:30 bis 17:00 (Lokale Zeit -> UTC-1 für den Z-String)
            from_date_time: format!("{}T15:30:00Z", today.format("%Y-%m-%d")),
            to_date_time: Some(format!("{}T16:00:00Z", today.format("%Y-%m-%d"))),
            attachment: None,
            rrule: None,
            recurrence_until: None,
            location: None,
            category: None,
            is_all_day: false,
            recurrence_id: None,
            overrides_datetime: None,
            skipped: false,
            created_at: "2026-02-15T10:00:00Z".to_string(),
            created_by: "7b3c6d5c-9154-4b4d-885a-75f2661fa44d".to_string(),
            last_mod: "2026-02-15T10:00:00Z".to_string(),
        },
        CalendarEventLight {
            id: Uuid::new_v4().to_string(),
            calendar_id: "f616fb30-b1e1-41c4-ab01-06d53ecf1a91".to_string(),
            summary: "Prv Termin erst nächste Woche".to_string(),
            description: Some("Kontrolle".to_string()),
            // Heute 16:30 bis 17:00 (Lokale Zeit -> UTC-1 für den Z-String)
            from_date_time: format!("{}T15:30:00Z", future_day.format("%Y-%m-%d")),
            to_date_time: Some(format!("{}T16:00:00Z", future_day.format("%Y-%m-%d"))),
            attachment: None,
            rrule: None,
            recurrence_until: None,
            location: None,
            category: None,
            is_all_day: false,
            recurrence_id: None,
            overrides_datetime: None,
            skipped: false,
            created_at: "2026-02-15T10:00:00Z".to_string(),
            created_by: "7b3c6d5c-9154-4b4d-885a-75f2661fa44d".to_string(),
            last_mod: "2026-02-15T10:00:00Z".to_string(),
        },
        CalendarEventLight {
            id: Uuid::new_v4().to_string(),
            calendar_id: "78e0df1a-9c10-4dee-a601-98b7b9ba83a4".to_string(),
            summary: "SEP Präsi".to_string(),
            description: Some("Projektabschluss".to_string()),
            from_date_time: format!(
                "{}T15:30:00Z",
                (yesterday - Duration::days(2)).format("%Y-%m-%d")
            ),
            to_date_time: Some(format!(
                "{}T16:00:00Z",
                (yesterday - Duration::days(2)).format("%Y-%m-%d")
            )),
            attachment: None,
            rrule: None,
            recurrence_until: None,
            location: None,
            category: None,
            is_all_day: false,
            recurrence_id: None,
            overrides_datetime: None,
            skipped: false,
            created_at: "2026-02-15T10:00:00Z".to_string(),
            created_by: "7b3c6d5c-9154-4b4d-885a-75f2661fa44d".to_string(),
            last_mod: "2026-02-15T10:00:00Z".to_string(),
        },
        CalendarEventLight {
            id: Uuid::new_v4().to_string(),
            calendar_id: "bfe5101d-2ee9-4a47-864c-a5d924766e25".to_string(),
            summary: "IT Sicherheit - Fortbildung".to_string(),
            description: Some("Betriebssysteme Angebot".to_string()),
            from_date_time: format!(
                "{}T15:30:00Z",
                (yesterday - Duration::days(3)).format("%Y-%m-%d")
            ),
            to_date_time: Some(format!(
                "{}T16:00:00Z",
                (yesterday - Duration::days(2)).format("%Y-%m-%d")
            )),
            attachment: None,
            rrule: None,
            recurrence_until: None,
            location: None,
            category: None,
            is_all_day: true,
            recurrence_id: None,
            overrides_datetime: None,
            skipped: false,
            created_at: "2026-02-15T10:00:00Z".to_string(),
            created_by: "7b3c6d5c-9154-4b4d-885a-75f2661fa44d".to_string(),
            last_mod: "2026-02-15T10:00:00Z".to_string(),
        },
    ]
}
