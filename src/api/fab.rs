use crate::api::error::EpicAPIError;
use crate::api::types::download_manifest::DownloadManifest;
use crate::api::types::fab_asset_manifest::DownloadInfo;
use crate::api::types::fab_library::FabLibrary;
use crate::api::EpicAPI;
use log::{debug, error, warn};
use std::borrow::BorrowMut;
use std::str::FromStr;
use url::Url;

impl EpicAPI {
    pub async fn fab_asset_manifest(
        &self,
        artifact_id: &str,
        namespace: &str,
        asset_id: &str,
        platform: Option<&str>,
    ) -> Result<Vec<DownloadInfo>, EpicAPIError> {
        let url = format!("https://www.fab.com/e/artifacts/{}/manifest", artifact_id);
        match self
            .authorized_post_client(Url::parse(&url).unwrap())
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
                    let text = response.text().await.unwrap();
                    match serde_json::from_str::<
                        crate::api::types::fab_asset_manifest::FabAssetManifest,
                    >(&text)
                    {
                        Ok(manifest) => Ok(manifest.download_info),
                        Err(e) => {
                            error!("{:?}", e);
                            debug!("{}", text);
                            Err(EpicAPIError::Unknown)
                        }
                    }
                } else if response.status() == reqwest::StatusCode::FORBIDDEN {
                    Err(EpicAPIError::FabTimeout)
                } else {
                    debug!("{:?}", response.headers());
                    warn!(
                        "{} result: {}",
                        response.status(),
                        response.text().await.unwrap()
                    );
                    Err(EpicAPIError::Unknown)
                }
            }
            Err(e) => {
                error!("{:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn fab_download_manifest(
        &self,
        download_info: DownloadInfo,
        distribution_point_url: &str,
    ) -> Result<DownloadManifest, EpicAPIError> {
        match download_info.get_distribution_point_by_base_url(distribution_point_url) {
            None => {
                error!("Distribution point not found");
                Err(EpicAPIError::Unknown)
            }
            Some(point) => {
                if point.signature_expiration < time::OffsetDateTime::now_utc() {
                    error!("Expired signature");
                    Err(EpicAPIError::Unknown)
                } else {
                    let client = EpicAPI::build_client().build().unwrap();
                    match client
                        .get(Url::from_str(&point.manifest_url).unwrap())
                        .send()
                        .await
                    {
                        Ok(response) => {
                            if response.status() == reqwest::StatusCode::OK {
                                match response.bytes().await {
                                    Ok(data) => match DownloadManifest::parse(data.to_vec()) {
                                        None => {
                                            error!("Unable to parse the Download Manifest");
                                            Err(EpicAPIError::Unknown)
                                        }
                                        Some(man) => Ok(man),
                                    },
                                    Err(_) => Err(EpicAPIError::Unknown),
                                }
                            } else {
                                warn!(
                                    "{} result: {}",
                                    response.status(),
                                    response.text().await.unwrap()
                                );
                                Err(EpicAPIError::Unknown)
                            }
                        }
                        Err(_) => Err(EpicAPIError::Unknown),
                    }
                }
            }
        }
    }

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

            match self
                .authorized_get_client(Url::parse(&url).unwrap())
                .send()
                .await
            {
                Ok(response) => {
                    if response.status() == reqwest::StatusCode::OK {
                        let text = response.text().await.unwrap();
                        match serde_json::from_str::<FabLibrary>(&text) {
                            Ok(mut api_library) => {
                                library.cursors.next = api_library.cursors.next;
                                library.results.append(api_library.results.borrow_mut());
                            }
                            Err(e) => {
                                error!("{:?}", e);
                                debug!("{}", text);
                                library.cursors.next = None;
                            }
                        }
                    } else {
                        debug!("{:?}", response.headers());
                        warn!(
                            "{} result: {}",
                            response.status(),
                            response.text().await.unwrap()
                        );
                    }
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
}
