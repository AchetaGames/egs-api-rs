#[path = "common.rs"]
mod common;

use egs_api::EpicGames;

/// Compare EGS and Fab download manifests side-by-side.
///
/// Fetches the first owned EGS asset manifest and the first Fab library asset
/// manifest, then prints their custom fields and chunk URL status to verify
/// that both paths produce valid download links.
#[tokio::main]
async fn main() {
    env_logger::init();
    let mut egs = EpicGames::new();

    if !common::login_or_restore(&mut egs).await {
        eprintln!("Authentication failed. Run the 'auth' example first.");
        std::process::exit(1);
    }

    // --- EGS asset manifest ---
    println!("========================================");
    println!("  EGS Asset Download Manifest");
    println!("========================================\n");

    let assets = egs.list_assets(None, None).await;
    println!("Total EGS assets: {}", assets.len());

    if let Some(asset) = assets.first() {
        println!(
            "Using asset: {} ({})",
            asset.app_name, asset.catalog_item_id
        );

        if let Some(manifest) = egs
            .asset_manifest(
                None,
                None,
                Some(asset.namespace.clone()),
                Some(asset.catalog_item_id.clone()),
                Some(asset.app_name.clone()),
            )
            .await
        {
            let download_manifests = egs.asset_download_manifests(manifest).await;
            println!("Got {} download manifest(s)\n", download_manifests.len());

            if let Some(dm) = download_manifests.first() {
                print_manifest_summary("EGS", dm);
            }
        } else {
            eprintln!("Failed to fetch EGS asset manifest");
        }
    } else {
        eprintln!("No EGS assets found");
    }

    // --- Fab asset manifest ---
    println!("\n========================================");
    println!("  Fab Asset Download Manifest");
    println!("========================================\n");

    let account_id = egs.user_details().account_id.unwrap_or_default();
    if account_id.is_empty() {
        eprintln!("No account ID available");
        std::process::exit(1);
    }

    match egs.fab_library_items(account_id).await {
        Some(library) => {
            println!("Total Fab library items: {}", library.results.len());

            if let Some(item) = library.results.first() {
                println!("Using Fab item: {:?}\n", item);

                // Use the Kite Demo as a known-good test asset, or fall back to first item
                let (artifact_id, namespace, asset_id) = (
                    "KiteDemo473",
                    "89efe5924d3d467c839449ab6ab52e7f",
                    "28166226c38a4ff3aa28bbe87dcbbe5b",
                );

                println!("Fetching Fab manifest for artifact: {}", artifact_id);

                match egs
                    .fab_asset_manifest(artifact_id, namespace, asset_id, None)
                    .await
                {
                    Ok(download_infos) => {
                        println!("Got {} download info(s)", download_infos.len());

                        if let Some(info) = download_infos.first() {
                            println!(
                                "Distribution point base URLs: {:?}",
                                info.distribution_point_base_urls
                            );

                            if let Some(base_url) = info.distribution_point_base_urls.first() {
                                match egs.fab_download_manifest(info.clone(), base_url).await {
                                    Ok(dm) => {
                                        print_manifest_summary("Fab", &dm);
                                    }
                                    Err(e) => eprintln!("Failed to download Fab manifest: {:?}", e),
                                }
                            } else {
                                eprintln!("No distribution point base URLs");
                            }
                        }
                    }
                    Err(egs_api::api::error::EpicAPIError::FabTimeout) => {
                        eprintln!("Fab API timed out (403). Try again.");
                    }
                    Err(e) => eprintln!("Failed to fetch Fab asset manifest: {:?}", e),
                }
            } else {
                eprintln!("Fab library is empty");
            }
        }
        None => eprintln!("Failed to fetch Fab library"),
    }
}

fn print_manifest_summary(
    label: &str,
    dm: &egs_api::api::types::download_manifest::DownloadManifest,
) {
    println!("[{}] App name:      {}", label, dm.app_name_string);
    println!("[{}] Build version: {}", label, dm.build_version_string);
    println!("[{}] Files:         {}", label, dm.file_manifest_list.len());
    println!("[{}] Chunks:        {}", label, dm.chunk_hash_list.len());
    println!("[{}] Total size:    {} bytes", label, dm.total_size());
    println!(
        "[{}] Download size: {} bytes",
        label,
        dm.total_download_size()
    );

    // Custom fields — the key comparison point
    println!("\n[{}] Custom fields:", label);
    if let Some(fields) = &dm.custom_fields {
        if fields.is_empty() {
            println!("  (none)");
        }
        for (k, v) in fields {
            let display_val = if v.len() > 120 {
                format!("{}...", &v[..120])
            } else {
                v.clone()
            };
            println!("  {} = {}", k, display_val);
        }
    } else {
        println!("  (none — custom_fields is None)");
    }

    // Check if files() produces valid download links
    let files = dm.files();
    let total_files = files.len();
    let mut files_with_links = 0;
    let mut files_without_links = 0;
    let mut total_chunks_with_links = 0;
    let mut total_chunks_without_links = 0;

    for (_filename, file_manifest) in &files {
        let has_any_link = file_manifest
            .file_chunk_parts
            .iter()
            .any(|p| p.link.is_some());
        if has_any_link {
            files_with_links += 1;
        } else if !file_manifest.file_chunk_parts.is_empty() {
            files_without_links += 1;
        }
        for part in &file_manifest.file_chunk_parts {
            if part.link.is_some() {
                total_chunks_with_links += 1;
            } else {
                total_chunks_without_links += 1;
            }
        }
    }

    println!("\n[{}] Download link status:", label);
    println!("  Total files:                {}", total_files);
    println!("  Files with chunk links:     {}", files_with_links);
    println!("  Files WITHOUT chunk links:  {}", files_without_links);
    println!("  Total chunk parts w/ link:  {}", total_chunks_with_links);
    println!(
        "  Total chunk parts NO link:  {}",
        total_chunks_without_links
    );

    if total_chunks_without_links > 0 {
        println!(
            "\n  *** WARNING: {} chunk parts have no download URL! Downloads will fail. ***",
            total_chunks_without_links
        );
    } else if total_chunks_with_links > 0 {
        println!("\n  All chunk parts have valid download URLs.");
    }

    // Show a sample chunk URL
    if let Some((filename, file_manifest)) = files.iter().next() {
        if let Some(part) = file_manifest.file_chunk_parts.first() {
            let url_display = match &part.link {
                Some(url) => {
                    let s = url.to_string();
                    if s.len() > 100 {
                        format!("{}...", &s[..100])
                    } else {
                        s
                    }
                }
                None => "(none)".to_string(),
            };
            println!(
                "\n[{}] Sample: file={}, chunk_guid={}, url={}",
                label,
                if filename.len() > 60 {
                    &filename[..60]
                } else {
                    filename
                },
                part.guid,
                url_display
            );
        }
    }
}
