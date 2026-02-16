use super::fab_search::{FabLicense, FabListingUeFormat, FabSearchCursors};
use serde::{Deserialize, Serialize};

/// Search params for `fab_library_entitlements()`.
#[derive(Default, Debug, Clone)]
pub struct FabEntitlementSearchParams {
    /// Sort order (e.g. `-createdAt`)
    pub sort_by: Option<String>,
    /// Cursor for pagination
    pub cursor: Option<String>,
    /// Listing type filter (e.g. `tool-and-plugin`)
    pub listing_types: Option<String>,
    /// Category filter
    pub categories: Option<String>,
    /// Tag filter
    pub tags: Option<String>,
    /// License filter
    pub licenses: Option<String>,
    /// Asset format filter
    pub asset_formats: Option<String>,
    /// Source filter (e.g. `acquired`)
    pub source: Option<String>,
    /// Fields to aggregate on (e.g. `categories,listingTypes`)
    pub aggregate_on: Option<String>,
    /// Results per page
    pub count: Option<u32>,
    /// Filter by entitlements added since this date
    pub added_since: Option<String>,
}

/// Paginated entitlements result from `GET /i/library/entitlements/search`.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FabEntitlementResults {
    pub results: Vec<FabEntitlement>,
    pub count: Option<u64>,
    pub cursors: Option<FabSearchCursors>,
    pub aggregations: Option<serde_json::Value>,
}

/// A single library entitlement.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabEntitlement {
    pub capabilities: Option<FabEntitlementCapabilities>,
    pub created_at: Option<String>,
    pub licenses: Option<Vec<FabLicense>>,
    pub listing: Option<FabEntitlementListing>,
}

/// Capabilities granted by an entitlement.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabEntitlementCapabilities {
    pub add_by_verse: Option<bool>,
    pub request_download_url: Option<bool>,
}

/// Listing metadata within an entitlement.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabEntitlementListing {
    pub uid: Option<String>,
    pub title: Option<String>,
    pub is_mature: Option<bool>,
    pub last_updated_at: Option<String>,
    pub asset_formats: Option<Vec<FabListingUeFormat>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_entitlement_results() {
        let json = r#"{
            "results": [
                {
                    "capabilities": {"addByVerse": true, "requestDownloadUrl": true},
                    "createdAt": "2025-01-15T10:00:00Z",
                    "licenses": [{"name": "Standard", "slug": "standard"}],
                    "listing": {
                        "uid": "ent-001",
                        "title": "My Plugin",
                        "isMature": false,
                        "lastUpdatedAt": "2025-06-01T00:00:00Z",
                        "assetFormats": []
                    }
                }
            ],
            "count": 1,
            "cursors": {"next": null, "previous": null},
            "aggregations": {"categories": []}
        }"#;
        let results: FabEntitlementResults = serde_json::from_str(json).unwrap();
        assert_eq!(results.count, Some(1));
        assert_eq!(results.results.len(), 1);
        let ent = &results.results[0];
        assert_eq!(ent.capabilities.as_ref().unwrap().add_by_verse, Some(true));
        assert_eq!(
            ent.listing.as_ref().unwrap().title,
            Some("My Plugin".to_string())
        );
    }

    #[test]
    fn deserialize_empty_results() {
        let json = r#"{"results": [], "count": 0}"#;
        let results: FabEntitlementResults = serde_json::from_str(json).unwrap();
        assert_eq!(results.count, Some(0));
        assert!(results.results.is_empty());
    }

    #[test]
    fn search_params_default() {
        let params = FabEntitlementSearchParams::default();
        assert!(params.sort_by.is_none());
        assert!(params.count.is_none());
        assert!(params.source.is_none());
    }
}
