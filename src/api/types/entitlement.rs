use serde::{Deserialize, Serialize};

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entitlement {
    pub id: String,
    pub entitlement_name: String,
    pub namespace: String,
    pub catalog_item_id: String,
    pub account_id: String,
    pub identity_id: String,
    pub entitlement_type: String,
    pub grant_date: String,
    pub consumable: bool,
    pub status: String,
    pub active: bool,
    pub use_count: i64,
    pub created: String,
    pub updated: String,
    pub group_entitlement: bool,
    pub original_use_count: Option<i64>,
    pub platform_type: Option<String>,
    pub country: Option<String>,
}
