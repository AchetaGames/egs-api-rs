use crate::api::types::account::{AccountData, AccountInfo};
use crate::api::types::epic_asset::EpicAsset;
use crate::api::types::fab_asset_manifest::DownloadInfo;
use crate::api::types::fab_library::FabLibrary;
use crate::api::types::friends::Friend;
use log::{debug, error, info, warn};
use reqwest::header::HeaderMap;
use reqwest::{Client, ClientBuilder, RequestBuilder, Response};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use types::account::UserData;
use types::asset_info::{AssetInfo, GameToken, OwnershipToken};
use types::asset_manifest::AssetManifest;
use types::download_manifest::DownloadManifest;
use types::entitlement::Entitlement;
use types::library::Library;
use url::Url;

/// Module holding the API types
pub mod types;

/// Various API Utils
pub mod utils;

#[derive(Default, Debug, Clone)]
pub(crate) struct EpicAPI {
    client: Client,
    pub(crate) user_data: UserData,
}

/// Error enum for the Epic API
#[derive(Debug)]
pub enum EpicAPIError {
    /// Wrong credentials
    InvalidCredentials,
    /// API error - see the contents
    APIError(String),
    /// Unknown error
    Unknown,
    /// Invalid parameters
    InvalidParams,
    /// Server error
    Server,
    /// FAB Timeout
    FabTimeout,
}

impl fmt::Display for EpicAPIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EpicAPIError::InvalidCredentials => {
                write!(f, "Invalid Credentials")
            }
            EpicAPIError::Unknown => {
                write!(f, "Unknown Error")
            }
            EpicAPIError::Server => {
                write!(f, "Server Error")
            }
            EpicAPIError::APIError(e) => {
                write!(f, "API Error: {}", e)
            }
            EpicAPIError::InvalidParams => {
                write!(f, "Invalid Input Parameters")
            }
            EpicAPIError::FabTimeout => {
                write!(f, "Fab Timeout Error")
            }
        }
    }
}

