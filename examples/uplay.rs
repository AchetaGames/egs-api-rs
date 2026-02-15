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

    println!("=== Uplay Codes ===\n");

    match egs.store_get_uplay_codes().await {
        Ok(response) => {
            if let Some(errors) = &response.errors {
                if !errors.is_empty() {
                    eprintln!("GraphQL errors: {:?}", errors);
                }
            }
            match response
                .data
                .and_then(|d| d.partner_integration)
                .and_then(|p| p.account_uplay_codes)
            {
                Some(codes) => {
                    println!("Total Uplay codes: {}", codes.len());
                    for code in &codes {
                        println!(
                            "  Game: {} | Uplay Account: {} | Redeemed: {}",
                            code.game_id.as_deref().unwrap_or("N/A"),
                            code.uplay_account_id.as_deref().unwrap_or("N/A"),
                            code.redeemed_on_uplay.unwrap_or(false),
                        );
                    }
                }
                None => println!("No Uplay codes found (account may not have Ubisoft-linked games)."),
            }
        }
        Err(e) => eprintln!("Failed to fetch Uplay codes: {:?}", e),
    }

    // Claiming and redeeming require a Uplay account ID and are
    // irreversible. Uncomment to try with your Uplay account ID:
    //
    // let uplay_id = "your-uplay-account-id";
    // match egs.store_redeem_uplay_codes(uplay_id).await {
    //     Ok(resp) => println!("Redeem result: {:#?}", resp),
    //     Err(e) => eprintln!("Redeem failed: {:?}", e),
    // }
}
