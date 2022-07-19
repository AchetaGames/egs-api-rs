#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

//! # Epic Games Store API
//!
//! A minimal asynchronous interface to Epic Games Store
//!
//! # This is under heavy development expect major breaking changes
//!
//! ## Current functionality
//!  - Authentication
//!  - Listing Assets
//!  - Get Asset metadata
//!  - Get Asset info
//!  - Get Ownership Token
//!  - Get Game Token
//!  - Get Entitlements
//!  - Get Library Items
//!  - Generate download links for chunks

use log::{error, info, warn};
use reqwest::header;

use api::types::asset_info::{AssetInfo, GameToken};
use api::types::asset_manifest::AssetManifest;
use api::types::download_manifest::DownloadManifest;
use api::types::entitlement::Entitlement;
use api::types::library::Library;

use crate::api::types::account::{AccountData, AccountInfo, UserData};
use crate::api::types::epic_asset::EpicAsset;
use crate::api::types::friends::Friend;
use crate::api::EpicAPI;

/// Module for authenticated API communication
pub mod api;

/// Struct to manage the communication with the Epic Games Store Api
#[derive(Default, Debug, Clone)]
pub struct EpicGames {
    egs: EpicAPI,
}

impl EpicGames {
    /// Creates new object
    pub fn new() -> Self {
        EpicGames {
            egs: EpicAPI::new(),
        }
    }

    /// Check whether the user is logged in
    pub fn is_logged_in(&self) -> bool {
        if let Some(exp) = self.egs.user_data.expires_at {
            let now = chrono::offset::Utc::now();
            let td = exp - now;
            if td.num_seconds() > 600 {
                return true;
            }
        }
        false
    }

    /// Get User details
    pub fn user_details(&self) -> UserData {
        self.egs.user_data.clone()
    }

    /// Update User Details
    pub fn set_user_details(&mut self, user_details: UserData) {
        self.egs.user_data.update(user_details);
    }

    /// Authenticate with sid
    pub async fn auth_sid(&self, sid: &str) -> Option<String> {
        // get first set of cookies (EPIC_BEARER_TOKEN etc.)
        let mut headers = header::HeaderMap::new();
        headers.insert("X-Epic-Event-Action", "login".parse().unwrap());
        headers.insert("X-Epic-Event-Category", "login".parse().unwrap());
        headers.insert("X-Epic-Strategy-Flags", "".parse().unwrap());
        headers.insert("X-Requested-With", "XMLHttpRequest".parse().unwrap());
        headers.insert(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) EpicGamesLauncher/11.0.1-14907503+++Portal+Release-Live UnrealEngine/4.23.0-14907503+++Portal+Release-Live Chrome/84.0.4147.38 Safari/537.36"
                .parse()
                .unwrap(),
        );
        let url = format!("https://www.epicgames.com/id/api/set-sid?sid={}", sid);
        let mut store = cookie_store::CookieStore::load_json("".as_bytes()).unwrap();
        store
            .parse(
                "EPIC_COUNTRY=US",
                &reqwest::Url::parse("https://www.epicgames.com").unwrap(),
            )
            .unwrap();
        let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(store);
        let cookie_store = std::sync::Arc::new(cookie_store);
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .cookie_provider(cookie_store)
            .default_headers(headers)
            .build()
            .unwrap();

        if let Ok(_resp) = client.get(&url).send().await {}

        let mut xsrf_token: String = "".to_string();

        if let Ok(resp) = client
            .get("https://www.epicgames.com/id/api/csrf")
            .send()
            .await
        {
            for cookie in resp.cookies() {
                if cookie.name().to_lowercase() == "xsrf-token" {
                    xsrf_token = cookie.value().to_string();
                    break;
                }
            }
        }

