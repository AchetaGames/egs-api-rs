use serde::{Deserialize, Serialize};

/// Response from a quick purchase (free claim) request.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickPurchaseResponse {
    pub order_id: Option<String>,
    pub status: Option<String>,
    pub namespace: Option<String>,
    pub offer_id: Option<String>,
}
