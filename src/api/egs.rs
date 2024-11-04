use crate::api::error::EpicAPIError;
use crate::api::types::asset_info::{AssetInfo, GameToken, OwnershipToken};
use crate::api::types::asset_manifest::AssetManifest;
use crate::api::types::download_manifest::DownloadManifest;
use crate::api::types::epic_asset::EpicAsset;
use crate::api::types::library::Library;
use crate::api::EpicAPI;
use log::{debug, error, warn};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::str::FromStr;
use url::Url;

impl EpicAPI {
    pub async fn assets(
        &mut self,
        platform: Option<String>,
        label: Option<String>,
    ) -> Result<Vec<EpicAsset>, EpicAPIError> {
        let plat = platform.unwrap_or_else(|| "Windows".to_string());
        let lab = label.unwrap_or_else(|| "Live".to_string());
        let url = format!("https://launcher-public-service-prod06.ol.epicgames.com/launcher/api/public/assets/{}?label={}", plat, lab);
        match self
            .authorized_get_client(Url::parse(&url).unwrap())
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(assets) => Ok(assets),
                        Err(e) => {
                            error!("{:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
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
            Err(e) => {
                error!("{:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn asset_manifest(
        &self,
        platform: Option<String>,
        label: Option<String>,
        namespace: Option<String>,
        item_id: Option<String>,
        app: Option<String>,
    ) -> Result<AssetManifest, EpicAPIError> {
        if namespace.is_none() {
            return Err(EpicAPIError::InvalidParams);
        };
        if item_id.is_none() {
            return Err(EpicAPIError::InvalidParams);
        };
        if app.is_none() {
            return Err(EpicAPIError::InvalidParams);
        };
        let url = format!("https://launcher-public-service-prod06.ol.epicgames.com/launcher/api/public/assets/v2/platform/{}/namespace/{}/catalogItem/{}/app/{}/label/{}",
                          platform.clone().unwrap_or_else(|| "Windows".to_string()), namespace.clone().unwrap(), item_id.clone().unwrap(), app.clone().unwrap(), label.clone().unwrap_or_else(|| "Live".to_string()));
        match self
            .authorized_get_client(Url::parse(&url).unwrap())
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json::<AssetManifest>().await {
                        Ok(mut manifest) => {
                            manifest.platform = platform;
                            manifest.label = label;
                            manifest.namespace = namespace;
                            manifest.item_id = item_id;
                            manifest.app = app;
                            Ok(manifest)
                        }
                        Err(e) => {
                            error!("{:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
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
            Err(e) => {
                error!("{:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn asset_download_manifests(
        &self,
        asset_manifest: AssetManifest,
    ) -> Vec<DownloadManifest> {
        let base_urls = asset_manifest.url_csv();
        let mut result: Vec<DownloadManifest> = Vec::new();
        for elem in asset_manifest.elements {
            for manifest in elem.manifests {
                let mut queries: Vec<String> = Vec::new();
                debug!("{:?}", manifest);
                for query in manifest.query_params {
                    queries.push(format!("{}={}", query.name, query.value));
                }
                let url = format!("{}?{}", manifest.uri, queries.join("&"));
                let client = EpicAPI::build_client().build().unwrap();
                match client.get(Url::from_str(&url).unwrap()).send().await {
                    Ok(response) => {
                        if response.status() == reqwest::StatusCode::OK {
                            match response.bytes().await {
                                Ok(data) => match DownloadManifest::parse(data.to_vec()) {
                                    None => {
                                        error!("Unable to parse the Download Manifest");
                                    }
                                    Some(mut man) => {
                                        let mut url = manifest.uri.clone();
                                        url.set_path(&match url.path_segments() {
                                            None => "".to_string(),
                                            Some(segments) => {
                                                let mut vec: Vec<&str> = segments.collect();
                                                vec.remove(vec.len() - 1);
                                                vec.join("/")
                                            }
                                        });
                                        url.set_query(None);
                                        url.set_fragment(None);
                                        man.set_custom_field(
                                            "BaseUrl".to_string(),
                                            base_urls.clone(),
                                        );

                                        if let Some(id) = asset_manifest.item_id.clone() {
                                            man.set_custom_field(
                                                "CatalogItemId".to_string(),
                                                id.clone(),
                                            );
                                        }
                                        if let Some(label) = asset_manifest.label.clone() {
                                            man.set_custom_field(
                                                "BuildLabel".to_string(),
                                                label.clone(),
                                            );
                                        }
                                        if let Some(ns) = asset_manifest.namespace.clone() {
                                            man.set_custom_field(
                                                "CatalogNamespace".to_string(),
                                                ns.clone(),
                                            );
                                        }

                                        if let Some(app) = asset_manifest.app.clone() {
                                            man.set_custom_field(
                                                "CatalogAssetName".to_string(),
                                                app.clone(),
                                            );
                                        }

                                        man.set_custom_field(
                                            "SourceURL".to_string(),
                                            url.to_string(),
                                        );
                                        result.push(man)
                                    }
                                },
                                Err(e) => {
                                    error!("{:?}", e);
                                }
                            }
                        } else {
                            warn!(
                                "{} result: {}",
                                response.status(),
                                response.text().await.unwrap()
                            );
                        }
                    }
                    Err(e) => {
                        error!("{:?}", e);
                    }
                }
            }
        }
        result
    }

    pub async fn asset_info(
        &self,
        asset: EpicAsset,
    ) -> Result<HashMap<String, AssetInfo>, EpicAPIError> {
        let url = format!("https://catalog-public-service-prod06.ol.epicgames.com/catalog/api/shared/namespace/{}/bulk/items?id={}&includeDLCDetails=true&includeMainGameDetails=true&country=us&locale=lc",
                          asset.namespace, asset.catalog_item_id);
        match self
            .authorized_get_client(Url::parse(&url).unwrap())
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(info) => Ok(info),
                        Err(e) => {
                            error!("{:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
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
            Err(e) => {
                error!("{:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn game_token(&self) -> Result<GameToken, EpicAPIError> {
        let url =
            "https://account-public-service-prod03.ol.epicgames.com/account/api/oauth/exchange"
                .to_string();
        match self
            .authorized_get_client(Url::parse(&url).unwrap())
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(token) => Ok(token),
                        Err(e) => {
                            error!("{:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
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
            Err(e) => {
                error!("{:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn ownership_token(&self, asset: EpicAsset) -> Result<OwnershipToken, EpicAPIError> {
        let url = match &self.user_data.account_id {
            None => {
                return Err(EpicAPIError::InvalidCredentials);
            }
            Some(id) => {
                format!("https://ecommerceintegration-public-service-ecomprod02.ol.epicgames.com/ecommerceintegration/api/public/platforms/EPIC/identities/{}/ownershipToken",
                        id)
            }
        };
        match self
            .authorized_post_client(Url::parse(&url).unwrap())
            .form(&[(
                "nsCatalogItemId".to_string(),
                format!("{}:{}", asset.namespace, asset.catalog_item_id),
            )])
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(token) => Ok(token),
                        Err(e) => {
                            error!("{:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
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
            Err(e) => {
                error!("{:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn library_items(&mut self, include_metadata: bool) -> Result<Library, EpicAPIError> {
        let mut library = Library {
            records: vec![],
            response_metadata: Default::default(),
        };
        let mut cursor: Option<String> = None;
        loop {
            let url = match &cursor {
                None => {
                    format!("https://library-service.live.use1a.on.epicgames.com/library/api/public/items?includeMetadata={}", include_metadata)
                }
                Some(c) => {
                    format!("https://library-service.live.use1a.on.epicgames.com/library/api/public/items?includeMetadata={}&cursor={}", include_metadata, c)
                }
            };

            match self
                .authorized_get_client(Url::parse(&url).unwrap())
                .send()
                .await
            {
                Ok(response) => {
                    if response.status() == reqwest::StatusCode::OK {
                        match response.json::<Library>().await {
                            Ok(mut records) => {
                                library.records.append(records.records.borrow_mut());
                                match records.response_metadata {
                                    None => {
                                        break;
                                    }
                                    Some(meta) => match meta.next_cursor {
                                        None => {
                                            break;
                                        }
                                        Some(curs) => {
                                            cursor = Some(curs);
                                        }
                                    },
                                }
                            }
                            Err(e) => {
                                error!("{:?}", e);
                            }
                        }
                    } else {
                        warn!(
                            "{} result: {}",
                            response.status(),
                            response.text().await.unwrap()
                        );
                    }
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            };
            if cursor.is_none() {
                break;
            }
        }
        Ok(library)
    }
}
