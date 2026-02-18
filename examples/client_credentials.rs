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

    println!("=== Client Credentials Auth ===\n");

    let mut client_egs = EpicGames::new();
    if client_egs.auth_client_credentials().await {
        println!("Client credentials auth succeeded");
        println!("Token type: client_credentials (limited permissions, no user context)");
    } else {
        eprintln!("Client credentials auth failed");
        return;
    }

    println!("\n=== Library State Token Status ===\n");

    let test_token = "test-token-id";
    match egs.library_state_token_status(test_token).await {
        Some(valid) => println!("Token '{}' valid: {}", test_token, valid),
        None => println!(
            "Token '{}': could not check status (API error or invalid)",
            test_token
        ),
    }
}
