use reqwest::header::HeaderMap;
use reqwest::{Client, ClientBuilder, RequestBuilder};
use types::account::UserData;
use url::Url;

/// Module holding the API types
pub mod types;

/// Various API Utils
pub mod utils;

/// Error type
pub mod error;

/// Fab Methods
pub mod fab;

///Account methods
pub mod account;

/// EGS Methods
pub mod egs;
/// Session Handling
pub mod login;

#[derive(Default, Debug, Clone)]
pub(crate) struct EpicAPI {
    client: Client,
    pub(crate) user_data: UserData,
}

impl EpicAPI {
    pub fn new() -> Self {
        let client = EpicAPI::build_client().build().unwrap();
        EpicAPI {
            client,
            user_data: Default::default(),
        }
    }

    fn build_client() -> ClientBuilder {
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            "UELauncher/17.0.1-37584233+++Portal+Release-Live Windows/10.0.19043.1.0.64bit"
                .parse()
                .unwrap(),
        );
        headers.insert(
            "X-Epic-Correlation-ID",
            "UE4-c176f7154c2cda1061cc43ab52598e2b-93AFB486488A22FDF70486BD1D883628-BFCD88F649E997BA203FF69F07CE578C".parse().unwrap()
        );
        reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
    }

    fn authorized_get_client(&self, url: Url) -> RequestBuilder {
        let client = EpicAPI::build_client().build().unwrap();
        self.set_authorization_header(client.get(url))
    }

    fn authorized_post_client(&self, url: Url) -> RequestBuilder {
        let client = EpicAPI::build_client().build().unwrap();
        self.set_authorization_header(client.post(url))
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

    
}
