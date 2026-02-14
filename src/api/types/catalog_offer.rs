use serde::{Deserialize, Serialize};

/// Paginated response for catalog offers within a namespace.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogOfferPage {
    #[serde(default)]
    pub elements: Vec<CatalogOffer>,
    pub paging: Option<super::catalog_item::Paging>,
}

/// A single catalog offer entry.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogOffer {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub namespace: Option<String>,
    pub status: Option<String>,
    pub creation_date: Option<String>,
    pub last_modified_date: Option<String>,
    pub offer_type: Option<String>,
    pub effective_date: Option<String>,
    pub expiry_date: Option<String>,
    pub is_code_redemption_only: Option<bool>,
    pub seller: Option<Seller>,
    #[serde(default)]
    pub categories: Vec<OfferCategory>,
    #[serde(default)]
    pub items: Vec<OfferItem>,
}

/// Seller information on a catalog offer.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Seller {
    pub id: String,
    pub name: Option<String>,
}

/// Category attached to a catalog offer.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferCategory {
    pub path: String,
}

/// Item reference within a catalog offer.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferItem {
    pub id: String,
    pub namespace: Option<String>,
}
