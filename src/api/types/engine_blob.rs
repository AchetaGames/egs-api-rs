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
/// The `download_url` field contains a presigned S3 URL with limited expiry
/// (typically 1 hour). The `name` field contains the filename,
/// e.g. `Linux_Unreal_Engine_5.7.4.zip`.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineBlob {
    pub name: String,
    pub created_at: String,
    pub size: u64,
    pub download_url: String,
    /// Engine/plugin version string (may be empty).
    #[serde(default)]
    pub version: String,
    /// Semver version string (may be empty).
    #[serde(default)]
    pub semver: String,
    /// Target OS (may be empty, inferred from the endpoint platform path).
    #[serde(default)]
    pub operating_system: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_engine_blobs_new_format() {
        let json = r#"{"blobs":[{"name":"Linux_Unreal_Engine_5.7.4.zip","createdAt":"2026-03-10T12:46:39.745Z","size":32073680226,"downloadUrl":"https://ucs-blob-store.s3-accelerate.amazonaws.com/blobs/40/f8/test","version":"","semver":"","operatingSystem":""}]}"#;
        let resp: EngineBlobsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.blobs.len(), 1);
        assert_eq!(resp.blobs[0].name, "Linux_Unreal_Engine_5.7.4.zip");
        assert_eq!(resp.blobs[0].size, 32073680226);
        assert!(resp.blobs[0].download_url.starts_with("https://"));
    }

    #[test]
    fn deserialize_engine_blobs_missing_optional_fields() {
        let json = r#"{"blobs":[{"name":"Linux_Unreal_Engine_5.7.3.zip","createdAt":"2026-01-15T10:00:00Z","size":12345678,"downloadUrl":"https://example.com/blob"}]}"#;
        let resp: EngineBlobsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.blobs.len(), 1);
        assert_eq!(resp.blobs[0].version, "");
        assert_eq!(resp.blobs[0].semver, "");
        assert_eq!(resp.blobs[0].operating_system, "");
    }

    #[test]
    fn deserialize_empty_blobs() {
        let json = r#"{"blobs":[]}"#;
        let resp: EngineBlobsResponse = serde_json::from_str(json).unwrap();
        assert!(resp.blobs.is_empty());
    }
}
