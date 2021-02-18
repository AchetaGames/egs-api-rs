use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpicAsset {
    pub app_name: String,
    pub label_name: String,
    pub build_version: String,
    pub catalog_item_id: String,
    pub namespace: String,
    pub asset_id: String,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetManifest {
    pub elements: Vec<Element>,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Element {
    pub app_name: String,
    pub label_name: String,
    pub build_version: String,
    pub hash: String,
    pub manifests: Vec<Manifest>,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub uri: String,
    pub query_params: Vec<QueryParam>,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryParam {
    pub name: String,
    pub value: String,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub long_description: Option<String>,
    pub technical_details: Option<String>,
    pub key_images: Vec<KeyImage>,
    pub categories: Vec<Category>,
    pub namespace: String,
    pub status: String,
    pub creation_date: String,
    pub last_modified_date: String,
    pub entitlement_name: String,
    pub entitlement_type: String,
    pub item_type: String,
    pub release_info: Vec<ReleaseInfo>,
    pub developer: String,
    pub developer_id: String,
    pub end_of_support: bool,
    pub unsearchable: bool,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyImage {
    #[serde(rename = "type")]
    pub type_field: String,
    pub url: String,
    pub md5: String,
    pub width: i64,
    pub height: i64,
    pub size: i64,
    pub uploaded_date: String,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub path: String,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseInfo {
    pub id: Option<String>,
    pub app_id: Option<String>,
    pub compatible_apps: Option<Vec<String>>,
    pub platform: Vec<String>,
    pub date_added: Option<String>,
    pub release_note: Option<String>,
    pub version_title: Option<String>,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameToken {
    pub expires_in_seconds: i64,
    pub code: String,
    pub creating_client_id: String,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OwnershipToken {
    pub token: String,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DownloadManifest {
    pub manifest_file_version: String,
    #[serde(rename = "bIsFileData")]
    pub b_is_file_data: bool,
    #[serde(rename = "AppID")]
    pub app_id: String,
    pub app_name_string: String,
    pub build_version_string: String,
    pub launch_exe_string: String,
    pub launch_command: String,
    pub prereq_ids: Option<Vec<::serde_json::Value>>,
    pub prereq_name: String,
    pub prereq_path: String,
    pub prereq_args: String,
    pub file_manifest_list: Vec<FileManifestList>,
    pub chunk_hash_list: HashMap<String, String>,
    pub chunk_sha_list: Option<HashMap<String, String>>,
    pub data_group_list: HashMap<String, String>,
    pub chunk_filesize_list: HashMap<String, String>,
    pub custom_fields: ::serde_json::Value,
}

impl DownloadManifest {
    /// Convert numbers in the Download Manifest from little indian and %03d concatenated string
    pub fn blob_to_num(str: String) -> u64 {
        let mut num: u64 = 0;
        let mut shift: u64 = 0;
        for i in (0..str.len()).step_by(3) {
            if let Ok(n) = str[i..i + 3].parse::<u64>() {
                num += n << shift;
                shift += 8;
            }
        };
        return num;
    }

    /// Get chunk dir based on the manifest version
    pub fn get_chunk_dir(version: u64) -> &'static str {
        if version >= 15 {
            "ChunksV4"
        } else if version >= 6 {
            "ChunksV3"
        } else if version >= 3 {
            "ChunksV2"
        } else {
            "Chunks"
        }
    }

    /// Get the download links from the downloaded manifest
    pub fn get_download_links(&self, manifest_url: String) -> Vec<String> {
        let url = manifest_url.split("/").collect::<Vec<&str>>().split_last().unwrap().1.join("/");


        let chunk_dir = DownloadManifest::get_chunk_dir(DownloadManifest::blob_to_num(self.manifest_file_version.to_string()));
        let mut result: Vec<String> = Vec::new();

        for (guid, hash) in self.chunk_hash_list.clone() {
            let group_num = match self.data_group_list.get(&guid) {
                None => { continue; }
                Some(group) => { DownloadManifest::blob_to_num(group.to_string()) }
            };
            result.push(format!("{} {}/{}/{:02}/{:016X}_{}.chunk", DownloadManifest::blob_to_num(self.chunk_filesize_list.get(&guid).unwrap().to_string()), url, chunk_dir, group_num, DownloadManifest::blob_to_num(hash), guid));
        }
        result
    }
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FileManifestList {
    pub filename: String,
    pub file_hash: String,
    pub file_chunk_parts: Vec<FileChunkPart>,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FileChunkPart {
    pub guid: String,
    pub offset: String,
    pub size: String,
}

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
    pub next_cursor: Option<String>,
}
