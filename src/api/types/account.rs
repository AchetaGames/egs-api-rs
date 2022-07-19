use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Structure that holds all account data
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountData {
    pub age_group: String,
    pub can_update_display_name: bool,
    pub company: String,
    pub country: String,
    pub display_name: String,
    pub email: String,
    pub email_verified: bool,
    pub failed_login_attempts: i64,
    pub headless: bool,
    pub id: String,
    pub last_display_name_change: String,
    pub last_login: String,
    pub last_name: String,
    pub minor_expected: bool,
    pub minor_status: String,
    pub minor_verified: bool,
    pub name: String,
    pub number_of_display_name_changes: i64,
    pub preferred_language: String,
    pub tfa_enabled: bool,
}

/// Structure that holds all user data
///
/// Needed for login
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserData {
    pub(crate) access_token: Option<String>,
    pub expires_in: Option<i64>,
    pub expires_at: Option<DateTime<Utc>>,
    pub token_type: Option<String>,
    pub(crate) refresh_token: Option<String>,
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
    /// Creates new UserData Structure
    pub fn new() -> Self {
        UserData {
            access_token: None,
            expires_in: None,
            expires_at: None,
            token_type: None,
            refresh_token: None,
            refresh_expires: None,
            refresh_expires_at: None,
            account_id: None,
            client_id: None,
            internal_client: None,
            client_service: None,
            display_name: None,
            app: None,
            in_app_id: None,
            device_id: None,
            error_message: None,
            error_code: None,
        }
    }

    /// Get access token
    pub fn access_token(&self) -> Option<String> {
        self.access_token.clone()
    }

    /// Get refresh token
    pub fn refresh_token(&self) -> Option<String> {
        self.refresh_token.clone()
    }

    /// Set access token
    pub fn set_access_token(&mut self, token: Option<String>) {
        self.access_token = token;
    }

    /// Set refresh token
    pub fn set_refresh_token(&mut self, token: Option<String>) {
        self.refresh_token = token;
    }

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

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub display_name: String,
    pub external_auths: HashMap<String, ExternalAuth>,
    pub id: String,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAuth {
    pub account_id: String,
    pub auth_ids: Vec<AuthId>,
    pub date_added: Option<String>,
    pub avatar: Option<String>,
    pub external_auth_id: Option<String>,
    pub external_auth_id_type: String,
    pub external_display_name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub external_auth_secondary_id: Option<String>,
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthId {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
}
