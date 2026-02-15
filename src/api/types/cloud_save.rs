use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Response from cloud save list/query endpoints.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudSaveResponse {
    pub files: HashMap<String, CloudSaveFile>,
}

/// Individual cloud save file metadata with read/write links.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudSaveFile {
    pub file_name: Option<String>,
    pub read_link: Option<String>,
    pub write_link: Option<String>,
    pub last_modified: Option<String>,
    pub length: Option<i64>,
    pub storage_type: Option<String>,
    pub etag: Option<String>,
    pub unique_filename: Option<String>,
    pub account_id: Option<String>,
}
