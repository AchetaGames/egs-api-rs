use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DefaultOnNull;

/// Fab Library Response
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabLibrary {
    /// Pagination cursors
    pub cursors: Cursor,
    /// Library contents
    pub results: Vec<FabAsset>,
}

/// Pagination Cursors
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cursor {
    /// next page cursor
    pub next: Option<String>,
}

/// Library item
#[serde_as]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabAsset {
    /// Asset ID
    pub asset_id: String,
    /// Asset Namespace
    pub asset_namespace: String,
    /// Asset Categories
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub categories: Vec<Category>,
    /// Custom Attributes
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub custom_attributes: Vec<std::collections::HashMap<String, String>>,
    /// Asset description
    pub description: String,
    /// Distribution Method
    pub distribution_method: String,
    /// Images
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub images: Vec<Image>,
    /// Legacy Item ID
    pub legacy_item_id: Option<String>,
    /// Project Versions
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub project_versions: Vec<ProjectVersion>,
    /// Source of listing
    pub source: String,
    /// Title
    pub title: String,
    /// Listing URL
    pub url: String,
}

/// Asset Category
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    /// Category ID
    pub id: String,
    /// Category Name
    pub name: Option<String>,
}

/// Asset image
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    /// Height
    pub height: String,
    /// checksum
    pub md5: Option<String>,
    /// Type
    #[serde(rename = "type")]
    pub type_field: String,
    /// Uploaded
    pub uploaded_date: String,
    /// url
    pub url: String,
    /// Width
    pub width: String,
}

/// Project Version
#[serde_as]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectVersion {
    /// Artifact ID
    pub artifact_id: String,
    /// Build Versions
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub build_versions: Vec<BuildVersion>,
    /// Engine Versions
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub engine_versions: Vec<String>,
    /// Target Platform
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub target_platforms: Vec<String>,
}

/// Build Version
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildVersion {
    /// Build Version
    pub build_version: String,
    /// Platform
    pub platform: String,
}
