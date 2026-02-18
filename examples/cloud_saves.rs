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

    println!("=== Cloud Save List (all games) ===\n");

    match egs.cloud_save_list(None, false).await {
        Ok(response) => {
            println!("Total cloud save files: {}", response.files.len());
            for (path, file) in response.files.iter().take(10) {
                println!(
                    "  {} — {} bytes, modified: {}",
                    path,
                    file.length.unwrap_or(0),
                    file.last_modified.as_deref().unwrap_or("unknown"),
                );
            }
            if response.files.len() > 10 {
                println!("  ... and {} more", response.files.len() - 10);
            }

            if let Some((path, file)) = response.files.iter().next() {
                println!("\n=== Cloud Save Details (first file) ===\n");
                println!("  Path:       {}", path);
                println!(
                    "  Filename:   {}",
                    file.file_name.as_deref().unwrap_or("N/A")
                );
                println!("  Length:     {} bytes", file.length.unwrap_or(0));
                println!(
                    "  Storage:    {}",
                    file.storage_type.as_deref().unwrap_or("N/A")
                );
                println!("  ETag:       {}", file.etag.as_deref().unwrap_or("N/A"));
                if let Some(link) = &file.read_link {
                    println!("  Read link:  {}...", &link[..link.len().min(80)]);
                }
            }
        }
        Err(e) => eprintln!("Failed to list cloud saves: {:?}", e),
    }

    println!("\n=== Cloud Save List (per game, first asset) ===\n");

    let assets = egs.list_assets(None, None).await;
    if let Some(asset) = assets.first() {
        println!("Checking cloud saves for: {}", asset.app_name);
        match egs.cloud_save_list(Some(&asset.app_name), false).await {
            Ok(response) => {
                println!("  Files: {}", response.files.len());
                for (path, file) in response.files.iter().take(5) {
                    println!("    {} — {} bytes", path, file.length.unwrap_or(0),);
                }
            }
            Err(e) => eprintln!("  No cloud saves or error: {:?}", e),
        }

        println!("\n=== Cloud Save Manifests ===\n");

        match egs.cloud_save_list(Some(&asset.app_name), true).await {
            Ok(response) => {
                println!("  Manifest files: {}", response.files.len());
                for (path, _) in response.files.iter().take(5) {
                    println!("    {}", path);
                }
            }
            Err(e) => eprintln!("  No manifests or error: {:?}", e),
        }
    } else {
        println!("No assets found, skipping per-game cloud save demo.");
    }

    // NOTE: cloud_save_query and cloud_save_delete are available but
    // require knowing specific filenames and could be destructive.
    // Uncomment below to try query with known filenames:
    //
    // let filenames = vec!["SaveGame1.sav".to_string()];
    // match egs.cloud_save_query("YourAppName", &filenames).await {
    //     Ok(response) => println!("Query result: {:#?}", response),
    //     Err(e) => eprintln!("Query failed: {:?}", e),
    // }
}
