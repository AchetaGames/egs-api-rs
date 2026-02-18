use serde::{Deserialize, Serialize};
/// Fab Asset Manifest
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabAssetManifest {
    /// Download info
    pub download_info: Vec<DownloadInfo>,
}

/// Download info
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadInfo {
    /// Artifact ID
    pub artifact_id: String,
    /// Asset format
    pub asset_format: String,
    /// Build Version
    pub build_version: String,
    /// Distribution Points Base URLs
    pub distribution_point_base_urls: Vec<String>,
    /// Distribution Points
    pub distribution_points: Vec<DistributionPoint>,
    /// Manifest Hash
    pub manifest_hash: String,
    /// Metadata
    pub metadata: Metadata,
    /// Type
    #[serde(rename = "type")]
    pub type_field: String,
}

impl DownloadInfo {
    /// Get Distribution Point by base url
    pub fn get_distribution_point_by_base_url(&self, base_url: &str) -> Option<&DistributionPoint> {
        self.distribution_points
            .iter()
            .find(|&distribution_point| distribution_point.manifest_url.starts_with(base_url))
    }
}

/// Distribution Point
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributionPoint {
    /// Manifest URL
    pub manifest_url: String,
    /// Signature expiration
    #[serde(with = "time::serde::rfc3339")]
    pub signature_expiration: time::OffsetDateTime,
}

/// Metadata
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_download_info(urls: Vec<&str>) -> DownloadInfo {
        DownloadInfo {
            artifact_id: "test".to_string(),
            asset_format: "".to_string(),
            build_version: "".to_string(),
            distribution_point_base_urls: vec![],
            distribution_points: urls
                .into_iter()
                .map(|url| DistributionPoint {
                    manifest_url: url.to_string(),
                    signature_expiration: time::OffsetDateTime::now_utc(),
                })
                .collect(),
            manifest_hash: "".to_string(),
            metadata: Metadata {},
            type_field: "".to_string(),
        }
    }

    #[test]
    fn get_distribution_point_by_base_url_found() {
        let di = make_download_info(vec![
            "https://cdn1.example.com/manifest.json",
            "https://cdn2.example.com/manifest.json",
        ]);
        let point = di
            .get_distribution_point_by_base_url("https://cdn1")
            .unwrap();
        assert!(point.manifest_url.starts_with("https://cdn1"));
    }

    #[test]
    fn get_distribution_point_by_base_url_not_found() {
        let di = make_download_info(vec!["https://cdn1.example.com/manifest.json"]);
        assert!(
            di.get_distribution_point_by_base_url("https://cdn3")
                .is_none()
        );
    }

    #[test]
    fn get_distribution_point_by_base_url_empty() {
        let di = make_download_info(vec![]);
        assert!(
            di.get_distribution_point_by_base_url("https://anything")
                .is_none()
        );
    }
}
