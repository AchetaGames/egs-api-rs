use serde::{Deserialize, Serialize};

/// Response from `GET /api/blobs/{platform}` — engine version downloads.
///
/// Contains presigned download URLs for Unreal Engine builds,
/// Fab plugins, and Bridge plugins for the specified platform.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineBlobsResponse {
    pub blobs: Vec<EngineBlob>,
}

/// A single engine download blob.
///
/// The `url` field contains a presigned S3 URL with limited expiry.
/// The `name` field contains the filename, e.g. `Linux_Unreal_Engine_5.7.3.zip`.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineBlob {
    pub name: String,
    pub created_at: String,
    pub size: u64,
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_engine_blobs() {
        let json = r#"{"blobs":[{"name":"Linux_Unreal_Engine_5.7.3.zip","createdAt":"2026-01-15T10:00:00Z","size":12345678,"url":"https://example.com/blob"}]}"#;
        let resp: EngineBlobsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.blobs.len(), 1);
        assert_eq!(resp.blobs[0].name, "Linux_Unreal_Engine_5.7.3.zip");
        assert_eq!(resp.blobs[0].size, 12345678);
    }

    #[test]
    fn deserialize_empty_blobs() {
        let json = r#"{"blobs":[]}"#;
        let resp: EngineBlobsResponse = serde_json::from_str(json).unwrap();
        assert!(resp.blobs.is_empty());
    }
}
