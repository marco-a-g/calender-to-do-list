use crate::database::local::sync_local_db::sync_function;
use std::time::Duration;
use tokio::time::sleep;

pub async fn start_heartbeat() {
    //startet neuen parallelen Task
    tokio::spawn(async move {
        loop {
            // 5 min warten
            sleep(Duration::from_secs(180)).await;
            println!("Heartbeat triggered: starting sync of local DB");
            match sync_function().await {
                Ok(_) => {
                    println!("Heartbeat-sync completed.");
                }
                Err(e) => {
                    eprintln!("Error on heartbeat-sync: {}", e);
                }
            }
        }
    });
}
