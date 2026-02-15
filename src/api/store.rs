use crate::api::error::EpicAPIError;
use crate::api::types::uplay::{
    UplayClaimResult, UplayCodesResult, UplayGraphQLResponse, UplayRedeemResult,
};
use crate::api::EpicAPI;
use log::error;

const UPLAY_CODES_QUERY: &str = r#"
query partnerIntegrationQuery($accountId: String!) {
  PartnerIntegration {
    accountUplayCodes(accountId: $accountId) {
      epicAccountId
      gameId
      uplayAccountId
      regionCode
      redeemedOnUplay
      redemptionTimestamp
    }
  }
}
"#;

const UPLAY_CLAIM_QUERY: &str = r#"
mutation claimUplayCode($accountId: String!, $uplayAccountId: String!, $gameId: String!) {
  PartnerIntegration {
    claimUplayCode(
      accountId: $accountId
      uplayAccountId: $uplayAccountId
      gameId: $gameId
    ) {
      data {
        assignmentTimestam
        epicAccountId
        epicEntitlement {
          entitlementId
          catalogItemId
          entitlementName
          country
        }
        gameId
        redeemedOnUplay
        redemptionTimestamp
        regionCode
        uplayAccountId
      }
      success
    }
  }
}
"#;

const UPLAY_REDEEM_QUERY: &str = r#"
mutation redeemAllPendingCodes($accountId: String!, $uplayAccountId: String!) {
  PartnerIntegration {
    redeemAllPendingCodes(accountId: $accountId, uplayAccountId: $uplayAccountId) {
      data {
        epicAccountId
        uplayAccountId
        redeemedOnUplay
        redemptionTimestamp
      }
      success
    }
  }
}
"#;

const STORE_GQL_URL: &str = "https://launcher.store.epicgames.com/graphql";
const STORE_USER_AGENT: &str = "EpicGamesLauncher/14.0.8-22004686+++Portal+Release-Live";

impl EpicAPI {
    /// Fetch Uplay codes linked to the user's Epic account via GraphQL.
    pub async fn store_get_uplay_codes(
        &self,
    ) -> Result<UplayGraphQLResponse<UplayCodesResult>, EpicAPIError> {
        let user_id = self
            .user_data
            .account_id
            .as_deref()
            .ok_or(EpicAPIError::InvalidCredentials)?;
        let body = serde_json::json!({
            "query": UPLAY_CODES_QUERY,
            "variables": { "accountId": user_id },
        });
        self.store_gql_request(&body).await
    }

    /// Claim a Uplay code for a specific game.
    pub async fn store_claim_uplay_code(
        &self,
        uplay_account_id: &str,
        game_id: &str,
    ) -> Result<UplayGraphQLResponse<UplayClaimResult>, EpicAPIError> {
        let user_id = self
            .user_data
            .account_id
            .as_deref()
            .ok_or(EpicAPIError::InvalidCredentials)?;
        let body = serde_json::json!({
            "query": UPLAY_CLAIM_QUERY,
            "variables": {
                "accountId": user_id,
                "uplayAccountId": uplay_account_id,
                "gameId": game_id,
            },
        });
        self.store_gql_request(&body).await
    }

    /// Redeem all pending Uplay codes for the user's account.
    pub async fn store_redeem_uplay_codes(
        &self,
        uplay_account_id: &str,
    ) -> Result<UplayGraphQLResponse<UplayRedeemResult>, EpicAPIError> {
        let user_id = self
            .user_data
            .account_id
            .as_deref()
            .ok_or(EpicAPIError::InvalidCredentials)?;
        let body = serde_json::json!({
            "query": UPLAY_REDEEM_QUERY,
            "variables": {
                "accountId": user_id,
                "uplayAccountId": uplay_account_id,
            },
        });
        self.store_gql_request(&body).await
    }

    /// Internal helper for store GraphQL requests with the required User-Agent.
    async fn store_gql_request<T: serde::de::DeserializeOwned>(
        &self,
        body: &serde_json::Value,
    ) -> Result<T, EpicAPIError> {
        let parsed_url =
            url::Url::parse(STORE_GQL_URL).map_err(|_| EpicAPIError::InvalidParams)?;
        let response = self
            .set_authorization_header(self.client.post(parsed_url))
            .header("User-Agent", STORE_USER_AGENT)
            .json(body)
            .send()
            .await
            .map_err(|e| {
                error!("{:?}", e);
                EpicAPIError::NetworkError(e)
            })?;
        if response.status() == reqwest::StatusCode::OK {
            response.json::<T>().await.map_err(|e| {
                error!("{:?}", e);
                EpicAPIError::DeserializationError(format!("{}", e))
            })
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            log::warn!("{} result: {}", status, body);
            Err(EpicAPIError::HttpError { status, body })
        }
    }
}
