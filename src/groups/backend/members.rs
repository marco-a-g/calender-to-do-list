// Local members backend (SQLite)

use dioxus::prelude::*;
use sqlx::{ConnectOptions, SqlitePool};
use std::str::FromStr;

// Compact DTO returned to the frontend
// (group_id, user_id, username, role)
pub type MemberTransfer = (String, String, String, String);

// Opens a connection pool to the local SQLite database
// We keep this as a helper so server functions stay focused on query logic
async fn get_local_db_pool() -> Result<SqlitePool, ServerFnError> {
    // Path is relative to the project root; adjust if your runtime working directory differs
    let db_path = "sqlite:src/database/local/local_Database.db";
    let connect_options = sqlx::sqlite::SqliteConnectOptions::from_str(db_path)
        .map_err(|e| ServerFnError::new(format!("DB path error: {e}")))?
        .create_if_missing(false)
        .foreign_keys(true)
        .disable_statement_logging();

    SqlitePool::connect_with(connect_options)
        .await
        .map_err(|e| ServerFnError::new(format!("DB connection error: {e}")))
}

// Returns all members for the given group_id from the local DB
// We LEFT JOIN profiles to enrich member rows with a username when available
#[server]
pub async fn fetch_members(group_id: String) -> Result<Vec<MemberTransfer>, ServerFnError> {
    let pool = get_local_db_pool().await?;

    // Query shape: (user_id, username (nullable), role)
    // Username can be NULL if there is no matching profile row
    let rows: Vec<(String, Option<String>, String)> = sqlx::query_as(
        r#"
        SELECT
            gm.user_id,
            p.username,
            gm.role
        FROM group_members gm
        LEFT JOIN profiles p ON gm.user_id = p.id
        WHERE gm.group_id = ?
        AND gm.role != 'invited'
        ORDER BY gm.joined_at ASC
        "#,
    )
    .bind(&group_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("DB query error (fetch_members): {e}")))?;

    // Attach the group_id to each row to match the frontend's expected transfer type
    // Use a fallback label if the user has no profile/username yet
    let result = rows
        .into_iter()
        .map(|(user_id, username_opt, role)| {
            (
                group_id.clone(),
                user_id,
                username_opt.unwrap_or_else(|| "<no profile>".to_string()),
                role,
            )
        })
        .collect();

    Ok(result)
}
