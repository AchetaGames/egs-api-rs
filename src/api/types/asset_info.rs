use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetInfo {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub key_images: Option<Vec<KeyImage>>,
    pub categories: Option<Vec<Category>>,
    pub namespace: String,
    pub status: Option<String>,
    pub creation_date: Option<DateTime<Utc>>,
    pub last_modified_date: Option<DateTime<Utc>>,
    pub custom_attributes: Option<HashMap<String, CustomAttribute>>,
    pub entitlement_name: Option<String>,
    pub entitlement_type: Option<String>,
    pub item_type: Option<String>,
    pub release_info: Option<Vec<ReleaseInfo>>,
    pub developer: Option<String>,
    pub developer_id: Option<String>,
    #[serde(default)]
    pub eula_ids: Vec<String>,
    pub end_of_support: Option<bool>,
    #[serde(default)]
    pub dlc_item_list: Vec<AssetInfo>,
    pub age_gatings: Option<::serde_json::Value>,
    pub application_id: Option<String>,
    pub unsearchable: bool,
    pub self_refundable: Option<bool>,
    pub requires_secure_account: Option<bool>,
    pub long_description: Option<String>,
    pub main_game_item: Box<Option<AssetInfo>>,
    pub esrb_game_rating_value: Option<String>,
    pub use_count: Option<i64>,
    pub technical_details: Option<String>,
    #[serde(default)]
    pub install_modes: Vec<::serde_json::Value>,
}

impl AssetInfo {
    /// Get the latest release by release_date
    pub fn get_latest_release(&self) -> Option<ReleaseInfo> {
        if let Some(releases) = self.get_sorted_releases() {
            if let Some(rel) = releases.first() {
                return Some(rel.clone());
            }
        }
        None
    }

    /// Get list of sorted releases newest to oldest
    pub fn get_sorted_releases(&self) -> Option<Vec<ReleaseInfo>> {
        if let Some(mut release_info) = self.release_info.clone() {
            release_info.sort_by_key(|ri| ri.date_added);
            release_info.reverse();
            Some(release_info)
        } else {
            None
        }
    }

    /// Get list of all compatible apps across all releases
    pub fn get_compatible_apps(&self) -> Option<Vec<String>> {
        match &self.release_info {
            None => {}
            Some(release_infos) => {
                let mut res: Vec<String> = Vec::new();
                for info in release_infos {
                    match &info.compatible_apps {
                        None => {}
                        Some(ca) => res.append(&mut ca.clone()),
                    };
                }
                res.sort();
                res.dedup();
                return Some(res);
            }
        }
        None
    }

    /// Get list of all platforms across all releases
    pub fn get_platforms(&self) -> Option<Vec<String>> {
        match &self.release_info {
            None => {}
            Some(release_infos) => {
                let mut res: Vec<String> = Vec::new();
                for info in release_infos {
                    match &info.platform {
                        None => {}
                        Some(p) => res.append(&mut p.clone()),
                    };
                }
                res.sort();
                res.dedup();
                return Some(res);
            }
        }
        None
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyImage {
    #[serde(rename = "type")]
    pub type_field: String,
    pub url: Url,
    pub md5: String,
    pub width: i64,
    pub height: i64,
    pub size: i64,
    pub uploaded_date: DateTime<Utc>,
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
pub struct CustomAttribute {
    #[serde(rename = "type")]
    pub type_field: String,
    pub value: String,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseInfo {
    pub id: Option<String>,
    pub app_id: Option<String>,
    pub compatible_apps: Option<Vec<String>>,
    pub platform: Option<Vec<String>>,
    pub date_added: Option<DateTime<Utc>>,
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
