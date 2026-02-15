use serde::{Deserialize, Serialize};

/// Paginated response for catalog items within a namespace.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogItemPage {
    #[serde(default)]
    pub elements: Vec<CatalogItem>,
    pub paging: Option<Paging>,
}

/// A single catalog item entry.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogItem {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub namespace: Option<String>,
    pub status: Option<String>,
    pub creation_date: Option<String>,
    pub last_modified_date: Option<String>,
    pub entitlement_name: Option<String>,
    pub entitlement_type: Option<String>,
    pub item_type: Option<String>,
    pub developer: Option<String>,
    pub developer_id: Option<String>,
    pub end_of_support: Option<bool>,
    pub unsearchable: Option<bool>,
}

/// Pagination metadata for catalog responses.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Paging {
    pub count: i64,
    pub start: i64,
    pub total: i64,
}
