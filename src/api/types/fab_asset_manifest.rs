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
        for distribution_point in &self.distribution_points {
            if distribution_point.manifest_url.starts_with(base_url) {
                return Some(distribution_point);
            }
        }
        None
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
