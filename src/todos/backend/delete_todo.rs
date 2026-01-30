//KOMMENTARE NICHT FERTIG

use crate::auth::backend::SUPABASE_URL;
use crate::auth::backend::*;
use crate::database::local::sync_local_db::sync_local_to_remote_db; // <--- NEUER IMPORT
use crate::utils::functions::get_user_id_and_session_token;
use dioxus::prelude::*;
use reqwest::StatusCode;
use uuid::Uuid;

// #[server]
pub async fn delete_todo_event(todo_id: Uuid) -> Result<StatusCode, ServerFnError> {
    println!("\n🗑️ START: delete_todo_event für ID '{}'", todo_id);

    // 1. AUTH
    let (_user_id, token) = match get_user_id_and_session_token().await {
        Ok(data) => data,
        Err(e) => {
            println!("❌ ABBRUCH: Auth fehlgeschlagen!");
            return Err(e);
        }
    };

    // 2. URL
    let url_delete = format!("{}/rest/v1/todo_events?id=eq.{}", SUPABASE_URL, todo_id);

    let client = reqwest::Client::new();

    // 3. REQUEST
    let response_result = client
        .delete(&url_delete)
        .bearer_auth(token)
        .header("apikey", ANON_KEY)
        .header("Content-Type", "application/json")
        .send()
        .await;

    // 4. CHECK
    match response_result {
        Ok(response) => {
            let status = response.status();

            if !status.is_success() {
                let error_text = response.text().await.unwrap_or_default();
                println!("❌ SUPABASE API FEHLER: {}", error_text);
                return Err(ServerFnError::new(format!(
                    "Supabase Error {}: {}",
                    status, error_text
                )));
            }

            println!("✅ SUCCESS! Todo gelöscht.");

            // --- NEU: SYNC AUFRUFEN ---
            println!("🔄 Trigger Sync nach Delete...");
            if let Err(e) = sync_local_to_remote_db().await {
                println!("⚠️ Sync Fehler (Delete): {:?}", e);
                // Wir returnen hier keinen Fehler, da das eigentliche Löschen erfolgreich war
            }
            // --------------------------

            Ok(status)
        }
        Err(e) => {
            println!("❌ NETZWERK FEHLER: {}", e);
            Err(ServerFnError::new(format!("Network Error: {}", e)))
        }
    }
}
