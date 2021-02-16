use reqwest::{Client, Response};
use reqwest::header::HeaderMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::error::Error;
use std::fmt;

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
        headers.insert("User-Agent", "UELauncher/11.0.1-14907503+++Portal+Release-Live Windows/10.0.19041.1.256.64bit".parse().unwrap());
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true).build().unwrap();
        EpicAPI { client, user_data: Default::default() }
    }

    pub async fn start_session(&mut self, refresh_token: Option<String>, exchange_token: Option<String>) -> Result<bool, EpicAPIError> {
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

    pub async fn resume_session(&mut self) -> Result<bool, EpicAPIError> {
        match self.client
            .get("https://account-public-service-prod03.ol.epicgames.com/account/api/oauth/verify")
            .header("Authorization", format!("{} {}", self.user_data.token_type.as_ref().unwrap_or(&"bearer".to_string()), self.user_data.access_token.as_ref().unwrap_or(&"".to_string())))
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
        match self.client
            .get(&url)
            .header("Authorization", format!("{} {}", self.user_data.token_type.as_ref().unwrap_or(&"bearer".to_string()), self.user_data.access_token.as_ref().unwrap_or(&"".to_string())))
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
}