use dioxus::prelude::*;
use serde_json::json;
use uuid::Uuid;

use crate::{
    auth::backend::{ANON_KEY, AuthStatus, SUPABASE_URL},
    utils::{functions::get_user_id_and_session_token, structs::Profile},
};

// todo: username validation

// #[server]
pub async fn get_user_by_username(username: &str) -> Result<Profile, ServerFnError> {
    let username = username.trim();
    let url = format!("{}/rest/v1/profiles?username=eq.{}", SUPABASE_URL, username); // theoretisch url manipulation, aber wegen rls egal
    let token = get_user_id_and_session_token().await?.1;

    let reqwest_client = reqwest::Client::new();
    let res = reqwest_client
        .get(url)
        .header("apikey", ANON_KEY)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("get_user_by_username: Reqwest error: {}", e)))?;

    if !res.status().is_success() {
        println!("Statuscode: {}\nText: {:?}", res.status(), res.text().await);
        return Err(ServerFnError::new("Request not successful"));
    }

    let mut user: Vec<Profile> = res.json().await.map_err(|e| {
        ServerFnError::new(format!(
            "get_user_by_username: error parsing result into json {}",
            e
        ))
    })?;

    if user.is_empty() {
        return Err(ServerFnError::new("User not found"));
    }

    let user = user.remove(0); // pull user out of Vec
    Ok(user)
}

// #[server]
pub async fn get_user_by_id(id: Uuid) -> Result<Profile, ServerFnError> {
    let url = format!("{}/rest/v1/profiles?id=eq.{}", SUPABASE_URL, id); // theoretisch url manipulation, aber wegen rls egal
    let token = get_user_id_and_session_token().await?.1;

    let reqwest_client = reqwest::Client::new();
    let res = reqwest_client
        .get(url)
        .header("apikey", ANON_KEY)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("get_user_by_id: Reqwest error: {}", e)))?;

    if !res.status().is_success() {
        println!("Statuscode: {}\nText: {:?}", res.status(), res.text().await);
        return Err(ServerFnError::new("Request not successful"));
    }

    let mut user: Vec<Profile> = res.json().await.map_err(|e| {
        ServerFnError::new(format!(
            "get_user_by_id: error parsing result into json {}",
            e
        ))
    })?;

    if user.is_empty() {
        return Err(ServerFnError::new("User not found"));
    }

    let user = user.remove(0); // pull user out of Vec
    Ok(user)
}

// #[server]
pub async fn is_username_available(username: &str) -> bool {
    if get_user_by_username(username).await.is_ok() {
        return false;
    }
    true
}

// #[server]
pub async fn get_own_username() -> Result<String, ServerFnError> {
    let id = get_user_id_and_session_token().await?.0;
    Ok(get_user_by_id(id).await?.username)
}

// #[server]
pub async fn create_profile(username: &str) -> Result<AuthStatus, ServerFnError> {
    let username = username.trim();
    if !is_username_available(username).await {
        return Err(ServerFnError::new("Username already taken"));
    }

    let url = format!("{}/rest/v1/profiles", SUPABASE_URL);
    let (id, token) = get_user_id_and_session_token().await?;

    let reqwest_client = reqwest::Client::new();
    let res = reqwest_client
        .post(url)
        .bearer_auth(token)
        .header("apikey", ANON_KEY)
        .header("Content-Type", "application/json")
        .json(&json!({"username": username}))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("create_profile: Reqwest error: {}", e)))?;

    if !res.status().is_success() {
        println!("Statuscode: {}\nText: {:?}", res.status(), res.text().await);
        return Err(ServerFnError::new("Request not successful"));
    }

    Ok(AuthStatus::Authenticated { user_id: id })
}

// #[server]
pub async fn update_username(username: &str) -> Result<(), ServerFnError> {
    let username = username.trim();
    if !is_username_available(username).await {
        return Err(ServerFnError::new("Username already taken"));
    }

    let (id, token) = get_user_id_and_session_token().await?;
    let url = format!("{}/rest/v1/profiles?id=eq.{}", SUPABASE_URL, id); // theoretisch url manipulation, aber wegen rls egal

    let reqwest_client = reqwest::Client::new();
    let res = reqwest_client
        .patch(url)
        .bearer_auth(token)
        .header("apikey", ANON_KEY)
        .header("Content-Type", "application/json")
        .json(&json!({"username": username}))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("update_username: Reqwest error: {}", e)))?;

    if !res.status().is_success() {
        println!("Statuscode: {}\nText: {:?}", res.status(), res.text().await);
        return Err(ServerFnError::new("Request not successful"));
    }

    Ok(())
}
