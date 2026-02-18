use crate::api::EpicAPI;
use crate::api::error::EpicAPIError;
use crate::api::types::billing_account::BillingAccount;
use crate::api::types::price::PriceResponse;
use crate::api::types::quick_purchase::QuickPurchaseResponse;

impl EpicAPI {
    /// Fetch offer prices from the price engine.
    pub async fn offer_prices(
        &self,
        namespace: &str,
        offer_ids: &[String],
        country: &str,
    ) -> Result<PriceResponse, EpicAPIError> {
        let url = "https://priceengine-public-service-ecomprod01.ol.epicgames.com/priceengine/api/shared/offers/price";
        let body = serde_json::json!({
            "namespace": namespace,
            "offers": offer_ids,
            "country": country,
        });
        self.authorized_post_json(url, &body).await
    }

    /// Execute a quick purchase (typically for free game claims).
    pub async fn quick_purchase(
        &self,
        namespace: &str,
        offer_id: &str,
    ) -> Result<QuickPurchaseResponse, EpicAPIError> {
        let url = match &self.user_data.account_id {
            Some(id) => format!(
                "https://orderprocessor-public-service-ecomprod01.ol.epicgames.com/orderprocessor/api/shared/accounts/{}/orders/quickPurchase",
                id
            ),
            None => return Err(EpicAPIError::InvalidCredentials),
        };
        let body = serde_json::json!({
            "salesChannel": "Launcher-purchase-client",
            "entitlementSource": "Launcher-purchase-client",
            "returnSplitPaymentItems": false,
            "lineOffers": [{
                "offerId": offer_id,
                "quantity": 1,
                "namespace": namespace,
            }],
        });
        self.authorized_post_json(&url, &body).await
    }

    /// Fetch the default billing account for the logged-in user.
    pub async fn billing_account(&self) -> Result<BillingAccount, EpicAPIError> {
        let url = match &self.user_data.account_id {
            Some(id) => format!(
                "https://launcher-public-service-prod06.ol.epicgames.com/launcher/api/public/payment/accounts/{}/billingaccounts/default",
                id
            ),
            None => return Err(EpicAPIError::InvalidCredentials),
        };
        self.authorized_get_json(&url).await
    }
}
