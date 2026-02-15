use log::{error, info, warn};
use reqwest::Response;
use crate::api::EpicAPI;
use crate::api::error::EpicAPIError;
use crate::api::types::account::UserData;

impl EpicAPI {
    /// Start a new OAuth session with exchange token, authorization code, or refresh token.
    pub async fn start_session(
        &mut self,
        exchange_token: Option<String>,
        authorization_code: Option<String>,
    ) -> Result<bool, EpicAPIError> {
        let params = match exchange_token {
            None => match authorization_code {
                None => [
                    ("grant_type".to_string(), "refresh_token".to_string()),
                    (
                        "refresh_token".to_string(),
                        self.user_data
                            .refresh_token
                            .as_ref()
                            .ok_or(EpicAPIError::InvalidCredentials)?
                            .clone(),
                    ),
                    ("token_type".to_string(), "eg1".to_string()),
                ],
                Some(auth) => [
                    ("grant_type".to_string(), "authorization_code".to_string()),
                    ("code".to_string(), auth),
                    ("token_type".to_string(), "eg1".to_string()),
                ],
            },
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
            Ok(response) => self.handle_login_response(response).await,
            Err(e) => {
                error!("{:?}", e);
                Err(EpicAPIError::NetworkError(e))
            }
        }
    }

    /// Handle the OAuth login response and update user data.
    async fn handle_login_response(&mut self, response: Response) -> Result<bool, EpicAPIError> {
        if response.status() == reqwest::StatusCode::INTERNAL_SERVER_ERROR {
            error!("Server Error");
            return Err(EpicAPIError::Server);
        }
        let new: UserData = match response.json().await {
            Ok(data) => data,
            Err(e) => {
                error!("{:?}", e);
                return Err(EpicAPIError::DeserializationError(format!("{}", e)));
            }
        };

        self.user_data.update(new);

        if let Some(m) = &self.user_data.error_message {
            error!("{}", m);
            return Err(EpicAPIError::APIError(m.to_string()));
        }
        Ok(true)
    }

    /// Resume an existing session by verifying the access token.
    pub async fn resume_session(&mut self) -> Result<bool, EpicAPIError> {
        let url = "https://account-public-service-prod03.ol.epicgames.com/account/api/oauth/verify";
        match self.authorized_get_client(
            url::Url::parse(url).map_err(|_| EpicAPIError::InvalidParams)?
        ).send().await {
            Ok(response) => {
                self.handle_login_response(response).await
            }
            Err(e) => {
                error!("{:?}", e);
                Err(EpicAPIError::NetworkError(e))
            }
        }
    }

    /// Start a client credentials session (app-level auth, no user context).
    pub async fn start_client_credentials_session(&mut self) -> Result<bool, EpicAPIError> {
        let params = [
            ("grant_type".to_string(), "client_credentials".to_string()),
            ("token_type".to_string(), "eg1".to_string()),
        ];

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
            Ok(response) => self.handle_login_response(response).await,
            Err(e) => {
                error!("{:?}", e);
                Err(EpicAPIError::NetworkError(e))
            }
        }
    }

    /// Authenticate via session ID (SID) from the Epic web login flow.
    ///
    /// Performs the web-based exchange: set-sid -> csrf -> exchange/generate,
    /// then uses the resulting exchange code to start a session.
    pub async fn auth_sid(&mut self, sid: &str) -> Result<bool, EpicAPIError> {
        let set_sid_url = format!(
            "https://www.epicgames.com/id/api/set-sid?sid={}",
            sid
        );
        self.client
            .get(&set_sid_url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) EpicGamesLauncher/17.0.1 UnrealEngine/4.23.0 Chrome/84.0.4147.38 Safari/537.36",
            )
            .send()
            .await
            .map_err(|e| {
                error!("{:?}", e);
                EpicAPIError::NetworkError(e)
            })?;

        let csrf_response = self
            .client
            .get("https://www.epicgames.com/id/api/csrf")
            .send()
            .await
            .map_err(|e| {
                error!("{:?}", e);
                EpicAPIError::NetworkError(e)
            })?;

        let xsrf_token = csrf_response
            .cookies()
            .find(|c| c.name() == "XSRF-TOKEN")
            .map(|c| c.value().to_string())
            .ok_or_else(|| {
                error!("XSRF-TOKEN cookie not found");
                EpicAPIError::InvalidCredentials
            })?;

        let exchange_response = self
            .client
            .post("https://www.epicgames.com/id/api/exchange/generate")
            .header("X-XSRF-TOKEN", &xsrf_token)
            .send()
            .await
            .map_err(|e| {
                error!("{:?}", e);
                EpicAPIError::NetworkError(e)
            })?;

        if exchange_response.status() != reqwest::StatusCode::OK {
            let body = exchange_response.text().await.unwrap_or_default();
            error!("Exchange code generation failed: {}", body);
            return Err(EpicAPIError::APIError(
                "Exchange code generation failed".to_string(),
            ));
        }

        let exchange_data: crate::api::types::exchange_code::ExchangeCode =
            exchange_response.json().await.map_err(|e| {
                error!("{:?}", e);
                EpicAPIError::DeserializationError(format!("{}", e))
            })?;

        let code = exchange_data
            .code
            .ok_or(EpicAPIError::InvalidCredentials)?;

        self.start_session(Some(code), None).await
    }

    /// Invalidate the current session (note: method name has a known typo).
    pub async fn invalidate_sesion(&mut self) -> bool {
        if let Some(access_token) = &self.user_data.access_token {
            let url = format!("https://account-public-service-prod03.ol.epicgames.com/account/api/oauth/sessions/kill/{}", access_token);
            match self.client.delete(&url).send().await {
                Ok(_) => {
                    info!("Session invalidated");
                    return true;
                }
                Err(e) => {
                    warn!("Unable to invalidate session: {}", e)
                }
            }
        };
        false
    }
}
