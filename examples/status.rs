#[path = "common.rs"]
mod common;

use egs_api::EpicGames;

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut egs = EpicGames::new();

    if !common::login_or_restore(&mut egs).await {
        eprintln!("Authentication failed. Run the 'auth' example first.");
        std::process::exit(1);
    }

    println!("=== Service Status (Fortnite) ===\n");

    match egs.service_status("fortnite").await {
        Some(statuses) => {
            for status in &statuses {
                println!("  Service: {}", status.service_instance_id);
                println!("  Status: {}", status.status);
                if let Some(msg) = &status.message {
                    println!("  Message: {}", msg);
                }
                if let Some(banned) = status.banned {
                    println!("  Banned: {}", banned);
                }
                if let Some(launcher) = &status.launcher_info_dto {
                    println!(
                        "  App: {}",
                        launcher.app_name.as_deref().unwrap_or("(none)")
                    );
                }
                println!();
            }
        }
        None => eprintln!("Failed to fetch service status"),
    }
}
