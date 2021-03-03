use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use chrono::{DateTime, Utc};
use reqwest::header::HeaderMap;
use reqwest::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};

use types::{
    AssetInfo, AssetManifest, DownloadManifest, Entitlement, EpicAsset, GameToken, Library,
    Manifest, OwnershipToken,
};
use url::Url;

/// Module holding the API types
pub mod types;

/// Structure that holds all user data
///
/// Needed for login
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserData {
    access_token: Option<String>,
    pub expires_in: Option<i64>,
    pub expires_at: Option<DateTime<Utc>>,
    pub token_type: Option<String>,
    refresh_token: Option<String>,
    pub refresh_expires: Option<i64>,
    pub refresh_expires_at: Option<DateTime<Utc>>,
    pub account_id: Option<String>,
    pub client_id: Option<String>,
    pub internal_client: Option<bool>,
    pub client_service: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub app: Option<String>,
    pub in_app_id: Option<String>,
    pub device_id: Option<String>,
    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
    #[serde(rename = "errorCode")]
    pub error_code: Option<String>,
}

impl UserData {
    /// Updates only the present values in the existing user data
    pub fn update(&mut self, new: UserData) {
        if let Some(n) = new.access_token {
            self.access_token = Some(n)
        }
        if let Some(n) = new.expires_in {
            self.expires_in = Some(n)
        }
        if let Some(n) = new.expires_at {
            self.expires_at = Some(n)
        }
        if let Some(n) = new.token_type {
            self.token_type = Some(n)
        }
        if let Some(n) = new.refresh_token {
            self.refresh_token = Some(n)
        }
        if let Some(n) = new.refresh_expires {
            self.refresh_expires = Some(n)
        }
        if let Some(n) = new.refresh_expires_at {
            self.refresh_expires_at = Some(n)
        }
        if let Some(n) = new.account_id {
            self.account_id = Some(n)
        }
        if let Some(n) = new.client_id {
            self.client_id = Some(n)
        }
        if let Some(n) = new.internal_client {
            self.internal_client = Some(n)
        }
        if let Some(n) = new.client_service {
            self.client_service = Some(n)
        }
        if let Some(n) = new.display_name {
            self.display_name = Some(n)
        }
        if let Some(n) = new.app {
            self.app = Some(n)
        }
        if let Some(n) = new.in_app_id {
            self.in_app_id = Some(n)
        }
        if let Some(n) = new.device_id {
            self.device_id = Some(n)
        }
        if let Some(n) = new.error_message {
            self.error_message = Some(n)
        }
        if let Some(n) = new.error_code {
            self.error_code = Some(n)
        }
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct EpicAPI {
    client: Client,
    pub(crate) user_data: UserData,
}

/// Error enum for the Epic API
#[derive(Debug)]
pub enum EpicAPIError {
    /// Wrong credentials
    InvalidCredentials,
    /// API error - see the contents
    APIError(String),
    /// Unknown error
    Unknown,
    /// Server error
    Server,
}

impl fmt::Display for EpicAPIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            EpicAPIError::InvalidCredentials => {
                write!(f, "Invalid Credentials")
            }
            EpicAPIError::Unknown => {
                write!(f, "Unknown Error")
            }
            EpicAPIError::Server => {
                write!(f, "Server Error")
            }
            EpicAPIError::APIError(e) => {
                write!(f, "API Error: {}", e)
            }
        }
    }
}

impl Error for EpicAPIError {
    fn description(&self) -> &str {
        match *self {
            EpicAPIError::InvalidCredentials => "Invalid Credentials",
            EpicAPIError::Unknown => "Unknown Error",
            EpicAPIError::Server => "Server Error",
            EpicAPIError::APIError(_) => "API Error",
        }
    }
}

impl EpicAPI {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            "UELauncher/12.0.5-15338009+++Portal+Release-Live Windows/6.1.7601.1.0.64bit"
                .parse()
                .unwrap(),
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

