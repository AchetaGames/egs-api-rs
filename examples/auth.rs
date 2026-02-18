#[path = "common.rs"]
mod common;

use egs_api::EpicGames;
use std::io;

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut egs = EpicGames::new();

    println!("=== Authentication Example ===\n");

    let args: Vec<String> = std::env::args().collect();
    let use_sid = args.iter().any(|a| a == "--sid");

    if use_sid {
        println!("SID auth mode. Enter your Epic session ID (SID):");
        let mut sid = String::new();
        io::stdin().read_line(&mut sid).unwrap();
        let sid = sid.trim();

        match egs.auth_sid(sid).await {
            Ok(true) => {
                println!("SID auth succeeded!");
                common::save_token(&egs);
            }
            Ok(false) => {
                eprintln!("SID auth returned false");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("SID auth failed: {:?}", e);
                std::process::exit(1);
            }
        }
    } else if !common::login_or_restore(&mut egs).await {
        eprintln!("Authentication failed");
        std::process::exit(1);
    }

    let details = egs.user_details();
    println!(
        "\nLogged in as: {}",
        details.display_name.unwrap_or_default()
    );
    println!("Account ID: {}", details.account_id.unwrap_or_default());

    println!("\nToken saved. Run other examples without re-authenticating.");
    println!("Tip: use --sid flag to authenticate via session ID instead.\n");
}
