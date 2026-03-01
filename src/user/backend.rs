//! User profile functionality
use dioxus::prelude::*;
use serde_json::json;
use server_fn::error::ServerFnError;
use uuid::Uuid;

use crate::{
    auth::backend::{ANON_KEY, AuthStatus, SUPABASE_URL},
    utils::{functions::get_user_id_and_session_token, structs::Profile},
};

/// Get Profile object by ``username``
///
/// ### Returns
///
/// Result: `Some(Profile)` if user exists
///
/// Result: `None` if user does not exist
///
/// ### Errors
///
/// Throws ``ServerFnError``
// #[server]
pub async fn get_user_by_username(username: &str) -> Result<Option<Profile>, ServerFnError> {
    let username = username.trim();
    let url = format!("{}/rest/v1/profiles?username=eq.{}", SUPABASE_URL, username); // theoretically possible url manipulation, but rls handles it anyway
    let token = get_user_id_and_session_token().await?.1;

    let reqwest_client = reqwest::Client::new();
    let res = reqwest_client
        .get(url)
        .header("apikey", ANON_KEY)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("get_user_by_username(): Reqwest error: {}", e)))?;

    if !res.status().is_success() {
        return Err(ServerFnError::new(format!(
            "get_user_by_username(): Request not successful: {}",
            res.status()
        )));
    }

    let mut user: Vec<Profile> = res.json().await.map_err(|e| {
        ServerFnError::new(format!(
            "get_user_by_username(): Error parsing result into json: {}",
            e
        ))
    })?;

    if user.is_empty() {
        return Ok(None);
    }

    let user = user.remove(0); // pull user out of Vec
    Ok(Some(user))
}

/// Get Profile object by ``id``
///
/// ### Returns
///
/// Result: `Some(Profile)` if user exists
///
/// Result: `None` if user does not exist
///
/// ### Errors
///
/// Throws ``ServerFnError``
// #[server]
pub async fn get_user_by_id(id: Uuid) -> Result<Option<Profile>, ServerFnError> {
    let url = format!("{}/rest/v1/profiles?id=eq.{}", SUPABASE_URL, id); // theoretically possible url manipulation, but rls handles it anyway
    let token = get_user_id_and_session_token().await?.1;

    let reqwest_client = reqwest::Client::new();
    let res = reqwest_client
        .get(url)
        .header("apikey", ANON_KEY)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("get_user_by_id(): Reqwest error: {}", e)))?;

    if !res.status().is_success() {
        return Err(ServerFnError::new(format!(
            "get_user_by_id(): Request not successful: {}",
            res.status()
        )));
    }

    let mut user: Vec<Profile> = res.json().await.map_err(|e| {
        ServerFnError::new(format!(
            "get_user_by_id(): Error parsing result into json {}",
            e
        ))
    })?;

    if user.is_empty() {
        return Ok(None);
    }

    let user = user.remove(0); // pull user out of Vec
    Ok(Some(user))
}

/// Checks if username ist available
///
/// This function treats errors as false
///
/// ### Returns
///
/// `true` if username is available else `false`
// #[server]
pub async fn is_username_available(username: &str) -> bool {
    match get_user_by_username(username).await {
        Ok(Some(_)) => false,
        Ok(None) => true,
        Err(_) => false, // treat error as not available, later maybe -> Result<bool, ServerFnError> with error handling in ui
    }
}

/// Get own username
///
/// ### Returns
///
/// Result: `String` if user exists
///
/// ### Errors
///
/// Throws ``ServerFnError``
// #[server]
pub async fn get_own_username() -> Result<String, ServerFnError> {
    let id = get_user_id_and_session_token().await?.0;
    match get_user_by_id(id).await? {
        Some(user) => Ok(user.username),
        None => Err(ServerFnError::new("User not found")),
    }
}

/// Create profile from `username`
///
/// Trims input
///
/// ### Returns
///
/// Result: `AuthStatus { user id }`
///
/// ### Errors
///
/// Throws `ServerFnError`
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
        .map_err(|e| ServerFnError::new(format!("create_profile(): Reqwest error: {}", e)))?;

    if !res.status().is_success() {
        return Err(ServerFnError::new(format!(
            "create_profile(): Request not successful: {}",
            res.status()
        )));
    }

    Ok(AuthStatus::Authenticated { user_id: id })
}

/// Update username
///
/// Trims input
///
/// ### Errors
///
/// Throws `ServerFnError`
// #[server]
pub async fn update_username(username: &str) -> Result<(), ServerFnError> {
    let username = username.trim();
    if !is_username_available(username).await {
        return Err(ServerFnError::new("Username already taken"));
    }

    let (id, token) = get_user_id_and_session_token().await?;
    let url = format!("{}/rest/v1/profiles?id=eq.{}", SUPABASE_URL, id); // theoretically possible url manipulation, but rls handles it anyway

    let reqwest_client = reqwest::Client::new();
    let res = reqwest_client
        .patch(url)
        .bearer_auth(token)
        .header("apikey", ANON_KEY)
        .header("Content-Type", "application/json")
        .json(&json!({"username": username}))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("update_username(): Reqwest error: {}", e)))?;

    if !res.status().is_success() {
        return Err(ServerFnError::new(format!(
            "update_username(): Request not successful: {}",
            res.status()
        )));
    }

    Ok(())
}
