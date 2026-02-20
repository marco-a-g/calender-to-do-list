use crate::auth::backend::SUPABASE_URL;
use crate::auth::backend::*;
use crate::database::local::sync_local_db::sync_local_to_remote_db;
use crate::utils::functions::get_user_id_and_session_token;
use dioxus::prelude::*;
use reqwest::StatusCode;
use uuid::Uuid;

// #[server]
pub async fn delete_todo_list(list_id: Uuid) -> Result<StatusCode, ServerFnError> {
    println!("Starting delete_todo_list for:'{}'", list_id);

    let (_user_id_str, token) = match get_user_id_and_session_token().await {
        Ok(data) => data,
        Err(e) => {
            println!("Error: Not authed");
            return Err(e);
        }
    };

    // HTTP Config
    let url_delete_list = format!("{}/rest/v1/todo_lists?id=eq.{}", SUPABASE_URL, list_id);
    let client = reqwest::Client::new();
    let response_result = client
        .delete(&url_delete_list)
        .bearer_auth(token)
        .header("apikey", ANON_KEY)
        .header("Content-Type", "application/json")
        .send()
        .await;

    // Response check
    match response_result {
        Ok(response) => {
            let status = response.status();
            if !status.is_success() {
                let error_text = response.text().await.unwrap_or_default();
                println!("Supabase response error: {}", error_text);
                return Err(ServerFnError::new(format!(
                    "Supabase Error {}: {}",
                    status, error_text
                )));
            }
            println!("Deleted ToDo-List in Supabase.");
            if let Err(e) = sync_local_to_remote_db().await {
                println!("Error on sync after delete_list: {:?}", e);
            }
            Ok(status)
        }
        Err(e) => Err(ServerFnError::new(format!("Network Error?: {}", e))),
    }
}
