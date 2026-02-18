use crate::EpicGames;
use crate::api::error::EpicAPIError;
use crate::api::types::artifact_service::ArtifactServiceTicket;
use crate::api::types::asset_info::AssetInfo;
use crate::api::types::asset_manifest::AssetManifest;
use crate::api::types::download_manifest::DownloadManifest;
use crate::api::types::engine_blob;
use crate::api::types::epic_asset::EpicAsset;

impl EpicGames {
    /// Like [`list_assets`](Self::list_assets), but returns a `Result` instead of swallowing errors.
    pub async fn try_list_assets(
        &mut self,
        platform: Option<String>,
        label: Option<String>,
    ) -> Result<Vec<EpicAsset>, EpicAPIError> {
        self.egs.assets(platform, label).await
    }

    /// List all owned assets.
    ///
    /// Defaults to platform="Windows" and label="Live" if not specified.
    /// Returns empty `Vec` on API errors.
    pub async fn list_assets(
        &mut self,
        platform: Option<String>,
        label: Option<String>,
    ) -> Vec<EpicAsset> {
        self.try_list_assets(platform, label)
            .await
            .unwrap_or_else(|_| Vec::new())
    }

    /// Like [`asset_manifest`](Self::asset_manifest), but returns a `Result` instead of swallowing errors.
    pub async fn try_asset_manifest(
        &mut self,
        platform: Option<String>,
        label: Option<String>,
        namespace: Option<String>,
        item_id: Option<String>,
        app: Option<String>,
    ) -> Result<AssetManifest, EpicAPIError> {
        self.egs
            .asset_manifest(platform, label, namespace, item_id, app)
            .await
    }

    /// Fetch asset manifest with CDN download URLs.
    ///
    /// Defaults to platform="Windows" and label="Live" if not specified.
    /// Returns `None` on API errors.
    pub async fn asset_manifest(
        &mut self,
        platform: Option<String>,
        label: Option<String>,
        namespace: Option<String>,
        item_id: Option<String>,
        app: Option<String>,
    ) -> Option<AssetManifest> {
        self.try_asset_manifest(platform, label, namespace, item_id, app)
            .await
            .ok()
    }

    /// Like [`asset_info`](Self::asset_info), but returns a `Result` instead of swallowing errors.
    pub async fn try_asset_info(
        &mut self,
        asset: &EpicAsset,
    ) -> Result<Option<AssetInfo>, EpicAPIError> {
        let mut info = self.egs.asset_info(asset).await?;
        log::debug!(
            "try_asset_info: catalog_item_id='{}', HashMap keys={:?}",
            asset.catalog_item_id,
            info.keys().collect::<Vec<_>>()
        );
        Ok(info.remove(asset.catalog_item_id.as_str()))
    }

    /// Fetch catalog metadata for an asset (includes DLC tree).
    ///
    /// Returns `None` on API errors.
    pub async fn asset_info(&mut self, asset: &EpicAsset) -> Option<AssetInfo> {
        self.try_asset_info(asset).await.ok().flatten()
    }

    /// Parse download manifests from all CDN mirrors.
    ///
    /// Fetches from all mirrors, parses binary/JSON format, and populates custom fields
    /// (BaseUrl, CatalogItemId, etc.). Returns empty `Vec` on API errors.
    pub async fn asset_download_manifests(&self, manifest: AssetManifest) -> Vec<DownloadManifest> {
        self.egs.asset_download_manifests(manifest).await
    }

    /// Fetch an artifact service ticket for manifest retrieval via EOS Helper.
    ///
    /// The `sandbox_id` is typically the game's namespace and `artifact_id`
    /// is the app name. Returns a signed ticket for use with
    /// [`game_manifest_by_ticket`](Self::game_manifest_by_ticket).
    pub async fn artifact_service_ticket(
        &self,
        sandbox_id: &str,
        artifact_id: &str,
        label: Option<&str>,
        platform: Option<&str>,
    ) -> Result<ArtifactServiceTicket, EpicAPIError> {
        self.egs
            .artifact_service_ticket(sandbox_id, artifact_id, label, platform)
            .await
    }

    /// Fetch a game manifest using a signed artifact service ticket.
    ///
    /// Alternative to [`asset_manifest`](Self::asset_manifest) using ticket-based
    /// auth from the EOS Helper service.
    pub async fn game_manifest_by_ticket(
        &self,
        artifact_id: &str,
        signed_ticket: &str,
        label: Option<&str>,
        platform: Option<&str>,
    ) -> Result<AssetManifest, EpicAPIError> {
        self.egs
            .game_manifest_by_ticket(artifact_id, signed_ticket, label, platform)
            .await
    }

    /// Fetch launcher manifests for self-update checks.
    pub async fn launcher_manifests(
        &self,
        platform: Option<&str>,
        label: Option<&str>,
    ) -> Result<AssetManifest, EpicAPIError> {
        self.egs.launcher_manifests(platform, label).await
    }

    /// Try to fetch a delta manifest for optimized patching between builds.
    ///
    /// Returns `None` if no delta is available or the builds are identical.
    pub async fn delta_manifest(
        &self,
        base_url: &str,
        old_build_id: &str,
        new_build_id: &str,
    ) -> Option<Vec<u8>> {
        self.egs
            .delta_manifest(base_url, old_build_id, new_build_id)
            .await
    }

    /// Fetch engine version download blobs for a platform. Returns `None` on error.
    pub async fn engine_versions(
        &self,
        platform: &str,
    ) -> Option<engine_blob::EngineBlobsResponse> {
        self.egs.engine_versions(platform).await.ok()
    }

    /// Fetch engine version download blobs. Returns full `Result`.
    pub async fn try_engine_versions(
        &self,
        platform: &str,
    ) -> Result<engine_blob::EngineBlobsResponse, EpicAPIError> {
        self.egs.engine_versions(platform).await
    }
}
