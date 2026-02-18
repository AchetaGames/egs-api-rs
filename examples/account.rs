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

    println!("=== Account Details ===\n");

    match egs.account_details().await {
        Some(details) => {
            println!("Display Name: {}", details.display_name);
            println!("Email: {}", details.email);
            println!("Country: {}", details.country);
            println!("2FA Enabled: {}", details.tfa_enabled);
            println!("Last Login: {}", details.last_login);
        }
        None => eprintln!("Failed to fetch account details"),
    }

    println!("\n=== Account ID Lookup ===\n");

    let account_id = egs.user_details().account_id.unwrap_or_default();
    if !account_id.is_empty() {
        match egs.account_ids_details(vec![account_id.clone()]).await {
            Some(infos) => {
                for info in &infos {
                    println!("ID: {} -> Display Name: {}", info.id, info.display_name);
                }
            }
            None => eprintln!("Failed to fetch account info"),
        }
    }

    println!("\n=== Friends List ===\n");

    match egs.account_friends(true).await {
        Some(friends) => {
            println!("Total friends (including pending): {}", friends.len());
            for friend in friends.iter().take(10) {
                println!("  {:?}", friend);
            }
            if friends.len() > 10 {
                println!("  ... and {} more", friends.len() - 10);
            }
        }
        None => eprintln!("Failed to fetch friends list"),
    }

    println!("\n=== External Auth Connections ===\n");

    match egs.external_auths(&account_id).await {
        Some(auths) => {
            println!("Linked platforms: {}", auths.len());
            for auth in &auths {
                println!("  {} - {}", auth.type_field, auth.external_display_name);
            }
        }
        None => eprintln!("Failed to fetch external auth connections"),
    }

    println!("\n=== SSO Domains ===\n");

    match egs.sso_domains().await {
        Some(domains) => {
            println!("SSO domains: {}", domains.len());
            for domain in domains.iter().take(20) {
                println!("  {}", domain);
            }
            if domains.len() > 20 {
                println!("  ... and {} more", domains.len() - 20);
            }
        }
        None => eprintln!("Failed to fetch SSO domains"),
    }
}
