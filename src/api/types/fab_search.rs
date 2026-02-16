use serde::{Deserialize, Serialize};

/// Paginated search results from `GET /i/listings/search`.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FabSearchResults {
    pub results: Vec<FabSearchListing>,
    pub count: Option<u64>,
    pub cursors: Option<FabSearchCursors>,
    pub aggregations: Option<serde_json::Value>,
}

/// Pagination cursors for Fab search results.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FabSearchCursors {
    pub next: Option<String>,
    pub previous: Option<String>,
}

/// A single listing in search results.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabSearchListing {
    pub uid: String,
    pub title: Option<String>,
    pub listing_type: Option<String>,
    pub user: Option<FabUser>,
    pub category: Option<FabListingCategory>,
    pub is_mature: Option<bool>,
    pub is_free: Option<bool>,
    pub is_discounted: Option<bool>,
    pub published_at: Option<String>,
    pub ratings: Option<serde_json::Value>,
    pub starting_price: Option<serde_json::Value>,
    pub tags: Option<Vec<FabTag>>,
    pub thumbnails: Option<Vec<FabThumbnail>>,
}

/// Seller/user info within a listing.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabUser {
    pub uid: Option<String>,
    pub seller_name: Option<String>,
    pub seller_id: Option<String>,
    pub profile_url: Option<String>,
    pub profile_image_url: Option<String>,
    pub is_seller: Option<bool>,
}

/// Tag on a listing.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FabTag {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub uid: Option<String>,
}

/// Thumbnail entry in a listing.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabThumbnail {
    pub uid: Option<String>,
    pub media_url: Option<String>,
    #[serde(rename = "type")]
    pub thumbnail_type: Option<String>,
}

/// Category in a listing (singular object, not array).
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FabListingCategory {
    pub uid: Option<String>,
    pub name: Option<String>,
    pub path: Option<String>,
    pub slug: Option<String>,
}

/// Full listing detail from `GET /i/listings/{uid}`.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabListingDetail {
    pub uid: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub listing_type: Option<String>,
    pub user: Option<FabUser>,
    pub category: Option<FabListingCategory>,
    pub ratings: Option<serde_json::Value>,
    pub is_mature: Option<bool>,
    pub is_free: Option<bool>,
    pub created_at: Option<String>,
    pub published_at: Option<String>,
    pub starting_price: Option<serde_json::Value>,
    pub tags: Option<Vec<FabTag>>,
    pub thumbnails: Option<Vec<FabThumbnail>>,
    pub review_count: Option<u64>,
}

/// UE-specific asset format info from `GET /i/listings/{uid}/asset-formats/unreal-engine`.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabListingUeFormat {
    pub asset_format_type: Option<FabAssetFormatType>,
    pub technical_specs: Option<FabTechnicalSpecs>,
}

/// Asset format type descriptor.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FabAssetFormatType {
    pub code: Option<String>,
    pub icon: Option<String>,
    pub name: Option<String>,
}

/// Technical specifications for a UE asset.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabTechnicalSpecs {
    pub technical_details: Option<String>,
    pub unreal_engine_engine_versions: Option<Vec<String>>,
    pub unreal_engine_target_platforms: Option<Vec<String>>,
    pub unreal_engine_distribution_method: Option<String>,
}

/// Bulk pricing response from `GET /i/listings/prices-infos`.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabPriceInfo {
    pub currency_code: Option<String>,
    pub price: Option<f64>,
    pub discount: Option<f64>,
    pub vat: Option<f64>,
}

/// Ownership info from `GET /i/listings/{uid}/ownership`.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FabOwnership {
    pub licenses: Option<Vec<FabLicense>>,
}

/// License type info.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FabLicense {
    pub name: Option<String>,
    pub slug: Option<String>,
}

