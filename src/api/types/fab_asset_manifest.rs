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

/// Distribution Point
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributionPoint {
    /// Manifest URL
    pub manifest_url: String,
    /// Signature expiration 
    /// Format: 2024-11-03T22:04:16.295Z
    pub signature_expiration: String,
}

/// Metadata
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {}
