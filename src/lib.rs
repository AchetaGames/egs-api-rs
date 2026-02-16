#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

//! # Epic Games Store API
//!
//! Async Rust client for the Epic Games Store API. Provides authentication,
//! asset management, download manifest parsing (binary + JSON), and
//! [Fab](https://www.fab.com/) marketplace integration.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use egs_api::EpicGames;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut egs = EpicGames::new();
//!
//!     // Authenticate with an authorization code
//!     let code = "your_authorization_code".to_string();
//!     if egs.auth_code(None, Some(code)).await {
//!         println!("Logged in as {}", egs.user_details().display_name.unwrap_or_default());
//!     }
//!
//!     // List all owned assets
//!     let assets = egs.list_assets(None, None).await;
//!     println!("You own {} assets", assets.len());
//! }
//! ```
//!
//! # Authentication
//!
//! Epic uses OAuth2 with a public launcher client ID. The flow is:
//!
//! 1. Open the [authorization URL] in a browser — the user logs in and gets
//!    redirected to a JSON page with an `authorizationCode`.
//! 2. Pass that code to [`EpicGames::auth_code`].
//! 3. Persist the session with [`EpicGames::user_details`] (implements
//!    `Serialize` / `Deserialize`).
//! 4. Restore it later with [`EpicGames::set_user_details`] +
//!    [`EpicGames::login`], which uses the refresh token.
//!
//! [authorization URL]: https://www.epicgames.com/id/login?redirectUrl=https%3A%2F%2Fwww.epicgames.com%2Fid%2Fapi%2Fredirect%3FclientId%3D34a02cf8f4414e29b15921876da36f9a%26responseType%3Dcode
//!
//! # Features
//!
//! - **Assets** — List owned assets, fetch catalog metadata (with DLC trees),
//!   retrieve asset manifests with CDN download URLs.
//! - **Download Manifests** — Parse Epic's binary and JSON manifest formats.
//!   Exposes file lists, chunk hashes, and custom fields for download
//!   reconstruction.
//! - **Fab Marketplace** — List Fab library items, fetch signed asset manifests,
//!   and download manifests from distribution points.
//! - **Account** — Details, bulk ID lookup, friends list.
//! - **Entitlements** — Games, DLC, subscriptions.
//! - **Library** — Paginated listing with optional metadata.
//! - **Tokens** — Game exchange tokens and per-asset ownership tokens (JWT).
//!
//! # Architecture
//!
//! [`EpicGames`] is the public facade. It wraps an internal `EpicAPI` struct
//! that holds the `reqwest::Client` (with cookie store) and session state.
//! Most public methods return `Option<T>` or `Vec<T>`, swallowing transport
//! errors for convenience. Fab methods return `Result<T, EpicAPIError>` to
//! expose timeout/error distinctions.
//!
//! # Examples
//!
//! The crate ships with examples covering every endpoint. See the
//! [`examples/`](https://github.com/AchetaGames/egs-api-rs/tree/master/examples)
//! directory or run:
//!
//! ```bash
//! cargo run --example auth                # Interactive login + token persistence
//! cargo run --example account             # Account details, ID lookup, friends, external auths, SSO
//! cargo run --example entitlements        # List all entitlements
//! cargo run --example library             # Paginated library listing
//! cargo run --example assets              # Full pipeline: list → info → manifest → download
//! cargo run --example game_token          # Exchange code + ownership token
//! cargo run --example fab                 # Fab library → asset manifest → download manifest
//! cargo run --example catalog             # Catalog items, offers, bulk lookup
//! cargo run --example commerce            # Currencies, prices, billing, quick purchase
//! cargo run --example status              # Service status (lightswitch API)
//! cargo run --example presence            # Update online presence
//! cargo run --example client_credentials  # App-level auth + library state tokens
//! ```

use crate::api::types::account::{AccountData, AccountInfo, ExternalAuth, UserData};
use crate::api::types::epic_asset::EpicAsset;
use crate::api::types::fab_asset_manifest::DownloadInfo;
use crate::api::types::friends::Friend;
use crate::api::{EpicAPI};

use api::types::asset_info::{AssetInfo, GameToken};
use api::types::asset_manifest::AssetManifest;
use api::types::artifact_service::ArtifactServiceTicket;
use api::types::billing_account::BillingAccount;
use api::types::catalog_item::CatalogItemPage;
use api::types::catalog_offer::CatalogOfferPage;
use api::types::cloud_save::CloudSaveResponse;
use api::types::currency::CurrencyPage;
use api::types::download_manifest::DownloadManifest;
use api::types::entitlement::Entitlement;
use api::types::library::Library;
use api::types::presence::PresenceUpdate;
use api::types::price::PriceResponse;
use api::types::quick_purchase::QuickPurchaseResponse;
use api::types::service_status::ServiceStatus;
use api::types::uplay::{
    UplayClaimResult, UplayCodesResult, UplayGraphQLResponse, UplayRedeemResult,
};
use api::types::cosmos;
use api::types::engine_blob;
use api::types::fab_search;
use api::types::fab_taxonomy;
use api::types::fab_entitlement;
use log::{error, info, warn};
use crate::api::error::EpicAPIError;

/// Module for authenticated API communication
pub mod api;

/// Client for the Epic Games Store API.
///
/// This is the main entry point for the library. Create an instance with
/// [`EpicGames::new`], authenticate with [`EpicGames::auth_code`] or
/// [`EpicGames::login`], then call API methods.
///
/// Most methods return `Option<T>` or `Vec<T>`, returning `None` / empty on
/// errors. Fab methods return `Result<T, EpicAPIError>` for richer error
/// handling (e.g., distinguishing timeouts from other failures).
///
/// Session state is stored in [`UserData`], which implements
/// `Serialize` / `Deserialize` for persistence across runs.
#[derive(Debug, Clone)]
pub struct EpicGames {
    egs: EpicAPI,
}