/// User listing state from `GET /i/users/me/listings-states/{uid}`.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabListingState {
    pub acquired: Option<bool>,
    pub entitlement_id: Option<String>,
    pub wishlisted: Option<bool>,
    pub ownership: Option<serde_json::Value>,
    pub review: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_search_results_with_listings() {
        let json = r#"{
            "results": [
                {
                    "uid": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                    "title": "Medieval Village Pack",
                    "listingType": "3d-model",
                    "user": {"sellerName": "Epic Marketplace", "uid": "seller-001", "isSeller": true},
                    "category": {"uid": "cat-env", "name": "Environments", "path": "environments", "slug": "environments"},
                    "isMature": false,
                    "isFree": false,
                    "publishedAt": "2025-06-15T10:30:00Z"
                }
            ],
            "count": 142,
            "cursors": {"next": "eyJwIjoyfQ==", "previous": null}
        }"#;
        let results: FabSearchResults = serde_json::from_str(json).unwrap();
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.count, Some(142));
        let listing = &results.results[0];
        assert_eq!(listing.uid, "a1b2c3d4-e5f6-7890-abcd-ef1234567890");
        assert_eq!(listing.title.as_deref(), Some("Medieval Village Pack"));
        assert_eq!(listing.listing_type.as_deref(), Some("3d-model"));
        assert_eq!(listing.is_mature, Some(false));
        let user = listing.user.as_ref().unwrap();
        assert_eq!(user.seller_name.as_deref(), Some("Epic Marketplace"));
        assert_eq!(user.is_seller, Some(true));
        let category = listing.category.as_ref().unwrap();
        assert_eq!(category.name.as_deref(), Some("Environments"));
        assert_eq!(category.slug.as_deref(), Some("environments"));
        let cursors = results.cursors.as_ref().unwrap();
        assert_eq!(cursors.next.as_deref(), Some("eyJwIjoyfQ=="));
        assert!(cursors.previous.is_none());
    }

    #[test]
    fn deserialize_search_results_empty() {
        let json = r#"{"results": [], "count": 0}"#;
        let results: FabSearchResults = serde_json::from_str(json).unwrap();
        assert!(results.results.is_empty());
        assert_eq!(results.count, Some(0));
        assert!(results.cursors.is_none());
    }

    #[test]
    fn deserialize_listing_detail() {
        let json = r#"{
            "uid": "listing-xyz-789",
            "title": "Sci-Fi Weapon Set",
            "description": "50 sci-fi weapons with PBR materials",
            "listingType": "3d-model",
            "user": {"sellerName": "GameArt Studio", "uid": "seller-042"},
            "category": {"uid": "cat-weapons", "name": "Weapons", "path": "weapons", "slug": "weapons"},
            "ratings": {"averageRating": 4.7, "total": 23},
            "isMature": false,
            "createdAt": "2025-03-01T00:00:00Z",
            "publishedAt": "2025-03-01T00:00:00Z",
            "reviewCount": 23
        }"#;
        let detail: FabListingDetail = serde_json::from_str(json).unwrap();
        assert_eq!(detail.uid, "listing-xyz-789");
        assert_eq!(detail.title.as_deref(), Some("Sci-Fi Weapon Set"));
        let category = detail.category.as_ref().unwrap();
        assert_eq!(category.name.as_deref(), Some("Weapons"));
        assert!(detail.ratings.is_some());
        assert_eq!(detail.review_count, Some(23));
    }

    #[test]
    fn deserialize_ue_format() {
        let json = r#"{
            "assetFormatType": {"code": "ue-asset", "icon": "unreal-icon", "name": "Unreal Engine Asset"},
            "technicalSpecs": {
                "technicalDetails": "Verts: 12000, Tris: 8000",
                "unrealEngineEngineVersions": ["5.3", "5.4", "5.5"],
                "unrealEngineTargetPlatforms": ["Windows", "Linux", "Mac"],
                "unrealEngineDistributionMethod": "asset-pack"
            }
        }"#;
        let format: FabListingUeFormat = serde_json::from_str(json).unwrap();
        let fmt_type = format.asset_format_type.as_ref().unwrap();
        assert_eq!(fmt_type.code.as_deref(), Some("ue-asset"));
        assert_eq!(fmt_type.name.as_deref(), Some("Unreal Engine Asset"));
        let specs = format.technical_specs.as_ref().unwrap();
        let versions = specs.unreal_engine_engine_versions.as_ref().unwrap();
        assert_eq!(versions, &["5.3", "5.4", "5.5"]);
        let platforms = specs.unreal_engine_target_platforms.as_ref().unwrap();
        assert_eq!(platforms.len(), 3);
        assert_eq!(
            specs.unreal_engine_distribution_method.as_deref(),
            Some("asset-pack")
        );
    }

    #[test]
    fn deserialize_price_info() {
        let json = r#"{"currencyCode": "USD", "price": 29.99, "discount": 0.0, "vat": 0.0}"#;
        let price: FabPriceInfo = serde_json::from_str(json).unwrap();
        assert_eq!(price.currency_code.as_deref(), Some("USD"));
        assert_eq!(price.price, Some(29.99));
        assert_eq!(price.discount, Some(0.0));
    }

    #[test]
    fn deserialize_ownership_with_licenses() {
        let json = r#"{"licenses": [{"name": "Standard License", "slug": "standard"}, {"name": "Enterprise", "slug": "enterprise"}]}"#;
        let ownership: FabOwnership = serde_json::from_str(json).unwrap();
        let licenses = ownership.licenses.as_ref().unwrap();
        assert_eq!(licenses.len(), 2);
        assert_eq!(licenses[0].name.as_deref(), Some("Standard License"));
        assert_eq!(licenses[0].slug.as_deref(), Some("standard"));
        assert_eq!(licenses[1].slug.as_deref(), Some("enterprise"));
    }

    #[test]
    fn deserialize_ownership_empty() {
        let json = r#"{"licenses": []}"#;
        let ownership: FabOwnership = serde_json::from_str(json).unwrap();
        assert!(ownership.licenses.as_ref().unwrap().is_empty());
    }

    #[test]
    fn deserialize_listing_state() {
        let json = r#"{"acquired": true, "entitlementId": "ent-abc-123", "wishlisted": false}"#;
        let state: FabListingState = serde_json::from_str(json).unwrap();
        assert_eq!(state.acquired, Some(true));
        assert_eq!(state.entitlement_id.as_deref(), Some("ent-abc-123"));
        assert_eq!(state.wishlisted, Some(false));
        assert!(state.ownership.is_none());
        assert!(state.review.is_none());
    }

    #[test]
    fn search_params_default() {
        let params = FabSearchParams::default();
        assert!(params.channels.is_none());
        assert!(params.listing_types.is_none());
        assert!(params.sort_by.is_none());
        assert!(params.count.is_none());
        assert!(params.is_discounted.is_none());
    }

    #[test]
    fn search_results_roundtrip() {
        let results = FabSearchResults {
            results: vec![FabSearchListing {
                uid: "test-uid".to_string(),
                title: Some("Test Asset".to_string()),
                ..Default::default()
            }],
            count: Some(1),
            cursors: Some(FabSearchCursors {
                next: Some("cursor-abc".to_string()),
                previous: None,
            }),
            aggregations: None,
        };
        let json = serde_json::to_string(&results).unwrap();
        let roundtrip: FabSearchResults = serde_json::from_str(&json).unwrap();
        assert_eq!(results, roundtrip);
    }

    #[test]
    fn deserialize_real_api_search_result() {
        let json = r#"{
            "aggregations": null,
            "cursors": {"next": "cD0yMDI2", "previous": null},
            "results": [{
                "uid": "a55fc08e-82ec-4332-8bed-dde44fff7847",
                "title": "Steam Integration for Unreal Engine",
                "listingType": "tool-and-plugin",
                "isMature": false,
                "isFree": false,
                "isDiscounted": false,
                "publishedAt": "2026-02-16T10:00:00Z",
                "category": {"uid": "afd8c50c", "name": "Engine Tools", "path": "engine-tools", "slug": "engine-tools"},
                "user": {"sellerName": "PloxTools", "sellerId": "o-hz52h7n", "uid": "86b03c21", "isSeller": true},
                "ratings": {"averageRating": 0.0, "total": 0},
                "tags": [{"name": "Steam", "slug": "steam", "uid": "tag-001"}],
                "thumbnails": [{"uid": "thumb-001", "mediaUrl": "https://media.fab.com/img.jpg", "type": "thumbnail"}]
            }]
        }"#;
        let results: FabSearchResults = serde_json::from_str(json).unwrap();
        assert_eq!(results.results.len(), 1);
        let listing = &results.results[0];
        assert_eq!(listing.listing_type.as_deref(), Some("tool-and-plugin"));
        assert_eq!(listing.is_free, Some(false));
        let user = listing.user.as_ref().unwrap();
        assert_eq!(user.seller_name.as_deref(), Some("PloxTools"));
        let category = listing.category.as_ref().unwrap();
        assert_eq!(category.name.as_deref(), Some("Engine Tools"));
        let tags = listing.tags.as_ref().unwrap();
        assert_eq!(tags[0].name.as_deref(), Some("Steam"));
        let thumbs = listing.thumbnails.as_ref().unwrap();
        assert_eq!(thumbs[0].thumbnail_type.as_deref(), Some("thumbnail"));
    }

    #[test]
    fn deserialize_real_api_listing_detail() {
        let json = r#"{
            "uid": "a55fc08e-82ec-4332-8bed-dde44fff7847",
            "title": "Steam Integration for Unreal Engine",
            "description": "Integrates Steam SDK with Unreal Engine",
            "listingType": "tool-and-plugin",
            "isMature": false,
            "isFree": false,
            "createdAt": "2026-01-15T00:00:00Z",
            "publishedAt": "2026-02-16T10:00:00Z",
            "category": {"uid": "afd8c50c", "name": "Engine Tools", "path": "engine-tools", "slug": "engine-tools"},
            "user": {"sellerName": "PloxTools", "uid": "86b03c21", "profileUrl": "https://www.fab.com/sellers/PloxTools"},
            "ratings": {"rating5": 0, "total": 0},
            "reviewCount": 0,
            "startingPrice": {"price": 49.99, "offerId": "offer-001"}
        }"#;
        let detail: FabListingDetail = serde_json::from_str(json).unwrap();
        assert_eq!(detail.uid, "a55fc08e-82ec-4332-8bed-dde44fff7847");
        assert_eq!(detail.listing_type.as_deref(), Some("tool-and-plugin"));
        assert_eq!(detail.review_count, Some(0));
        let user = detail.user.as_ref().unwrap();
        assert_eq!(user.seller_name.as_deref(), Some("PloxTools"));
        assert_eq!(
            user.profile_url.as_deref(),
            Some("https://www.fab.com/sellers/PloxTools")
        );
    }
}

/// Search filter parameters for `fab_search()`.
#[derive(Default, Debug, Clone)]
pub struct FabSearchParams {
    /// Channel filter (e.g. `unreal-engine`)
    pub channels: Option<String>,
    /// Listing type filter (e.g. `3d-model`, `tool-and-plugin`, `audio`, `material`)
    pub listing_types: Option<String>,
    /// Category filter
    pub categories: Option<String>,
    /// Sort order: `-relevance`, `-createdAt`, `createdAt`, `-price`, `price`
    pub sort_by: Option<String>,
    /// Results per page
    pub count: Option<u32>,
    /// Pagination cursor (from previous result)
    pub cursor: Option<String>,
    /// Aggregation type: `category_per_listing_type`, `channel`, `listing_type`
    pub aggregate_on: Option<String>,
    /// Filter to wishlisted items: set to `"wishlist"`
    pub in_filter: Option<String>,
    /// Only discounted items
    pub is_discounted: Option<bool>,
}
