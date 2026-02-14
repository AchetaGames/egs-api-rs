use serde::{Deserialize, Serialize};

/// Response from the price engine for offer pricing.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceResponse {
    #[serde(default)]
    pub offers: Vec<OfferPrice>,
}

/// Price details for a single offer.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferPrice {
    pub offer_id: String,
    pub namespace: Option<String>,
    pub effective_date: Option<String>,
    pub expiry_date: Option<String>,
    pub current_price: Option<PriceDetail>,
}

/// Breakdown of a price entry.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceDetail {
    pub currency_code: Option<String>,
    pub discount_price: Option<i64>,
    pub original_price: Option<i64>,
    pub voucher_discount: Option<i64>,
    pub discount_percentage: Option<i64>,
    pub currency_info: Option<PriceCurrencyInfo>,
    pub fmt_price: Option<FormattedPrice>,
}

/// Currency info embedded in a price response.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceCurrencyInfo {
    pub decimals: Option<i32>,
}

/// Pre-formatted price strings.
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormattedPrice {
    pub original_price: Option<String>,
    pub discount_price: Option<String>,
    pub intermediate_price: Option<String>,
}
