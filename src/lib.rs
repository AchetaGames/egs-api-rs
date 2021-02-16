use reqwest::{header};

pub mod api;

use crate::api::{EpicAPI, EpicAPIError, EpicAsset, AssetManifest, AssetInfo, GameToken, Entitlement, Library};
use chrono::{Utc, DateTime};
use std::future::Future;
use std::collections::HashMap;


pub struct EpicGames {
    egs: EpicAPI
}

impl EpicGames {
    pub fn new() -> Self {
        EpicGames {
            egs: EpicAPI::new()
        }
    }

    pub async fn auth_sid(&self, sid: &str) -> Option<String> {
        // get first set of cookies (EPIC_BEARER_TOKEN etc.)
        let mut headers = header::HeaderMap::new();
        headers.insert("X-Epic-Event-Action", "login".parse().unwrap());
        headers.insert("X-Epic-Event-Category", "login".parse().unwrap());
        headers.insert("X-Epic-Strategy-Flags", "".parse().unwrap());
        headers.insert("X-Requested-With", "XMLHttpRequest".parse().unwrap());
        headers.insert("User-Agent", "EpicGamesLauncher/11.0.1-14907503+++Portal+Release-Live ".parse().unwrap());
        let url = format!("https://www.epicgames.com/id/api/set-sid?sid={}", sid);
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .default_headers(headers)
            .build().unwrap();
        match client.get(&url).send().await {
            Ok(_resp) => {}
            _ => {}
        }

        let mut xsrf_token: String = "".to_string();

        match client.get("https://www.epicgames.com/id/api/csrf").send().await {
            Ok(resp) => {
                for cookie in resp.cookies() {
                    if cookie.name().to_lowercase() == "xsrf-token" {
                        xsrf_token = cookie.value().to_string();
                    }
                }
            }
            _ => {}
        }

        match client.post("https://www.epicgames.com/id/api/exchange/generate").header("X-XSRF-TOKEN", xsrf_token).send().await {
            Ok(resp) => {
                if resp.status() == reqwest::StatusCode::OK {
                    let echo_json: serde_json::Value = resp.json().await.unwrap();
                    match echo_json["code"].as_str() {
                        Some(t) => { Some(t.to_string()) }
                        None => None
                    }
                } else {
                    //let echo_json: serde_json::Value = resp.json().await.unwrap();
                    //TODO: return the error from echo_json
                    None
                }
            }
            _ => { None }
        }
    }

    pub async fn auth_code(&mut self, code: String) -> bool {
        match self.egs.start_session(None, Some(code)).await {
            Ok(b) => { return b; }
            Err(_) => { return false; }
        }
    }

    pub async fn login(&mut self) -> bool {
        match self.egs.user_data.expires_at {
            None => {}
            Some(exp) => {
                let now = chrono::offset::Utc::now();
                let td = exp - now;
                if td.num_seconds() > 600 {
                    println!("Trying to re-use existing login session... ");
                    match self.egs.resume_session().await {
                        Ok(b) => {
                            if b {
                                println!("Logged in");
                                return true;
                            }
                            return false;
                        }
                        Err(e) => {
                            println!("Error: {}", e)
                        }
                    };
                }
            }
        }
        println!("Logging in...");
        match self.egs.user_data.refresh_expires_at {
            None => {}
            Some(exp) => {
                let now = chrono::offset::Utc::now();
                let td = exp - now;
                if td.num_seconds() > 600 {
                    match &self.egs.user_data.refresh_token {
                        None => {}
                        Some(rt) => {
                            match self.egs.start_session(Some(rt.to_string()), None).await {
                                Ok(b) => {
                                    if b {
                                        println!("Logged in");
                                        return true;
                                    }
                                    return false;
                                }
                                Err(e) => { println!("Error: {}", e) }
                            }
                        }
                    }
                }
            }
        }
        false
    }


    pub async fn list_assets(&mut self) -> Vec<EpicAsset> {
        match self.egs.get_assets(None, None).await {
            Ok(b) => { b }
            Err(_) => { Vec::new() }
        }
    }

    pub async fn get_asset_metadata(&mut self, asset: EpicAsset) -> Option<AssetManifest> {
        match self.egs.get_asset_manifest(None, None, asset).await {
            Ok(a) => { Some(a) }
            Err(_) => { None }
        }
    }

    pub async fn get_asset_info(&mut self, asset: EpicAsset) -> Option<HashMap<String, AssetInfo>> {
        match self.egs.get_asset_info(asset).await {
            Ok(a) => { Some(a) }
            Err(_) => { None }
        }
    }

    pub async fn get_game_token(&mut self) -> Option<GameToken> {
        match self.egs.get_game_token().await {
            Ok(a) => { Some(a) }
            Err(_) => { None }
        }
    }

    pub async fn get_ownership_token(&mut self, asset: EpicAsset) -> Option<String> {
        match self.egs.get_ownership_token(asset).await {
            Ok(a) => { Some(a.token) }
            Err(_) => { None }
        }
    }

    pub async fn get_user_entitlements(&mut self) -> Vec<Entitlement> {
        match self.egs.get_user_entitlements().await {
            Ok(a) => { a }
            Err(_) => { Vec::new() }
        }
    }

    pub async fn get_library_items(&mut self, include_metadata: bool) -> Option<Library> {
        match self.egs.get_library_items(include_metadata).await {
            Ok(a) => { Some(a) }
            Err(_) => { None }
        }
    }
}