    pub async fn start_session(
        &mut self,
        exchange_token: Option<String>,
    ) -> Result<bool, EpicAPIError> {
        let params = match exchange_token {
            None => [
                ("grant_type".to_string(), "refresh_token".to_string()),
                (
                    "refresh_token".to_string(),
                    self.user_data.refresh_token.clone().unwrap(),
                ),
                ("token_type".to_string(), "eg1".to_string()),
            ],
            Some(exchange) => [
                ("grant_type".to_string(), "exchange_code".to_string()),
                ("exchange_code".to_string(), exchange),
                ("token_type".to_string(), "eg1".to_string()),
            ],
        };

        match self
            .client
            .post("https://account-public-service-prod03.ol.epicgames.com/account/api/oauth/token")
            .form(&params)
            .basic_auth(
                "34a02cf8f4414e29b15921876da36f9a",
                Some("daafbccc737745039dffe53d94fc76cf"),
            )
            .send()
            .await
        {
            Ok(response) => {
                return self.handle_login_response(response).await;
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    async fn handle_login_response(&mut self, response: Response) -> Result<bool, EpicAPIError> {
        if response.status() == reqwest::StatusCode::INTERNAL_SERVER_ERROR {
            println!("Server Error");
            return Err(EpicAPIError::Server);
        }
        let new: UserData = match response.json().await {
            Ok(data) => data,
            Err(e) => {
                println!("Error: {:?}", e);
                return Err(EpicAPIError::Unknown);
            }
        };

        self.user_data.update(new);

        match &self.user_data.error_message {
            None => {}
            Some(m) => {
                println!("Error: {}", m);
                return Err(EpicAPIError::APIError(m.to_string()));
            }
        }
        Ok(true)
    }

    fn get_authorized_get_client(&self, url: Url) -> RequestBuilder {
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            "UELauncher/12.0.5-15338009+++Portal+Release-Live Windows/6.1.7601.1.0.64bit"
                .parse()
                .unwrap(),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .build()
            .unwrap();
        self.set_authorization_header(client.clone().get(url))
    }

    fn get_authorized_post_client(&self, url: Url) -> RequestBuilder {
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            "UELauncher/12.0.5-15338009+++Portal+Release-Live Windows/6.1.7601.1.0.64bit"
                .parse()
                .unwrap(),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .build()
            .unwrap();
        self.set_authorization_header(client.clone().post(url))
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

    pub async fn resume_session(&mut self) -> Result<bool, EpicAPIError> {
        match self.get_authorized_get_client(Url::parse("https://account-public-service-prod03.ol.epicgames.com/account/api/oauth/verify").unwrap()).send().await {
            Ok(response) => {
                return self.handle_login_response(response).await;
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn get_assets(
        &mut self,
        platform: Option<String>,
        label: Option<String>,
    ) -> Result<Vec<EpicAsset>, EpicAPIError> {
        let plat = platform.unwrap_or("Windows".to_string());
        let lab = label.unwrap_or("Live".to_string());
        let url = format!("https://launcher-public-service-prod06.ol.epicgames.com/launcher/api/public/assets/{}?label={}", plat, lab);
        match self
            .get_authorized_get_client(Url::parse(&url).unwrap())
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(assets) => Ok(assets),
                        Err(e) => {
                            println!("Error: {:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
                    }
                } else {
                    println!(
                        "{} result: {}",
                        response.status(),
                        response.text().await.unwrap()
                    );
                    Err(EpicAPIError::Unknown)
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn get_asset_manifest(
        &self,
        platform: Option<String>,
        label: Option<String>,
        app: Option<String>,
        asset: EpicAsset,
    ) -> Result<AssetManifest, EpicAPIError> {
        let url = format!("https://launcher-public-service-prod06.ol.epicgames.com/launcher/api/public/assets/v2/platform/{}/namespace/{}/catalogItem/{}/app/{}/label/{}",
                          platform.unwrap_or("Windows".to_string()), asset.namespace, asset.catalog_item_id, app.unwrap_or(asset.app_name), label.unwrap_or("Live".to_string()));
        match self
            .get_authorized_get_client(Url::parse(&url).unwrap())
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(manifest) => Ok(manifest),
                        Err(e) => {
                            println!("Error: {:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
                    }
                } else {
                    println!(
                        "{} result: {}",
                        response.status(),
                        response.text().await.unwrap()
                    );
                    Err(EpicAPIError::Unknown)
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn get_asset_download_manifest(
        &self,
        manifest: Manifest,
    ) -> Result<DownloadManifest, EpicAPIError> {
        let mut queries: Vec<(String, String)> = Vec::new();
        for query in manifest.query_params {
            queries.push((query.name, query.value));
        }
        match self
            .get_authorized_get_client(manifest.uri.clone())
            .query(&queries)
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.text().await {
                        Ok(s) => {
                            println!("{}", s);
                            match serde_json::from_str::<DownloadManifest>(&s) {
                                Ok(mut man) => {
                                    let mut url = manifest.uri.clone();
                                    url.set_path(&match url.path_segments() {
                                        None => "".to_string(),
                                        Some(segments) => {
                                            let mut vec: Vec<&str> = segments.collect();
                                            vec.remove(vec.len() - 1);
                                            vec.join("/")
                                        }
                                    });
                                    url.set_query(None);
                                    url.set_fragment(None);
                                    man.base_url = Some(url);
                                    Ok(man)
                                }
                                Err(e) => {
                                    println!("Error: {:?}", e);
                                    Err(EpicAPIError::Unknown)
                                }
                            }}
                        Err(_) => {Err(EpicAPIError::Unknown)}
                    }
                } else {
                    println!(
                        "{} result: {}",
                        response.status(),
                        response.text().await.unwrap()
                    );
                    Err(EpicAPIError::Unknown)
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn get_asset_info(
        &self,
        asset: EpicAsset,
    ) -> Result<HashMap<String, AssetInfo>, EpicAPIError> {
        let url = format!("https://catalog-public-service-prod06.ol.epicgames.com/catalog/api/shared/namespace/{}/bulk/items?id={}&includeDLCDetails=true&includeMainGameDetails=true&country=us&locale=lc",
                          asset.namespace, asset.catalog_item_id);
        match self
            .get_authorized_get_client(Url::parse(&url).unwrap())
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    // match response.text().await {
                    //     Ok(t) => { println!("4321574 {},", t); }
                    //     Err(_) => {}
                    // }

                    match response.json().await {
                        Ok(info) => Ok(info),
                        Err(e) => {
                            println!("Error: {:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
                    }
                } else {
                    println!(
                        "{} result: {}",
                        response.status(),
                        response.text().await.unwrap()
                    );
                    Err(EpicAPIError::Unknown)
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn get_game_token(&self) -> Result<GameToken, EpicAPIError> {
        let url = format!(
            "https://account-public-service-prod03.ol.epicgames.com/account/api/oauth/exchange"
        );
        match self
            .get_authorized_get_client(Url::parse(&url).unwrap())
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(token) => Ok(token),
                        Err(e) => {
                            println!("Error: {:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
                    }
                } else {
                    println!(
                        "{} result: {}",
                        response.status(),
                        response.text().await.unwrap()
                    );
                    Err(EpicAPIError::Unknown)
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn get_ownership_token(
        &self,
        asset: EpicAsset,
    ) -> Result<OwnershipToken, EpicAPIError> {
        let url = match &self.user_data.account_id {
            None => {
                return Err(EpicAPIError::InvalidCredentials);
            }
            Some(id) => {
                format!("https://ecommerceintegration-public-service-ecomprod02.ol.epicgames.com/ecommerceintegration/api/public/platforms/EPIC/identities/{}/ownershipToken",
                        id)
            }
        };
        match self
            .get_authorized_post_client(Url::parse(&url).unwrap())
            .form(&[(
                "nsCatalogItemId".to_string(),
                format!("{}:{}", asset.namespace, asset.catalog_item_id),
            )])
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(token) => Ok(token),
                        Err(e) => {
                            println!("Error: {:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
                    }
                } else {
                    println!(
                        "{} result: {}",
                        response.status(),
                        response.text().await.unwrap()
                    );
                    Err(EpicAPIError::Unknown)
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn get_user_entitlements(&self) -> Result<Vec<Entitlement>, EpicAPIError> {
        let url = match &self.user_data.account_id {
            None => {
                return Err(EpicAPIError::InvalidCredentials);
            }
            Some(id) => {
                format!("https://entitlement-public-service-prod08.ol.epicgames.com/entitlement/api/account/{}/entitlements?start=0&count=5000",
                        id)
            }
        };
        match self
            .get_authorized_get_client(Url::parse(&url).unwrap())
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(ent) => Ok(ent),
                        Err(e) => {
                            println!("Error: {:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
                    }
                } else {
                    println!(
                        "{} result: {}",
                        response.status(),
                        response.text().await.unwrap()
                    );
                    Err(EpicAPIError::Unknown)
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn get_library_items(
        &mut self,
        include_metadata: bool,
    ) -> Result<Library, EpicAPIError> {
        let mut library = Library {
            records: vec![],
            response_metadata: Default::default(),
        };
        let mut cursor: Option<String> = None;
        loop {
            let url = match &cursor {
                None => {
                    format!("https://library-service.live.use1a.on.epicgames.com/library/api/public/items?includeMetadata={}", include_metadata)
                }
                Some(c) => {
                    format!("https://library-service.live.use1a.on.epicgames.com/library/api/public/items?includeMetadata={}&cursor={}", include_metadata, c)
                }
            };

            match self
                .get_authorized_get_client(Url::parse(&url).unwrap())
                .send()
                .await
            {
                Ok(response) => {
                    if response.status() == reqwest::StatusCode::OK {
                        match response.json::<Library>().await {
                            Ok(mut records) => {
                                library.records.append(records.records.borrow_mut());
                                match records.response_metadata {
                                    None => {
                                        break;
                                    }
                                    Some(meta) => match meta.next_cursor {
                                        None => {
                                            break;
                                        }
                                        Some(curs) => {
                                            cursor = Some(curs);
                                        }
                                    },
                                }
                            }
                            Err(e) => {
                                println!("Error: {:?}", e);
                            }
                        }
                    } else {
                        println!(
                            "{} result: {}",
                            response.status(),
                            response.text().await.unwrap()
                        );
                    }
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            };
            if cursor.is_none() {
                break;
            }
        }
        Ok(library)
    }
}
