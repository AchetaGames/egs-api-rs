use serde::{Deserialize, Serialize};

/// Status of an Epic online service (from lightswitch API).
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceStatus {
    pub service_instance_id: String,
    pub status: String,
    pub message: Option<String>,
    pub maintenance_uri: Option<String>,
    #[serde(default)]
    pub override_catalog_ids: Vec<String>,
    pub allowed_actions: Option<Vec<String>>,
    pub banned: Option<bool>,
    pub launcher_info_dto: Option<LauncherInfo>,
}

/// Launcher-specific info within a service status.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherInfo {
    pub app_name: Option<String>,
    pub catalog_item_id: Option<String>,
    pub namespace: Option<String>,
}
