use crate::api::error::EpicAPIError;
use crate::api::types::download_manifest::DownloadManifest;
use crate::api::types::fab_asset_manifest::DownloadInfo;
use crate::api::types::fab_library::FabLibrary;
use crate::api::EpicAPI;
use log::{debug, error, warn};
use std::borrow::BorrowMut;
use url::Url;

impl EpicAPI {
    /// Fetch Fab asset manifest with signed distribution points. Returns `FabTimeout` on 403.
    pub async fn fab_asset_manifest(
        &self,
        artifact_id: &str,
        namespace: &str,
        asset_id: &str,
        platform: Option<&str>,
    ) -> Result<Vec<DownloadInfo>, EpicAPIError> {
        let url = format!("https://www.fab.com/e/artifacts/{}/manifest", artifact_id);
        let parsed_url = Url::parse(&url).map_err(|_| EpicAPIError::InvalidParams)?;
        match self
            .authorized_post_client(parsed_url)
            .json(&serde_json::json!({
                "item_id": asset_id,
                "namespace": namespace,
                "platform": platform.unwrap_or("Windows"),
            }))
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    let text = response.text().await.unwrap_or_default();
                    match serde_json::from_str::<
                        crate::api::types::fab_asset_manifest::FabAssetManifest,
                    >(&text)
                    {
                        Ok(manifest) => Ok(manifest.download_info),
                        Err(e) => {
                            error!("{:?}", e);
                            debug!("{}", text);
                            Err(EpicAPIError::DeserializationError(format!("{}", e)))
                        }
                    }
                } else if response.status() == reqwest::StatusCode::FORBIDDEN {
                    Err(EpicAPIError::FabTimeout)
                } else {
                    debug!("{:?}", response.headers());
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    warn!("{} result: {}", status, body);
                    Err(EpicAPIError::HttpError { status, body })
                }
            }
            Err(e) => {
                error!("{:?}", e);
                Err(EpicAPIError::NetworkError(e))
            }
        }
    }

    /// Download and parse a Fab manifest from a distribution point.
    pub async fn fab_download_manifest(
        &self,
        download_info: DownloadInfo,
        distribution_point_url: &str,
    ) -> Result<DownloadManifest, EpicAPIError> {
        match download_info.get_distribution_point_by_base_url(distribution_point_url) {
            None => {
                error!("Distribution point not found");
                Err(EpicAPIError::InvalidParams)
            }
            Some(point) => {
                if point.signature_expiration < time::OffsetDateTime::now_utc() {
                    error!("Expired signature");
                    Err(EpicAPIError::InvalidParams)
                } else {
                    let data = self.get_bytes(&point.manifest_url).await?;
                    match DownloadManifest::parse(data) {
                        None => {
                            error!("Unable to parse the Download Manifest");
                            Err(EpicAPIError::DeserializationError(
                                "Unable to parse the Download Manifest".to_string(),
                            ))
                        }
                        Some(man) => Ok(man),
                    }
                }
            }
        }
    }

    /// Fetch all Fab library items, paginating internally.
    pub async fn fab_library_items(
        &mut self,
        account_id: String,
    ) -> Result<FabLibrary, EpicAPIError> {
        let mut library = FabLibrary::default();

        loop {
            let url = match &library.cursors.next {
                None => {
                    format!(
                        "https://www.fab.com/e/accounts/{}/ue/library?count=100",
                        account_id
                    )
                }
                Some(c) => {
                    format!(
                        "https://www.fab.com/e/accounts/{}/ue/library?cursor={}&count=100",
                        account_id, c
                    )
                }
            };

            match self.authorized_get_json::<FabLibrary>(&url).await {
                Ok(mut api_library) => {
                    library.cursors.next = api_library.cursors.next;
                    library.results.append(api_library.results.borrow_mut());
                }
                Err(e) => {
                    error!("{:?}", e);
                    library.cursors.next = None;
                }
            }
            if library.cursors.next.is_none() {
                break;
            }
        }

        Ok(library)
    }

    /// Fetch download info for a specific file within a Fab listing.
    pub async fn fab_file_download_info(
        &self,
        listing_id: &str,
        format_id: &str,
        file_id: &str,
    ) -> Result<DownloadInfo, EpicAPIError> {
        let url = format!(
            "https://www.fab.com/p/egl/listings/{}/asset-formats/{}/files/{}/download-info",
            listing_id, format_id, file_id
        );
        self.authorized_get_json(&url).await
    }
}
