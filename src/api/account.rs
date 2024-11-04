use crate::api::error::EpicAPIError;
use crate::api::types::account::{AccountData, AccountInfo};
use crate::api::types::friends::Friend;
use crate::api::EpicAPI;
use log::{error, warn};
use url::Url;
use crate::api::types::entitlement::Entitlement;

impl EpicAPI {
    pub async fn account_details(&mut self) -> Result<AccountData, EpicAPIError> {
        let id = match &self.user_data.account_id {
            Some(id) => id,
            None => return Err(EpicAPIError::InvalidParams),
        };
        let url = format!(
            "https://account-public-service-prod03.ol.epicgames.com/account/api/public/account/{}",
            id
        );
        match self
            .authorized_get_client(Url::parse(&url).unwrap())
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(details) => Ok(details),
                        Err(e) => {
                            error!("{:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
                    }
                } else {
                    warn!(
                        "{} result: {}",
                        response.status(),
                        response.text().await.unwrap()
                    );
                    Err(EpicAPIError::Unknown)
                }
            }
            Err(e) => {
                error!("{:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn account_ids_details(
        &mut self,
        ids: Vec<String>,
    ) -> Result<Vec<AccountInfo>, EpicAPIError> {
        if ids.is_empty() {
            return Err(EpicAPIError::InvalidParams);
        }
        let url =
            "https://account-public-service-prod03.ol.epicgames.com/account/api/public/account"
                .to_string();
        let mut parsed_url = Url::parse(&url).unwrap();
        let mut query = "accountId=".to_string();
        query.push_str(&ids.join("&accountId="));
        parsed_url.set_query(Some(&query));
        match self.authorized_get_client(parsed_url).send().await {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(details) => Ok(details),
                        Err(e) => {
                            error!("{:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
                    }
                } else {
                    warn!(
                        "{} result: {}",
                        response.status(),
                        response.text().await.unwrap()
                    );
                    Err(EpicAPIError::Unknown)
                }
            }
            Err(e) => {
                error!("{:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn account_friends(
        &mut self,
        include_pending: bool,
    ) -> Result<Vec<Friend>, EpicAPIError> {
        let id = match &self.user_data.account_id {
            Some(id) => id,
            None => return Err(EpicAPIError::InvalidParams),
        };
        let url = format!(
            "https://friends-public-service-prod06.ol.epicgames.com/friends/api/public/friends/{}?includePending={}", id, include_pending);
        match self
            .authorized_get_client(Url::parse(&url).unwrap())
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(details) => Ok(details),
                        Err(e) => {
                            error!("{:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
                    }
                } else {
                    warn!(
                        "{} result: {}",
                        response.status(),
                        response.text().await.unwrap()
                    );
                    Err(EpicAPIError::Unknown)
                }
            }
            Err(e) => {
                error!("{:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn user_entitlements(&self) -> Result<Vec<Entitlement>, EpicAPIError> {
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
            .authorized_get_client(Url::parse(&url).unwrap())
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(ent) => Ok(ent),
                        Err(e) => {
                            error!("{:?}", e);
                            Err(EpicAPIError::Unknown)
                        }
                    }
                } else {
                    warn!(
                        "{} result: {}",
                        response.status(),
                        response.text().await.unwrap()
                    );
                    Err(EpicAPIError::Unknown)
                }
            }
            Err(e) => {
                error!("{:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }
}
