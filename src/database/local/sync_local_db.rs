use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::collections::HashSet;
use std::str::FromStr;
use supabase::Client;

// Config -> Später raus sobald auth steht?
const SUPABASE_URL: &str = "https://tixtjdlkhnnxvneduxvb.supabase.co";
const SUPABASE_SERVICE_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InRpeHRqZGxraG5ueHZuZWR1eHZiIiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImlhdCI6MTc2NjkzMjUzNSwiZXhwIjoyMDgyNTA4NTM1fQ.YjnAzOQJ3GxlAGGAfNtbNtytfhKiDBG-OHqr7tex-5A";
const MOCK_USER_ID: &str = "a0000000-0000-0000-0000-000000000003"; //User

// Data-Stucts

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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupMember {
    pub id: String,
    pub user_id: String,
    pub group_id: String,
    pub role: String,
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
    pub last_mod: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub calendar_id: String,
    pub summary: String,
    pub description: Option<String>,
    pub date: String,
    pub from_time: Option<String>,
    pub to_time: Option<String>,
    pub seq: bool,
    pub rrule: Option<String>,
    pub last_mod: String,
}

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
    pub seq: bool,
    pub last_mod: String,
}

// Sync

//Sync api function
#[server]
pub async fn sync_remote_to_local() -> Result<(), ServerFnError> {
    sync_function().await
}

