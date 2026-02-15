use serde::{Deserialize, Serialize};

/// Response from the artifact service ticket endpoint.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactServiceTicket {
    pub expires_in_seconds: Option<i64>,
    pub signed_ticket: Option<String>,
    pub expiration: Option<String>,
}
