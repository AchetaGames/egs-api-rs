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

    println!("=== Currencies ===\n");

    match egs.currencies(0, 20).await {
        Some(page) => {
            if let Some(paging) = &page.paging {
                println!("Total currencies: {}", paging.total);
            }
            for currency in &page.elements {
                println!(
                    "  {} ({}) - {} decimals",
                    currency.code,
                    currency.symbol.as_deref().unwrap_or("?"),
                    currency.decimals.unwrap_or(0),
                );
            }
        }
        None => eprintln!("Failed to fetch currencies"),
    }

    println!("\n=== Billing Account ===\n");

    match egs.billing_account().await {
        Some(account) => {
            println!("Billing ID: {}", account.id.as_deref().unwrap_or("(none)"));
            println!(
                "Country: {}",
                account.country.as_deref().unwrap_or("(none)")
            );
        }
        None => eprintln!("Failed to fetch billing account"),
    }

    println!("\n=== Offer Prices ===\n");

    let assets = egs.list_assets(None, None).await;
    if let Some(asset) = assets.first() {
        let billing_country = egs
            .billing_account()
            .await
            .and_then(|b| b.country)
            .unwrap_or_else(|| "US".to_string());

        let offer_ids = vec![asset.catalog_item_id.clone()];
        match egs
            .offer_prices(&asset.namespace, &offer_ids, &billing_country)
            .await
        {
            Some(response) => {
                for offer in &response.offers {
                    println!("  Offer: {}", offer.offer_id);
                    if let Some(price) = &offer.current_price {
                        if let Some(fmt) = &price.fmt_price {
                            println!(
                                "    Original: {}",
                                fmt.original_price.as_deref().unwrap_or("N/A")
                            );
                            println!(
                                "    Discount: {}",
                                fmt.discount_price.as_deref().unwrap_or("N/A")
                            );
                        }
                    }
                }
            }
            None => eprintln!("Failed to fetch offer prices"),
        }
    } else {
        println!("No assets found, skipping offer prices demo.");
    }

    println!("\n=== Quick Purchase (dry run - commented out) ===\n");
    println!("Quick purchase is destructive (claims a free game). Example usage:");
    println!("  egs.quick_purchase(\"namespace\", \"offer_id\").await");
    println!("Uncomment the code below to try it with a real free offer.");
    // match egs.quick_purchase("namespace", "offer_id").await {
    //     Some(response) => {
    //         println!("  Order ID: {}", response.order_id.unwrap_or_default());
    //         println!("  Status: {}", response.status.unwrap_or_default());
    //     }
    //     None => eprintln!("  Quick purchase failed"),
    // }
}
