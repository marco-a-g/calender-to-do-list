use crate::utils::structs::ProfileLight;
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
    let profiles: Vec<ProfileLight> =
        serde_json::from_value(serde_json::Value::Array(profiles_json))
            .map_err(|e| ServerFnError::new(format!("JSON Parse Profiles: {}", e)))?;

    //neues set aus Remote-Db Id's für Löschung von verwaisten einträgen
    let remote_ids = profiles
        .iter()
        .map(|p| format!("'{}'", p.id))
        .collect::<Vec<String>>()
        .join(",");

    //über Vec mit profilen itterieren und in local db (erst in tx, noch nicht direkt speichern -> in Änderungsqueue) speichern
    for p in profiles {
        sqlx::query("INSERT INTO profiles (id, username, created_at) VALUES (?, ?, datetime('now')) ON CONFLICT(id) DO UPDATE SET username = excluded.username")
            .bind(p.id)
            .bind(p.username)
            .execute(&mut **tx).await
            .map_err(|e| ServerFnError::new(format!("SQL Error Profile: {}", e)))?;
    } //created_at nochmal anschauen

    // CleanUp: Profile die lokal noch da sind aber nicht in remote -> löschen
    //ist remote table für profiles leer?
    if remote_ids.is_empty() {
        sqlx::query("DELETE FROM profiles")
            .execute(&mut **tx)
            .await
            .map_err(|e| ServerFnError::new(format!("SQL Cleanup (Delete All): {}", e)))?;
    } else {
        // sonst lösche die "Waisen"
        let cleanup_sql = format!("DELETE FROM profiles WHERE id NOT IN ({})", remote_ids);

        sqlx::query(&cleanup_sql)
            .execute(&mut **tx)
            .await
            .map_err(|e| ServerFnError::new(format!("SQL Cleanup (Sync Deletion): {}", e)))?;
    }

    Ok(())
}
