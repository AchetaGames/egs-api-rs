#[path = "common.rs"]
mod common;

use egs_api::EpicGames;

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut egs = EpicGames::new();

    println!("=== Authentication Example ===\n");

    if !common::login_or_restore(&mut egs).await {
        eprintln!("Authentication failed");
        std::process::exit(1);
    }

    let details = egs.user_details();
    println!("\nLogged in as: {}", details.display_name.unwrap_or_default());
    println!("Account ID: {}", details.account_id.unwrap_or_default());

    println!("\nToken saved. Run other examples without re-authenticating.");
    println!("To logout, uncomment the logout section below.\n");

    // Uncomment to invalidate the session and delete the saved token:
    // if egs.logout().await {
    //     println!("Logged out successfully");
    //     let _ = std::fs::remove_file(dirs_or_home().join(".egs-api/token.json"));
    // }
}
