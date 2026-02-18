#[path = "common.rs"]
mod common;

use egs_api::EpicGames;
use egs_api::api::types::fab_search::FabSearchParams;

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

    // --- Text search with q parameter (Phase 1 enhancement) ---

    println!("=== Fab Text Search (q=\"landscape\") ===\n");

    let text_params = FabSearchParams {
        q: Some("landscape".to_string()),
        channels: Some("unreal-engine".to_string()),
        count: Some(3),
        ..Default::default()
    };

    match egs.fab_search(&text_params).await {
        Some(results) => {
            println!(
                "Found {} results for \"landscape\":",
                results.count.unwrap_or(0)
            );
            for listing in &results.results {
                let title = listing.title.as_deref().unwrap_or("(untitled)");
                let seller = listing
                    .user
                    .as_ref()
                    .and_then(|u| u.seller_name.as_deref())
                    .unwrap_or("unknown");
                println!("  {} — by {}", title, seller);
            }
        }
        None => {
            eprintln!("Text search failed");
        }
    }

    // --- Search (public, no auth required) ---

    println!("\n=== Fab Search (UE plugins, newest first) ===\n");

    let params = FabSearchParams {
        channels: Some("unreal-engine".to_string()),
        listing_types: Some("tool-and-plugin".to_string()),
        sort_by: Some("-createdAt".to_string()),
        count: Some(5),
        ..Default::default()
    };

    match egs.fab_search(&params).await {
        Some(results) => {
            println!(
                "Found {} results (showing first {})",
                results.count.unwrap_or(0),
                results.results.len()
            );
            for listing in &results.results {
                let title = listing.title.as_deref().unwrap_or("(untitled)");
                let seller = listing
                    .user
                    .as_ref()
                    .and_then(|u| u.seller_name.as_deref())
                    .unwrap_or("unknown");
                let listing_type = listing.listing_type.as_deref().unwrap_or("?");
                println!("  [{}] {} — by {}", listing_type, title, seller);

                if let Some(cat) = &listing.category {
                    if let Some(name) = &cat.name {
                        println!("    Category: {}", name);
                    }
                }
            }
            if let Some(ref cursors) = results.cursors {
                if cursors.next.is_some() {
                    println!("  (more results available via cursor pagination)");
                }
            }

            // --- Listing Detail for the first result ---

            if let Some(first) = results.results.first() {
                println!(
                    "\n=== Listing Detail: {} ===\n",
                    first.title.as_deref().unwrap_or(&first.uid)
                );

                if let Some(detail) = egs.fab_listing(&first.uid).await {
                    println!("  UID:         {}", detail.uid);
                    println!(
                        "  Title:       {}",
                        detail.title.as_deref().unwrap_or("(none)")
                    );
                    println!(
                        "  Type:        {}",
                        detail.listing_type.as_deref().unwrap_or("(none)")
                    );
                    println!(
                        "  Seller:      {}",
                        detail
                            .user
                            .as_ref()
                            .and_then(|u| u.seller_name.as_deref())
                            .unwrap_or("unknown")
                    );
                    println!("  Mature:      {}", detail.is_mature.unwrap_or(false));
                    println!(
                        "  Created:     {}",
                        detail.created_at.as_deref().unwrap_or("?")
                    );
                    println!(
                        "  Published:   {}",
                        detail.published_at.as_deref().unwrap_or("?")
                    );
                    if let Some(ratings) = &detail.ratings {
                        println!("  Ratings:     {}", ratings);
                    }
                } else {
                    eprintln!("  Failed to fetch listing detail");
                }

                // --- UE Format Details ---

                println!("\n=== UE Format Details ===\n");

                match egs.fab_listing_ue_formats(&first.uid).await {
                    Some(formats) => {
                        println!("  {} format(s):", formats.len());
                        for fmt in &formats {
                            if let Some(ref ft) = fmt.asset_format_type {
                                println!(
                                    "    Format: {} ({})",
                                    ft.name.as_deref().unwrap_or("?"),
                                    ft.code.as_deref().unwrap_or("?")
                                );
                            }
                            if let Some(ref specs) = fmt.technical_specs {
                                if let Some(ref versions) = specs.unreal_engine_engine_versions {
                                    println!("    Engine versions: {}", versions.join(", "));
                                }
                                if let Some(ref platforms) = specs.unreal_engine_target_platforms {
                                    println!("    Platforms: {}", platforms.join(", "));
                                }
                                if let Some(ref method) = specs.unreal_engine_distribution_method {
                                    println!("    Distribution: {}", method);
                                }
                            }
                        }
                    }
                    None => {
                        println!("  No UE format info available for this listing");
                    }
                }

                // --- Listing State (requires auth) ---

                println!("\n=== Listing State ===\n");

                match egs.fab_listing_state(&first.uid).await {
                    Some(state) => {
                        println!("  Acquired:    {}", state.acquired.unwrap_or(false));
                        println!("  Wishlisted:  {}", state.wishlisted.unwrap_or(false));
                        if let Some(ref eid) = state.entitlement_id {
                            println!("  Entitlement: {}", eid);
                        }
                    }
                    None => {
                        println!("  Could not fetch listing state (may require Fab session)");
                    }
                }

                // --- Ownership ---

                println!("\n=== Listing Ownership ===\n");

                match egs.fab_listing_ownership(&first.uid).await {
                    Some(ownership) => {
                        if let Some(ref licenses) = ownership.licenses {
                            if licenses.is_empty() {
                                println!("  Not owned");
                            } else {
                                for lic in licenses {
                                    println!(
                                        "  License: {} ({})",
                                        lic.name.as_deref().unwrap_or("?"),
                                        lic.slug.as_deref().unwrap_or("?")
                                    );
                                }
                            }
                        }
                    }
                    None => {
                        println!("  Could not fetch ownership info");
                    }
                }

                // --- Pricing ---

                println!("\n=== Listing Prices ===\n");

                match egs.fab_listing_prices(&first.uid).await {
                    Some(prices) => {
                        if prices.is_empty() {
                            println!("  No pricing info (may be free)");
                        } else {
                            for price in &prices {
                                let currency = price.currency_code.as_deref().unwrap_or("?");
                                let amount = price
                                    .price
                                    .map(|p| format!("{:.2}", p))
                                    .unwrap_or_else(|| "N/A".to_string());
                                let discount_pct = price.discount_percentage.unwrap_or(0);
                                if discount_pct > 0 {
                                    let discounted = price
                                        .discounted_price
                                        .map(|p| format!("{:.2}", p))
                                        .unwrap_or_else(|| "N/A".to_string());
                                    println!(
                                        "  {} {} → {} ({}% off)",
                                        currency, amount, discounted, discount_pct
                                    );
                                } else {
                                    println!("  {} {}", currency, amount);
                                }
                            }
                        }
                    }
                    None => {
                        println!("  Could not fetch pricing info");
                    }
                }

                // --- Reviews ---

                println!("\n=== Listing Reviews ===\n");

                match egs
                    .fab_listing_reviews(&first.uid, Some("-createdAt"), None)
                    .await
                {
                    Some(reviews_resp) => {
                        println!("  Total reviews: {}", reviews_resp.count.unwrap_or(0));
                        for review in reviews_resp.results.iter().take(3) {
                            let author = review
                                .user
                                .as_ref()
                                .and_then(|u| u.display_name.as_deref())
                                .unwrap_or("anonymous");
                            let rating = review
                                .rating
                                .map(|r| format!("{}/5", r))
                                .unwrap_or_else(|| "?".to_string());
                            let title = review.title.as_deref().unwrap_or("(no title)");
                            println!("  [{}] {} — {}", rating, title, author);
                            if let Some(ref content) = review.content {
                                let preview: String = content.chars().take(80).collect();
                                if content.len() > 80 {
                                    println!("    {}...", preview);
                                } else {
                                    println!("    {}", preview);
                                }
                            }
                        }
                    }
                    None => {
                        println!("  Could not fetch reviews");
                    }
                }
            }
        }
        None => {
            eprintln!("Fab search failed");
        }
    }

    // --- Library (requires auth) ---

    println!("\n=== Fab Library ===\n");

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

    // --- Asset Manifest ---

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

    // --- File Download Info ---

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
