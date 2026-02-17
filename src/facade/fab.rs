use crate::api::error::EpicAPIError;
use crate::api::types::download_manifest::DownloadManifest;
use crate::api::types::fab_asset_manifest::DownloadInfo;
use crate::api::types::fab_entitlement;
use crate::api::types::fab_search;
use crate::api::types::fab_taxonomy;
use crate::EpicGames;

impl EpicGames {
    /// Fetch Fab asset manifest with signed distribution points.
    ///
    /// Returns `Result` to expose timeout errors (403 → `EpicAPIError::FabTimeout`).
    pub async fn fab_asset_manifest(
        &self,
        artifact_id: &str,
        namespace: &str,
        asset_id: &str,
        platform: Option<&str>,
    ) -> Result<Vec<DownloadInfo>, EpicAPIError> {
        match self
            .egs
            .fab_asset_manifest(artifact_id, namespace, asset_id, platform)
            .await
        {
            Ok(a) => Ok(a),
            Err(e) => Err(e),
        }
    }

    /// Like [`fab_library_items`](Self::fab_library_items), but returns a `Result` instead of swallowing errors.
    pub async fn try_fab_library_items(
        &mut self,
        account_id: String,
    ) -> Result<crate::api::types::fab_library::FabLibrary, EpicAPIError> {
        self.egs.fab_library_items(account_id).await
    }

    /// Fetch the user Fab library.
    ///
    /// Paginates internally and returns all records at once. Returns `None` on API errors.
    pub async fn fab_library_items(
        &mut self,
        account_id: String,
    ) -> Option<crate::api::types::fab_library::FabLibrary> {
        self.try_fab_library_items(account_id).await.ok()
    }

    /// Parse a Fab download manifest from a specific distribution point.
    ///
    /// Checks signature expiration before fetching. Returns `Result` to expose timeout errors.
    pub async fn fab_download_manifest(
        &self,
        download_info: DownloadInfo,
        distribution_point_url: &str,
    ) -> Result<DownloadManifest, EpicAPIError> {
        self.egs
            .fab_download_manifest(download_info, distribution_point_url)
            .await
    }

    /// Like [`fab_file_download_info`](Self::fab_file_download_info), but returns a `Result` instead of swallowing errors.
    pub async fn try_fab_file_download_info(
        &self,
        listing_id: &str,
        format_id: &str,
        file_id: &str,
    ) -> Result<DownloadInfo, EpicAPIError> {
        self.egs
            .fab_file_download_info(listing_id, format_id, file_id)
            .await
    }

    /// Fetch download info for a specific file within a Fab listing.
    ///
    /// Returns signed [`DownloadInfo`] for a single file identified by
    /// `listing_id`, `format_id`, and `file_id`. Use this for targeted
    /// downloads of individual files from a Fab asset rather than fetching
    /// the entire asset manifest.
    ///
    /// Returns `None` on API errors.
    pub async fn fab_file_download_info(
        &self,
        listing_id: &str,
        format_id: &str,
        file_id: &str,
    ) -> Option<DownloadInfo> {
        self.try_fab_file_download_info(listing_id, format_id, file_id)
            .await
            .ok()
    }

    // ── Fab Search/Browse ──

    /// Search Fab listings. Returns `None` on error.
    pub async fn fab_search(
        &self,
        params: &fab_search::FabSearchParams,
    ) -> Option<fab_search::FabSearchResults> {
        self.egs.fab_search(params).await.ok()
    }

    /// Search Fab listings. Returns full `Result`.
    pub async fn try_fab_search(
        &self,
        params: &fab_search::FabSearchParams,
    ) -> Result<fab_search::FabSearchResults, EpicAPIError> {
        self.egs.fab_search(params).await
    }

    /// Get full listing detail. Returns `None` on error.
    pub async fn fab_listing(&self, uid: &str) -> Option<fab_search::FabListingDetail> {
        self.egs.fab_listing(uid).await.ok()
    }

    /// Get full listing detail. Returns full `Result`.
    pub async fn try_fab_listing(
        &self,
        uid: &str,
    ) -> Result<fab_search::FabListingDetail, EpicAPIError> {
        self.egs.fab_listing(uid).await
    }

    /// Get UE-specific format details for a listing. Returns `None` on error.
    pub async fn fab_listing_ue_formats(
        &self,
        uid: &str,
    ) -> Option<Vec<fab_search::FabListingUeFormat>> {
        self.egs.fab_listing_ue_formats(uid).await.ok()
    }

