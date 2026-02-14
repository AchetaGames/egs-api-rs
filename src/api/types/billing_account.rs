use serde::{Deserialize, Serialize};

/// Default billing account for payment processing.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BillingAccount {
    pub id: Option<String>,
    pub country: Option<String>,
}
