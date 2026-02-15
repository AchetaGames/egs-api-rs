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

    println!("=== List Assets ===\n");

    let assets = egs.list_assets(None, None).await;
    println!("Total assets: {}", assets.len());

    let sample = assets.iter().take(5).collect::<Vec<_>>();
    for asset in &sample {
        println!(
            "  {} (namespace: {}, catalog: {})",
            asset.app_name, asset.namespace, asset.catalog_item_id
        );
    }

    let test_asset = match assets.first() {
        Some(a) => a,
        None => {
            println!("No assets found, nothing more to demo.");
            return;
        }
    };

    println!("\n=== Asset Info (first asset) ===\n");

    match egs.asset_info(test_asset).await {
        Some(info) => println!("{:#?}", info),
        None => eprintln!("Failed to fetch asset info for {}", test_asset.app_name),
    }

    println!("\n=== Asset Manifest ===\n");

    let manifest = egs
        .asset_manifest(
            None,
            None,
            Some(test_asset.namespace.clone()),
            Some(test_asset.catalog_item_id.clone()),
            Some(test_asset.app_name.clone()),
        )
        .await;

    match manifest {
        Some(m) => {
            println!("Manifest elements: {}", m.elements.len());
            println!("{:#?}", m);

            println!("\n=== Download Manifests ===\n");

            let download_manifests = egs.asset_download_manifests(m).await;
            println!("Got {} download manifest(s)", download_manifests.len());
            for dm in &download_manifests {
                println!(
                    "  App: {} | Files: {} | Chunks: {}",
                    dm.app_name_string,
                    dm.file_manifest_list.len(),
                    dm.chunk_hash_list.len()
                );
            }
        }
        None => eprintln!(
            "Failed to fetch asset manifest for {}",
            test_asset.app_name
        ),
    }

    println!("\n=== Artifact Service Ticket ===\n");

    match egs
        .artifact_service_ticket(&test_asset.namespace, &test_asset.app_name, None, None)
        .await
    {
        Ok(ticket) => {
            println!(
                "Ticket expires in: {} seconds",
                ticket.expires_in_seconds.unwrap_or(0)
            );
            if let Some(signed) = &ticket.signed_ticket {
                println!(
                    "Signed ticket (first 80 chars): {}...",
                    &signed[..signed.len().min(80)]
                );

                println!("\n=== Game Manifest by Ticket ===\n");

                match egs
                    .game_manifest_by_ticket(&test_asset.app_name, signed, None, None)
                    .await
                {
                    Ok(m) => println!("Manifest elements: {}", m.elements.len()),
                    Err(e) => eprintln!("Failed to fetch manifest by ticket: {:?}", e),
                }
            }
        }
        Err(e) => eprintln!("Failed to get artifact service ticket: {:?}", e),
    }

    println!("\n=== Launcher Manifests ===\n");

    match egs.launcher_manifests(None, None).await {
        Ok(m) => {
            println!("Launcher manifest elements: {}", m.elements.len());
            for elem in &m.elements {
                for manifest in &elem.manifests {
                    println!("  URI: {}", manifest.uri);
                }
            }
        }
        Err(e) => eprintln!("Failed to fetch launcher manifests: {:?}", e),
    }

    println!("\n=== Delta Manifest ===\n");

    println!("Delta manifests require two different build IDs.");
    println!("Trying with dummy IDs to demonstrate the API (expected to return None):");
    match egs
        .delta_manifest("https://example.com", "old-build", "new-build")
        .await
    {
        Some(data) => println!("Got delta manifest: {} bytes", data.len()),
        None => println!("No delta manifest available (expected for most assets)."),
    }
}
