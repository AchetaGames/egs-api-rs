use webbrowser;
use std::io::{self};
use egs_api::EpicGames;
use egs_api::api::EpicAsset;
use std::collections::HashMap;

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

    match egs.auth_sid(sid.as_str()).await {
        None => { println!("No exchange token, cannot login.") }
        Some(exchange_token) => {
            egs.auth_code(exchange_token).await;
            egs.login().await;
            let assets = egs.list_assets().await;
            let mut ueasset_map: HashMap<String, EpicAsset> = HashMap::new();
            let mut non_ueasset_map: HashMap<String, EpicAsset> = HashMap::new();
            for asset in assets {
                if asset.namespace=="ue" {
                    ueasset_map.insert(asset.catalog_item_id.clone(), asset.clone());
                } else {
                    non_ueasset_map.insert(asset.catalog_item_id.clone(), asset.clone());
                }
            }

            println!("Got {} assets", ueasset_map.len()+ non_ueasset_map.len());
            println!("From that {} unreal assets", ueasset_map.len());
            println!("From that {} non unreal assets", non_ueasset_map.len());
        }
    }
}