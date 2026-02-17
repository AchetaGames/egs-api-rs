use crate::api::error::EpicAPIError;
use crate::api::types::asset_info::AssetInfo;
use crate::api::types::billing_account::BillingAccount;
use crate::api::types::catalog_item::CatalogItemPage;
use crate::api::types::catalog_offer::CatalogOfferPage;
use crate::api::types::currency::CurrencyPage;
use crate::api::types::price::PriceResponse;
use crate::api::types::quick_purchase::QuickPurchaseResponse;
use crate::api::types::uplay::{
    UplayClaimResult, UplayCodesResult, UplayGraphQLResponse, UplayRedeemResult,
};
use crate::EpicGames;

impl EpicGames {
    /// Like [`catalog_items`](Self::catalog_items), but returns a `Result` instead of swallowing errors.
    pub async fn try_catalog_items(
        &self,
        namespace: &str,
        start: i64,
        count: i64,
    ) -> Result<CatalogItemPage, EpicAPIError> {
        self.egs.catalog_items(namespace, start, count).await
    }

    /// Fetch paginated catalog items for a namespace.
    ///
    /// Queries the Epic catalog service for items within a given namespace
    /// (e.g., a game's namespace). Results are paginated — use `start` and
    /// `count` to page through. Each [`CatalogItemPage`] includes a `paging`
    /// field with the total count.
    ///
    /// Returns `None` on API errors.
    pub async fn catalog_items(
        &self,
        namespace: &str,
        start: i64,
        count: i64,
    ) -> Option<CatalogItemPage> {
        self.try_catalog_items(namespace, start, count).await.ok()
    }

    /// Like [`catalog_offers`](Self::catalog_offers), but returns a `Result` instead of swallowing errors.
    pub async fn try_catalog_offers(
        &self,
        namespace: &str,
        start: i64,
        count: i64,
    ) -> Result<CatalogOfferPage, EpicAPIError> {
        self.egs.catalog_offers(namespace, start, count).await
    }

    /// Fetch paginated catalog offers for a namespace.
    ///
    /// Queries the Epic catalog service for offers (purchasable items) within
    /// a namespace. Offers include pricing metadata, seller info, and linked
    /// catalog items. Use `start` and `count` to paginate.
    ///
    /// Returns `None` on API errors.
    pub async fn catalog_offers(
        &self,
        namespace: &str,
        start: i64,
        count: i64,
    ) -> Option<CatalogOfferPage> {
        self.try_catalog_offers(namespace, start, count).await.ok()
    }

    /// Like [`bulk_catalog_items`](Self::bulk_catalog_items), but returns a `Result` instead of swallowing errors.
    pub async fn try_bulk_catalog_items(
        &self,
        items: &[(&str, &str)],
    ) -> Result<std::collections::HashMap<String, std::collections::HashMap<String, AssetInfo>>, EpicAPIError> {
        self.egs.bulk_catalog_items(items).await
    }

    /// Bulk fetch catalog items across multiple namespaces.
    ///
    /// Accepts a slice of `(namespace, item_id)` pairs and returns them grouped
    /// by namespace → item_id → [`AssetInfo`]. Useful for resolving catalog
    /// metadata for items from different games in a single request.
    ///
    /// Returns `None` on API errors.
    pub async fn bulk_catalog_items(
        &self,
        items: &[(&str, &str)],
    ) -> Option<std::collections::HashMap<String, std::collections::HashMap<String, AssetInfo>>> {
        self.try_bulk_catalog_items(items).await.ok()
    }

    /// Like [`currencies`](Self::currencies), but returns a `Result` instead of swallowing errors.
    pub async fn try_currencies(&self, start: i64, count: i64) -> Result<CurrencyPage, EpicAPIError> {
        self.egs.currencies(start, count).await
    }

    /// Fetch available currencies from the Epic catalog.
    ///
    /// Returns paginated currency definitions including code, symbol, and
    /// decimal precision. Use `start` and `count` to paginate.
    ///
    /// Returns `None` on API errors.
    pub async fn currencies(&self, start: i64, count: i64) -> Option<CurrencyPage> {
        self.try_currencies(start, count).await.ok()
    }

