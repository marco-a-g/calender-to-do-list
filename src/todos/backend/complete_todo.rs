use crate::auth::backend::SUPABASE_URL;
use crate::auth::backend::*;
use crate::database::local::sync_local_db::sync_local_to_remote_db;
use crate::utils::functions::get_user_id_and_session_token;
use dioxus::prelude::*;
use reqwest::StatusCode;
use serde::Serialize;
use uuid::Uuid;

// LLM: Lieber Payload reduzieren so:
#[derive(Serialize)]
struct UpdateTodoStatus {
    completed: bool,
}

// #[server]
pub async fn complete_todo_event(todo_id: Uuid) -> Result<StatusCode, ServerFnError> {
    println!("Starting complete_todo_event on:'{}'", todo_id);
    let (_user_id_str, token) = match get_user_id_and_session_token().await {
        Ok(data) => data,
        Err(e) => {
            println!("Not Authed!");
            return Err(e);
        }
    };

    //Http Config
    let url_update = format!("{}/rest/v1/todo_events?id=eq.{}", SUPABASE_URL, todo_id);
    // LLM: Nur zu änderndes Feld senden
    let payload = UpdateTodoStatus { completed: true };
    let client = reqwest::Client::new();
    let response_result = client
        .patch(&url_update)
        .bearer_auth(token)
        .header("apikey", ANON_KEY)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await;

    // Response check
    match response_result {
        Ok(response) => {
            let status = response.status();
            if !status.is_success() {
                let error_text = response.text().await.unwrap_or_default();
                println!("Supabase respons error: {}", error_text);
                return Err(ServerFnError::new(format!(
                    "Supabase Error {}: {}",
                    status, error_text
                )));
            }
            println!("Completed ToDo in Supabase.");
            if let Err(e) = sync_local_to_remote_db().await {
                println!("Error on sync after complete_todo: {:?}", e);
            }
            Ok(status)
        }
        Err(e) => Err(ServerFnError::new(format!("Network Error?: {}", e))),
    }
}