impl Default for EpicGames {
    fn default() -> Self {
        Self::new()
    }
}

impl EpicGames {
    /// Creates a new [`EpicGames`] client.
    pub fn new() -> Self {
        EpicGames {
            egs: EpicAPI::new(),
        }
    }

    /// Check whether the user is logged in.
    ///
    /// Returns `true` if the access token exists and has more than 600 seconds
    /// remaining before expiry.
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

    /// Returns a clone of the current session state.
    ///
    /// The returned [`UserData`] implements `Serialize` / `Deserialize`,
    /// so you can persist it to disk and restore it later with
    /// [`set_user_details`](Self::set_user_details).
    pub fn user_details(&self) -> UserData {
        self.egs.user_data.clone()
    }

    /// Restore session state from a previously saved [`UserData`].
    ///
    /// Only merges `Some` fields — existing values are preserved for any
    /// field that is `None` in the input. Call [`login`](Self::login)
    /// afterward to refresh the access token.
    pub fn set_user_details(&mut self, user_details: UserData) {
        self.egs.user_data.update(user_details);
    }

    /// Like [`auth_code`](Self::auth_code), but returns a `Result` instead of swallowing errors.
    pub async fn try_auth_code(
        &mut self,
        exchange_token: Option<String>,
        authorization_code: Option<String>,
    ) -> Result<bool, EpicAPIError> {
        self.egs
            .start_session(exchange_token, authorization_code)
            .await
    }

    /// Authenticate with an authorization code or exchange token.
    ///
    /// Returns `true` on success, `false` on failure. Returns `None` on API errors.
    pub async fn auth_code(
        &mut self,
        exchange_token: Option<String>,
        authorization_code: Option<String>,
    ) -> bool {
        self.try_auth_code(exchange_token, authorization_code)
            .await
            .unwrap_or(false)
    }

    /// Invalidate the current session and log out.
    pub async fn logout(&mut self) -> bool {
        self.egs.invalidate_sesion().await
    }

    /// Like [`login`](Self::login), but returns a `Result` instead of swallowing errors.
    pub async fn try_login(&mut self) -> Result<bool, EpicAPIError> {
        if let Some(exp) = self.egs.user_data.expires_at {
            let now = chrono::offset::Utc::now();
            let td = exp - now;
            if td.num_seconds() > 600 {
                info!("Trying to re-use existing login session... ");
                let resumed = self.egs.resume_session().await.map_err(|e| {
                    warn!("{}", e);
                    e
                })?;
                if resumed {
                    info!("Logged in");
                    return Ok(true);
                }
                return Ok(false);
            }
        }
        info!("Logging in...");
        if let Some(exp) = self.egs.user_data.refresh_expires_at {
            let now = chrono::offset::Utc::now();
            let td = exp - now;
            if td.num_seconds() > 600 {
                let started = self.egs.start_session(None, None).await.map_err(|e| {
                    error!("{}", e);
                    e
                })?;
                if started {
                    info!("Logged in");
                    return Ok(true);
                }
                return Ok(false);
            }
        }
        Ok(false)
    }

