//! Cosmos session setup, EULA check, and engine version listing.
//!
//! Run: `cargo run --example cosmos`

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut egs = egs_api::EpicGames::new();

    if !common::login_or_restore(&mut egs).await {
        eprintln!("Authentication failed. Run the 'auth' example first.");
        std::process::exit(1);
    }

    // Get a game token (exchange code) to bootstrap the Cosmos session
    let token = egs
        .try_game_token()
        .await
        .expect("Failed to get game token");

    println!("Exchange code obtained, setting up Cosmos session...");

    // Set up cookie-based Cosmos session
    let auth = egs
        .cosmos_session_setup(&token.code)
        .await
        .expect("Cosmos session setup failed");

    println!(
        "Cosmos auth: bearerTokenValid={}, upgradedBearerToken={}",
        auth.bearer_token_valid, auth.upgraded_bearer_token
    );

    // Check EULA acceptance (IDs may change, known: unreal_engine, unreal_engine2)
    match egs.try_cosmos_eula_check("unreal_engine2", "en").await {
        Ok(eula) => println!("UE EULA accepted: {}", eula.accepted),
        Err(e) => eprintln!("EULA check failed (ID may have changed): {}", e),
    }

    // Get Cosmos account info
    if let Some(account) = egs.cosmos_account().await {
        println!("Account: {} ({})", account.display_name, account.country);
    }

    // Fetch engine versions for Linux
    if let Some(versions) = egs.engine_versions("linux").await {
        println!("\nLinux engine builds ({} blobs):", versions.blobs.len());
        for blob in versions.blobs.iter().take(5) {
            println!("  {} ({} bytes)", blob.name, blob.size);
        }
        if versions.blobs.len() > 5 {
            println!("  ... and {} more", versions.blobs.len() - 5);
        }
    }
}
