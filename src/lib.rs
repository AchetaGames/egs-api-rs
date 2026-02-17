#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

//! # Epic Games Store API
//!
//! Async Rust client for the Epic Games Store API. Provides authentication,
//! asset management, download manifest parsing (binary + JSON), and
//! [Fab](https://www.fab.com/) marketplace integration.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use egs_api::EpicGames;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut egs = EpicGames::new();
//!
//!     // Authenticate with an authorization code
//!     let code = "your_authorization_code".to_string();
//!     if egs.auth_code(None, Some(code)).await {
//!         println!("Logged in as {}", egs.user_details().display_name.unwrap_or_default());
//!     }
//!
//!     // List all owned assets
//!     let assets = egs.list_assets(None, None).await;
//!     println!("You own {} assets", assets.len());
//! }
//! ```
//!
//! # Authentication
//!
//! Epic uses OAuth2 with a public launcher client ID. The flow is:
//!
//! 1. Open the [authorization URL] in a browser — the user logs in and gets
//!    redirected to a JSON page with an `authorizationCode`.
//! 2. Pass that code to [`EpicGames::auth_code`].
//! 3. Persist the session with [`EpicGames::user_details`] (implements
//!    `Serialize` / `Deserialize`).
//! 4. Restore it later with [`EpicGames::set_user_details`] +
//!    [`EpicGames::login`], which uses the refresh token.
//!
//! [authorization URL]: https://www.epicgames.com/id/login?redirectUrl=https%3A%2F%2Fwww.epicgames.com%2Fid%2Fapi%2Fredirect%3FclientId%3D34a02cf8f4414e29b15921876da36f9a%26responseType%3Dcode
//!
//! # Features
//!
//! - **Assets** — List owned assets, fetch catalog metadata (with DLC trees),
//!   retrieve asset manifests with CDN download URLs.
//! - **Download Manifests** — Parse Epic's binary and JSON manifest formats.
//!   Exposes file lists, chunk hashes, and custom fields for download
//!   reconstruction.
//! - **Fab Marketplace** — List Fab library items, fetch signed asset manifests,
//!   and download manifests from distribution points.
//! - **Account** — Details, bulk ID lookup, friends list.
//! - **Entitlements** — Games, DLC, subscriptions.
//! - **Library** — Paginated listing with optional metadata.
//! - **Tokens** — Game exchange tokens and per-asset ownership tokens (JWT).
//!
//! # Architecture
//!
//! [`EpicGames`] is the public facade. It wraps an internal `EpicAPI` struct
//! that holds the `reqwest::Client` (with cookie store) and session state.
//! Most public methods return `Option<T>` or `Vec<T>`, swallowing transport
//! errors for convenience. Fab methods return `Result<T, EpicAPIError>` to
//! expose timeout/error distinctions.
//!
//! # Examples
//!
//! The crate ships with examples covering every endpoint. See the
//! [`examples/`](https://github.com/AchetaGames/egs-api-rs/tree/master/examples)
//! directory or run:
//!
//! ```bash
//! cargo run --example auth                # Interactive login + token persistence
//! cargo run --example account             # Account details, ID lookup, friends, external auths, SSO
//! cargo run --example entitlements        # List all entitlements
//! cargo run --example library             # Paginated library listing
//! cargo run --example assets              # Full pipeline: list → info → manifest → download
//! cargo run --example game_token          # Exchange code + ownership token
//! cargo run --example fab                 # Fab library → asset manifest → download manifest
//! cargo run --example catalog             # Catalog items, offers, bulk lookup
//! cargo run --example commerce            # Currencies, prices, billing, quick purchase
//! cargo run --example status              # Service status (lightswitch API)
//! cargo run --example presence            # Update online presence
//! cargo run --example client_credentials  # App-level auth + library state tokens
//! ```

use crate::api::EpicAPI;

/// Module for authenticated API communication
pub mod api;

mod facade;

/// Client for the Epic Games Store API.
///
/// This is the main entry point for the library. Create an instance with
/// [`EpicGames::new`], authenticate with [`EpicGames::auth_code`] or
/// [`EpicGames::login`], then call API methods.
///
/// Most methods return `Option<T>` or `Vec<T>`, returning `None` / empty on
/// errors. Fab methods return `Result<T, EpicAPIError>` for richer error
/// handling (e.g., distinguishing timeouts from other failures).
///
/// Session state is stored in [`UserData`](crate::api::types::account::UserData),
/// which implements `Serialize` / `Deserialize` for persistence across runs.
#[derive(Debug, Clone)]
pub struct EpicGames {
    egs: EpicAPI,
}

impl Default for EpicGames {
    fn default() -> Self {
        Self::new()
    }
}

impl EpicGames {
    /// Creates a new [`EpicGames`] client.
    pub fn new() -> Self {
        EpicGames {
            egs: EpicAPI::new(),
        }
    }
}

#[cfg(test)]
mod facade_tests {
    use super::*;
    use crate::api::types::account::UserData;
    use chrono::{Duration, Utc};

    #[test]
    fn new_creates_instance() {
        let egs = EpicGames::new();
        assert!(!egs.is_logged_in());
    }

    #[test]
    fn default_same_as_new() {
        let egs = EpicGames::default();
        assert!(!egs.is_logged_in());
    }

    #[test]
    fn user_details_default_empty() {
        let egs = EpicGames::new();
        assert!(egs.user_details().access_token.is_none());
    }

    #[test]
    fn set_and_get_user_details() {
        let mut egs = EpicGames::new();
        let mut ud = UserData::new();
        ud.display_name = Some("TestUser".to_string());
        egs.set_user_details(ud);
        assert_eq!(egs.user_details().display_name, Some("TestUser".to_string()));
    }

    #[test]
    fn is_logged_in_expired() {
        let mut egs = EpicGames::new();
        let mut ud = UserData::new();
        ud.expires_at = Some(Utc::now() - Duration::hours(1));
        egs.set_user_details(ud);
        assert!(!egs.is_logged_in());
    }

    #[test]
    fn is_logged_in_valid() {
        let mut egs = EpicGames::new();
        let mut ud = UserData::new();
        ud.expires_at = Some(Utc::now() + Duration::hours(2));
        egs.set_user_details(ud);
        assert!(egs.is_logged_in());
    }

    #[test]
    fn is_logged_in_within_600s_threshold() {
        let mut egs = EpicGames::new();
        let mut ud = UserData::new();
        ud.expires_at = Some(Utc::now() + Duration::seconds(500));
        egs.set_user_details(ud);
        assert!(!egs.is_logged_in());
    }
}