    /// Resume session using the saved refresh token.
    ///
    /// Returns `true` on success, `false` if the refresh token has expired or is invalid.
    /// Unlike [`try_login`](Self::try_login), this method falls through to
    /// refresh-token login if session resume fails.
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
                match self.egs.start_session(None, None).await {
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

    /// Like [`list_assets`](Self::list_assets), but returns a `Result` instead of swallowing errors.
    pub async fn try_list_assets(
        &mut self,
        platform: Option<String>,
        label: Option<String>,
    ) -> Result<Vec<EpicAsset>, EpicAPIError> {
        self.egs.assets(platform, label).await
    }

    /// List all owned assets.
    ///
    /// Defaults to platform="Windows" and label="Live" if not specified.
    /// Returns empty `Vec` on API errors.
    pub async fn list_assets(
        &mut self,
        platform: Option<String>,
        label: Option<String>,
    ) -> Vec<EpicAsset> {
        self.try_list_assets(platform, label)
            .await
            .unwrap_or_else(|_| Vec::new())
    }

    /// Like [`asset_manifest`](Self::asset_manifest), but returns a `Result` instead of swallowing errors.
    pub async fn try_asset_manifest(
        &mut self,
        platform: Option<String>,
        label: Option<String>,
        namespace: Option<String>,
        item_id: Option<String>,
        app: Option<String>,
    ) -> Result<AssetManifest, EpicAPIError> {
        self.egs
            .asset_manifest(platform, label, namespace, item_id, app)
            .await
    }

    /// Fetch asset manifest with CDN download URLs.
    ///
    /// Defaults to platform="Windows" and label="Live" if not specified.
    /// Returns `None` on API errors.
    pub async fn asset_manifest(
        &mut self,
        platform: Option<String>,
        label: Option<String>,
        namespace: Option<String>,
        item_id: Option<String>,
        app: Option<String>,
    ) -> Option<AssetManifest> {
        self.try_asset_manifest(platform, label, namespace, item_id, app)
            .await
            .ok()
    }

    /// Fetch Fab asset manifest with signed distribution points.
    ///
    /// Returns `Result` to expose timeout errors (403 → `EpicAPIError::FabTimeout`).
    pub async fn fab_asset_manifest(
        &self,
        artifact_id: &str,
        namespace: &str,
        asset_id: &str,
        platform: Option<&str>,
    ) -> Result<Vec<DownloadInfo>, EpicAPIError> {
        match self
            .egs
            .fab_asset_manifest(artifact_id, namespace, asset_id, platform)
            .await
        {
            Ok(a) => Ok(a),
            Err(e) => Err(e),
        }
    }

    /// Like [`asset_info`](Self::asset_info), but returns a `Result` instead of swallowing errors.
    pub async fn try_asset_info(
        &mut self,
        asset: &EpicAsset,
    ) -> Result<Option<AssetInfo>, EpicAPIError> {
        let mut info = self.egs.asset_info(asset).await?;
        Ok(info.remove(asset.catalog_item_id.as_str()))
    }

    /// Fetch catalog metadata for an asset (includes DLC tree).
    ///
    /// Returns `None` on API errors.
    pub async fn asset_info(&mut self, asset: &EpicAsset) -> Option<AssetInfo> {
        self.try_asset_info(asset).await.ok().flatten()
    }

    /// Like [`account_details`](Self::account_details), but returns a `Result` instead of swallowing errors.
    pub async fn try_account_details(&mut self) -> Result<AccountData, EpicAPIError> {
        self.egs.account_details().await
    }

    /// Fetch account details (email, display name, country, 2FA status).
    ///
    /// Returns `None` on API errors.
    pub async fn account_details(&mut self) -> Option<AccountData> {
        self.try_account_details().await.ok()
    }

    /// Like [`account_ids_details`](Self::account_ids_details), but returns a `Result` instead of swallowing errors.
    pub async fn try_account_ids_details(
        &mut self,
        ids: Vec<String>,
    ) -> Result<Vec<AccountInfo>, EpicAPIError> {
        self.egs.account_ids_details(ids).await
    }

    /// Bulk lookup of account IDs to display names.
    ///
    /// Returns `None` on API errors.
    pub async fn account_ids_details(&mut self, ids: Vec<String>) -> Option<Vec<AccountInfo>> {
        self.try_account_ids_details(ids).await.ok()
    }

    /// Like [`account_friends`](Self::account_friends), but returns a `Result` instead of swallowing errors.
    pub async fn try_account_friends(
        &mut self,
        include_pending: bool,
    ) -> Result<Vec<Friend>, EpicAPIError> {
        self.egs.account_friends(include_pending).await
    }

    /// Fetch friends list (including pending requests if `include_pending` is true).
    ///
    /// Returns `None` on API errors.
    pub async fn account_friends(&mut self, include_pending: bool) -> Option<Vec<Friend>> {
        self.try_account_friends(include_pending).await.ok()
    }

    /// Like [`game_token`](Self::game_token), but returns a `Result` instead of swallowing errors.
    pub async fn try_game_token(&mut self) -> Result<GameToken, EpicAPIError> {
        self.egs.game_token().await
    }

    /// Fetch a short-lived exchange code for game launches.
    ///
    /// Returns `None` on API errors.
    pub async fn game_token(&mut self) -> Option<GameToken> {
        self.try_game_token().await.ok()
    }

    /// Like [`ownership_token`](Self::ownership_token), but returns a `Result` instead of swallowing errors.
    pub async fn try_ownership_token(&mut self, asset: &EpicAsset) -> Result<String, EpicAPIError> {
        self.egs.ownership_token(asset).await.map(|a| a.token)
    }

    /// Fetch a JWT proving ownership of an asset.
    ///
    /// Returns `None` on API errors.
    pub async fn ownership_token(&mut self, asset: &EpicAsset) -> Option<String> {
        self.try_ownership_token(asset).await.ok()
    }

    /// Like [`user_entitlements`](Self::user_entitlements), but returns a `Result` instead of swallowing errors.
    pub async fn try_user_entitlements(&mut self) -> Result<Vec<Entitlement>, EpicAPIError> {
        self.egs.user_entitlements().await
    }

    /// Fetch all user entitlements (games, DLC, subscriptions).
    ///
    /// Returns empty `Vec` on API errors.
    pub async fn user_entitlements(&mut self) -> Vec<Entitlement> {
        self.try_user_entitlements().await.unwrap_or_else(|_| Vec::new())
    }

    /// Like [`library_items`](Self::library_items), but returns a `Result` instead of swallowing errors.
    pub async fn try_library_items(&mut self, include_metadata: bool) -> Result<Library, EpicAPIError> {
        self.egs.library_items(include_metadata).await
    }

    /// Fetch the user library with optional metadata.
    ///
    /// Paginates internally and returns all records at once. Returns `None` on API errors.
    pub async fn library_items(&mut self, include_metadata: bool) -> Option<Library> {
        self.try_library_items(include_metadata).await.ok()
    }

    /// Like [`fab_library_items`](Self::fab_library_items), but returns a `Result` instead of swallowing errors.
    pub async fn try_fab_library_items(
        &mut self,
        account_id: String,
    ) -> Result<api::types::fab_library::FabLibrary, EpicAPIError> {
        self.egs.fab_library_items(account_id).await
    }

    /// Fetch the user Fab library.
    ///
    /// Paginates internally and returns all records at once. Returns `None` on API errors.
    pub async fn fab_library_items(
        &mut self,
        account_id: String,
    ) -> Option<api::types::fab_library::FabLibrary> {
        self.try_fab_library_items(account_id).await.ok()
    }

    /// Parse download manifests from all CDN mirrors.
    ///
    /// Fetches from all mirrors, parses binary/JSON format, and populates custom fields
    /// (BaseUrl, CatalogItemId, etc.). Returns empty `Vec` on API errors.
    pub async fn asset_download_manifests(&self, manifest: AssetManifest) -> Vec<DownloadManifest> {
        self.egs.asset_download_manifests(manifest).await
    }

    /// Parse a Fab download manifest from a specific distribution point.
    ///
    /// Checks signature expiration before fetching. Returns `Result` to expose timeout errors.
    pub async fn fab_download_manifest(
        &self,
        download_info: DownloadInfo,
        distribution_point_url: &str,
    ) -> Result<DownloadManifest, EpicAPIError> {
        self.egs
            .fab_download_manifest(download_info, distribution_point_url)
            .await
    }

    /// Like [`auth_client_credentials`](Self::auth_client_credentials), but returns a `Result` instead of swallowing errors.
    pub async fn try_auth_client_credentials(&mut self) -> Result<bool, EpicAPIError> {
        self.egs.start_client_credentials_session().await
    }

    /// Authenticate with client credentials (app-level, no user context).
    ///
    /// Uses the launcher's public client ID/secret to obtain an access token
    /// without any user interaction. The resulting session has limited
    /// permissions — it can query public endpoints (catalog, service status,
    /// currencies) but cannot access user-specific data (library, entitlements).
    ///
    /// Returns `true` on success, `false` on failure.
    pub async fn auth_client_credentials(&mut self) -> bool {
        self.try_auth_client_credentials().await.unwrap_or(false)
    }

    /// Like [`external_auths`](Self::external_auths), but returns a `Result` instead of swallowing errors.
    pub async fn try_external_auths(&self, account_id: &str) -> Result<Vec<ExternalAuth>, EpicAPIError> {
        self.egs.external_auths(account_id).await
    }

    /// Fetch external auth connections linked to an account.
    ///
    /// Returns linked platform accounts (Steam, PSN, Xbox, Nintendo, etc.)
    /// with external display names and account IDs. Requires a valid user session.
    ///
    /// Returns `None` on API errors.
    pub async fn external_auths(&self, account_id: &str) -> Option<Vec<ExternalAuth>> {
        self.try_external_auths(account_id).await.ok()
    }

    /// Like [`sso_domains`](Self::sso_domains), but returns a `Result` instead of swallowing errors.
    pub async fn try_sso_domains(&self) -> Result<Vec<String>, EpicAPIError> {
        self.egs.sso_domains().await
    }

    /// Fetch the list of SSO (Single Sign-On) domains.
    ///
    /// Returns domain strings that support Epic's SSO flow. Used by the
    /// launcher to determine which domains can share authentication cookies.
    ///
    /// Returns `None` on API errors.
    pub async fn sso_domains(&self) -> Option<Vec<String>> {
        self.try_sso_domains().await.ok()
    }

    /// Like [`catalog_items`](Self::catalog_items), but returns a `Result` instead of swallowing errors.
    pub async fn try_catalog_items(
        &self,
        namespace: &str,
        start: i64,
        count: i64,
    ) -> Result<CatalogItemPage, EpicAPIError> {
        self.egs.catalog_items(namespace, start, count).await
    }

    /// Fetch paginated catalog items for a namespace.
    ///
    /// Queries the Epic catalog service for items within a given namespace
    /// (e.g., a game's namespace). Results are paginated — use `start` and
    /// `count` to page through. Each [`CatalogItemPage`] includes a `paging`
    /// field with the total count.
    ///
    /// Returns `None` on API errors.
    pub async fn catalog_items(
        &self,
        namespace: &str,
        start: i64,
        count: i64,
    ) -> Option<CatalogItemPage> {
        self.try_catalog_items(namespace, start, count).await.ok()
    }

    /// Like [`catalog_offers`](Self::catalog_offers), but returns a `Result` instead of swallowing errors.
    pub async fn try_catalog_offers(
        &self,
        namespace: &str,
        start: i64,
        count: i64,
    ) -> Result<CatalogOfferPage, EpicAPIError> {
        self.egs.catalog_offers(namespace, start, count).await
    }

    /// Fetch paginated catalog offers for a namespace.
    ///
    /// Queries the Epic catalog service for offers (purchasable items) within
    /// a namespace. Offers include pricing metadata, seller info, and linked
    /// catalog items. Use `start` and `count` to paginate.
    ///
    /// Returns `None` on API errors.
    pub async fn catalog_offers(
        &self,
        namespace: &str,
        start: i64,
        count: i64,
    ) -> Option<CatalogOfferPage> {
        self.try_catalog_offers(namespace, start, count).await.ok()
    }

    /// Like [`bulk_catalog_items`](Self::bulk_catalog_items), but returns a `Result` instead of swallowing errors.
    pub async fn try_bulk_catalog_items(
        &self,
        items: &[(&str, &str)],
    ) -> Result<std::collections::HashMap<String, std::collections::HashMap<String, AssetInfo>>, EpicAPIError> {
        self.egs.bulk_catalog_items(items).await
    }

    /// Bulk fetch catalog items across multiple namespaces.
    ///
    /// Accepts a slice of `(namespace, item_id)` pairs and returns them grouped
    /// by namespace → item_id → [`AssetInfo`]. Useful for resolving catalog
    /// metadata for items from different games in a single request.
    ///
    /// Returns `None` on API errors.
    pub async fn bulk_catalog_items(
        &self,
        items: &[(&str, &str)],
    ) -> Option<std::collections::HashMap<String, std::collections::HashMap<String, AssetInfo>>> {
        self.try_bulk_catalog_items(items).await.ok()
    }

    /// Like [`currencies`](Self::currencies), but returns a `Result` instead of swallowing errors.
    pub async fn try_currencies(&self, start: i64, count: i64) -> Result<CurrencyPage, EpicAPIError> {
        self.egs.currencies(start, count).await
    }

    /// Fetch available currencies from the Epic catalog.
    ///
    /// Returns paginated currency definitions including code, symbol, and
    /// decimal precision. Use `start` and `count` to paginate.
    ///
    /// Returns `None` on API errors.
    pub async fn currencies(&self, start: i64, count: i64) -> Option<CurrencyPage> {
        self.try_currencies(start, count).await.ok()
    }

    /// Like [`library_state_token_status`](Self::library_state_token_status), but returns a `Result` instead of swallowing errors.
    pub async fn try_library_state_token_status(
        &self,
        token_id: &str,
    ) -> Result<bool, EpicAPIError> {
        self.egs.library_state_token_status(token_id).await
    }

    /// Check the validity of a library state token.
    ///
    /// Returns `Some(true)` if the token is still valid, `Some(false)` if
    /// expired or invalid, or `None` on API errors. Library state tokens are
    /// used to detect changes to the user's library since the last sync.
    ///
    /// Returns `None` on API errors.
    pub async fn library_state_token_status(&self, token_id: &str) -> Option<bool> {
        self.try_library_state_token_status(token_id).await.ok()
    }

    /// Like [`service_status`](Self::service_status), but returns a `Result` instead of swallowing errors.
    pub async fn try_service_status(
        &self,
        service_id: &str,
    ) -> Result<Vec<ServiceStatus>, EpicAPIError> {
        self.egs.service_status(service_id).await
    }

    /// Fetch service status from Epic's lightswitch API.
    ///
    /// Returns the operational status of an Epic online service (e.g., a game's
    /// backend). The response includes whether the service is UP/DOWN, any
    /// maintenance message, and whether the current user is banned.
    ///
    /// Returns `None` on API errors.
    pub async fn service_status(&self, service_id: &str) -> Option<Vec<ServiceStatus>> {
        self.try_service_status(service_id).await.ok()
    }

    /// Like [`offer_prices`](Self::offer_prices), but returns a `Result` instead of swallowing errors.
    pub async fn try_offer_prices(
        &self,
        namespace: &str,
        offer_ids: &[String],
        country: &str,
    ) -> Result<PriceResponse, EpicAPIError> {
        self.egs.offer_prices(namespace, offer_ids, country).await
    }

    /// Fetch offer prices from Epic's price engine.
    ///
    /// Queries current pricing for one or more offers within a namespace,
    /// localized to a specific country. The response includes original price,
    /// discount price, and pre-formatted display strings.
    ///
    /// Returns `None` on API errors.
    pub async fn offer_prices(
        &self,
        namespace: &str,
        offer_ids: &[String],
        country: &str,
    ) -> Option<PriceResponse> {
        self.try_offer_prices(namespace, offer_ids, country).await.ok()
    }

    /// Like [`quick_purchase`](Self::quick_purchase), but returns a `Result` instead of swallowing errors.
    pub async fn try_quick_purchase(
        &self,
        namespace: &str,
        offer_id: &str,
    ) -> Result<QuickPurchaseResponse, EpicAPIError> {
        self.egs.quick_purchase(namespace, offer_id).await
    }

    /// Execute a quick purchase (typically for free game claims).
    ///
    /// Initiates a purchase order for a free offer. The response contains the
    /// order ID and its processing status. For paid offers, use the full
    /// checkout flow in the Epic Games launcher instead.
    ///
    /// Returns `None` on API errors.
    pub async fn quick_purchase(
        &self,
        namespace: &str,
        offer_id: &str,
    ) -> Option<QuickPurchaseResponse> {
        self.try_quick_purchase(namespace, offer_id).await.ok()
    }

    /// Like [`billing_account`](Self::billing_account), but returns a `Result` instead of swallowing errors.
    pub async fn try_billing_account(&self) -> Result<BillingAccount, EpicAPIError> {
        self.egs.billing_account().await
    }

    /// Fetch the default billing account for payment processing.
    ///
    /// Returns the account's billing country, which is used to determine
    /// regional pricing and payment availability.
    ///
    /// Returns `None` on API errors.
    pub async fn billing_account(&self) -> Option<BillingAccount> {
        self.try_billing_account().await.ok()
    }

    /// Update the user's presence status.
    ///
    /// Sends a PATCH request to update the user's online presence (e.g.,
    /// "online", "away") and optionally set an activity with custom properties.
    /// The `session_id` is the OAuth session token from login. Returns `Ok(())`
    /// on success (204 No Content) or an [`EpicAPIError`] on failure.
    pub async fn update_presence(
        &self,
        session_id: &str,
        body: &PresenceUpdate,
    ) -> Result<(), EpicAPIError> {
        self.egs.update_presence(session_id, body).await
    }

    /// Like [`fab_file_download_info`](Self::fab_file_download_info), but returns a `Result` instead of swallowing errors.
    pub async fn try_fab_file_download_info(
        &self,
        listing_id: &str,
        format_id: &str,
        file_id: &str,
    ) -> Result<DownloadInfo, EpicAPIError> {
        self.egs
            .fab_file_download_info(listing_id, format_id, file_id)
            .await
    }

    /// Fetch download info for a specific file within a Fab listing.
    ///
    /// Returns signed [`DownloadInfo`] for a single file identified by
    /// `listing_id`, `format_id`, and `file_id`. Use this for targeted
    /// downloads of individual files from a Fab asset rather than fetching
    /// the entire asset manifest.
    ///
    /// Returns `None` on API errors.
    pub async fn fab_file_download_info(
        &self,
        listing_id: &str,
        format_id: &str,
        file_id: &str,
    ) -> Option<DownloadInfo> {
        self.try_fab_file_download_info(listing_id, format_id, file_id)
            .await
            .ok()
    }

    // ── Cloud Saves ──

    /// List cloud save files for the logged-in user.
    ///
    /// If `app_name` is provided, lists saves for that specific game.
    /// If `manifests` is true (only relevant when `app_name` is set), lists manifest files.
    pub async fn cloud_save_list(
        &self,
        app_name: Option<&str>,
        manifests: bool,
    ) -> Result<CloudSaveResponse, EpicAPIError> {
        self.egs.cloud_save_list(app_name, manifests).await
    }

    /// Query cloud save files by specific filenames.
    ///
    /// Returns metadata including read/write links for the specified files.
    pub async fn cloud_save_query(
        &self,
        app_name: &str,
        filenames: &[String],
    ) -> Result<CloudSaveResponse, EpicAPIError> {
        self.egs.cloud_save_query(app_name, filenames).await
    }

    /// Delete a cloud save file by its storage path.
    pub async fn cloud_save_delete(&self, path: &str) -> Result<(), EpicAPIError> {
        self.egs.cloud_save_delete(path).await
    }

    // ── Artifact Service & Manifests ──

    /// Fetch an artifact service ticket for manifest retrieval via EOS Helper.
    ///
    /// The `sandbox_id` is typically the game's namespace and `artifact_id`
    /// is the app name. Returns a signed ticket for use with
    /// [`game_manifest_by_ticket`](Self::game_manifest_by_ticket).
    pub async fn artifact_service_ticket(
        &self,
        sandbox_id: &str,
        artifact_id: &str,
        label: Option<&str>,
        platform: Option<&str>,
    ) -> Result<ArtifactServiceTicket, EpicAPIError> {
        self.egs
            .artifact_service_ticket(sandbox_id, artifact_id, label, platform)
            .await
    }

    /// Fetch a game manifest using a signed artifact service ticket.
    ///
    /// Alternative to [`asset_manifest`](Self::asset_manifest) using ticket-based
    /// auth from the EOS Helper service.
    pub async fn game_manifest_by_ticket(
        &self,
        artifact_id: &str,
        signed_ticket: &str,
        label: Option<&str>,
        platform: Option<&str>,
    ) -> Result<AssetManifest, EpicAPIError> {
        self.egs
            .game_manifest_by_ticket(artifact_id, signed_ticket, label, platform)
            .await
    }

    /// Fetch launcher manifests for self-update checks.
    pub async fn launcher_manifests(
        &self,
        platform: Option<&str>,
        label: Option<&str>,
    ) -> Result<AssetManifest, EpicAPIError> {
        self.egs.launcher_manifests(platform, label).await
    }

    /// Try to fetch a delta manifest for optimized patching between builds.
    ///
    /// Returns `None` if no delta is available or the builds are identical.
    pub async fn delta_manifest(
        &self,
        base_url: &str,
        old_build_id: &str,
        new_build_id: &str,
    ) -> Option<Vec<u8>> {
        self.egs
            .delta_manifest(base_url, old_build_id, new_build_id)
            .await
    }

    // ── SID Auth ──

    /// Authenticate via session ID (SID) from the Epic web login flow.
    ///
    /// Performs the multi-step web exchange: set-sid → CSRF → exchange code,
    /// then starts a session with the resulting code. Returns `true` on success.
    pub async fn auth_sid(&mut self, sid: &str) -> Result<bool, EpicAPIError> {
        self.egs.auth_sid(sid).await
    }

    // ── Uplay / Ubisoft Store ──

    /// Fetch Uplay codes linked to the user's Epic account.
    pub async fn store_get_uplay_codes(
        &self,
    ) -> Result<UplayGraphQLResponse<UplayCodesResult>, EpicAPIError> {
        self.egs.store_get_uplay_codes().await
    }

    /// Claim a Uplay code for a specific game.
    pub async fn store_claim_uplay_code(
        &self,
        uplay_account_id: &str,
        game_id: &str,
    ) -> Result<UplayGraphQLResponse<UplayClaimResult>, EpicAPIError> {
        self.egs
            .store_claim_uplay_code(uplay_account_id, game_id)
            .await
    }

    /// Redeem all pending Uplay codes for the user's account.
    pub async fn store_redeem_uplay_codes(
        &self,
        uplay_account_id: &str,
    ) -> Result<UplayGraphQLResponse<UplayRedeemResult>, EpicAPIError> {
        self.egs.store_redeem_uplay_codes(uplay_account_id).await
    }

    // --- Cosmos Session ---

    /// Set up a Cosmos cookie session from an exchange code.
    /// Typically called with a code from `game_token()`.
    pub async fn cosmos_session_setup(
        &self,
        exchange_code: &str,
    ) -> Result<cosmos::CosmosAuthResponse, EpicAPIError> {
        self.egs.cosmos_session_setup(exchange_code).await
    }

    /// Upgrade bearer token to Cosmos session (step 5 of session setup).
    pub async fn cosmos_auth_upgrade(
        &self,
    ) -> Result<cosmos::CosmosAuthResponse, EpicAPIError> {
        self.egs.cosmos_auth_upgrade().await
    }

    /// Check if a EULA has been accepted. Returns `None` on error.
    pub async fn cosmos_eula_check(&self, eula_id: &str, locale: &str) -> Option<bool> {
        self.egs
            .cosmos_eula_check(eula_id, locale)
            .await
            .ok()
            .map(|r| r.accepted)
    }

    /// Check if a EULA has been accepted. Returns full `Result`.
    pub async fn try_cosmos_eula_check(
        &self,
        eula_id: &str,
        locale: &str,
    ) -> Result<cosmos::CosmosEulaResponse, EpicAPIError> {
        self.egs.cosmos_eula_check(eula_id, locale).await
    }

    /// Accept a EULA. Returns `None` on error.
    pub async fn cosmos_eula_accept(
        &self,
        eula_id: &str,
        locale: &str,
        version: u32,
    ) -> Option<bool> {
        self.egs
            .cosmos_eula_accept(eula_id, locale, version)
            .await
            .ok()
            .map(|r| r.accepted)
    }

    /// Accept a EULA. Returns full `Result`.
    pub async fn try_cosmos_eula_accept(
        &self,
        eula_id: &str,
        locale: &str,
        version: u32,
    ) -> Result<cosmos::CosmosEulaResponse, EpicAPIError> {
        self.egs.cosmos_eula_accept(eula_id, locale, version).await
    }

    /// Get Cosmos account details. Returns `None` on error.
    pub async fn cosmos_account(&self) -> Option<cosmos::CosmosAccount> {
        self.egs.cosmos_account().await.ok()
    }

    /// Get Cosmos account details. Returns full `Result`.
    pub async fn try_cosmos_account(&self) -> Result<cosmos::CosmosAccount, EpicAPIError> {
        self.egs.cosmos_account().await
    }

    /// Fetch engine version download blobs for a platform. Returns `None` on error.
    pub async fn engine_versions(
        &self,
        platform: &str,
    ) -> Option<engine_blob::EngineBlobsResponse> {
        self.egs.engine_versions(platform).await.ok()
    }

    /// Fetch engine version download blobs. Returns full `Result`.
    pub async fn try_engine_versions(
        &self,
        platform: &str,
    ) -> Result<engine_blob::EngineBlobsResponse, EpicAPIError> {
        self.egs.engine_versions(platform).await
    }

    // --- Fab Search/Browse ---

    /// Search Fab listings. Returns `None` on error.
    pub async fn fab_search(
        &self,
        params: &fab_search::FabSearchParams,
    ) -> Option<fab_search::FabSearchResults> {
        self.egs.fab_search(params).await.ok()
    }

    /// Search Fab listings. Returns full `Result`.
    pub async fn try_fab_search(
        &self,
        params: &fab_search::FabSearchParams,
    ) -> Result<fab_search::FabSearchResults, EpicAPIError> {
        self.egs.fab_search(params).await
    }

    /// Get full listing detail. Returns `None` on error.
    pub async fn fab_listing(&self, uid: &str) -> Option<fab_search::FabListingDetail> {
        self.egs.fab_listing(uid).await.ok()
    }

    /// Get full listing detail. Returns full `Result`.
    pub async fn try_fab_listing(
        &self,
        uid: &str,
    ) -> Result<fab_search::FabListingDetail, EpicAPIError> {
        self.egs.fab_listing(uid).await
    }

    /// Get UE-specific format details for a listing. Returns `None` on error.
    pub async fn fab_listing_ue_formats(
        &self,
        uid: &str,
    ) -> Option<Vec<fab_search::FabListingUeFormat>> {
        self.egs.fab_listing_ue_formats(uid).await.ok()
    }

    /// Get UE-specific format details. Returns full `Result`.
    pub async fn try_fab_listing_ue_formats(
        &self,
        uid: &str,
    ) -> Result<Vec<fab_search::FabListingUeFormat>, EpicAPIError> {
        self.egs.fab_listing_ue_formats(uid).await
    }

    /// Get listing state (ownership, wishlist, review). Returns `None` on error.
    pub async fn fab_listing_state(
        &self,
        uid: &str,
    ) -> Option<fab_search::FabListingState> {
        self.egs.fab_listing_state(uid).await.ok()
    }

    /// Get listing state. Returns full `Result`.
    pub async fn try_fab_listing_state(
        &self,
        uid: &str,
    ) -> Result<fab_search::FabListingState, EpicAPIError> {
        self.egs.fab_listing_state(uid).await
    }

    /// Bulk check listing states. Returns `None` on error.
    pub async fn fab_listing_states_bulk(
        &self,
        listing_ids: &[&str],
    ) -> Option<Vec<fab_search::FabListingState>> {
        self.egs.fab_listing_states_bulk(listing_ids).await.ok()
    }

    /// Bulk check listing states. Returns full `Result`.
    pub async fn try_fab_listing_states_bulk(
        &self,
        listing_ids: &[&str],
    ) -> Result<Vec<fab_search::FabListingState>, EpicAPIError> {
        self.egs.fab_listing_states_bulk(listing_ids).await
    }

    /// Bulk fetch pricing for multiple offer IDs. Returns `None` on error.
    pub async fn fab_bulk_prices(
        &self,
        offer_ids: &[&str],
    ) -> Option<Vec<fab_search::FabPriceInfo>> {
        self.egs.fab_bulk_prices(offer_ids).await.ok()
    }

    /// Bulk fetch pricing. Returns full `Result`.
    pub async fn try_fab_bulk_prices(
        &self,
        offer_ids: &[&str],
    ) -> Result<Vec<fab_search::FabPriceInfo>, EpicAPIError> {
        self.egs.fab_bulk_prices(offer_ids).await
    }

    /// Get listing ownership info. Returns `None` on error.
    pub async fn fab_listing_ownership(
        &self,
        uid: &str,
    ) -> Option<fab_search::FabOwnership> {
        self.egs.fab_listing_ownership(uid).await.ok()
    }

    /// Get listing ownership info. Returns full `Result`.
    pub async fn try_fab_listing_ownership(
        &self,
        uid: &str,
    ) -> Result<fab_search::FabOwnership, EpicAPIError> {
        self.egs.fab_listing_ownership(uid).await
    }

    /// Get pricing for a specific listing. Returns `None` on error.
    pub async fn fab_listing_prices(
        &self,
        uid: &str,
    ) -> Option<Vec<fab_search::FabPriceInfo>> {
        self.egs.fab_listing_prices(uid).await.ok()
    }

    /// Get pricing for a specific listing. Returns full `Result`.
    pub async fn try_fab_listing_prices(
        &self,
        uid: &str,
    ) -> Result<Vec<fab_search::FabPriceInfo>, EpicAPIError> {
        self.egs.fab_listing_prices(uid).await
    }

    /// Get reviews for a listing. Returns `None` on error.
    pub async fn fab_listing_reviews(
        &self,
        uid: &str,
        sort_by: Option<&str>,
        cursor: Option<&str>,
    ) -> Option<fab_search::FabReviewsResponse> {
        self.egs.fab_listing_reviews(uid, sort_by, cursor).await.ok()
    }

    /// Get reviews for a listing. Returns full `Result`.
    pub async fn try_fab_listing_reviews(
        &self,
        uid: &str,
        sort_by: Option<&str>,
        cursor: Option<&str>,
    ) -> Result<fab_search::FabReviewsResponse, EpicAPIError> {
        self.egs.fab_listing_reviews(uid, sort_by, cursor).await
    }

    // --- Fab Taxonomy ---

    /// Fetch available license types. Returns `None` on error.
    pub async fn fab_licenses(&self) -> Option<Vec<fab_taxonomy::FabLicenseType>> {
        self.egs.fab_licenses().await.ok()
    }

    /// Fetch available license types. Returns full `Result`.
    pub async fn try_fab_licenses(
        &self,
    ) -> Result<Vec<fab_taxonomy::FabLicenseType>, EpicAPIError> {
        self.egs.fab_licenses().await
    }

    /// Fetch asset format groups. Returns `None` on error.
    pub async fn fab_format_groups(&self) -> Option<Vec<fab_taxonomy::FabFormatGroup>> {
        self.egs.fab_format_groups().await.ok()
    }

    /// Fetch asset format groups. Returns full `Result`.
    pub async fn try_fab_format_groups(
        &self,
    ) -> Result<Vec<fab_taxonomy::FabFormatGroup>, EpicAPIError> {
        self.egs.fab_format_groups().await
    }

    /// Fetch tag groups with nested tags. Returns `None` on error.
    pub async fn fab_tag_groups(&self) -> Option<Vec<fab_taxonomy::FabTagGroup>> {
        self.egs.fab_tag_groups().await.ok()
    }

    /// Fetch tag groups with nested tags. Returns full `Result`.
    pub async fn try_fab_tag_groups(
        &self,
    ) -> Result<Vec<fab_taxonomy::FabTagGroup>, EpicAPIError> {
        self.egs.fab_tag_groups().await
    }

    /// Fetch available UE versions. Returns `None` on error.
    pub async fn fab_ue_versions(&self) -> Option<Vec<String>> {
        self.egs.fab_ue_versions().await.ok()
    }

    /// Fetch available UE versions. Returns full `Result`.
    pub async fn try_fab_ue_versions(&self) -> Result<Vec<String>, EpicAPIError> {
        self.egs.fab_ue_versions().await
    }

    /// Fetch channel info by slug. Returns `None` on error.
    pub async fn fab_channel(&self, slug: &str) -> Option<fab_taxonomy::FabChannel> {
        self.egs.fab_channel(slug).await.ok()
    }

    /// Fetch channel info by slug. Returns full `Result`.
    pub async fn try_fab_channel(
        &self,
        slug: &str,
    ) -> Result<fab_taxonomy::FabChannel, EpicAPIError> {
        self.egs.fab_channel(slug).await
    }

    // --- Fab Library Entitlements ---

    /// Search library entitlements. Returns `None` on error.
    pub async fn fab_library_entitlements(
        &self,
        params: &fab_entitlement::FabEntitlementSearchParams,
    ) -> Option<fab_entitlement::FabEntitlementResults> {
        self.egs.fab_library_entitlements(params).await.ok()
    }

    /// Search library entitlements. Returns full `Result`.
    pub async fn try_fab_library_entitlements(
        &self,
        params: &fab_entitlement::FabEntitlementSearchParams,
    ) -> Result<fab_entitlement::FabEntitlementResults, EpicAPIError> {
        self.egs.fab_library_entitlements(params).await
    }

    // --- Cosmos Policy/Communication ---

    /// Check Age of Digital Consent policy. Returns `None` on error.
    pub async fn cosmos_policy_aodc(&self) -> Option<cosmos::CosmosPolicyAodc> {
        self.egs.cosmos_policy_aodc().await.ok()
    }

    /// Check Age of Digital Consent policy. Returns full `Result`.
    pub async fn try_cosmos_policy_aodc(
        &self,
    ) -> Result<cosmos::CosmosPolicyAodc, EpicAPIError> {
        self.egs.cosmos_policy_aodc().await
    }

    /// Check communication opt-in status. Returns `None` on error.
    pub async fn cosmos_comm_opt_in(
        &self,
        setting: &str,
    ) -> Option<cosmos::CosmosCommOptIn> {
        self.egs.cosmos_comm_opt_in(setting).await.ok()
    }

    /// Check communication opt-in status. Returns full `Result`.
    pub async fn try_cosmos_comm_opt_in(
        &self,
        setting: &str,
    ) -> Result<cosmos::CosmosCommOptIn, EpicAPIError> {
        self.egs.cosmos_comm_opt_in(setting).await
    }
}

#[cfg(test)]
mod facade_tests {
    use super::*;
    use crate::api::types::account::UserData;
    use chrono::{Duration, Utc};

