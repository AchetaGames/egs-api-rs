use crate::api::error::EpicAPIError;
use crate::api::types::asset_info::{AssetInfo, GameToken, OwnershipToken};
use crate::api::types::asset_manifest::AssetManifest;
use crate::api::types::catalog_item::CatalogItemPage;
use crate::api::types::catalog_offer::CatalogOfferPage;
use crate::api::types::currency::CurrencyPage;
use crate::api::types::download_manifest::DownloadManifest;
use crate::api::types::epic_asset::EpicAsset;
use crate::api::types::library::Library;
use crate::api::EpicAPI;
use log::{debug, error};
use std::borrow::BorrowMut;
use std::collections::HashMap;

impl EpicAPI {
    /// Fetch all owned assets for the given platform and label.
    pub async fn assets(
        &mut self,
        platform: Option<String>,
        label: Option<String>,
    ) -> Result<Vec<EpicAsset>, EpicAPIError> {
        let plat = platform.unwrap_or_else(|| "Windows".to_string());
        let lab = label.unwrap_or_else(|| "Live".to_string());
        let url = format!("https://launcher-public-service-prod06.ol.epicgames.com/launcher/api/public/assets/{}?label={}", plat, lab);
        self.authorized_get_json(&url).await
    }

    /// Fetch the asset manifest with CDN download URLs.
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
        let mut manifest: AssetManifest = self.authorized_get_json(&url).await?;
        manifest.platform = platform;
        manifest.label = label;
        manifest.namespace = namespace;
        manifest.item_id = item_id;
        manifest.app = app;
        Ok(manifest)
    }

    /// Download and parse manifests from all CDN mirrors in the asset manifest.
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
                match self.get_bytes(&url).await {
                    Ok(data) => match DownloadManifest::parse(data) {
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
            }
        }
        result
    }

    /// Fetch catalog metadata for an asset, including DLC details.
    pub async fn asset_info(
        &self,
        asset: EpicAsset,
    ) -> Result<HashMap<String, AssetInfo>, EpicAPIError> {
        let url = format!("https://catalog-public-service-prod06.ol.epicgames.com/catalog/api/shared/namespace/{}/bulk/items?id={}&includeDLCDetails=true&includeMainGameDetails=true&country=us&locale=lc",
                          asset.namespace, asset.catalog_item_id);
        self.authorized_get_json(&url).await
    }

    /// Fetch a short-lived game exchange token.
    pub async fn game_token(&self) -> Result<GameToken, EpicAPIError> {
        self.authorized_get_json(
            "https://account-public-service-prod03.ol.epicgames.com/account/api/oauth/exchange",
        )
        .await
    }

    /// Fetch a JWT ownership token for the given asset.
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
        self.authorized_post_form_json(
            &url,
            &[(
                "nsCatalogItemId".to_string(),
                format!("{}:{}", asset.namespace, asset.catalog_item_id),
            )],
        )
        .await
    }

    /// Fetch all library items, paginating internally.
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

            match self.authorized_get_json::<Library>(&url).await {
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
                    break;
                }
            };
        }
        Ok(library)
    }

    /// Fetch paginated catalog items for a namespace.
    pub async fn catalog_items(
        &self,
        namespace: &str,
        start: i64,
        count: i64,
    ) -> Result<CatalogItemPage, EpicAPIError> {
        let url = format!(
            "https://catalog-public-service-prod06.ol.epicgames.com/catalog/api/shared/namespace/{}/items?start={}&count={}",
            namespace, start, count
        );
        self.authorized_get_json(&url).await
    }

    /// Fetch paginated catalog offers for a namespace.
    pub async fn catalog_offers(
        &self,
        namespace: &str,
        start: i64,
        count: i64,
    ) -> Result<CatalogOfferPage, EpicAPIError> {
        let url = format!(
            "https://catalog-public-service-prod06.ol.epicgames.com/catalog/api/shared/namespace/{}/offers?start={}&count={}",
            namespace, start, count
        );
        self.authorized_get_json(&url).await
    }

    /// Bulk fetch catalog items across multiple namespaces.
    pub async fn bulk_catalog_items(
        &self,
        items: &[(&str, &str)],
    ) -> Result<HashMap<String, HashMap<String, AssetInfo>>, EpicAPIError> {
        let body: Vec<serde_json::Value> = items
            .iter()
            .map(|(ns, id)| {
                serde_json::json!({
                    "id": id,
                    "namespace": ns,
                    "includeDLCDetails": true,
                    "includeMainGameDetails": true,
                    "country": "us",
                    "locale": "lc",
                })
            })
            .collect();
        self.authorized_post_json(
            "https://catalog-public-service-prod06.ol.epicgames.com/catalog/api/shared/bulk/namespaces/items",
            &body,
        )
        .await
    }

    /// Fetch available currencies.
    pub async fn currencies(
        &self,
        start: i64,
        count: i64,
    ) -> Result<CurrencyPage, EpicAPIError> {
        let url = format!(
            "https://catalog-public-service-prod06.ol.epicgames.com/catalog/api/shared/currencies?start={}&count={}",
            start, count
        );
        self.authorized_get_json(&url).await
    }

    /// Check the status of a library state token.
    pub async fn library_state_token_status(
        &self,
        token_id: &str,
    ) -> Result<bool, EpicAPIError> {
        let url = format!(
            "https://library-service.live.use1a.on.epicgames.com/library/api/public/stateToken/{}/status",
            token_id
        );
        self.authorized_get_json(&url).await
    }
}
