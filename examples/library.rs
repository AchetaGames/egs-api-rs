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

    println!("=== Library Items ===\n");

    match egs.library_items(true).await {
        Some(library) => {
            println!("Total library records: {}", library.records.len());
            for record in library.records.iter().take(20) {
                println!("  {:?}", record);
            }
            if library.records.len() > 20 {
                println!("  ... and {} more", library.records.len() - 20);
            }
        }
        None => eprintln!("Failed to fetch library items"),
    }
}
