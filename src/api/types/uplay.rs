use serde::{Deserialize, Serialize};

/// GraphQL response wrapper for Uplay partner integration queries.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UplayGraphQLResponse<T> {
    pub data: Option<UplayPartnerData<T>>,
    pub errors: Option<Vec<serde_json::Value>>,
}

/// Partner integration data container.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UplayPartnerData<T> {
    pub partner_integration: Option<T>,
}

/// Container for Uplay codes query result.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UplayCodesResult {
    pub account_uplay_codes: Option<Vec<UplayCode>>,
}

/// Container for Uplay code claim mutation result.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UplayClaimResult {
    pub claim_uplay_code: Option<UplayMutationResponse>,
}

/// Container for Uplay redeem-all mutation result.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UplayRedeemResult {
    pub redeem_all_pending_codes: Option<UplayMutationResponse>,
}

/// A Uplay code entry linking an Epic game to a Ubisoft account.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UplayCode {
    pub epic_account_id: Option<String>,
    pub game_id: Option<String>,
    pub uplay_account_id: Option<String>,
    pub region_code: Option<String>,
    pub redeemed_on_uplay: Option<bool>,
    pub redemption_timestamp: Option<String>,
}

/// Response from a Uplay claim or redeem mutation.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UplayMutationResponse {
    pub data: Option<Vec<UplayCode>>,
    pub success: Option<bool>,
}
