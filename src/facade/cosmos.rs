use crate::EpicGames;
use crate::api::error::EpicAPIError;
use crate::api::types::cosmos;

impl EpicGames {
    /// Set up a Cosmos cookie session from an exchange code.
    /// Typically called with a code from `game_token()`.
    pub async fn cosmos_session_setup(
        &self,
        exchange_code: &str,
    ) -> Result<cosmos::CosmosAuthResponse, EpicAPIError> {
        self.egs.cosmos_session_setup(exchange_code).await
    }

    /// Upgrade bearer token to Cosmos session (step 5 of session setup).
    pub async fn cosmos_auth_upgrade(&self) -> Result<cosmos::CosmosAuthResponse, EpicAPIError> {
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

    /// Check Age of Digital Consent policy. Returns `None` on error.
    pub async fn cosmos_policy_aodc(&self) -> Option<cosmos::CosmosPolicyAodc> {
        self.egs.cosmos_policy_aodc().await.ok()
    }

    /// Check Age of Digital Consent policy. Returns full `Result`.
    pub async fn try_cosmos_policy_aodc(&self) -> Result<cosmos::CosmosPolicyAodc, EpicAPIError> {
        self.egs.cosmos_policy_aodc().await
    }

    /// Check communication opt-in status. Returns `None` on error.
    pub async fn cosmos_comm_opt_in(&self, setting: &str) -> Option<cosmos::CosmosCommOptIn> {
        self.egs.cosmos_comm_opt_in(setting).await.ok()
    }

    /// Check communication opt-in status. Returns full `Result`.
    pub async fn try_cosmos_comm_opt_in(
        &self,
        setting: &str,
    ) -> Result<cosmos::CosmosCommOptIn, EpicAPIError> {
        self.egs.cosmos_comm_opt_in(setting).await
    }

    /// Search unrealengine.com content. Requires an active Cosmos session.
    ///
    /// Returns `None` on any error.
    pub async fn cosmos_search(
        &self,
        query: &str,
        slug: Option<&str>,
        locale: Option<&str>,
        filter: Option<&str>,
    ) -> Option<cosmos::CosmosSearchResults> {
        self.try_cosmos_search(query, slug, locale, filter)
            .await
            .ok()
    }

    /// Like [`cosmos_search`](Self::cosmos_search), but returns a `Result` instead of swallowing errors.
    pub async fn try_cosmos_search(
        &self,
        query: &str,
        slug: Option<&str>,
        locale: Option<&str>,
        filter: Option<&str>,
    ) -> Result<cosmos::CosmosSearchResults, EpicAPIError> {
        self.egs.cosmos_search(query, slug, locale, filter).await
    }
}
