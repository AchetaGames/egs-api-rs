use log::{debug, error, warn};
use reqwest::header::HeaderMap;
use reqwest::{Client, RequestBuilder};
use serde::de::DeserializeOwned;
use types::account::UserData;
use url::Url;

use crate::api::error::EpicAPIError;

/// Module holding the API types
pub mod types;

/// Various API Utils
pub mod utils;

/// Binary reader/writer for manifest parsing
#[allow(dead_code)]
pub(crate) mod binary_rw;

/// Error type
pub mod error;

/// Fab Methods
pub mod fab;

///Account methods
pub mod account;

/// EGS Methods
pub mod egs;
#[allow(dead_code)]
/// Cloud Save Methods
pub mod cloud_save;
/// Session Handling
pub mod login;

/// Commerce Methods (pricing, purchases, billing)
pub mod commerce;

/// Service Status Methods
pub mod status;

/// Presence Methods
pub mod presence;

/// Uplay/Ubisoft Store Methods
pub mod store;

/// Cosmos session and API methods (unrealengine.com cookie-based)
pub mod cosmos;

#[derive(Debug, Clone)]
pub(crate) struct EpicAPI {
    client: Client,
    pub(crate) user_data: UserData,
}

impl Default for EpicAPI {
    fn default() -> Self {
        Self::new()
    }
}

impl EpicAPI {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            "UELauncher/17.0.1-37584233+++Portal+Release-Live Windows/10.0.19043.1.0.64bit"
                .parse()
                .unwrap(),
        );
        headers.insert(
            "X-Epic-Correlation-ID",
            "UE4-c176f7154c2cda1061cc43ab52598e2b-93AFB486488A22FDF70486BD1D883628-BFCD88F649E997BA203FF69F07CE578C".parse().unwrap()
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .build()
            .unwrap();
        EpicAPI {
            client,
            user_data: Default::default(),
        }
    }

    fn authorized_get_client(&self, url: Url) -> RequestBuilder {
        self.set_authorization_header(self.client.get(url))
    }

    fn authorized_post_client(&self, url: Url) -> RequestBuilder {
        self.set_authorization_header(self.client.post(url))
    }

    fn set_authorization_header(&self, rb: RequestBuilder) -> RequestBuilder {
        rb.header(
            "Authorization",
            format!(
                "{} {}",
                self.user_data
                    .token_type
                    .as_ref()
                    .unwrap_or(&"bearer".to_string()),
                self.user_data
                    .access_token
                    .as_ref()
                    .unwrap_or(&"".to_string())
            ),
        )
    }

    async fn send(request: RequestBuilder) -> Result<reqwest::Response, EpicAPIError> {
        request.send().await.map_err(|e| {
            error!("{:?}", e);
            EpicAPIError::NetworkError(e)
        })
    }

    fn require_ok(response: &reqwest::Response) -> Result<(), ()> {
        if response.status() == reqwest::StatusCode::OK {
            Ok(())
        } else {
            Err(())
        }
    }

    async fn error_response(response: reqwest::Response) -> EpicAPIError {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        warn!("{} result: {}", status, body);
        EpicAPIError::HttpError { status, body }
    }

    async fn read_json<T: DeserializeOwned>(
        response: reqwest::Response,
        url: &str,
    ) -> Result<T, EpicAPIError> {
        let body = response.text().await.map_err(|e| {
            error!("Failed to read response body from {}: {:?}", url, e);
            EpicAPIError::DeserializationError(format!("{}", e))
        })?;
        serde_json::from_str::<T>(&body).map_err(|e| {
            error!("Deserialization failed for {}: {:?}", url, e);
            error!("Response body: {}", &body[..body.len().min(2048)]);
            EpicAPIError::DeserializationError(format!("{}", e))
        })
    }

    async fn send_and_deserialize<T: DeserializeOwned>(
        request: RequestBuilder,
        url: &str,
    ) -> Result<T, EpicAPIError> {
        let response = Self::send(request).await?;
        if Self::require_ok(&response).is_ok() {
            Self::read_json(response, url).await
        } else {
            Err(Self::error_response(response).await)
        }
    }

    pub(crate) async fn authorized_get_json<T: DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<T, EpicAPIError> {
        let parsed_url = Url::parse(url).map_err(|_| EpicAPIError::InvalidParams)?;
        debug!("authorized_get_json: {}", url);
        Self::send_and_deserialize(self.authorized_get_client(parsed_url), url).await
    }

    pub(crate) async fn authorized_post_form_json<T: DeserializeOwned>(
        &self,
        url: &str,
        form: &[(String, String)],
    ) -> Result<T, EpicAPIError> {
        let parsed_url = Url::parse(url).map_err(|_| EpicAPIError::InvalidParams)?;
        debug!("authorized_post_form_json: {}", url);
        Self::send_and_deserialize(self.authorized_post_client(parsed_url).form(form), url).await
    }

    pub(crate) async fn authorized_post_json<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T, EpicAPIError> {
        let parsed_url = Url::parse(url).map_err(|_| EpicAPIError::InvalidParams)?;
        debug!("authorized_post_json: {}", url);
        Self::send_and_deserialize(self.authorized_post_client(parsed_url).json(body), url).await
    }

    pub(crate) async fn get_bytes(&self, url: &str) -> Result<Vec<u8>, EpicAPIError> {
        let parsed_url = Url::parse(url).map_err(|_| EpicAPIError::InvalidParams)?;
        let response = Self::send(self.client.get(parsed_url)).await?;
        if Self::require_ok(&response).is_ok() {
            response.bytes().await.map(|b| b.to_vec()).map_err(|e| {
                error!("{:?}", e);
                EpicAPIError::DeserializationError(format!("{}", e))
            })
        } else {
            Err(Self::error_response(response).await)
        }
    }

    pub(crate) async fn get_json<T: DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<T, EpicAPIError> {
        let parsed_url = Url::parse(url).map_err(|_| EpicAPIError::InvalidParams)?;
        debug!("get_json: {}", url);
        Self::send_and_deserialize(self.client.get(parsed_url), url).await
    }

    #[allow(dead_code)]
    pub(crate) async fn authorized_delete(&self, url: &str) -> Result<(), EpicAPIError> {
        let parsed_url = Url::parse(url).map_err(|_| EpicAPIError::InvalidParams)?;
        let response =
            Self::send(self.set_authorization_header(self.client.delete(parsed_url))).await?;
        if response.status() == reqwest::StatusCode::OK
            || response.status() == reqwest::StatusCode::NO_CONTENT
        {
            Ok(())
        } else {
            Err(Self::error_response(response).await)
        }
    }
}
