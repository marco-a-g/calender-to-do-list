use crate::database::local::sync_local_db::sync_local_to_remote_db;
use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

/// Starts background task that synchronizes the databases periodically.
///
/// Triggers the `sync_local_to_remote_db` upon starting and then pauses for (a given timeframe) between each execution.
///
/// Errors encountered during sync are caught and printed, so that a single failed sync attempt does not crash the App or stop the heartbeat mechanism.
pub async fn start_heartbeat() {
    spawn(async move {
        loop {
            println!("Heartbeat triggered: starting sync of local DB");
            match sync_local_to_remote_db().await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error on heartbeat-sync: {}", e);
                }
            }
            sleep(Duration::from_secs(180)).await;
        }
    });
}