// Sync Logik
pub async fn sync_function() -> Result<(), ServerFnError> {
    println!("Start sync for User: {}", MOCK_USER_ID);

    //Client aufsetzen
    let client = Client::new(SUPABASE_URL, SUPABASE_SERVICE_KEY)
        .map_err(|e| ServerFnError::new(format!("Supabase Init Error: {}", e)))?;
    //Pfad local DB
    let db_path = "sqlite:src/database/local/local_Database.db";

    //Connectionoptions; Foreign Keys aktivieren sonst geht es nicht? Keine Ahnung...
    let opts = sqlx::sqlite::SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path))
        .map_err(|e| ServerFnError::new(format!("Path Error: {}", e)))?
        .create_if_missing(true)
        .foreign_keys(true);

    // connection zur local db mit error
    let pool = SqlitePoolOptions::new()
        .connect_with(opts)
        .await
        .map_err(|e| ServerFnError::new(format!("DB Connect Error: {}.", e)))?;

    //öffnet "Änderungs-Warteschlange", tx = transaction, läuft querys ab hier durch und ändert erst ab tx.commit die Inhalte, bisschen wie ein Lock
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| ServerFnError::new(format!("Transaction Error: {}", e)))?;

    // Profile laden
    println!("Loading Profiles...");
    let profiles_json = client
        .database()
        .from("profiles")
        .select("*")
        .execute()
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Profiles Error: {}", e)))?;

    //Profile in Vec parsen
    let profiles: Vec<Profile> = serde_json::from_value(serde_json::Value::Array(profiles_json))
        .map_err(|e| ServerFnError::new(format!("JSON Parse Profiles: {}", e)))?;

    //über Vec mit profilen itterieren und in local db (erst in tx, noch nicht direkt speichern -> in Änderungsqueue) speichern
    for p in profiles {
        sqlx::query("INSERT INTO profiles (id, username) VALUES (?, ?) ON CONFLICT(id) DO UPDATE SET username = excluded.username")
            .bind(p.id).bind(p.username).execute(&mut *tx).await
            .map_err(|e| ServerFnError::new(format!("SQL Error Profile: {}", e)))?;
    }

    // Mitglieder laden
    println!("Loading Members...");
    let members_json = client
        .database()
        .from("group_members")
        .select("*")
        .execute()
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Members Error: {}", e)))?;

    //Mitglieder in Vec parsen
    let members: Vec<GroupMember> = serde_json::from_value(serde_json::Value::Array(members_json))
        .map_err(|e| ServerFnError::new(format!("JSON Parse Members: {}", e)))?;

    //group ids des derzeitigen users sammeln
    let user_group_ids: Vec<String> = members
        .iter()
        .filter(|m| m.user_id == MOCK_USER_ID)
        .map(|m| m.group_id.clone())
        .collect();

    // Gruppen laden
    println!("Loading Groups...");
    let group_ids_for_groups_query = user_group_ids.clone();
    let groups_json = client
        .database()
        .from("groups")
        .select("*")
        .or(move |q| {
            let q = q.eq("owner_id", MOCK_USER_ID);
            if !group_ids_for_groups_query.is_empty() {
                let refs: Vec<&str> = group_ids_for_groups_query
                    .iter()
                    .map(|s| s.as_str())
                    .collect();
                q.r#in("id", &refs)
            } else {
                q
            }
        })
        .execute()
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Groups Error: {}", e)))?;

    //Gruppen in Vec parsen
    let groups: Vec<Group> = serde_json::from_value(serde_json::Value::Array(groups_json))
        .map_err(|e| ServerFnError::new(format!("JSON Parse Groups: {}", e)))?;

    // Set Für Löschung von Gruppen IDs
    let mut remote_group_ids = HashSet::new();

    //nimmt remote Gruppen und packt sie in neues Set remote_group_ids
    for g in groups {
        remote_group_ids.insert(g.id.clone());
        sqlx::query("INSERT INTO groups (id, name, owner_id) VALUES (?, ?, ?) ON CONFLICT(id) DO UPDATE SET name=excluded.name, owner_id=excluded.owner_id")
            .bind(g.id).bind(g.name).bind(g.owner_id).execute(&mut *tx).await
            .map_err(|e| ServerFnError::new(format!("SQL Error Group: {}", e)))?;
    }

    // Cleanup: erstelle Set aus lokalen gruppen ids
    let local_group_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM groups")
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Local Group IDs: {}", e)))?;

    //Cleanup: ist local_id nicht in remote_ids -> löschen
    for local_id in local_group_ids {
        if !remote_group_ids.contains(&local_id) {
            println!("Deleting orphan group: {}", local_id);
            sqlx::query("DELETE FROM groups WHERE id = ?")
                .bind(local_id)
                .execute(&mut *tx)
                .await
                .ok();
        }
    }

    //speichert Mitglieder, die in Gruppen des users sind
    for m in members {
        if !user_group_ids.contains(&m.group_id) {
            continue;
        }
        sqlx::query("INSERT INTO group_members (id, user_id, group_id, role) VALUES (?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET role=excluded.role, group_id=excluded.group_id")
            .bind(m.id).bind(m.user_id).bind(m.group_id).bind(m.role).execute(&mut *tx).await
            .map_err(|e| ServerFnError::new(format!("SQL Error Member: {}", e)))?;
    }

    // Cleanup: löscht alle Members, die nicht zu Gruppen gehören die entfernt worden
    //set aus localen membern erstellen
    let local_member_rows: Vec<(String, String)> =
        sqlx::query_as("SELECT id, group_id FROM group_members")
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| ServerFnError::new(format!("Fetch Local Members: {}", e)))?;
    //Cleanup: wenn member nicht in remote DB -> löschen
    for (mem_id, grp_id) in local_member_rows {
        if !remote_group_ids.contains(&grp_id) {
            sqlx::query("DELETE FROM group_members WHERE id = ?")
                .bind(mem_id)
                .execute(&mut *tx)
                .await
                .ok();
        }
    }

    // Kalender laden
    println!("Loading Calendars...");
    let group_ids_for_cals = user_group_ids.clone();
    let cals_json = client
        .database()
        .from("calendars")
        .select("*")
        .or(move |q| {
            let q = q.eq("owner_id", MOCK_USER_ID);
            if !group_ids_for_cals.is_empty() {
                let refs: Vec<&str> = group_ids_for_cals.iter().map(|s| s.as_str()).collect();
                q.r#in("group_id", &refs)
            } else {
                q
            }
        })
        .execute()
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Calendars Error: {}", e)))?;

    //Kalender in Vec parsen
    let cals: Vec<Calendar> = serde_json::from_value(serde_json::Value::Array(cals_json))
        .map_err(|e| ServerFnError::new(format!("JSON Parse Calendars: {}", e)))?;

    //temporäres set mit den validen keys der Kalender -> für später bei ToDos und Events
    let mut valid_calendar_ids = HashSet::new();
    //temporäres set mit den keys der remote Kalender
    let mut remote_cal_ids = HashSet::new();

    //über Vec mit Kalendern itterieren und in local db (erst in tx, noch nicht direkt speichern -> in Änderungsqueue) speichern
    for c in cals {
        valid_calendar_ids.insert(c.id.clone());
        remote_cal_ids.insert(c.id.clone());
        sqlx::query(r#"INSERT INTO calendars (id, name, type, description, owner_id, group_id, last_mod) 
            VALUES (?, ?, ?, ?, ?, ?, ?) 
            ON CONFLICT(id) DO UPDATE SET 
                name=excluded.name, type=excluded.type, description=excluded.description, 
                owner_id=excluded.owner_id, group_id=excluded.group_id, last_mod=excluded.last_mod"#)
            .bind(c.id).bind(c.name).bind(c.calendar_type).bind(c.description).bind(c.owner_id).bind(c.group_id).bind(c.last_mod)
            .execute(&mut *tx).await.map_err(|e| ServerFnError::new(format!("SQL Error Calendar: {}", e)))?;
    }

    // Cleanup: Kalender die user nicht betreffen entfernen
    //set aus localen ids erstellen
    let local_cal_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM calendars")
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Local Cal IDs: {}", e)))?;

    //sind local ids nicht in remote_ids -> löschen
    for local_id in local_cal_ids {
        if !remote_cal_ids.contains(&local_id) {
            println!("Deleting orphan calendar: {}", local_id);
            sqlx::query("DELETE FROM calendars WHERE id = ?")
                .bind(local_id)
                .execute(&mut *tx)
                .await
                .ok();
        }
    }

    // To-Do Listen laden
    println!("Loading To-Do Lists");
    let group_ids_for_lists = user_group_ids.clone();
    let lists_json = client
        .database()
        .from("todo_lists")
        .select("*")
        .or(move |q| {
            let q = q.eq("owner_id", MOCK_USER_ID);
            if !group_ids_for_lists.is_empty() {
                let refs: Vec<&str> = group_ids_for_lists.iter().map(|s| s.as_str()).collect();
                q.r#in("group_id", &refs)
            } else {
                q
            }
        })
        .execute()
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Todo Lists Error: {}", e)))?;

    //To-Do Listen in Vec parsen
    let lists: Vec<TodoList> = serde_json::from_value(serde_json::Value::Array(lists_json))
        .map_err(|e| ServerFnError::new(format!("JSON Parse Todo Lists: {}", e)))?;

    //temporäres set mit den validen keys der To-Do-Listen
    let mut valid_list_ids = HashSet::new();
    //temporäres set mit den keys der remote Listen
    let mut remote_list_ids = HashSet::new();

    //über Vec mit To-Do Listen itterieren und in local db (erst in tx, noch nicht direkt speichern -> in Änderungsqueue) speichern
    for l in lists {
        valid_list_ids.insert(l.id.clone());
        remote_list_ids.insert(l.id.clone());
        sqlx::query(r#"INSERT INTO todo_lists (id, name, type, description, owner_id, group_id, due_datetime, priority, last_mod) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) 
            ON CONFLICT(id) DO UPDATE SET 
                name=excluded.name, type=excluded.type, description=excluded.description, 
                owner_id=excluded.owner_id, group_id=excluded.group_id, 
                due_datetime=excluded.due_datetime, priority=excluded.priority, last_mod=excluded.last_mod"#)
            .bind(l.id).bind(l.name).bind(l.list_type).bind(l.description).bind(l.owner_id).bind(l.group_id).bind(l.due_datetime).bind(l.priority).bind(l.last_mod)
            .execute(&mut *tx).await.map_err(|e| ServerFnError::new(format!("SQL Error TodoList: {}", e)))?;
    }

    // Cleanup: listen die local sind aber nicht remote -> löschen
    let local_list_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM todo_lists")
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Local List IDs: {}", e)))?;
    for local_id in local_list_ids {
        if !remote_list_ids.contains(&local_id) {
            println!("Deleting orphan list: {}", local_id);
            sqlx::query("DELETE FROM todo_lists WHERE id = ?")
                .bind(local_id)
                .execute(&mut *tx)
                .await
                .ok();
        }
    }

    //Events laden
    println!("Loading Events...");

    //Set in Vektor, damit Supabase ihn als Filter benutzen kann
    let valid_cal_ids_vec: Vec<&str> = valid_calendar_ids.iter().map(|s| s.as_str()).collect();

    //Nur Request starten, wenn Kalender vorhanden
    let ev_json_rows: Vec<serde_json::Value> = if valid_cal_ids_vec.is_empty() {
        vec![]
    } else {
        client
            .database()
            .from("calendar_events")
            .select("*")
            .r#in("calendar_id", &valid_cal_ids_vec)
            .execute()
            .await
            .map_err(|e| ServerFnError::new(format!("Fetch Events Error: {}", e)))?
    };

    //Events in Vec parsen
    let events: Vec<CalendarEvent> = serde_json::from_value(serde_json::Value::Array(ev_json_rows))
        .map_err(|e| ServerFnError::new(format!("JSON Parse Events: {}", e)))?;

    //temporäres set mit den keys der remote Events
    let mut remote_event_ids = HashSet::new();

    //über Vec mit Events itterieren und in local db (erst in tx, noch nicht direkt speichern -> in Änderungsqueue) speichern
    for e in events {
        remote_event_ids.insert(e.id.clone());
        sqlx::query(r#"
            INSERT INTO calendar_events (id, calendar_id, summary, description, date, from_time, to_time, seq, rrule, last_mod) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET 
                summary=excluded.summary, description=excluded.description, date=excluded.date, 
                from_time=excluded.from_time, to_time=excluded.to_time, 
                seq=excluded.seq, rrule=excluded.rrule, last_mod=excluded.last_mod
        "#)
        .bind(e.id).bind(e.calendar_id).bind(e.summary).bind(e.description).bind(e.date).bind(e.from_time).bind(e.to_time).bind(e.seq).bind(e.rrule).bind(e.last_mod)
        .execute(&mut *tx).await.map_err(|e| ServerFnError::new(format!("SQL Error Event: {}", e)))?;
    }

    // Cleanup: local Event ids laden
    let local_event_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM calendar_events")
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Local Event IDs: {}", e)))?;
    // Cleanup: local Event ids nicht in remote event ids -> löschen
    for local_id in local_event_ids {
        if !remote_event_ids.contains(&local_id) {
            println!("Deleting orphan event: {}", local_id);
            sqlx::query("DELETE FROM calendar_events WHERE id = ?")
                .bind(local_id)
                .execute(&mut *tx)
                .await
                .ok();
        }
    }

    //ToDos Laden
    println!("Loading To-Do's...");

    //Set in Vektor, damit Supabase ihn als Filter benutzen kann
    let valid_list_ids_vec: Vec<&str> = valid_list_ids.iter().map(|s| s.as_str()).collect();

    let todo_json_rows: Vec<serde_json::Value> = if valid_list_ids_vec.is_empty() {
        vec![]
    } else {
        client
            .database()
            .from("todo_events")
            .select("*")
            .r#in("todo_list_id", &valid_list_ids_vec)
            .execute()
            .await
            .map_err(|e| ServerFnError::new(format!("Fetch Todo Items Error: {}", e)))?
    };

    //To-Do's in Vec parsen
    let todos: Vec<TodoEvent> = serde_json::from_value(serde_json::Value::Array(todo_json_rows))
        .map_err(|e| ServerFnError::new(format!("JSON Parse Todo Items: {}", e)))?;

    //temporäres set mit den keys der remote ToDo's
    let mut remote_todo_ids = HashSet::new();

    //über Vec mit To-Do's itterieren und in local db (erst in tx, noch nicht direkt speichern -> in Änderungsqueue) speichern
    for t in todos {
        remote_todo_ids.insert(t.id.clone());
        sqlx::query(r#"
            INSERT INTO todo_events (id, todo_list_id, summary, description, completed, due_datetime, priority, seq, last_mod) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET 
                summary=excluded.summary, description=excluded.description, completed=excluded.completed, 
                due_datetime=excluded.due_datetime, priority=excluded.priority, seq=excluded.seq, last_mod=excluded.last_mod
        "#)
        .bind(t.id).bind(t.todo_list_id).bind(t.summary).bind(t.description).bind(t.completed).bind(t.due_datetime).bind(t.priority).bind(t.seq).bind(t.last_mod)
        .execute(&mut *tx).await.map_err(|e| ServerFnError::new(format!("SQL Error TodoItem: {}", e)))?;
    }

    // Cleanup: set aus lokalen todo ids erstellen
    let local_todo_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM todo_events")
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Local Todo IDs: {}", e)))?;

    //Cleanup: locale Todo id nicht in remote ToDo ids -> löschen
    for local_id in local_todo_ids {
        if !remote_todo_ids.contains(&local_id) {
            println!("Deleting orphan todo: {}", local_id);
            sqlx::query("DELETE FROM todo_events WHERE id = ?")
                .bind(local_id)
                .execute(&mut *tx)
                .await
                .ok();
        }
    }

    //Hier Änderungsqueue zusammenfügen und "commiten"
    tx.commit()
        .await
        .map_err(|e| ServerFnError::new(format!("Commit Error: {}", e)))?;

    println!("Sync completed");
    Ok(())
}
