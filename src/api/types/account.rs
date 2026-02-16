use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub fn access_token(&self) -> Option<&str> {
        self.access_token.as_deref()
    }

    /// Get refresh token
    pub fn refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_deref()
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

/// Account info returned by bulk account ID lookup.
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
    pub external_auth_id_type: Option<String>,
    pub external_display_name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub external_auth_secondary_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_data_new_defaults() {
        let ud = UserData::new();
        assert_eq!(ud.access_token, None);
        assert_eq!(ud.refresh_token, None);
        assert_eq!(ud.account_id, None);
        assert_eq!(ud.display_name, None);
        assert_eq!(ud.expires_at, None);
    }

    #[test]
    fn user_data_update_merges_some_fields() {
        let mut ud = UserData::new();
        ud.access_token = Some("old_token".to_string());
        ud.account_id = Some("id123".to_string());

        let mut new_ud = UserData::new();
        new_ud.access_token = Some("new_token".to_string());
        // account_id left as None

        ud.update(new_ud);
        assert_eq!(ud.access_token, Some("new_token".to_string()));
        assert_eq!(ud.account_id, Some("id123".to_string()));
    }

    #[test]
    fn user_data_update_all_none_preserves() {
        let mut ud = UserData::new();
        ud.access_token = Some("token".to_string());
        ud.account_id = Some("id".to_string());
        ud.display_name = Some("user".to_string());

        let original = ud.clone();
        ud.update(UserData::new());
        assert_eq!(ud, original);
    }

    #[test]
    fn user_data_serialization_roundtrip() {
        let mut ud = UserData::new();
        ud.access_token = Some("tok123".to_string());
        ud.display_name = Some("TestUser".to_string());
        ud.account_id = Some("acc456".to_string());

        let json = serde_json::to_string(&ud).unwrap();
        let deserialized: UserData = serde_json::from_str(&json).unwrap();
        assert_eq!(ud, deserialized);
    }

    #[test]
    fn user_data_access_token_getters() {
        let mut ud = UserData::new();
        assert_eq!(ud.access_token(), None);
        ud.set_access_token(Some("my_token".to_string()));
        assert_eq!(ud.access_token(), Some("my_token"));
    }

    #[test]
    fn user_data_refresh_token_getters() {
        let mut ud = UserData::new();
        assert_eq!(ud.refresh_token(), None);
        ud.set_refresh_token(Some("refresh_tok".to_string()));
        assert_eq!(ud.refresh_token(), Some("refresh_tok"));
    }

    #[test]
    fn deserialize_token_verification() {
        let json = r#"{"token":"abc123","sessionId":"sess1","tokenType":"bearer","clientId":"34a02cf8f4414e29b15921876da36f9a","internalClient":true,"clientService":"launcher","accountId":"8645b4947bbc4c0092a8b7236df169d1","expiresIn":28800,"expiresAt":"2026-02-16T20:00:00.000Z","authMethod":"exchange_code","displayName":"TestUser","app":"launcher","inAppId":"8645b4947bbc4c0092a8b7236df169d1","deviceId":"device1","perms":[{"resource":"account:public:account","action":1}]}"#;
        let v: TokenVerification = serde_json::from_str(json).unwrap();
        assert_eq!(
            v.account_id,
            Some("8645b4947bbc4c0092a8b7236df169d1".to_string())
        );
        assert_eq!(v.display_name, Some("TestUser".to_string()));
        assert_eq!(v.auth_method, Some("exchange_code".to_string()));
        let perms = v.perms.unwrap();
        assert_eq!(perms.len(), 1);
        assert_eq!(
            perms[0].resource,
            Some("account:public:account".to_string())
        );
        assert_eq!(perms[0].action, Some(1));
    }

    #[test]
    fn deserialize_token_verification_no_perms() {
        let json = r#"{"token":"abc","sessionId":"s1","tokenType":"bearer","clientId":"cid","internalClient":false,"clientService":"launcher","accountId":"aid","expiresIn":100,"expiresAt":"2026-01-01T00:00:00.000Z","authMethod":"refresh_token","displayName":"User","app":"launcher","inAppId":null,"deviceId":null,"perms":null}"#;
        let v: TokenVerification = serde_json::from_str(json).unwrap();
        assert_eq!(v.account_id, Some("aid".to_string()));
        assert!(v.perms.is_none());
        assert!(v.in_app_id.is_none());
    }
}

#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthId {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

/// Response from `GET /account/api/oauth/verify` — token introspection.
///
/// Returns details about the current OAuth token including account info,
/// client info, expiration times, and optionally granted permissions.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenVerification {
    pub token: Option<String>,
    pub session_id: Option<String>,
    pub token_type: Option<String>,
    pub client_id: Option<String>,
    pub internal_client: Option<bool>,
    pub client_service: Option<String>,
    pub account_id: Option<String>,
    pub expires_in: Option<i64>,
    pub expires_at: Option<String>,
    pub auth_method: Option<String>,
    pub display_name: Option<String>,
    pub app: Option<String>,
    pub in_app_id: Option<String>,
    pub device_id: Option<String>,
    pub perms: Option<Vec<TokenPermission>>,
}

/// A single permission entry from token verification.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPermission {
    pub resource: Option<String>,
    pub action: Option<u32>,
}