    #[test]
    fn new_creates_instance() {
        let egs = EpicGames::new();
        assert!(!egs.is_logged_in());
    }

    #[test]
    fn default_same_as_new() {
        let egs = EpicGames::default();
        assert!(!egs.is_logged_in());
    }

    #[test]
    fn user_details_default_empty() {
        let egs = EpicGames::new();
        assert!(egs.user_details().access_token.is_none());
    }

    #[test]
    fn set_and_get_user_details() {
        let mut egs = EpicGames::new();
        let mut ud = UserData::new();
        ud.display_name = Some("TestUser".to_string());
        egs.set_user_details(ud);
        assert_eq!(egs.user_details().display_name, Some("TestUser".to_string()));
    }

    #[test]
    fn is_logged_in_expired() {
        let mut egs = EpicGames::new();
        let mut ud = UserData::new();
        ud.expires_at = Some(Utc::now() - Duration::hours(1));
        egs.set_user_details(ud);
        assert!(!egs.is_logged_in());
    }

    #[test]
    fn is_logged_in_valid() {
        let mut egs = EpicGames::new();
        let mut ud = UserData::new();
        ud.expires_at = Some(Utc::now() + Duration::hours(2));
        egs.set_user_details(ud);
        assert!(egs.is_logged_in());
    }

    #[test]
    fn is_logged_in_within_600s_threshold() {
        let mut egs = EpicGames::new();
        let mut ud = UserData::new();
        ud.expires_at = Some(Utc::now() + Duration::seconds(500));
        egs.set_user_details(ud);
        assert!(!egs.is_logged_in());
    }
}