        match client
            .post("https://www.epicgames.com/id/api/exchange/generate")
            .header("X-XSRF-TOKEN", xsrf_token)
            .send()
            .await
        {
            Ok(resp) => {
                if resp.status() == reqwest::StatusCode::OK {
                    let echo_json: serde_json::Value = resp.json().await.unwrap();
                    echo_json["code"].as_str().map(|t| t.to_string())
                } else {
                    let echo_json: serde_json::Value = resp.json().await.unwrap();
                    error!("{:?}", echo_json);
                    //TODO: return the error from echo_json
                    None
                }
            }
            _ => None,
        }
    }

    /// Start session with auth code
    pub async fn auth_code(&mut self, code: String) -> bool {
        self.egs.start_session(Some(code)).await.unwrap_or(false)
    }

    /// Invalidate existing session
    pub async fn logout(&mut self) -> bool {
        self.egs.invalidate_sesion().await
    }

    /// Perform login based on previous authentication
    pub async fn login(&mut self) -> bool {
        if let Some(exp) = self.egs.user_data.expires_at {
            let now = chrono::offset::Utc::now();
            let td = exp - now;
            if td.num_seconds() > 600 {
                info!("Trying to re-use existing login session... ");
                match self.egs.resume_session().await {
                    Ok(b) => {
                        if b {
                            info!("Logged in");
                            return true;
                        }
                        return false;
                    }
                    Err(e) => {
                        warn!("{}", e)
                    }
                };
            }
        }
        info!("Logging in...");
        if let Some(exp) = self.egs.user_data.refresh_expires_at {
            let now = chrono::offset::Utc::now();
            let td = exp - now;
            if td.num_seconds() > 600 {
                match self.egs.start_session(None).await {
                    Ok(b) => {
                        if b {
                            info!("Logged in");
                            return true;
                        }
                        return false;
                    }
                    Err(e) => {
                        error!("{}", e)
                    }
                }
            }
        }
        false
    }

    /// Returns all assets
    pub async fn list_assets(
        &mut self,
        platform: Option<String>,
        label: Option<String>,
    ) -> Vec<EpicAsset> {
        match self.egs.assets(platform, label).await {
            Ok(b) => b,
            Err(_) => Vec::new(),
        }
    }

    /// Return asset
    pub async fn asset_manifest(
        &mut self,
        platform: Option<String>,
        label: Option<String>,
        namespace: Option<String>,
        item_id: Option<String>,
        app: Option<String>,
    ) -> Option<AssetManifest> {
        match self
            .egs
            .asset_manifest(platform, label, namespace, item_id, app)
            .await
        {
            Ok(a) => Some(a),
            Err(_) => None,
        }
    }

    /// Returns info for an asset
    pub async fn asset_info(&mut self, asset: EpicAsset) -> Option<AssetInfo> {
        match self.egs.asset_info(asset.clone()).await {
            Ok(mut a) => a.remove(asset.catalog_item_id.as_str()),
            Err(_) => None,
        }
    }

    /// Returns account details
    pub async fn account_details(&mut self) -> Option<AccountData> {
        match self.egs.account_details().await {
            Ok(a) => Some(a),
            Err(_) => None,
        }
    }

    /// Returns account id info
    pub async fn account_ids_details(&mut self, ids: Vec<String>) -> Option<Vec<AccountInfo>> {
        match self.egs.account_ids_details(ids).await {
            Ok(a) => Some(a),
            Err(_) => None,
        }
    }

    /// Returns account id info
    pub async fn account_friends(&mut self, include_pending: bool) -> Option<Vec<Friend>> {
        match self.egs.account_friends(include_pending).await {
            Ok(a) => Some(a),
            Err(_) => None,
        }
    }

    /// Returns game token
    pub async fn game_token(&mut self) -> Option<GameToken> {
        match self.egs.game_token().await {
            Ok(a) => Some(a),
            Err(_) => None,
        }
    }

    /// Returns ownership token for an Asset
    pub async fn ownership_token(&mut self, asset: EpicAsset) -> Option<String> {
        match self.egs.ownership_token(asset).await {
            Ok(a) => Some(a.token),
            Err(_) => None,
        }
    }

    ///Returns user entitlements
    pub async fn user_entitlements(&mut self) -> Vec<Entitlement> {
        match self.egs.user_entitlements().await {
            Ok(a) => a,
            Err(_) => Vec::new(),
        }
    }

    /// Returns the user library
    pub async fn library_items(&mut self, include_metadata: bool) -> Option<Library> {
        match self.egs.library_items(include_metadata).await {
            Ok(a) => Some(a),
            Err(_) => None,
        }
    }

    /// Returns a DownloadManifest for a specified file manifest
    pub async fn asset_download_manifests(&self, manifest: AssetManifest) -> Vec<DownloadManifest> {
        self.egs.asset_download_manifests(manifest).await
    }
}
