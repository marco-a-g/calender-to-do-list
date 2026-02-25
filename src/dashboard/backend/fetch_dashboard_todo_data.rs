use crate::database::local::init_fetch::init_fetch_local_db::{
    fetch_groups_lokal_db, fetch_todo_events_lokal_db, fetch_todo_lists_lokal_db,
};
use crate::todos::backend::handle_recurrence_todos::expand_recurring_todos;
use crate::utils::functions::get_user_id_and_session_token;
use crate::utils::structs::TodoEventLight;
use chrono::{DateTime, Datelike, Duration, Local};
use tokio::join;

/// Fetches, expands and filters to-do events for the dashboard view.
///
/// Retrieves all to-do events, lists, and groups from the local database, expands recurring tasks to generate "fake"" instances and filters them to include only incomplete todos that are due this week and are assigned to the current User.
///
/// Todos get sorted and mapped into a tuple of Strings (task_name, due_date, group_name, group_color)` for read-only rendering in dashboard.
///
/// # Errors
///
/// Returns a boxed dynamic error if the user session cannot be validated, if the database queries fail or the recurrence expansion fails.
pub async fn fetch_todos_dashboard_tuples()
-> Result<Vec<(String, Option<String>, String, String)>, Box<dyn std::error::Error>> {
    // Id Holen für filterung nach ID der Users
    let user_id = get_user_id_and_session_token()
        .await
        .map_err(|e| e.to_string())?
        .0;
    //todos, listen, gruppen holen und joinen
    let (todos_res, lists_res, groups_res) = join!(
        fetch_todo_events_lokal_db(),
        fetch_todo_lists_lokal_db(),
        fetch_groups_lokal_db()
    );
    //aus join in einzelne Vecs
    let pool = todos_res.map_err(|e| e.to_string())?;
    let all_lists = lists_res.map_err(|e| e.to_string())?;
    let all_groups = groups_res.map_err(|e| e.to_string())?;

    // Todos Expanden
    let expanded_pool = expand_recurring_todos(pool)?;
    let user_id_str = user_id.to_string();

    // Datumsgrenzen für Wochenansicht
    let now = Local::now();
    let today = now.date_naive();
    let days_to_sunday = 6 - now.weekday().num_days_from_monday() as i64;
    let end_of_week = today + Duration::days(days_to_sunday);

    //Filtern nach ID=User, Completed=false, Due Date in der Woche
    let mut filtered_pool: Vec<TodoEventLight> = expanded_pool
        .into_iter()
        .filter(|todo| {
            if todo.assigned_to_user.as_deref() != Some(&user_id_str) {
                return false;
            }
            if todo.completed {
                return false;
            }
            //alles was über berechneten Enddatum der Woche ist = false
            if let Some(due_str) = &todo.due_datetime {
                if let Ok(date) = DateTime::parse_from_rfc3339(due_str) {
                    let todo_date = date.with_timezone(&Local).date_naive();

                    return todo_date <= end_of_week;
                }
            }
            false
        })
        .collect();

    // nach Datum Due Date Sortiern
    filtered_pool.sort_by(|a, b| {
        let date_a = a
            .due_datetime
            .as_deref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok());
        let date_b = b
            .due_datetime
            .as_deref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok());
        date_a.cmp(&date_b)
    });

    // Auf Tupel mappen für Ansicht, hier brauchts keine Structs da keine Interaktion
    let result_tuples: Vec<(String, Option<String>, String, String)> = filtered_pool
        .into_iter()
        .map(|todo| {
            let (group_name, group_color) =
                if let Some(list) = all_lists.iter().find(|l| l.id == todo.todo_list_id) {
                    if let Some(gid) = &list.group_id {
                        if let Some(group) = all_groups.iter().find(|g| g.id == *gid) {
                            //Group id der Liste matched -> name und color der nehmen
                            (group.name.clone(), group.color.clone())
                        } else {
                            //Fallback keine passende Groupe zur Group id gefunden, sollte eigentlich nicht
                            ("Unknown Group".to_string(), "#9ca3af".to_string())
                        }
                    } else {
                        //Liste hat keine Gruppen ID -> privat
                        ("Private".to_string(), "#9ca3af".to_string())
                    }
                } else {
                    //Fallback, sollte eig nicht passieren
                    ("Unknown List".to_string(), "#9ca3af".to_string())
                };
            //Tupel zusammensetzen
            (todo.summary, todo.due_datetime, group_name, group_color)
        })
        .collect();

    Ok(result_tuples)
}
