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
                        Some(mut man) => {
                            man.set_custom_field("SourceURL", distribution_point_url);
                            Ok(man)
                        }
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

    /// Search Fab listings. Public endpoint — no auth required.
    ///
    /// Use `FabSearchParams` to specify filters, sorting, and pagination.
    pub async fn fab_search(
        &self,
        params: &crate::api::types::fab_search::FabSearchParams,
    ) -> Result<crate::api::types::fab_search::FabSearchResults, EpicAPIError> {
        let mut url = "https://www.fab.com/i/listings/search?".to_string();
        let mut query_parts = Vec::new();

        if let Some(ref q) = params.q {
            query_parts.push(format!("q={}", q));
        }
        if let Some(ref channels) = params.channels {
            query_parts.push(format!("channels={}", channels));
        }
        if let Some(ref listing_types) = params.listing_types {
            query_parts.push(format!("listing_types={}", listing_types));
        }
        if let Some(ref categories) = params.categories {
            query_parts.push(format!("categories={}", categories));
        }
        if let Some(ref sort_by) = params.sort_by {
            query_parts.push(format!("sort_by={}", sort_by));
        }
        if let Some(count) = params.count {
            query_parts.push(format!("count={}", count));
        }
        if let Some(ref cursor) = params.cursor {
            query_parts.push(format!("cursor={}", cursor));
        }
        if let Some(ref aggregate_on) = params.aggregate_on {
            query_parts.push(format!("aggregate_on={}", aggregate_on));
        }
        if let Some(ref in_filter) = params.in_filter {
            query_parts.push(format!("in={}", in_filter));
        }
        if let Some(is_discounted) = params.is_discounted {
            if is_discounted {
                query_parts.push("is_discounted=true".to_string());
            }
        }

        url.push_str(&query_parts.join("&"));
        self.get_json(&url).await
    }

    /// Get full listing detail. Public endpoint — no auth required.
    pub async fn fab_listing(
        &self,
        uid: &str,
    ) -> Result<crate::api::types::fab_search::FabListingDetail, EpicAPIError> {
        let url = format!("https://www.fab.com/i/listings/{}", uid);
        self.get_json(&url).await
    }

    /// Get UE-specific format details for a listing. Public endpoint.
    pub async fn fab_listing_ue_formats(
        &self,
        uid: &str,
    ) -> Result<Vec<crate::api::types::fab_search::FabListingUeFormat>, EpicAPIError> {
        let url = format!(
            "https://www.fab.com/i/listings/{}/asset-formats/unreal-engine",
            uid
        );
        self.get_json(&url).await
    }

    /// Get user's listing state (ownership, wishlist, review). Requires Fab session.
    pub async fn fab_listing_state(
        &self,
        uid: &str,
    ) -> Result<crate::api::types::fab_search::FabListingState, EpicAPIError> {
        let url = format!("https://www.fab.com/i/users/me/listings-states/{}", uid);
        self.authorized_get_json(&url).await
    }

    /// Bulk check listing states for multiple IDs. Requires Fab session.
    pub async fn fab_listing_states_bulk(
        &self,
        listing_ids: &[&str],
    ) -> Result<Vec<crate::api::types::fab_search::FabListingState>, EpicAPIError> {
        let ids = listing_ids.join(",");
        let url = format!(
            "https://www.fab.com/i/users/me/listings-states?listing_ids={}",
            ids
        );
        self.authorized_get_json(&url).await
    }

    /// Bulk fetch pricing for multiple offer IDs. Public endpoint.
    pub async fn fab_bulk_prices(
        &self,
        offer_ids: &[&str],
    ) -> Result<Vec<crate::api::types::fab_search::FabPriceInfo>, EpicAPIError> {
        let ids = offer_ids
            .iter()
            .map(|id| format!("offer_ids={}", id))
            .collect::<Vec<_>>()
            .join("&");
        let url = format!("https://www.fab.com/i/listings/prices-infos?{}", ids);
        self.get_json(&url).await
    }

    /// Get listing ownership info. Requires Fab session.
    pub async fn fab_listing_ownership(
        &self,
        uid: &str,
    ) -> Result<crate::api::types::fab_search::FabOwnership, EpicAPIError> {
        let url = format!("https://www.fab.com/i/listings/{}/ownership", uid);
        self.authorized_get_json(&url).await
    }

    /// Get pricing for a specific listing. Public endpoint.
    pub async fn fab_listing_prices(
        &self,
        uid: &str,
    ) -> Result<Vec<crate::api::types::fab_search::FabPriceInfo>, EpicAPIError> {
        let url = format!("https://www.fab.com/i/listings/{}/prices-infos", uid);
        self.get_json(&url).await
    }

    /// Get reviews for a listing. Public endpoint.
    pub async fn fab_listing_reviews(
        &self,
        uid: &str,
        sort_by: Option<&str>,
        cursor: Option<&str>,
    ) -> Result<crate::api::types::fab_search::FabReviewsResponse, EpicAPIError> {
        let mut query_parts = Vec::new();
        if let Some(sort) = sort_by {
            query_parts.push(format!("sort_by={}", sort));
        }
        if let Some(c) = cursor {
            query_parts.push(format!("cursor={}", c));
        }
        let url = if query_parts.is_empty() {
            format!("https://www.fab.com/i/store/listings/{}/reviews", uid)
        } else {
            format!(
                "https://www.fab.com/i/store/listings/{}/reviews?{}",
                uid,
                query_parts.join("&")
            )
        };
        self.get_json(&url).await
    }

    /// Fetch available license types. Public endpoint.
    pub async fn fab_licenses(
        &self,
    ) -> Result<Vec<crate::api::types::fab_taxonomy::FabLicenseType>, EpicAPIError> {
        self.get_json("https://www.fab.com/i/taxonomy/licenses").await
    }

    /// Fetch asset format groups. Public endpoint.
    pub async fn fab_format_groups(
        &self,
    ) -> Result<Vec<crate::api::types::fab_taxonomy::FabFormatGroup>, EpicAPIError> {
        self.get_json("https://www.fab.com/i/taxonomy/asset-format-groups").await
    }

    /// Fetch tag groups with nested tags. Public endpoint.
    pub async fn fab_tag_groups(
        &self,
    ) -> Result<Vec<crate::api::types::fab_taxonomy::FabTagGroup>, EpicAPIError> {
        self.get_json("https://www.fab.com/i/tags/groups").await
    }

    /// Fetch available UE versions. Public endpoint.
    pub async fn fab_ue_versions(
        &self,
    ) -> Result<Vec<String>, EpicAPIError> {
        self.get_json("https://www.fab.com/i/unreal-engine/versions").await
    }

    /// Fetch channel info by slug. Public endpoint.
    pub async fn fab_channel(
        &self,
        slug: &str,
    ) -> Result<crate::api::types::fab_taxonomy::FabChannel, EpicAPIError> {
        let url = format!("https://www.fab.com/i/channels/{}", slug);
        self.get_json(&url).await
    }
}
