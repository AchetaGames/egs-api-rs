use crate::api::error::EpicAPIError;
use crate::api::types::account::{AccountData, AccountInfo, ExternalAuth};
use crate::api::types::entitlement::Entitlement;
use crate::api::types::friends::Friend;
use crate::api::EpicAPI;

impl EpicAPI {
    /// Fetch account details for the logged-in user.
    pub async fn account_details(&mut self) -> Result<AccountData, EpicAPIError> {
        let id = match &self.user_data.account_id {
            Some(id) => id,
            None => return Err(EpicAPIError::InvalidParams),
        };
        let url = format!(
            "https://account-public-service-prod03.ol.epicgames.com/account/api/public/account/{}",
            id
        );
        self.authorized_get_json(&url).await
    }

    /// Fetch display names for a list of account IDs.
    pub async fn account_ids_details(
        &mut self,
        ids: Vec<String>,
    ) -> Result<Vec<AccountInfo>, EpicAPIError> {
        if ids.is_empty() {
            return Err(EpicAPIError::InvalidParams);
        }
        let url = format!(
            "https://account-public-service-prod03.ol.epicgames.com/account/api/public/account?accountId={}",
            ids.join("&accountId=")
        );
        self.authorized_get_json(&url).await
    }

    /// Fetch the friends list, optionally including pending requests.
    pub async fn account_friends(
        &mut self,
        include_pending: bool,
    ) -> Result<Vec<Friend>, EpicAPIError> {
        let id = match &self.user_data.account_id {
            Some(id) => id,
            None => return Err(EpicAPIError::InvalidParams),
        };
        let url = format!(
            "https://friends-public-service-prod06.ol.epicgames.com/friends/api/public/friends/{}?includePending={}",
            id, include_pending
        );
        self.authorized_get_json(&url).await
    }

    /// Fetch all entitlements for the logged-in user.
    pub async fn user_entitlements(&self) -> Result<Vec<Entitlement>, EpicAPIError> {
        let url = match &self.user_data.account_id {
            None => {
                return Err(EpicAPIError::InvalidCredentials);
            }
            Some(id) => {
                format!("https://entitlement-public-service-prod08.ol.epicgames.com/entitlement/api/account/{}/entitlements?start=0&count=5000",
                        id)
            }
        };
        self.authorized_get_json(&url).await
    }

    /// Fetch external auth connections for an account.
    pub async fn external_auths(
        &self,
        account_id: &str,
    ) -> Result<Vec<ExternalAuth>, EpicAPIError> {
        let url = format!(
            "https://account-public-service-prod03.ol.epicgames.com/account/api/public/account/{}/externalAuths",
            account_id
        );
        self.authorized_get_json(&url).await
    }

    /// Fetch SSO domain list for Epic accounts.
    pub async fn sso_domains(&self) -> Result<Vec<String>, EpicAPIError> {
        self.authorized_get_json(
            "https://account-public-service-prod03.ol.epicgames.com/account/api/epicdomains/ssodomains",
        )
        .await
    }
}
