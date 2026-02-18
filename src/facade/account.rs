use crate::EpicGames;
use crate::api::error::EpicAPIError;
use crate::api::types::account::{AccountData, AccountInfo, ExternalAuth};
use crate::api::types::asset_info::GameToken;
use crate::api::types::entitlement::Entitlement;
use crate::api::types::epic_asset::EpicAsset;
use crate::api::types::friends::Friend;
use crate::api::types::library::Library;

impl EpicGames {
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

    /// Like [`external_auths`](Self::external_auths), but returns a `Result` instead of swallowing errors.
    pub async fn try_external_auths(
        &self,
        account_id: &str,
    ) -> Result<Vec<ExternalAuth>, EpicAPIError> {
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
        self.try_user_entitlements()
            .await
            .unwrap_or_else(|_| Vec::new())
    }

    /// Like [`library_items`](Self::library_items), but returns a `Result` instead of swallowing errors.
    pub async fn try_library_items(
        &mut self,
        include_metadata: bool,
    ) -> Result<Library, EpicAPIError> {
        self.egs.library_items(include_metadata).await
    }

    /// Fetch the user library with optional metadata.
    ///
    /// Paginates internally and returns all records at once. Returns `None` on API errors.
    pub async fn library_items(&mut self, include_metadata: bool) -> Option<Library> {
        self.try_library_items(include_metadata).await.ok()
    }
}
