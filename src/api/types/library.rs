use serde::{Deserialize, Serialize};
use serde_with::rust::string_empty_as_none;

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub records: Vec<Record>,
    pub response_metadata: Option<ResponseMetadata>,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub app_name: String,
    pub catalog_item_id: String,
    pub namespace: String,
    pub product_id: String,
    pub sandbox_name: String,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMetadata {
    #[serde(with = "string_empty_as_none")]
    pub next_cursor: Option<String>,
}
