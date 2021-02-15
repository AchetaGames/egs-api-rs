use reqwest::{Client};
use reqwest::header::HeaderMap;
use serde::{Serialize, Deserialize};


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserData {
    pub access_token: String,
    pub expires_in: i64,
    pub expires_at: String,
    pub token_type: String,
    pub refresh_token: String,
    pub refresh_expires: i64,
    pub refresh_expires_at: String,
    pub account_id: String,
    pub client_id: String,
    pub internal_client: bool,
    pub client_service: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub app: String,
    pub in_app_id: String,
    pub device_id: String,
}


pub(crate) struct EpicAPI {
    client: Client,
    user_data: UserData,
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

    pub async fn start_session(&mut self, refresh_token: Option<String>, exchange_token: Option<String>) -> bool {
        let params = match refresh_token {
            None => {
                match exchange_token {
                    None => { return false; }
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
            Ok(result) => {
                if result.status() == reqwest::StatusCode::INTERNAL_SERVER_ERROR {
                    println!("Server Error");
                    return false;
                }
                self.user_data = match result.json().await {
                    Ok(data) => data,
                    Err(e) => {
                        println!("Error: {:?}", e);
                        return false;
                    }
                };
                true
            }
            Err(e) => {
                println!("Error: {:?}", e);
                false
            }
        }
    }
}