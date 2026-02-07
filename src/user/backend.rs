use dioxus::{fullstack::headers::Server, prelude::*};
use uuid::Uuid;

use crate::{
    auth::{
        backend::{ANON_KEY, AuthError, AuthStatus, SUPABASE_URL, get_client},
        ui::RegisterView,
    },
    utils::{
        functions::get_user_id_and_session_token,
        structs::{Profile, ProfileWrite},
    },
};

// #[server]
pub async fn get_user_by_username(username: &str) -> Result<Profile, ServerFnError> {
    let url = format!("{}/rest/v1/profiles?username=eq.{}", SUPABASE_URL, username);
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

    let user = user.remove(0);
    Ok(user)
}

// #[server]
pub async fn get_user_by_id(id: Uuid) -> Result<Profile, ServerFnError> {
    let url = format!("{}/rest/v1/profiles?id=eq.{}", SUPABASE_URL, id);
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

    let user = user.remove(0);
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
    if !is_username_available(username).await {
        return Err(ServerFnError::new("Username already taken"));
    }

    let url = format!("{}/rest/v1/profiles", SUPABASE_URL);
    let (id, token) = get_user_id_and_session_token().await?;

    let profile = ProfileWrite {
        username: username.into(),
    };

    let reqwest_client = reqwest::Client::new();
    let res = reqwest_client
        .post(url)
        .bearer_auth(token)
        .header("apikey", ANON_KEY)
        .header("Content-Type", "application/json")
        .json(&profile)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("create_profile: Reqwest error: {}", e)))?;

    Ok(AuthStatus::Authenticated { user_id: id })
}

// #[server]
pub async fn update_username(username: &str) -> Result<(), ServerFnError> {
    if !is_username_available(username).await {
        return Err(ServerFnError::new("Username already taken"));
    }

    let (id, token) = get_user_id_and_session_token().await?;
    let url = format!("{}/rest/v1/profiles?id=eq.{}", SUPABASE_URL, id);

    let profile = ProfileWrite {
        username: username.into(),
    };

    let reqwest_client = reqwest::Client::new();
    let res = reqwest_client
        .patch(url)
        .bearer_auth(token)
        .header("apikey", ANON_KEY)
        .header("Content-Type", "application/json")
        .json(&profile)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("create_profile: Reqwest error: {}", e)))?;

    println!("Result: {:?}", res);

    Ok(())
}