    /// Like [`library_state_token_status`](Self::library_state_token_status), but returns a `Result` instead of swallowing errors.
    pub async fn try_library_state_token_status(
        &self,
        token_id: &str,
    ) -> Result<bool, EpicAPIError> {
        self.egs.library_state_token_status(token_id).await
    }

    /// Check the validity of a library state token.
    ///
    /// Returns `Some(true)` if the token is still valid, `Some(false)` if
    /// expired or invalid, or `None` on API errors. Library state tokens are
    /// used to detect changes to the user's library since the last sync.
    ///
    /// Returns `None` on API errors.
    pub async fn library_state_token_status(&self, token_id: &str) -> Option<bool> {
        self.try_library_state_token_status(token_id).await.ok()
    }

    /// Like [`offer_prices`](Self::offer_prices), but returns a `Result` instead of swallowing errors.
    pub async fn try_offer_prices(
        &self,
        namespace: &str,
        offer_ids: &[String],
        country: &str,
    ) -> Result<PriceResponse, EpicAPIError> {
        self.egs.offer_prices(namespace, offer_ids, country).await
    }

    /// Fetch offer prices from Epic's price engine.
    ///
    /// Queries current pricing for one or more offers within a namespace,
    /// localized to a specific country. The response includes original price,
    /// discount price, and pre-formatted display strings.
    ///
    /// Returns `None` on API errors.
    pub async fn offer_prices(
        &self,
        namespace: &str,
        offer_ids: &[String],
        country: &str,
    ) -> Option<PriceResponse> {
        self.try_offer_prices(namespace, offer_ids, country).await.ok()
    }

    /// Like [`quick_purchase`](Self::quick_purchase), but returns a `Result` instead of swallowing errors.
    pub async fn try_quick_purchase(
        &self,
        namespace: &str,
        offer_id: &str,
    ) -> Result<QuickPurchaseResponse, EpicAPIError> {
        self.egs.quick_purchase(namespace, offer_id).await
    }

    /// Execute a quick purchase (typically for free game claims).
    ///
    /// Initiates a purchase order for a free offer. The response contains the
    /// order ID and its processing status. For paid offers, use the full
    /// checkout flow in the Epic Games launcher instead.
    ///
    /// Returns `None` on API errors.
    pub async fn quick_purchase(
        &self,
        namespace: &str,
        offer_id: &str,
    ) -> Option<QuickPurchaseResponse> {
        self.try_quick_purchase(namespace, offer_id).await.ok()
    }

    /// Like [`billing_account`](Self::billing_account), but returns a `Result` instead of swallowing errors.
    pub async fn try_billing_account(&self) -> Result<BillingAccount, EpicAPIError> {
        self.egs.billing_account().await
    }

    /// Fetch the default billing account for payment processing.
    ///
    /// Returns the account's billing country, which is used to determine
    /// regional pricing and payment availability.
    ///
    /// Returns `None` on API errors.
    pub async fn billing_account(&self) -> Option<BillingAccount> {
        self.try_billing_account().await.ok()
    }

    // ── Uplay / Ubisoft Store ──

    /// Fetch Uplay codes linked to the user's Epic account.
    pub async fn store_get_uplay_codes(
        &self,
    ) -> Result<UplayGraphQLResponse<UplayCodesResult>, EpicAPIError> {
        self.egs.store_get_uplay_codes().await
    }

    /// Claim a Uplay code for a specific game.
    pub async fn store_claim_uplay_code(
        &self,
        uplay_account_id: &str,
        game_id: &str,
    ) -> Result<UplayGraphQLResponse<UplayClaimResult>, EpicAPIError> {
        self.egs
            .store_claim_uplay_code(uplay_account_id, game_id)
            .await
    }

    /// Redeem all pending Uplay codes for the user's account.
    pub async fn store_redeem_uplay_codes(
        &self,
        uplay_account_id: &str,
    ) -> Result<UplayGraphQLResponse<UplayRedeemResult>, EpicAPIError> {
        self.egs.store_redeem_uplay_codes(uplay_account_id).await
    }
}
