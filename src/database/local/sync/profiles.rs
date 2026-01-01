use crate::database::local::sync_local_db::Profile;
use dioxus::prelude::ServerFnError;
use sqlx::{Sqlite, Transaction};
use supabase::Client;

pub async fn sync_profiles(
    client: &Client,
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), ServerFnError> {
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
            .bind(p.id).bind(p.username).execute(&mut **tx).await
            .map_err(|e| ServerFnError::new(format!("SQL Error Profile: {}", e)))?;
    }

    Ok(())
}
