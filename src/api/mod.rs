use reqwest::{Client, Response, RequestBuilder};
use reqwest::header::HeaderMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::error::Error;
use std::fmt;
use std::collections::HashMap;
use std::borrow::BorrowMut;
use std::num::ParseIntError;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserData {
    pub access_token: Option<String>,
    pub expires_in: Option<i64>,
    pub expires_at: Option<DateTime<Utc>>,
    pub token_type: Option<String>,
    pub refresh_token: Option<String>,
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpicAsset {
    pub app_name: String,
    pub label_name: String,
    pub build_version: String,
    pub catalog_item_id: String,
    pub namespace: String,
    pub asset_id: String,
}


impl UserData {
    pub fn update(&mut self, new: UserData) {
        if let Some(n) = new.access_token { self.access_token = Some(n) }
        if let Some(n) = new.expires_in { self.expires_in = Some(n) }
        if let Some(n) = new.expires_at { self.expires_at = Some(n) }
        if let Some(n) = new.token_type { self.token_type = Some(n) }
        if let Some(n) = new.refresh_token { self.refresh_token = Some(n) }
        if let Some(n) = new.refresh_expires { self.refresh_expires = Some(n) }
        if let Some(n) = new.refresh_expires_at { self.refresh_expires_at = Some(n) }
        if let Some(n) = new.account_id { self.account_id = Some(n) }
        if let Some(n) = new.client_id { self.client_id = Some(n) }
        if let Some(n) = new.internal_client { self.internal_client = Some(n) }
        if let Some(n) = new.client_service { self.client_service = Some(n) }
        if let Some(n) = new.display_name { self.display_name = Some(n) }
        if let Some(n) = new.app { self.app = Some(n) }
        if let Some(n) = new.in_app_id { self.in_app_id = Some(n) }
        if let Some(n) = new.device_id { self.device_id = Some(n) }
        if let Some(n) = new.error_message { self.error_message = Some(n) }
        if let Some(n) = new.error_code { self.error_code = Some(n) }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetManifest {
    pub elements: Vec<Element>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Element {
    pub app_name: String,
    pub label_name: String,
    pub build_version: String,
    pub hash: String,
    pub manifests: Vec<Manifest>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub uri: String,
    pub query_params: Vec<QueryParam>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryParam {
    pub name: String,
    pub value: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub long_description: String,
    pub technical_details: String,
    pub key_images: Vec<KeyImage>,
    pub categories: Vec<Category>,
    pub namespace: String,
    pub status: String,
    pub creation_date: String,
    pub last_modified_date: String,
    pub entitlement_name: String,
    pub entitlement_type: String,
    pub item_type: String,
    pub release_info: Vec<ReleaseInfo>,
    pub developer: String,
    pub developer_id: String,
    pub end_of_support: bool,
    pub unsearchable: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyImage {
    #[serde(rename = "type")]
    pub type_field: String,
    pub url: String,
    pub md5: String,
    pub width: i64,
    pub height: i64,
    pub size: i64,
    pub uploaded_date: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub path: String,
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseInfo {
    pub id: Option<String>,
    pub app_id: Option<String>,
    pub compatible_apps: Vec<String>,
    pub platform: Vec<String>,
    pub date_added: Option<String>,
    pub release_note: Option<String>,
    pub version_title: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameToken {
    pub expires_in_seconds: i64,
    pub code: String,
    pub creating_client_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OwnershipToken {
    pub token: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entitlement {
    pub id: String,
    pub entitlement_name: String,
    pub namespace: String,
    pub catalog_item_id: String,
    pub account_id: String,
    pub identity_id: String,
    pub entitlement_type: String,
    pub grant_date: String,
    pub consumable: bool,
    pub status: String,
    pub active: bool,
    pub use_count: i64,
    pub created: String,
    pub updated: String,
    pub group_entitlement: bool,
    pub original_use_count: Option<i64>,
    pub platform_type: Option<String>,
    pub country: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub records: Vec<Record>,
    pub response_metadata: Option<ResponseMetadata>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub app_name: String,
    pub catalog_item_id: String,
    pub namespace: String,
    pub product_id: String,
    pub sandbox_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMetadata {
    pub next_cursor: Option<String>,
}

pub(crate) struct EpicAPI {
    client: Client,
    pub(crate) user_data: UserData,
}

#[derive(Debug)]
pub enum EpicAPIError {
    InvalidCredentials,
    APIError(String),
    Unknown,
    Server,
}

impl fmt::Display for EpicAPIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            EpicAPIError::InvalidCredentials => { write!(f, "Invalid Credentials") }
            EpicAPIError::Unknown => { write!(f, "Unknown Error") }
            EpicAPIError::Server => { write!(f, "Server Error") }
            EpicAPIError::APIError(e) => { write!(f, "API Error: {}", e) }
        }
    }
}

impl Error for EpicAPIError {
    fn description(&self) -> &str {
        match *self {
            EpicAPIError::InvalidCredentials => { "Invalid Credentials" }
            EpicAPIError::Unknown => { "Unknown Error" }
            EpicAPIError::Server => { "Server Error" }
            EpicAPIError::APIError(_) => { "API Error" }
        }
    }
}

impl EpicAPI {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", "UELauncher/12.0.5-15338009+++Portal+Release-Live Windows/6.1.7601.1.0.64bit".parse().unwrap());
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true).build().unwrap();
        EpicAPI { client, user_data: Default::default() }
    }

    pub fn blob_to_num(str: String) -> u64 {
        let mut num: u64 = 0;
        let mut shift: u64 = 0;
        for i in (0..str.len()).step_by(3) {
            if let Ok(n) = str[i..i + 3].parse::<u64>() {
                num += n << shift;
                shift += 8;
            }
        };
        return num;
    }

    pub async fn start_session(&mut self, refresh_token: Option<String>, exchange_token: Option<String>) -> Result<bool, EpicAPIError> {
        println!("{:016X}", EpicAPI::blob_to_num("215058166135082031144025".to_string()));
        let params = match refresh_token {
            None => {
                match exchange_token {
                    None => { return Err(EpicAPIError::InvalidCredentials); }
                    Some(exchange) => { [("grant_type".to_string(), "exchange_code".to_string()), ("exchange_code".to_string(), exchange), ("token_type".to_string(), "eg1".to_string())] }
                }
            }
            Some(refresh) => { [("grant_type".to_string(), "refresh_token".to_string()), ("refresh_token".to_string(), refresh), ("token_type".to_string(), "eg1".to_string())] }
        };

        match self.client
            .post("https://account-public-service-prod03.ol.epicgames.com/account/api/oauth/token")
            .form(&params)
            .basic_auth("34a02cf8f4414e29b15921876da36f9a", Some("daafbccc737745039dffe53d94fc76cf"))
            .send().await {
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

    fn get_authorized_get_client(&self, url: &str) -> RequestBuilder {
        self.set_authorization_header(self.client.get(url))
    }

    fn get_authorized_post_client(&self, url: &str) -> RequestBuilder {
        self.set_authorization_header(self.client.post(url))
    }

    fn get_authorized_delete_client(&self, url: &str) -> RequestBuilder {
        self.set_authorization_header(self.client.delete(url))
    }

    fn set_authorization_header(&self, rb: RequestBuilder) -> RequestBuilder {
        rb.header("Authorization", format!("{} {}", self.user_data.token_type.as_ref().unwrap_or(&"bearer".to_string()), self.user_data.access_token.as_ref().unwrap_or(&"".to_string())))
    }

    pub async fn resume_session(&mut self) -> Result<bool, EpicAPIError> {
        match self.get_authorized_get_client("https://account-public-service-prod03.ol.epicgames.com/account/api/oauth/verify")
            .send()
            .await {
            Ok(response) => {
                return self.handle_login_response(response).await;
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn get_assets(&mut self, platform: Option<String>, label: Option<String>) -> Result<Vec<EpicAsset>, EpicAPIError> {
        let plat = platform.unwrap_or("Windows".to_string());
        let lab = label.unwrap_or("Live".to_string());
        let url = format!("https://launcher-public-service-prod06.ol.epicgames.com/launcher/api/public/assets/{}?label={}", plat, lab);
        match self.get_authorized_get_client(&url)
            .send()
            .await {
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
                    println!("{} result: {}", response.status(), response.text().await.unwrap());
                    Err(EpicAPIError::Unknown)
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn get_asset_manifest(&self, platform: Option<String>, label: Option<String>, asset: EpicAsset) -> Result<AssetManifest, EpicAPIError> {
        let url = format!("https://launcher-public-service-prod06.ol.epicgames.com/launcher/api/public/assets/v2/platform/{}/namespace/{}/catalogItem/{}/app/{}/label/{}",
                          platform.unwrap_or("Windows".to_string()), asset.namespace, asset.catalog_item_id, asset.app_name, label.unwrap_or("Live".to_string()));
        match self.get_authorized_get_client(&url).send().await {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(manifest) => { Ok(manifest) }
                        Err(e) => {
                            println!("Error: {:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
                    }
                } else {
                    println!("{} result: {}", response.status(), response.text().await.unwrap());
                    Err(EpicAPIError::Unknown)
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn get_asset_info(&self, asset: EpicAsset) -> Result<HashMap<String, AssetInfo>, EpicAPIError> {
        let url = format!("https://catalog-public-service-prod06.ol.epicgames.com/catalog/api/shared/namespace/{}/bulk/items?id={}&includeDLCDetails=true&includeMainGameDetails=true&country=us&locale=lc",
                          asset.namespace, asset.catalog_item_id);
        match self.get_authorized_get_client(&url).send().await {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(info) => { Ok(info) }
                        Err(e) => {
                            println!("Error: {:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
                    }
                } else {
                    println!("{} result: {}", response.status(), response.text().await.unwrap());
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
        let url = format!("https://account-public-service-prod03.ol.epicgames.com/account/api/oauth/exchange");
        match self.get_authorized_get_client(&url).send().await {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(token) => { Ok(token) }
                        Err(e) => {
                            println!("Error: {:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
                    }
                } else {
                    println!("{} result: {}", response.status(), response.text().await.unwrap());
                    Err(EpicAPIError::Unknown)
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn get_ownership_token(&self, asset: EpicAsset) -> Result<OwnershipToken, EpicAPIError> {
        let url = match &self.user_data.account_id {
            None => { return Err(EpicAPIError::InvalidCredentials); }
            Some(id) => {
                format!("https://ecommerceintegration-public-service-ecomprod02.ol.epicgames.com/ecommerceintegration/api/public/platforms/EPIC/identities/{}/ownershipToken",
                        id)
            }
        };
        match self.get_authorized_post_client(&url)
            .form(&[("nsCatalogItemId".to_string(), format!("{}:{}", asset.namespace, asset.catalog_item_id))])
            .send().await {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(token) => { Ok(token) }
                        Err(e) => {
                            println!("Error: {:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
                    }
                } else {
                    println!("{} result: {}", response.status(), response.text().await.unwrap());
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
            None => { return Err(EpicAPIError::InvalidCredentials); }
            Some(id) => {
                format!("https://entitlement-public-service-prod08.ol.epicgames.com/entitlement/api/account/{}/entitlements?start=0&count=5000",
                        id)
            }
        };
        match self.get_authorized_get_client(&url).send().await {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(ent) => { Ok(ent) }
                        Err(e) => {
                            println!("Error: {:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
                    }
                } else {
                    println!("{} result: {}", response.status(), response.text().await.unwrap());
                    Err(EpicAPIError::Unknown)
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn get_library_items(&mut self, include_metadata: bool) -> Result<Library, EpicAPIError> {
        let mut library = Library { records: vec![], response_metadata: Default::default() };
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

            match self.get_authorized_get_client(&url)
                .send()
                .await {
                Ok(response) => {
                    if response.status() == reqwest::StatusCode::OK {
                        match response.json::<Library>().await {
                            Ok(mut records) => {
                                library.records.append(records.records.borrow_mut());
                                match records.response_metadata {
                                    None => { break; }
                                    Some(meta) => {
                                        match meta.next_cursor {
                                            None => { break; }
                                            Some(curs) => {
                                                cursor = Some(curs);
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Error: {:?}", e);
                            }
                        }
                    } else {
                        println!("{} result: {}", response.status(), response.text().await.unwrap());
                    }
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            };
            if cursor.is_none() {
                break;
            }
        };
        Ok(library)
    }
}