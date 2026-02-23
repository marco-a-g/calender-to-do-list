// Local members backend (SQLite)
// Uses the central database pool and maps data for frontend display
use server_fn::error::ServerFnError;

use crate::database::local::init_fetch::init_fetch_local_db::{
    fetch_group_members_lokal_db, fetch_profiles_lokal_db,
};
use dioxus::prelude::*;

// Compact DTO returned to the frontend
// (group_id, user_id, username, role)
pub type MemberTransfer = (String, String, String, String);

// Returns all members for the given group_id from the local DB
// Filters out 'invited' roles and enriches with username from profiles
//#[server]
pub async fn fetch_members(group_id: String) -> Result<Vec<MemberTransfer>, ServerFnError> {
    // Fetch all members and profiles from central functions
    let all_members = fetch_group_members_lokal_db().await?;
    let all_profiles = fetch_profiles_lokal_db().await?;

    // Build a username lookup map
    let username_map: std::collections::HashMap<String, String> = all_profiles
        .into_iter()
        .map(|p| (p.id, p.username))
        .collect();

    // Filter by group_id, exclude 'invited', and map to transfer type
    let result: Vec<MemberTransfer> = all_members
        .into_iter()
        .filter(|m| m.group_id == group_id && m.role != "invited")
        .map(|m| {
            let username = username_map
                .get(&m.user_id)
                .cloned()
                .unwrap_or_else(|| "<no profile>".to_string());
            (group_id.clone(), m.user_id, username, m.role)
        })
        .collect();

    Ok(result)
}
