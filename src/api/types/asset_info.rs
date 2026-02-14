use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Detailed catalog metadata for an asset, including DLC list and release info.
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
    pub fn latest_release(&self) -> Option<ReleaseInfo> {
        if let Some(releases) = self.sorted_releases() {
            if let Some(rel) = releases.first() {
                return Some(rel.clone());
            }
        }
        None
    }

    /// Get list of sorted releases newest to oldest
    pub fn sorted_releases(&self) -> Option<Vec<ReleaseInfo>> {
        if let Some(mut release_info) = self.release_info.clone() {
            release_info.sort_by_key(|ri| ri.date_added);
            release_info.reverse();
            Some(release_info)
        } else {
            None
        }
    }

    /// Get release info based on the release id
    pub fn release_info(&self, id: &str) -> Option<ReleaseInfo> {
        if let Some(releases) = self.release_info.clone() {
            for release in releases {
                if release.id.clone().unwrap_or_default().eq(id) {
                    return Some(release);
                }
            }
        };
        None
    }

    /// Get release info based on the release name
    pub fn release_name(&self, name: &str) -> Option<ReleaseInfo> {
        if let Some(releases) = self.release_info.clone() {
            for release in releases {
                if release.app_id.clone().unwrap_or_default().eq(name) {
                    return Some(release);
                }
            }
        };
        None
    }

    /// Get list of all compatible apps across all releases
    pub fn compatible_apps(&self) -> Option<Vec<String>> {
        if let Some(release_infos) = &self.release_info {
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
        None
    }

    /// Get list of all platforms across all releases
    pub fn platforms(&self) -> Option<Vec<String>> {
        if let Some(release_infos) = &self.release_info {
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

/// A short-lived exchange code for game launches.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameToken {
    pub expires_in_seconds: i64,
    pub code: String,
    pub creating_client_id: String,
}

/// JWT token proving ownership of an asset.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OwnershipToken {
    pub token: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn make_release(
        id: &str,
        app_id: &str,
        days_ago: i64,
        platforms: Vec<&str>,
        compat: Vec<&str>,
    ) -> ReleaseInfo {
        ReleaseInfo {
            id: Some(id.to_string()),
            app_id: Some(app_id.to_string()),
            date_added: Some(Utc::now() - Duration::days(days_ago)),
            platform: if platforms.is_empty() {
                None
            } else {
                Some(platforms.iter().map(|s| s.to_string()).collect())
            },
            compatible_apps: if compat.is_empty() {
                None
            } else {
                Some(compat.iter().map(|s| s.to_string()).collect())
            },
            ..Default::default()
        }
    }

    #[test]
    fn latest_release_returns_newest() {
        let info = AssetInfo {
            release_info: Some(vec![
                make_release("old", "app_old", 30, vec![], vec![]),
                make_release("new", "app_new", 1, vec![], vec![]),
                make_release("mid", "app_mid", 15, vec![], vec![]),
            ]),
            ..Default::default()
        };
        let latest = info.latest_release().unwrap();
        assert_eq!(latest.id, Some("new".to_string()));
    }

    #[test]
    fn sorted_releases_order() {
        let info = AssetInfo {
            release_info: Some(vec![
                make_release("old", "a", 30, vec![], vec![]),
                make_release("new", "b", 1, vec![], vec![]),
                make_release("mid", "c", 15, vec![], vec![]),
            ]),
            ..Default::default()
        };
        let sorted = info.sorted_releases().unwrap();
        assert_eq!(sorted[0].id, Some("new".to_string()));
        assert_eq!(sorted[1].id, Some("mid".to_string()));
        assert_eq!(sorted[2].id, Some("old".to_string()));
    }

    #[test]
    fn latest_release_none_when_no_releases() {
        let info = AssetInfo {
            release_info: None,
            ..Default::default()
        };
        assert!(info.latest_release().is_none());
    }

    #[test]
    fn release_info_by_id() {
        let info = AssetInfo {
            release_info: Some(vec![
                make_release("r1", "a", 10, vec![], vec![]),
                make_release("r2", "b", 5, vec![], vec![]),
            ]),
            ..Default::default()
        };
        assert_eq!(
            info.release_info("r2").unwrap().app_id,
            Some("b".to_string())
        );
        assert!(info.release_info("nonexistent").is_none());
    }

    #[test]
    fn release_name_lookup() {
        let info = AssetInfo {
            release_info: Some(vec![
                make_release("r1", "MyApp", 10, vec![], vec![]),
                make_release("r2", "OtherApp", 5, vec![], vec![]),
            ]),
            ..Default::default()
        };
        assert_eq!(
            info.release_name("OtherApp").unwrap().id,
            Some("r2".to_string())
        );
        assert!(info.release_name("Missing").is_none());
    }

    #[test]
    fn compatible_apps_deduplicates() {
        let info = AssetInfo {
            release_info: Some(vec![
                make_release("r1", "a", 10, vec![], vec!["UE_5.0", "UE_4.27"]),
                make_release("r2", "b", 5, vec![], vec!["UE_5.0", "UE_5.1"]),
            ]),
            ..Default::default()
        };
        let apps = info.compatible_apps().unwrap();
        assert_eq!(apps, vec!["UE_4.27", "UE_5.0", "UE_5.1"]);
    }

    #[test]
    fn compatible_apps_none_when_no_releases() {
        let info = AssetInfo {
            release_info: None,
            ..Default::default()
        };
        assert!(info.compatible_apps().is_none());
    }

    #[test]
    fn platforms_aggregates() {
        let info = AssetInfo {
            release_info: Some(vec![
                make_release("r1", "a", 10, vec!["Windows"], vec![]),
                make_release("r2", "b", 5, vec!["Windows", "Mac"], vec![]),
            ]),
            ..Default::default()
        };
        let plats = info.platforms().unwrap();
        assert_eq!(plats, vec!["Mac", "Windows"]);
    }

    #[test]
    fn platforms_none_when_no_releases() {
        let info = AssetInfo {
            release_info: None,
            ..Default::default()
        };
        assert!(info.platforms().is_none());
    }
}
