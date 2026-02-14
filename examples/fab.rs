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

    let account_id = egs.user_details().account_id.unwrap_or_default();
    if account_id.is_empty() {
        eprintln!("No account ID available");
        std::process::exit(1);
    }

    println!("=== Fab Library ===\n");

    match egs.fab_library_items(account_id).await {
        Some(library) => {
            println!("Total Fab library items: {}", library.results.len());
            for item in library.results.iter().take(10) {
                println!("  {:?}", item);
            }
            if library.results.len() > 10 {
                println!("  ... and {} more", library.results.len() - 10);
            }
        }
        None => {
            eprintln!("Failed to fetch Fab library");
            return;
        }
    }

    println!("\n=== Fab Asset Manifest (Kite Demo) ===\n");

    let manifest_result = egs
        .fab_asset_manifest(
            "KiteDemo473",
            "89efe5924d3d467c839449ab6ab52e7f",
            "28166226c38a4ff3aa28bbe87dcbbe5b",
            None,
        )
        .await;

    match manifest_result {
        Ok(download_infos) => {
            println!("Got {} download info(s)", download_infos.len());

            for info in &download_infos {
                println!("  Manifest hash: {}", info.manifest_hash);
                println!(
                    "  Distribution points: {:?}",
                    info.distribution_point_base_urls
                );

                println!("\n=== Fab Download Manifest ===\n");

                for url in &info.distribution_point_base_urls {
                    println!("Trying distribution point: {}", url);
                    match egs.fab_download_manifest(info.clone(), url).await {
                        Ok(dm) => {
                            println!("  App: {}", dm.app_name_string);
                            println!("  Build: {}", dm.build_version_string);
                            println!("  Files: {}", dm.file_manifest_list.len());
                            println!("  Chunks: {}", dm.chunk_hash_list.len());
                            println!(
                                "  Hash match: {} == {}",
                                info.manifest_hash,
                                dm.custom_field("DownloadedManifestHash")
                                    .unwrap_or_default()
                            );
                            break;
                        }
                        Err(e) => {
                            eprintln!("  Failed from {}: {:?}", url, e);
                        }
                    }
                }
            }
        }
        Err(egs_api::api::error::EpicAPIError::FabTimeout) => {
            eprintln!("Fab API timed out (403). Try running the example again.");
        }
        Err(e) => eprintln!("Failed to fetch Fab asset manifest: {:?}", e),
    }

    println!("\n=== Fab File Download Info ===\n");

    match egs
        .fab_file_download_info("some-listing-id", "some-format-id", "some-file-id")
        .await
    {
        Some(info) => {
            println!("  Manifest hash: {}", info.manifest_hash);
            println!(
                "  Distribution points: {:?}",
                info.distribution_point_base_urls
            );
        }
        None => {
            println!("  fab_file_download_info requires valid Fab listing/format/file IDs.");
            println!("  Replace the placeholder IDs above with real values from your Fab library.");
        }
    }
}
