use std::io::{self};

use egs_api::api::types::epic_asset::EpicAsset;
use egs_api::EpicGames;
use std::collections::{HashMap, HashSet};
use webbrowser;

#[tokio::main]
async fn main() {
    if !webbrowser::open("https://www.epicgames.com/id/login?redirectUrl=https%3A%2F%2Fwww.epicgames.com%2Fid%2Fapi%2Fredirect").is_ok() {
        println!("Please go to https://www.epicgames.com/id/login?redirectUrl=https%3A%2F%2Fwww.epicgames.com%2Fid%2Fapi%2Fredirect")
    }
    println!("Please enter the 'sid' value from the JSON response");
    let mut sid = String::new();
    let stdin = io::stdin(); // We get `Stdin` here.
    stdin.read_line(&mut sid).unwrap();
    sid = sid.trim().to_string();
    sid = sid.replace(|c: char| c == '"', "");
    let mut egs = EpicGames::new();
    println!("Using sid: {}", sid);

    match egs.auth_sid(sid.as_str()).await {
        None => {
            println!("No exchange token, cannot login.")
        }
        Some(exchange_token) => {
            egs.auth_code(exchange_token).await;
            egs.login().await;
            let details = egs.account_details().await;
            println!("Account details: {:?}", details);
            let info = egs
                .account_ids_details(vec![egs.user_details().account_id.unwrap_or_default()])
                .await;
            println!("Account info: {:?}", info);
            // let assets = egs.list_assets().await;
            // let mut ueasset_map: HashMap<String, HashMap<String, EpicAsset>> = HashMap::new();
            // let mut non_ueasset_map: HashMap<String, HashMap<String, EpicAsset>> = HashMap::new();
            // for asset in assets {
            //     if asset.namespace == "ue" {
            //         if !ueasset_map.contains_key(&asset.catalog_item_id.clone()) {
            //             ueasset_map.insert(asset.catalog_item_id.clone(), HashMap::new());
            //         };
            //         match ueasset_map.get_mut(&asset.catalog_item_id.clone()) {
            //             None => {}
            //             Some(old) => {
            //                 old.insert(asset.app_name.clone(), asset.clone());
            //             }
            //         };
            //     } else {
            //         if !non_ueasset_map.contains_key(&asset.catalog_item_id.clone()) {
            //             non_ueasset_map.insert(asset.catalog_item_id.clone(), HashMap::new());
            //         };
            //         match non_ueasset_map.get_mut(&asset.catalog_item_id.clone()) {
            //             None => {}
            //             Some(old) => {
            //                 old.insert(asset.app_name.clone(), asset.clone());
            //             }
            //         };
            //     }
            // }
            //
            // println!("Got {} assets", ueasset_map.len() + non_ueasset_map.len());
            // println!("From that {} unreal assets", ueasset_map.len());
            // println!("From that {} non unreal assets", non_ueasset_map.len());
            //
            // println!("Getting the asset metadata");
            // let test_asset = ueasset_map
            //     .values()
            //     .last()
            //     .unwrap()
            //     .values()
            //     .last()
            //     .unwrap()
            //     .to_owned();
            // egs.asset_manifest(
            //     None,
            //     None,
            //     Some(test_asset.namespace.clone()),
            //     Some(test_asset.catalog_item_id.clone()),
            //     Some(test_asset.app_name.clone()),
            // )
            // .await;
            // println!("{:#?}", test_asset.clone());
            // println!("Getting the asset info");
            // let mut categories: HashSet<String> = HashSet::new();
            // for (_guid, asset) in non_ueasset_map.clone() {
            //     match egs
            //         .asset_info(asset.values().last().unwrap().to_owned())
            //         .await
            //     {
            //         None => {}
            //         Some(info) => {
            //             for category in info.categories.unwrap() {
            //                 categories.insert(category.path);
            //             }
            //         }
            //     };
            // }
            // let mut cat: Vec<String> = categories.into_iter().collect();
            // cat.sort();
            // for category in cat {
            //     println!("{}", category);
            // }
            // let _asset_info = egs.asset_info(test_asset.clone()).await;
            // println!("Getting ownership token");
            // egs.ownership_token(test_asset.clone()).await;
            // println!("Getting the game token");
            // egs.game_token().await;
            // println!("Getting the entitlements");
            // egs.user_entitlements().await;
            // println!("Getting the library items");
            // egs.library_items(true).await;
            // println!("Getting Asset manifest");
            // let manifest = egs
            //     .asset_manifest(
            //         None,
            //         None,
            //         Some(test_asset.namespace.clone()),
            //         Some(test_asset.catalog_item_id.clone()),
            //         Some(test_asset.app_name.clone()),
            //     )
            //     .await;
            // println!("{:?}", manifest);
            //
            // let download_manifest = egs.asset_download_manifests(manifest.unwrap()).await;
        }
    }
}
