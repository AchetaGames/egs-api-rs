use crate::api::error::EpicAPIError;
use crate::api::types::account::{TokenVerification, UserData};
use crate::EpicGames;
use log::{error, info, warn};

impl EpicGames {
    /// Check whether the user is logged in.
    ///
    /// Returns `true` if the access token exists and has more than 600 seconds
    /// remaining before expiry.
    pub fn is_logged_in(&self) -> bool {
        if let Some(exp) = self.egs.user_data.expires_at {
            let now = chrono::offset::Utc::now();
            let td = exp - now;
            if td.num_seconds() > 600 {
                return true;
            }
        }
        false
    }

    /// Returns a clone of the current session state.
    ///
    /// The returned [`UserData`] implements `Serialize` / `Deserialize`,
    /// so you can persist it to disk and restore it later with
    /// [`set_user_details`](Self::set_user_details).
    pub fn user_details(&self) -> UserData {
        self.egs.user_data.clone()
    }

    /// Restore session state from a previously saved [`UserData`].
    ///
    /// Only merges `Some` fields — existing values are preserved for any
    /// field that is `None` in the input. Call [`login`](Self::login)
    /// afterward to refresh the access token.
    pub fn set_user_details(&mut self, user_details: UserData) {
        self.egs.user_data.update(user_details);
    }

    /// Like [`auth_code`](Self::auth_code), but returns a `Result` instead of swallowing errors.
    pub async fn try_auth_code(
        &mut self,
        exchange_token: Option<String>,
        authorization_code: Option<String>,
    ) -> Result<bool, EpicAPIError> {
        self.egs
            .start_session(exchange_token, authorization_code)
            .await
    }

    /// Authenticate with an authorization code or exchange token.
    ///
    /// Returns `true` on success, `false` on failure. Returns `None` on API errors.
    pub async fn auth_code(
        &mut self,
        exchange_token: Option<String>,
        authorization_code: Option<String>,
    ) -> bool {
        self.try_auth_code(exchange_token, authorization_code)
            .await
            .unwrap_or(false)
    }

    /// Invalidate the current session and log out.
    pub async fn logout(&mut self) -> bool {
        self.egs.invalidate_sesion().await
    }

    /// Like [`login`](Self::login), but returns a `Result` instead of swallowing errors.
    pub async fn try_login(&mut self) -> Result<bool, EpicAPIError> {
        if let Some(exp) = self.egs.user_data.expires_at {
            let now = chrono::offset::Utc::now();
            let td = exp - now;
            if td.num_seconds() > 600 {
                info!("Trying to re-use existing login session... ");
                let resumed = self.egs.resume_session().await.map_err(|e| {
                    warn!("{}", e);
                    e
                })?;
                if resumed {
                    info!("Logged in");
                    return Ok(true);
                }
                return Ok(false);
            }
        }
        info!("Logging in...");
        if let Some(exp) = self.egs.user_data.refresh_expires_at {
            let now = chrono::offset::Utc::now();
            let td = exp - now;
            if td.num_seconds() > 600 {
                let started = self.egs.start_session(None, None).await.map_err(|e| {
                    error!("{}", e);
                    e
                })?;
                if started {
                    info!("Logged in");
                    return Ok(true);
                }
                return Ok(false);
            }
        }
        Ok(false)
    }

    /// Resume session using the saved refresh token.
    ///
    /// Returns `true` on success, `false` if the refresh token has expired or is invalid.
    /// Unlike [`try_login`](Self::try_login), this method falls through to
    /// refresh-token login if session resume fails.
    pub async fn login(&mut self) -> bool {
        if let Some(exp) = self.egs.user_data.expires_at {
            let now = chrono::offset::Utc::now();
            let td = exp - now;
            if td.num_seconds() > 600 {
                info!("Trying to re-use existing login session... ");
                match self.egs.resume_session().await {
                    Ok(b) => {
                        if b {
                            info!("Logged in");
                            return true;
                        }
                        return false;
                    }
                    Err(e) => {
                        warn!("{}", e)
                    }
                };
            }
        }
        info!("Logging in...");
        if let Some(exp) = self.egs.user_data.refresh_expires_at {
            let now = chrono::offset::Utc::now();
            let td = exp - now;
            if td.num_seconds() > 600 {
                match self.egs.start_session(None, None).await {
                    Ok(b) => {
                        if b {
                            info!("Logged in");
                            return true;
                        }
                        return false;
                    }
                    Err(e) => {
                        error!("{}", e)
                    }
                }
            }
        }
        false
    }

    /// Like [`auth_client_credentials`](Self::auth_client_credentials), but returns a `Result` instead of swallowing errors.
    pub async fn try_auth_client_credentials(&mut self) -> Result<bool, EpicAPIError> {
        self.egs.start_client_credentials_session().await
    }

    /// Authenticate with client credentials (app-level, no user context).
    ///
    /// Uses the launcher's public client ID/secret to obtain an access token
    /// without any user interaction. The resulting session has limited
    /// permissions — it can query public endpoints (catalog, service status,
    /// currencies) but cannot access user-specific data (library, entitlements).
    ///
    /// Returns `true` on success, `false` on failure.
    pub async fn auth_client_credentials(&mut self) -> bool {
        self.try_auth_client_credentials().await.unwrap_or(false)
    }

    /// Authenticate via session ID (SID) from the Epic web login flow.
    ///
    /// Performs the multi-step web exchange: set-sid → CSRF → exchange code,
    /// then starts a session with the resulting code. Returns `true` on success.
    pub async fn auth_sid(&mut self, sid: &str) -> Result<bool, EpicAPIError> {
        self.egs.auth_sid(sid).await
    }

    /// Verify the current OAuth token and get account/session info.
    ///
    /// Returns `None` on any error.
    pub async fn verify_access_token(&self, include_perms: bool) -> Option<TokenVerification> {
        self.try_verify_access_token(include_perms).await.ok()
    }

    /// Like [`verify_access_token`](Self::verify_access_token), but returns a `Result` instead of swallowing errors.
    pub async fn try_verify_access_token(
        &self,
        include_perms: bool,
    ) -> Result<TokenVerification, EpicAPIError> {
        self.egs.verify_token(include_perms).await
    }
}
