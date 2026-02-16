use serde::{Deserialize, Serialize};

/// Response from `GET /api/cosmos/auth` — session upgrade result.
///
/// After calling `set-sid`, this endpoint upgrades the bearer token and
/// issues EPIC_EG1 / EPIC_EG1_REFRESH JWTs (set as cookies) required
/// by all other `/api/cosmos/*` endpoints.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CosmosAuthResponse {
    pub bearer_token_valid: bool,
    pub cleared_offline: bool,
    pub upgraded_bearer_token: bool,
    pub account_id: String,
}

/// Error response from Cosmos when not authenticated.
///
/// Returned as `{"error": "Not logged in", "isLoggedIn": false}`
/// when EPIC_EG1 cookie is missing or expired.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CosmosAuthError {
    pub error: String,
    pub is_logged_in: bool,
}

/// Response from `GET /api/cosmos/account` — account info.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CosmosAccount {
    pub country: String,
    pub display_name: String,
    pub email: String,
    pub id: String,
    pub preferred_language: String,
    pub cabined_mode: bool,
    pub is_logged_in: bool,
}

/// Response from `GET/POST /api/cosmos/eula/accept`.
///
/// GET checks if a EULA is accepted; POST accepts it.
/// Known EULA IDs: `unreal_engine`, `unreal_engine2`, `realityscan`, `mhc`, `content`
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CosmosEulaResponse {
    pub accepted: bool,
}

/// Response from `GET /api/cosmos/policy/aodc` — Age of Digital Consent.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CosmosPolicyAodc {
    pub failed: bool,
}

/// Response from `GET /api/cosmos/communication/opt-in`.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CosmosCommOptIn {
    pub setting_value: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_cosmos_auth() {
        let json = r#"{"bearerTokenValid":true,"clearedOffline":false,"upgradedBearerToken":true,"accountId":"8645b4947bbc4c0092a8b7236df169d1"}"#;
        let auth: CosmosAuthResponse = serde_json::from_str(json).unwrap();
        assert!(auth.bearer_token_valid);
        assert!(auth.upgraded_bearer_token);
        assert_eq!(auth.account_id, "8645b4947bbc4c0092a8b7236df169d1");
    }

    #[test]
    fn deserialize_cosmos_auth_error() {
        let json = r#"{"error":"Not logged in","isLoggedIn":false}"#;
        let err: CosmosAuthError = serde_json::from_str(json).unwrap();
        assert_eq!(err.error, "Not logged in");
        assert!(!err.is_logged_in);
    }

    #[test]
    fn deserialize_cosmos_eula() {
        let json = r#"{"accepted":true}"#;
        let eula: CosmosEulaResponse = serde_json::from_str(json).unwrap();
        assert!(eula.accepted);
    }

    #[test]
    fn deserialize_cosmos_account() {
        let json = r#"{"country":"CZ","displayName":"Acheta Games","email":"m***n@stastnej.ch","id":"8645b4947bbc4c0092a8b7236df169d1","preferredLanguage":"en","cabinedMode":false,"isLoggedIn":true}"#;
        let account: CosmosAccount = serde_json::from_str(json).unwrap();
        assert_eq!(account.country, "CZ");
        assert_eq!(account.display_name, "Acheta Games");
        assert!(account.is_logged_in);
    }

    #[test]
    fn deserialize_policy_aodc() {
        let json = r#"{"failed":false}"#;
        let policy: CosmosPolicyAodc = serde_json::from_str(json).unwrap();
        assert!(!policy.failed);
    }

    #[test]
    fn deserialize_comm_opt_in() {
        let json = r#"{"settingValue":false}"#;
        let opt: CosmosCommOptIn = serde_json::from_str(json).unwrap();
        assert!(!opt.setting_value);
    }
}
