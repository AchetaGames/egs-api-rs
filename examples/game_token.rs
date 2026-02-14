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

    println!("=== Game Token ===\n");

    match egs.game_token().await {
        Some(token) => {
            println!("Exchange code: {}", token.code);
            println!("Expires in: {} seconds", token.expires_in_seconds);
        }
        None => eprintln!("Failed to fetch game token"),
    }

    println!("\n=== Ownership Token ===\n");

    let assets = egs.list_assets(None, None).await;
    match assets.first() {
        Some(asset) => match egs.ownership_token(asset.clone()).await {
            Some(token) => {
                println!(
                    "Ownership token for {} (first 80 chars): {}...",
                    asset.app_name,
                    &token[..token.len().min(80)]
                );
            }
            None => eprintln!("Failed to get ownership token for {}", asset.app_name),
        },
        None => println!("No assets found, skipping ownership token demo"),
    }
}
