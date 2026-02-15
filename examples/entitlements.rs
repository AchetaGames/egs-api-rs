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

    println!("=== User Entitlements ===\n");

    let entitlements = egs.user_entitlements().await;
    println!("Total entitlements: {}", entitlements.len());

    for ent in entitlements.iter().take(20) {
        println!("  {:?}", ent);
    }
    if entitlements.len() > 20 {
        println!("  ... and {} more", entitlements.len() - 20);
    }
}
