#[path = "common.rs"]
mod common;

use egs_api::api::types::presence::{PresenceActivity, PresenceUpdate};
use egs_api::EpicGames;

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut egs = EpicGames::new();

    if !common::login_or_restore(&mut egs).await {
        eprintln!("Authentication failed. Run the 'auth' example first.");
        std::process::exit(1);
    }

    println!("=== Update Presence ===\n");

    let session_id = egs
        .user_details()
        .account_id
        .unwrap_or_default();

    if session_id.is_empty() {
        eprintln!("No access token available for presence update");
        std::process::exit(1);
    }

    let update = PresenceUpdate {
        status: Some("online".to_string()),
        activity: Some(PresenceActivity {
            r#type: Some("playing".to_string()),
            properties: Some(serde_json::json!({
                "FriendableGame": "Fortnite"
            })),
        }),
    };

    match egs.update_presence(&session_id, &update).await {
        Ok(()) => println!("Presence updated successfully"),
        Err(e) => eprintln!("Failed to update presence: {:?}", e),
    }
}