impl Error for EpicAPIError {
    fn description(&self) -> &str {
        match *self {
            EpicAPIError::InvalidCredentials => "Invalid Credentials",
            EpicAPIError::Unknown => "Unknown Error",
            EpicAPIError::Server => "Server Error",
            EpicAPIError::APIError(_) => "API Error",
            EpicAPIError::InvalidParams => "Invalid Input Parameters",
            EpicAPIError::FabTimeout => "Fab Timeout Error",
        }
    }
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
                        self.user_data.refresh_token.clone().unwrap(),
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
                Err(EpicAPIError::Unknown)
            }
        }
    }

    async fn handle_login_response(&mut self, response: Response) -> Result<bool, EpicAPIError> {
        if response.status() == reqwest::StatusCode::INTERNAL_SERVER_ERROR {
            error!("Server Error");
            return Err(EpicAPIError::Server);
        }
        let new: UserData = match response.json().await {
            Ok(data) => data,
            Err(e) => {
                error!("{:?}", e);
                return Err(EpicAPIError::Unknown);
            }
        };

        self.user_data.update(new);

        if let Some(m) = &self.user_data.error_message {
            error!("{}", m);
            return Err(EpicAPIError::APIError(m.to_string()));
        }
        Ok(true)
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

    pub async fn resume_session(&mut self) -> Result<bool, EpicAPIError> {
        match self.authorized_get_client(Url::parse("https://account-public-service-prod03.ol.epicgames.com/account/api/oauth/verify").unwrap()).send().await {
            Ok(response) => {
                self.handle_login_response(response).await
            }
            Err(e) => {
                error!("{:?}", e);
                Err(EpicAPIError::Unknown)
            }
        }
    }

    pub async fn invalidate_sesion(&mut self) -> bool {
        if let Some(access_token) = &self.user_data.access_token {
            let url = format!("https://account-public-service-prod03.ol.epicgames.com/account/api/oauth/sessions/kill/{}", access_token);
            let client = EpicAPI::build_client().build().unwrap();
            match client.delete(Url::from_str(&url).unwrap()).send().await {
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

    pub async fn assets(
        &mut self,
        platform: Option<String>,
        label: Option<String>,
    ) -> Result<Vec<EpicAsset>, EpicAPIError> {
        let plat = platform.unwrap_or_else(|| "Windows".to_string());
        let lab = label.unwrap_or_else(|| "Live".to_string());
        let url = format!("https://launcher-public-service-prod06.ol.epicgames.com/launcher/api/public/assets/{}?label={}", plat, lab);
        match self
            .authorized_get_client(Url::parse(&url).unwrap())
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(assets) => Ok(assets),
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

    pub async fn fab_asset_manifest(
        &self,
        artifact_id: &str,
        namespace: &str,
        asset_id: &str,
        platform: Option<&str>,
    ) -> Result<Vec<DownloadInfo>, EpicAPIError> {
        let url = format!("https://www.fab.com/e/artifacts/{}/manifest", artifact_id);
        match self
            .authorized_post_client(Url::parse(&url).unwrap())
            .json(&serde_json::json!({
                "item_id": asset_id,
                "namespace": namespace,
                "platform": platform.unwrap_or("Windows"),
            }))
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    let text = response.text().await.unwrap();
                    match serde_json::from_str::<types::fab_asset_manifest::FabAssetManifest>(&text)
                    {
                        Ok(manifest) => Ok(manifest.download_info),
                        Err(e) => {
                            error!("{:?}", e);
                            debug!("{}", text);
                            Err(EpicAPIError::Unknown)
                        }
                    }
                } else if response.status() == reqwest::StatusCode::FORBIDDEN {
                    Err(EpicAPIError::FabTimeout)
                } else {
                    debug!("{:?}", response.headers());
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

    pub async fn asset_manifest(
        &self,
        platform: Option<String>,
        label: Option<String>,
        namespace: Option<String>,
        item_id: Option<String>,
        app: Option<String>,
    ) -> Result<AssetManifest, EpicAPIError> {
        if namespace.is_none() {
            return Err(EpicAPIError::InvalidParams);
        };
        if item_id.is_none() {
            return Err(EpicAPIError::InvalidParams);
        };
        if app.is_none() {
            return Err(EpicAPIError::InvalidParams);
        };
        let url = format!("https://launcher-public-service-prod06.ol.epicgames.com/launcher/api/public/assets/v2/platform/{}/namespace/{}/catalogItem/{}/app/{}/label/{}",
                          platform.clone().unwrap_or_else(|| "Windows".to_string()), namespace.clone().unwrap(), item_id.clone().unwrap(), app.clone().unwrap(), label.clone().unwrap_or_else(|| "Live".to_string()));
        match self
            .authorized_get_client(Url::parse(&url).unwrap())
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json::<AssetManifest>().await {
                        Ok(mut manifest) => {
                            manifest.platform = platform;
                            manifest.label = label;
                            manifest.namespace = namespace;
                            manifest.item_id = item_id;
                            manifest.app = app;
                            Ok(manifest)
                        }
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

    pub async fn fab_download_manifest(
        &self,
        download_info: DownloadInfo,
        distribution_point_url: &str,
    ) -> Result<DownloadManifest, EpicAPIError> {
        match download_info.get_distribution_point_by_base_url(distribution_point_url) {
            None => {
                error!("Distribution point not found");
                Err(EpicAPIError::Unknown)
            }
            Some(point) => {
                if point.signature_expiration < time::OffsetDateTime::now_utc() {
                    error!("Expired signature");
                    Err(EpicAPIError::Unknown)
                } else {
                    let client = EpicAPI::build_client().build().unwrap();
                    match client
                        .get(Url::from_str(&point.manifest_url).unwrap())
                        .send()
                        .await
                    {
                        Ok(response) => {
                            if response.status() == reqwest::StatusCode::OK {
                                match response.bytes().await {
                                    Ok(data) => match DownloadManifest::parse(data.to_vec()) {
                                        None => {
                                            error!("Unable to parse the Download Manifest");
                                            Err(EpicAPIError::Unknown)
                                        }
                                        Some(man) => Ok(man),
                                    },
                                    Err(_) => Err(EpicAPIError::Unknown),
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
                        Err(_) => Err(EpicAPIError::Unknown),
                    }
                }
            }
        }
    }

    pub async fn asset_download_manifests(
        &self,
        asset_manifest: AssetManifest,
    ) -> Vec<DownloadManifest> {
        let base_urls = asset_manifest.url_csv();
        let mut result: Vec<DownloadManifest> = Vec::new();
        for elem in asset_manifest.elements {
            for manifest in elem.manifests {
                let mut queries: Vec<String> = Vec::new();
                debug!("{:?}", manifest);
                for query in manifest.query_params {
                    queries.push(format!("{}={}", query.name, query.value));
                }
                let url = format!("{}?{}", manifest.uri, queries.join("&"));
                let client = EpicAPI::build_client().build().unwrap();
                match client.get(Url::from_str(&url).unwrap()).send().await {
                    Ok(response) => {
                        if response.status() == reqwest::StatusCode::OK {
                            match response.bytes().await {
                                Ok(data) => match DownloadManifest::parse(data.to_vec()) {
                                    None => {
                                        error!("Unable to parse the Download Manifest");
                                    }
                                    Some(mut man) => {
                                        let mut url = manifest.uri.clone();
                                        url.set_path(&match url.path_segments() {
                                            None => "".to_string(),
                                            Some(segments) => {
                                                let mut vec: Vec<&str> = segments.collect();
                                                vec.remove(vec.len() - 1);
                                                vec.join("/")
                                            }
                                        });
                                        url.set_query(None);
                                        url.set_fragment(None);
                                        man.set_custom_field(
                                            "BaseUrl".to_string(),
                                            base_urls.clone(),
                                        );

                                        if let Some(id) = asset_manifest.item_id.clone() {
                                            man.set_custom_field(
                                                "CatalogItemId".to_string(),
                                                id.clone(),
                                            );
                                        }
                                        if let Some(label) = asset_manifest.label.clone() {
                                            man.set_custom_field(
                                                "BuildLabel".to_string(),
                                                label.clone(),
                                            );
                                        }
                                        if let Some(ns) = asset_manifest.namespace.clone() {
                                            man.set_custom_field(
                                                "CatalogNamespace".to_string(),
                                                ns.clone(),
                                            );
                                        }

                                        if let Some(app) = asset_manifest.app.clone() {
                                            man.set_custom_field(
                                                "CatalogAssetName".to_string(),
                                                app.clone(),
                                            );
                                        }

                                        man.set_custom_field(
                                            "SourceURL".to_string(),
                                            url.to_string(),
                                        );
                                        result.push(man)
                                    }
                                },
                                Err(e) => {
                                    error!("{:?}", e);
                                }
                            }
                        } else {
                            warn!(
                                "{} result: {}",
                                response.status(),
                                response.text().await.unwrap()
                            );
                        }
                    }
                    Err(e) => {
                        error!("{:?}", e);
                    }
                }
            }
        }
        result
    }

    pub async fn asset_info(
        &self,
        asset: EpicAsset,
    ) -> Result<HashMap<String, AssetInfo>, EpicAPIError> {
        let url = format!("https://catalog-public-service-prod06.ol.epicgames.com/catalog/api/shared/namespace/{}/bulk/items?id={}&includeDLCDetails=true&includeMainGameDetails=true&country=us&locale=lc",
                          asset.namespace, asset.catalog_item_id);
        match self
            .authorized_get_client(Url::parse(&url).unwrap())
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(info) => Ok(info),
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

    pub async fn game_token(&self) -> Result<GameToken, EpicAPIError> {
        let url =
            "https://account-public-service-prod03.ol.epicgames.com/account/api/oauth/exchange"
                .to_string();
        match self
            .authorized_get_client(Url::parse(&url).unwrap())
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(token) => Ok(token),
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

    pub async fn ownership_token(&self, asset: EpicAsset) -> Result<OwnershipToken, EpicAPIError> {
        let url = match &self.user_data.account_id {
            None => {
                return Err(EpicAPIError::InvalidCredentials);
            }
            Some(id) => {
                format!("https://ecommerceintegration-public-service-ecomprod02.ol.epicgames.com/ecommerceintegration/api/public/platforms/EPIC/identities/{}/ownershipToken",
                        id)
            }
        };
        match self
            .authorized_post_client(Url::parse(&url).unwrap())
            .form(&[(
                "nsCatalogItemId".to_string(),
                format!("{}:{}", asset.namespace, asset.catalog_item_id),
            )])
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::OK {
                    match response.json().await {
                        Ok(token) => Ok(token),
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

    pub async fn fab_library_items(
        &mut self,
        account_id: String,
    ) -> Result<FabLibrary, EpicAPIError> {
        let mut library = FabLibrary::default();

        loop {
            let url = match &library.cursors.next {
                None => {
                    format!(
                        "https://www.fab.com/e/accounts/{}/ue/library?count=100",
                        account_id
                    )
                }
                Some(c) => {
                    format!(
                        "https://www.fab.com/e/accounts/{}/ue/library?cursor={}&count=100",
                        account_id, c
                    )
                }
            };

            match self
                .authorized_get_client(Url::parse(&url).unwrap())
                .send()
                .await
            {
                Ok(response) => {
                    if response.status() == reqwest::StatusCode::OK {
                        let text = response.text().await.unwrap();
                        match serde_json::from_str::<FabLibrary>(&text) {
                            Ok(mut api_library) => {
                                library.cursors.next = api_library.cursors.next;
                                library.results.append(api_library.results.borrow_mut());
                            }
                            Err(e) => {
                                error!("{:?}", e);
                                debug!("{}", text);
                                library.cursors.next = None;
                            }
                        }
                    } else {
                        debug!("{:?}", response.headers());
                        warn!(
                            "{} result: {}",
                            response.status(),
                            response.text().await.unwrap()
                        );
                    }
                }
                Err(e) => {
                    error!("{:?}", e);
                    library.cursors.next = None;
                }
            }
            if library.cursors.next.is_none() {
                break;
            }
        }

        Ok(library)
    }

    pub async fn library_items(&mut self, include_metadata: bool) -> Result<Library, EpicAPIError> {
        let mut library = Library {
            records: vec![],
            response_metadata: Default::default(),
        };
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

            match self
                .authorized_get_client(Url::parse(&url).unwrap())
                .send()
                .await
            {
                Ok(response) => {
                    if response.status() == reqwest::StatusCode::OK {
                        match response.json::<Library>().await {
                            Ok(mut records) => {
                                library.records.append(records.records.borrow_mut());
                                match records.response_metadata {
                                    None => {
                                        break;
                                    }
                                    Some(meta) => match meta.next_cursor {
                                        None => {
                                            break;
                                        }
                                        Some(curs) => {
                                            cursor = Some(curs);
                                        }
                                    },
                                }
                            }
                            Err(e) => {
                                error!("{:?}", e);
                            }
                        }
                    } else {
                        warn!(
                            "{} result: {}",
                            response.status(),
                            response.text().await.unwrap()
                        );
                    }
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            };
            if cursor.is_none() {
                break;
            }
        }
        Ok(library)
    }
}
