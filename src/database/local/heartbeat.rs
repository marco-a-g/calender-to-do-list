use crate::database::local::sync_local_db::sync_local_to_remote_db;
use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

pub async fn start_heartbeat() {
    //startet neuen parallelen Task für heartbeat
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
