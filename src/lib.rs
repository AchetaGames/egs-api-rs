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

use chrono;
use reqwest::header;
use log::{error,info,warn};

use api::types::asset_info::{AssetInfo, GameToken};
use api::types::asset_manifest::{AssetManifest, Manifest};
use api::types::download_manifest::DownloadManifest;
use api::types::library::Library;
use api::types::entitlement::Entitlement;

use crate::api::types::epic_asset::EpicAsset;
use crate::api::{EpicAPI, EpicAPIError, UserData};

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
        match self.egs.user_data.expires_at {
            None => {}
            Some(exp) => {
                let now = chrono::offset::Utc::now();
                let td = exp - now;
                if td.num_seconds() > 600 {
                    return true;
                }
            }
        }
        return false;
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
            "EpicGamesLauncher/11.0.1-14907503+++Portal+Release-Live "
                .parse()
                .unwrap(),
        );
        let url = format!("https://www.epicgames.com/id/api/set-sid?sid={}", sid);
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .default_headers(headers)
            .build()
            .unwrap();
        match client.get(&url).send().await {
            Ok(_resp) => {}
            _ => {}
        }

        let mut xsrf_token: String = "".to_string();

        match client
            .get("https://www.epicgames.com/id/api/csrf")
            .send()
            .await
        {
            Ok(resp) => {
                for cookie in resp.cookies() {
                    if cookie.name().to_lowercase() == "xsrf-token" {
                        xsrf_token = cookie.value().to_string();
                    }
                }
            }
            _ => {}
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
                    match echo_json["code"].as_str() {
                        Some(t) => Some(t.to_string()),
                        None => None,
                    }
                } else {
                    //let echo_json: serde_json::Value = resp.json().await.unwrap();
                    //TODO: return the error from echo_json
                    None
                }
            }
            _ => None,
        }
    }

    /// Start session with auth code
    pub async fn auth_code(&mut self, code: String) -> bool {
        match self.egs.start_session(Some(code)).await {
            Ok(b) => {
                return b;
            }
            Err(_) => {
                return false;
            }
        }
    }

    /// Invalidate existing session
    pub async fn logout(&mut self) -> bool {
        self.egs.invalidate_sesion().await
    }

    /// Perform login based on previous authentication
    pub async fn login(&mut self) -> bool {
        match self.egs.user_data.expires_at {
            None => {}
            Some(exp) => {
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
        }
        info!("Logging in...");
        match self.egs.user_data.refresh_expires_at {
            None => {}
            Some(exp) => {
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
        }
        false
    }

    /// Returns all assets
    pub async fn list_assets(&mut self) -> Vec<EpicAsset> {
        match self.egs.assets(None, None).await {
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
    pub async fn asset_download_manifest(
        &self,
        manifest: Manifest,
    ) -> Result<DownloadManifest, EpicAPIError> {
        match self.egs.asset_download_manifest(manifest).await {
            Ok(manifest) => Ok(manifest),
            Err(e) => Err(e),
        }
    }
}
