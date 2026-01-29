use dioxus::prelude::*;
use sqlx::{ConnectOptions, SqlitePool};
use std::str::FromStr;

// (group_id, user_id, username, role)
pub type MemberTransfer = (String, String, String, String);

async fn get_local_db_pool() -> Result<SqlitePool, ServerFnError> {
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

#[server]
pub async fn fetch_members(group_id: String) -> Result<Vec<MemberTransfer>, ServerFnError> {
    let pool = get_local_db_pool().await?;

    let rows: Vec<(String, Option<String>, String)> = sqlx::query_as(
        r#"
        SELECT
            gm.user_id,
            p.username,
            gm.role
        FROM group_members gm
        LEFT JOIN profiles p ON gm.user_id = p.id
        WHERE gm.group_id = ?
        ORDER BY gm.joined_at ASC
        "#,
    )
    .bind(&group_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("DB query error (fetch_members): {e}")))?;

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