use serde::{Deserialize, Serialize};

/// Paginated library response containing owned asset records.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub records: Vec<Record>,
    pub response_metadata: Option<ResponseMetadata>,
}

/// A single library entry referencing an owned asset.
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
    pub next_cursor: Option<String>,
}
