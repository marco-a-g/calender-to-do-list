use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use supabase::Client;

// Config -> Später raus sobald auth steht

const SUPABASE_URL: &str = "https://wyqawnnkpusgtnhmeebn.supabase.co";
const SUPABASE_SERVICE_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Ind5cWF3bm5rcHVzZ3RuaG1lZWJuIiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImlhdCI6MTc2NTg0MzkyOSwiZXhwIjoyMDgxNDE5OTI5fQ.s3Gmfv0u89h5ZjguByboQbfjPADR3p9iVfcIeYyAoFY";
const MOCK_USER_ID: &str = "24074bae-904b-44ab-b9d0-0934c309027e";

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
//sync logik
pub async fn sync_function() -> Result<(), ServerFnError> {
    println!("Start sync for User: {}", MOCK_USER_ID);

    // Client aufsetzen
    let client = Client::new(SUPABASE_URL, SUPABASE_SERVICE_KEY)
        .map_err(|e| ServerFnError::new(format!("Supabase Init Error: {}", e)))?;

    // Pfat für locale db
    let db_path = "sqlite:src/database/local/local_Database.db";
    // connection zur local db mit error
    let pool = SqlitePoolOptions::new()
        .connect(db_path)
        .await
        .map_err(|e| ServerFnError::new(format!("DB Connect Error: {}.", e)))?;

    //öffnet "Änderungs-Warteschlange", läuft querys ab hier durch und ändert erst ab tx.commit die Inhalte, bisschen wie ein Lock
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
            .bind(p.id)
            .bind(p.username)
            .execute(&mut *tx).await
            .map_err(|e| ServerFnError::new(format!("SQL Error Profile: {}", e)))?;
    }

    // Gruppen laden
    println!("Loading Groups...");
    let groups_json = client
        .database()
        .from("groups")
        .select("*")
        .execute()
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Groups Error: {}", e)))?;

    //Gruppen in Vec parsen
    let groups: Vec<Group> = serde_json::from_value(serde_json::Value::Array(groups_json))
        .map_err(|e| ServerFnError::new(format!("JSON Parse Groups: {}", e)))?;

    //über Vec mit Gruppen itterieren und in local db (erst in tx, noch nicht direkt speichern -> in Änderungsqueue) speichern
    for g in groups {
        sqlx::query("INSERT INTO groups (id, name, owner_id) VALUES (?, ?, ?) ON CONFLICT(id) DO UPDATE SET name=excluded.name, owner_id=excluded.owner_id")
            .bind(g.id)
            .bind(g.name)
            .bind(g.owner_id)
            .execute(&mut *tx).await
            .map_err(|e| ServerFnError::new(format!("SQL Error Group: {}", e)))?;
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

    //itteriert und sammeld alle Gruppen die dem user gehören
    let user_group_ids: Vec<String> = members
        .iter()
        .filter(|m| m.user_id == MOCK_USER_ID)
        .map(|m| m.group_id.clone())
        .collect();

    //über Vec mit Mitgliedern itterieren und in local db (erst in tx, noch nicht direkt speichern -> in Änderungsqueue) speichern
    for m in members {
        sqlx::query("INSERT INTO group_members (id, user_id, group_id, role) VALUES (?, ?, ?, ?) ON CONFLICT(id) DO NOTHING")
            .bind(m.id)
            .bind(m.user_id)
            .bind(m.group_id)
            .bind(m.role)
            .execute(&mut *tx).await
            .map_err(|e| ServerFnError::new(format!("SQL Error Member: {}", e)))?;
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

    //über Vec mit Kalendern itterieren und in local db (erst in tx, noch nicht direkt speichern -> in Änderungsqueue) speichern
    for c in cals {
        sqlx::query(
            r#"
            INSERT INTO calendars (id, name, type, description, owner_id, group_id, last_mod) 
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET last_mod=excluded.last_mod
        "#,
        )
        .bind(c.id)
        .bind(c.name)
        .bind(c.calendar_type)
        .bind(c.description)
        .bind(c.owner_id)
        .bind(c.group_id)
        .bind(c.last_mod)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServerFnError::new(format!("SQL Error Calendar: {}", e)))?;
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

    //über Vec mit To-Do Listen itterieren und in local db (erst in tx, noch nicht direkt speichern -> in Änderungsqueue) speichern
    for l in lists {
        sqlx::query(r#"
            INSERT INTO todo_lists (id, name, type, description, owner_id, group_id, due_datetime, priority, last_mod) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET last_mod=excluded.last_mod
        "#)
        .bind(l.id)
        .bind(l.name)
        .bind(l.list_type)
        .bind(l.description)
        .bind(l.owner_id)
        .bind(l.group_id)
        .bind(l.due_datetime)
        .bind(l.priority)
        .bind(l.last_mod)
        .execute(&mut *tx).await
        .map_err(|e| ServerFnError::new(format!("SQL Error TodoList: {}", e)))?;
    }

    //Kalenderevents laden
    println!("Loading Events");
    let ev_json = client
        .database()
        .from("calendar_events")
        .select("*")
        .execute()
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Events Error: {}", e)))?;

    //Events in Vec parsen
    let events: Vec<CalendarEvent> = serde_json::from_value(serde_json::Value::Array(ev_json))
        .map_err(|e| ServerFnError::new(format!("JSON Parse Events: {}", e)))?;

    //über Vec mit Events itterieren und in local db (erst in tx, noch nicht direkt speichern -> in Änderungsqueue) speichern
    for e in events {
        sqlx::query(r#"
            INSERT OR IGNORE INTO calendar_events (id, calendar_id, summary, description, date, from_time, to_time, seq, rrule, last_mod) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(e.id)
        .bind(e.calendar_id)
        .bind(e.summary)
        .bind(e.description)
        .bind(e.date)
        .bind(e.from_time)
        .bind(e.to_time)
        .bind(e.seq)
        .bind(e.rrule)
        .bind(e.last_mod)
        .execute(&mut *tx).await
        .map_err(|e| ServerFnError::new(format!("SQL Error Event: {}", e)))?;
    }

    //ToDos Laden
    println!("Loading To-Do's");
    let todo_json = client
        .database()
        .from("todo_events")
        .select("*")
        .execute()
        .await
        .map_err(|e| ServerFnError::new(format!("Fetch Todo Items Error: {}", e)))?;

    //To-Do's in Vec parsen
    let todos: Vec<TodoEvent> = serde_json::from_value(serde_json::Value::Array(todo_json))
        .map_err(|e| ServerFnError::new(format!("JSON Parse Todo Items: {}", e)))?;

    //über Vec mit To-Do's itterieren und in local db (erst in tx, noch nicht direkt speichern -> in Änderungsqueue) speichern
    for t in todos {
        sqlx::query(r#"
            INSERT OR IGNORE INTO todo_events (id, todo_list_id, summary, description, completed, due_datetime, priority, seq, last_mod) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(t.id)
        .bind(t.todo_list_id)
        .bind(t.summary)
        .bind(t.description)
        .bind(t.completed)
        .bind(t.due_datetime)
        .bind(t.priority)
        .bind(t.seq)
        .bind(t.last_mod)
        .execute(&mut *tx).await
        .map_err(|e| ServerFnError::new(format!("SQL Error TodoItem: {}", e)))?;
    }

    //Hier Änderungsqueue zusammenfügen und "commiten"
    tx.commit()
        .await
        .map_err(|e| ServerFnError::new(format!("Commit Error: {}", e)))?;

    println!("Sync completed");
    Ok(())
}
