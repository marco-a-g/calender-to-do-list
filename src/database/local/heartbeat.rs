use crate::database::local::sync_local_db::sync_local_to_remote_db;
use std::time::Duration;
use tokio::time::sleep;

pub async fn start_heartbeat() {
    //startet neuen parallelen Task für heartbeat
    tokio::spawn(async move {
        loop {
            println!("Heartbeat triggered: starting sync of local DB");
            match sync_local_to_remote_db().await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error on heartbeat-sync: {}", e);
                }
            }
            // bestimmte Zeit warten
            sleep(Duration::from_secs(300)).await;
        }
    });
}