    /// Get UE-specific format details. Returns full `Result`.
    pub async fn try_fab_listing_ue_formats(
        &self,
        uid: &str,
    ) -> Result<Vec<fab_search::FabListingUeFormat>, EpicAPIError> {
        self.egs.fab_listing_ue_formats(uid).await
    }

    /// Get listing state (ownership, wishlist, review). Returns `None` on error.
    pub async fn fab_listing_state(
        &self,
        uid: &str,
    ) -> Option<fab_search::FabListingState> {
        self.egs.fab_listing_state(uid).await.ok()
    }

    /// Get listing state. Returns full `Result`.
    pub async fn try_fab_listing_state(
        &self,
        uid: &str,
    ) -> Result<fab_search::FabListingState, EpicAPIError> {
        self.egs.fab_listing_state(uid).await
    }

    /// Bulk check listing states. Returns `None` on error.
    pub async fn fab_listing_states_bulk(
        &self,
        listing_ids: &[&str],
    ) -> Option<Vec<fab_search::FabListingState>> {
        self.egs.fab_listing_states_bulk(listing_ids).await.ok()
    }

    /// Bulk check listing states. Returns full `Result`.
    pub async fn try_fab_listing_states_bulk(
        &self,
        listing_ids: &[&str],
    ) -> Result<Vec<fab_search::FabListingState>, EpicAPIError> {
        self.egs.fab_listing_states_bulk(listing_ids).await
    }

    /// Bulk fetch pricing for multiple offer IDs. Returns `None` on error.
    pub async fn fab_bulk_prices(
        &self,
        offer_ids: &[&str],
    ) -> Option<fab_search::FabBulkPricesResponse> {
        self.egs.fab_bulk_prices(offer_ids).await.ok()
    }

    /// Bulk fetch pricing. Returns full `Result`.
    pub async fn try_fab_bulk_prices(
        &self,
        offer_ids: &[&str],
    ) -> Result<fab_search::FabBulkPricesResponse, EpicAPIError> {
        self.egs.fab_bulk_prices(offer_ids).await
    }

    /// Get listing ownership info. Returns `None` on error.
    pub async fn fab_listing_ownership(
        &self,
        uid: &str,
    ) -> Option<fab_search::FabOwnership> {
        self.egs.fab_listing_ownership(uid).await.ok()
    }

    /// Get listing ownership info. Returns full `Result`.
    pub async fn try_fab_listing_ownership(
        &self,
        uid: &str,
    ) -> Result<fab_search::FabOwnership, EpicAPIError> {
        self.egs.fab_listing_ownership(uid).await
    }

    /// Get pricing for a specific listing. Returns `None` on error.
    pub async fn fab_listing_prices(
        &self,
        uid: &str,
    ) -> Option<Vec<fab_search::FabPriceInfo>> {
        self.egs.fab_listing_prices(uid).await.ok()
    }

    /// Get pricing for a specific listing. Returns full `Result`.
    pub async fn try_fab_listing_prices(
        &self,
        uid: &str,
    ) -> Result<Vec<fab_search::FabPriceInfo>, EpicAPIError> {
        self.egs.fab_listing_prices(uid).await
    }

    /// Get reviews for a listing. Returns `None` on error.
    pub async fn fab_listing_reviews(
        &self,
        uid: &str,
        sort_by: Option<&str>,
        cursor: Option<&str>,
    ) -> Option<fab_search::FabReviewsResponse> {
        self.egs.fab_listing_reviews(uid, sort_by, cursor).await.ok()
    }

    /// Get reviews for a listing. Returns full `Result`.
    pub async fn try_fab_listing_reviews(
        &self,
        uid: &str,
        sort_by: Option<&str>,
        cursor: Option<&str>,
    ) -> Result<fab_search::FabReviewsResponse, EpicAPIError> {
        self.egs.fab_listing_reviews(uid, sort_by, cursor).await
    }

    // ── Fab Taxonomy ──

    /// Fetch available license types. Returns `None` on error.
    pub async fn fab_licenses(&self) -> Option<Vec<fab_taxonomy::FabLicenseType>> {
        self.egs.fab_licenses().await.ok()
    }

