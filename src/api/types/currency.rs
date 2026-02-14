use serde::{Deserialize, Serialize};

/// Paginated response for currencies.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyPage {
    #[serde(default)]
    pub elements: Vec<Currency>,
    pub paging: Option<super::catalog_item::Paging>,
}

/// A single currency entry.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Currency {
    pub r#type: String,
    pub code: String,
    pub symbol: Option<String>,
    pub description: Option<String>,
    pub decimals: Option<i32>,
    pub truncation_length: Option<i32>,
}
