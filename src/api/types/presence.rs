use serde::{Deserialize, Serialize};

/// Response from a presence update (PATCH).
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresenceUpdate {
    pub status: Option<String>,
    pub activity: Option<PresenceActivity>,
}

/// Activity details within a presence payload.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresenceActivity {
    pub r#type: Option<String>,
    pub properties: Option<serde_json::Value>,
}