    /// Fetch available license types. Returns full `Result`.
    pub async fn try_fab_licenses(
        &self,
    ) -> Result<Vec<fab_taxonomy::FabLicenseType>, EpicAPIError> {
        self.egs.fab_licenses().await
    }

    /// Fetch asset format groups. Returns `None` on error.
    pub async fn fab_format_groups(&self) -> Option<Vec<fab_taxonomy::FabFormatGroup>> {
        self.egs.fab_format_groups().await.ok()
    }

    /// Fetch asset format groups. Returns full `Result`.
    pub async fn try_fab_format_groups(
        &self,
    ) -> Result<Vec<fab_taxonomy::FabFormatGroup>, EpicAPIError> {
        self.egs.fab_format_groups().await
    }

    /// Fetch tag groups with nested tags. Returns `None` on error.
    pub async fn fab_tag_groups(&self) -> Option<Vec<fab_taxonomy::FabTagGroup>> {
        self.egs.fab_tag_groups().await.ok()
    }

    /// Fetch tag groups with nested tags. Returns full `Result`.
    pub async fn try_fab_tag_groups(
        &self,
    ) -> Result<Vec<fab_taxonomy::FabTagGroup>, EpicAPIError> {
        self.egs.fab_tag_groups().await
    }

    /// Fetch available UE versions. Returns `None` on error.
    pub async fn fab_ue_versions(&self) -> Option<Vec<String>> {
        self.egs.fab_ue_versions().await.ok()
    }

    /// Fetch available UE versions. Returns full `Result`.
    pub async fn try_fab_ue_versions(&self) -> Result<Vec<String>, EpicAPIError> {
        self.egs.fab_ue_versions().await
    }

    /// Fetch channel info by slug. Returns `None` on error.
    pub async fn fab_channel(&self, slug: &str) -> Option<fab_taxonomy::FabChannel> {
        self.egs.fab_channel(slug).await.ok()
    }

    /// Fetch channel info by slug. Returns full `Result`.
    pub async fn try_fab_channel(
        &self,
        slug: &str,
    ) -> Result<fab_taxonomy::FabChannel, EpicAPIError> {
        self.egs.fab_channel(slug).await
    }

    // ── Fab Library Entitlements ──

    /// Search library entitlements. Returns `None` on error.
    pub async fn fab_library_entitlements(
        &self,
        params: &fab_entitlement::FabEntitlementSearchParams,
    ) -> Option<fab_entitlement::FabEntitlementResults> {
        self.egs.fab_library_entitlements(params).await.ok()
    }

    /// Search library entitlements. Returns full `Result`.
    pub async fn try_fab_library_entitlements(
        &self,
        params: &fab_entitlement::FabEntitlementSearchParams,
    ) -> Result<fab_entitlement::FabEntitlementResults, EpicAPIError> {
        self.egs.fab_library_entitlements(params).await
    }

    // ── Fab Session ──

    /// Initialize Fab CSRF token. Sets cookies on the shared HTTP client.
    pub async fn fab_csrf(&self) -> Result<(), EpicAPIError> {
        self.egs.fab_csrf().await
    }

    /// Fetch Fab user context. Returns `None` on error.
    pub async fn fab_user_context(&self) -> Option<fab_search::FabUserContext> {
        self.egs.fab_user_context().await.ok()
    }

    /// Fetch Fab user context. Returns full `Result`.
    pub async fn try_fab_user_context(
        &self,
    ) -> Result<fab_search::FabUserContext, EpicAPIError> {
        self.egs.fab_user_context().await
    }

    /// Add a free listing to the user's library. Returns `Ok(())` on success.
    pub async fn fab_add_to_library(&self, listing_uid: &str) -> Result<(), EpicAPIError> {
        self.egs.fab_add_to_library(listing_uid).await
    }

    /// Fetch all available asset formats for a listing. Returns `None` on error.
    pub async fn fab_listing_formats(
        &self,
        listing_uid: &str,
    ) -> Option<Vec<fab_search::FabListingFormat>> {
        self.egs.fab_listing_formats(listing_uid).await.ok()
    }

    /// Fetch all available asset formats. Returns full `Result`.
    pub async fn try_fab_listing_formats(
        &self,
        listing_uid: &str,
    ) -> Result<Vec<fab_search::FabListingFormat>, EpicAPIError> {
        self.egs.fab_listing_formats(listing_uid).await
    }
}
