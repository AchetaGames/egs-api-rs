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

    println!("=== Catalog Items ===\n");

    let namespace = "fn";
    match egs.catalog_items(namespace, 0, 10).await {
        Some(page) => {
            if let Some(paging) = &page.paging {
                println!("Total items in '{}': {}", namespace, paging.total);
            }
            for item in &page.elements {
                println!(
                    "  [{}] {} (type: {})",
                    item.id,
                    item.title.as_deref().unwrap_or("(untitled)"),
                    item.item_type.as_deref().unwrap_or("unknown"),
                );
            }
        }
        None => eprintln!("Failed to fetch catalog items for '{}'", namespace),
    }

    println!("\n=== Catalog Offers ===\n");

    match egs.catalog_offers(namespace, 0, 10).await {
        Some(page) => {
            if let Some(paging) = &page.paging {
                println!("Total offers in '{}': {}", namespace, paging.total);
            }
            for offer in &page.elements {
                println!(
                    "  [{}] {} (type: {}, items: {})",
                    offer.id,
                    offer.title.as_deref().unwrap_or("(untitled)"),
                    offer.offer_type.as_deref().unwrap_or("unknown"),
                    offer.items.len(),
                );
            }
        }
        None => eprintln!("Failed to fetch catalog offers for '{}'", namespace),
    }

    println!("\n=== Bulk Catalog Items ===\n");

    let assets = egs.list_assets(None, None).await;
    let items: Vec<(&str, &str)> = assets
        .iter()
        .take(3)
        .map(|a| (a.namespace.as_str(), a.catalog_item_id.as_str()))
        .collect();

    if items.is_empty() {
        println!("No assets found, skipping bulk catalog demo.");
        return;
    }

    println!("Looking up {} items across namespaces...", items.len());
    match egs.bulk_catalog_items(&items).await {
        Some(result) => {
            for (ns, items_map) in &result {
                for (item_id, info) in items_map {
                    println!(
                        "  {}/{}: {}",
                        ns,
                        item_id,
                        info.title.as_deref().unwrap_or("(untitled)")
                    );
                }
            }
        }
        None => eprintln!("Failed to fetch bulk catalog items"),
    }
}
