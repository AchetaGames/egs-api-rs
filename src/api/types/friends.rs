use serde::{Deserialize, Serialize};

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Friend {
    pub account_id: String,
    pub created: String,
    pub direction: String,
    pub favorite: bool,
    pub status: String,
}