use crate::api::EpicAPI;
use crate::api::error::EpicAPIError;
use crate::api::types::cosmos::{
    CosmosAccount, CosmosAuthResponse, CosmosCommOptIn, CosmosEulaResponse, CosmosPolicyAodc,
    CosmosSearchResults,
};
use crate::api::types::engine_blob::EngineBlobsResponse;
use log::{debug, error, warn};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RedirectResponse {
    #[allow(dead_code)]
    redirect_url: Option<String>,
    #[allow(dead_code)]
    authorization_code: Option<serde_json::Value>,
    sid: Option<String>,
}

impl EpicAPI {
    /// Set up a Cosmos cookie session from an exchange code.
    ///
    /// Performs the full web-based flow:
    /// 1. `GET /id/api/reputation` — sets XSRF-TOKEN cookie
    /// 2. `POST /id/api/exchange` — sends exchange code with XSRF
    /// 3. `GET /id/api/redirect` — gets SID
    /// 4. `GET /id/api/set-sid?sid=X` — sets session cookies on unrealengine.com
    /// 5. `GET /api/cosmos/auth` — upgrades bearer token, sets EPIC_EG1 JWTs
    ///
    /// After success, all Cosmos endpoints (`cosmos_*` methods)
    /// work automatically via cookies in the client's cookie jar.
    pub async fn cosmos_session_setup(
        &self,
        exchange_code: &str,
    ) -> Result<CosmosAuthResponse, EpicAPIError> {
        let rep_response = self
            .client
            .get("https://www.epicgames.com/id/api/reputation")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to get XSRF token: {:?}", e);
                EpicAPIError::NetworkError(e)
            })?;

        let xsrf = rep_response
            .cookies()
            .find(|c| c.name() == "XSRF-TOKEN")
            .map(|c| c.value().to_string())
            .ok_or_else(|| {
                error!("XSRF-TOKEN cookie not found in reputation response");
                EpicAPIError::InvalidCredentials
            })?;

        let exchange_body = serde_json::json!({"exchangeCode": exchange_code});
        let exchange_resp = self
            .client
            .post("https://www.epicgames.com/id/api/exchange")
            .json(&exchange_body)
            .header("x-xsrf-token", &xsrf)
            .send()
            .await
            .map_err(|e| {
                error!("Failed exchange code: {:?}", e);
                EpicAPIError::NetworkError(e)
            })?;

        if !exchange_resp.status().is_success() {
            let status = exchange_resp.status();
            let body = exchange_resp.text().await.unwrap_or_default();
            warn!("Exchange failed: {} {}", status, body);
            return Err(EpicAPIError::HttpError { status, body });
        }

        let redirect_resp = self
            .client
            .get("https://www.epicgames.com/id/api/redirect?")
            .send()
            .await
            .map_err(|e| {
                error!("Failed redirect: {:?}", e);
                EpicAPIError::NetworkError(e)
            })?;

        let redirect: RedirectResponse = redirect_resp.json().await.map_err(|e| {
            error!("Failed to parse redirect response: {:?}", e);
            EpicAPIError::DeserializationError(format!("{}", e))
        })?;

        let sid = redirect.sid.ok_or_else(|| {
            error!("No SID in redirect response");
            EpicAPIError::InvalidCredentials
        })?;

        let set_sid_resp = self
            .client
            .get(format!(
                "https://www.unrealengine.com/id/api/set-sid?sid={}",
                sid
            ))
            .send()
            .await
            .map_err(|e| {
                error!("Failed set-sid: {:?}", e);
                EpicAPIError::NetworkError(e)
            })?;
        debug!("set-sid status={}", set_sid_resp.status());
        for cookie in set_sid_resp.cookies() {
            debug!("set-sid cookie: {}={} (domain={:?})", cookie.name(), &cookie.value()[..20.min(cookie.value().len())], cookie.domain());
        }

        self.cosmos_auth_upgrade().await
    }

    /// Upgrade the session by calling `GET /api/cosmos/auth`.
    ///
    /// Converts base session cookies (from `set-sid`) into upgraded EPIC_EG1 JWTs.
    pub async fn cosmos_auth_upgrade(&self) -> Result<CosmosAuthResponse, EpicAPIError> {
        let response = self
            .client
            .get("https://www.unrealengine.com/api/cosmos/auth")
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                error!("Failed cosmos auth: {:?}", e);
                EpicAPIError::NetworkError(e)
            })?;

        if response.status().is_success() {
            response.json::<CosmosAuthResponse>().await.map_err(|e| {
                error!("Failed to parse cosmos auth response: {:?}", e);
                EpicAPIError::DeserializationError(format!("{}", e))
            })
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("cosmos/auth failed: {} {}", status, body);
            Err(EpicAPIError::HttpError { status, body })
        }
    }

    /// Check if a EULA has been accepted.
    ///
    /// Requires an active Cosmos session (call `cosmos_session_setup` first).
    /// Known EULA IDs: `unreal_engine`, `unreal_engine2`, `ue`, `mhc`.
    pub async fn cosmos_eula_check(
        &self,
        eula_id: &str,
        locale: &str,
    ) -> Result<CosmosEulaResponse, EpicAPIError> {
        let url = format!(
            "https://www.unrealengine.com/api/cosmos/eula/accept?eulaId={}&locale={}",
            eula_id, locale
        );
        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                error!("Failed EULA check: {:?}", e);
                EpicAPIError::NetworkError(e)
            })?;

        if response.status().is_success() {
            response.json::<CosmosEulaResponse>().await.map_err(|e| {
                error!("Failed to parse EULA response: {:?}", e);
                EpicAPIError::DeserializationError(format!("{}", e))
            })
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("EULA check failed: {} {}", status, body);
            Err(EpicAPIError::HttpError { status, body })
        }
    }

    /// Accept a EULA.
    ///
    /// Requires an active Cosmos session. The web UI sends
    /// `eulaId=unreal_engine2&locale=en&version=3`.
    pub async fn cosmos_eula_accept(
        &self,
        eula_id: &str,
        locale: &str,
        version: u32,
    ) -> Result<CosmosEulaResponse, EpicAPIError> {
        let url = format!(
            "https://www.unrealengine.com/api/cosmos/eula/accept?eulaId={}&locale={}&version={}",
            eula_id, locale, version
        );
        let response = self
            .client
            .post(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                error!("Failed EULA accept: {:?}", e);
                EpicAPIError::NetworkError(e)
            })?;

        if response.status().is_success() {
            response.json::<CosmosEulaResponse>().await.map_err(|e| {
                error!("Failed to parse EULA accept response: {:?}", e);
                EpicAPIError::DeserializationError(format!("{}", e))
            })
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("EULA accept failed: {} {}", status, body);
            Err(EpicAPIError::HttpError { status, body })
        }
    }

    /// Get Cosmos account details. Requires an active Cosmos session.
    pub async fn cosmos_account(&self) -> Result<CosmosAccount, EpicAPIError> {
        let response = self
            .client
            .get("https://www.unrealengine.com/api/cosmos/account")
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                error!("Failed cosmos account: {:?}", e);
                EpicAPIError::NetworkError(e)
            })?;

        if response.status().is_success() {
            response.json::<CosmosAccount>().await.map_err(|e| {
                error!("Failed to parse cosmos account response: {:?}", e);
                EpicAPIError::DeserializationError(format!("{}", e))
            })
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("cosmos/account failed: {} {}", status, body);
            Err(EpicAPIError::HttpError { status, body })
        }
    }

    /// Check Age of Digital Consent policy. Requires an active Cosmos session.
    pub async fn cosmos_policy_aodc(&self) -> Result<CosmosPolicyAodc, EpicAPIError> {
        let response = self
            .client
            .get("https://www.unrealengine.com/api/cosmos/policy/aodc")
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                error!("Failed cosmos policy check: {:?}", e);
                EpicAPIError::NetworkError(e)
            })?;

        if response.status().is_success() {
            response.json::<CosmosPolicyAodc>().await.map_err(|e| {
                error!("Failed to parse policy response: {:?}", e);
                EpicAPIError::DeserializationError(format!("{}", e))
            })
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("cosmos/policy/aodc failed: {} {}", status, body);
            Err(EpicAPIError::HttpError { status, body })
        }
    }

    /// Check communication opt-in status.
    ///
    /// Known settings: `email:ue` (Unreal Engine), likely also `email:fn` (Fortnite).
    /// Requires an active Cosmos session.
    pub async fn cosmos_comm_opt_in(&self, setting: &str) -> Result<CosmosCommOptIn, EpicAPIError> {
        let url = format!(
            "https://www.unrealengine.com/api/cosmos/communication/opt-in?setting={}",
            setting
        );
        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                error!("Failed cosmos comm opt-in: {:?}", e);
                EpicAPIError::NetworkError(e)
            })?;

        if response.status().is_success() {
            response.json::<CosmosCommOptIn>().await.map_err(|e| {
                error!("Failed to parse comm opt-in response: {:?}", e);
                EpicAPIError::DeserializationError(format!("{}", e))
            })
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("cosmos/communication/opt-in failed: {} {}", status, body);
            Err(EpicAPIError::HttpError { status, body })
        }
    }

    /// Fetch engine version blobs (download URLs) for a platform.
    ///
    /// Requires an active Cosmos session (cookies from set-sid + cosmos/auth).
    /// Platform values: `linux`, `windows`, `mac`
    ///
    /// Tries the JSON API (`/api/blobs/{platform}`) first.
    /// If that fails or returns empty on `linux`, it falls back to scraping
    /// the Linux download page HTML. For `windows` and `mac`, scraping is skipped.
    pub async fn engine_versions(
        &self,
        platform: &str,
    ) -> Result<EngineBlobsResponse, EpicAPIError> {
        match self.engine_versions_api(platform).await {
            Ok(resp) if !resp.blobs.is_empty() => Ok(resp),
            Ok(resp) => {
                if platform == "linux" {
                    debug!("API returned empty blobs, trying Linux page fallback");
                    self.engine_versions_from_page().await
                } else {
                    warn!(
                        "API returned empty blobs for '{}'. Linux page fallback is disabled for this platform",
                        platform
                    );
                    Ok(resp)
                }
            }
            Err(e) => {
                if platform == "linux" {
                    debug!("API failed ({}), trying Linux page fallback", e);
                    self.engine_versions_from_page().await
                } else {
                    warn!(
                        "API failed for '{}'. Linux page fallback is disabled for this platform",
                        platform
                    );
                    Err(e)
                }
            }
        }
    }

    /// Try the direct JSON API endpoint (may be deprecated).
    async fn engine_versions_api(
        &self,
        platform: &str,
    ) -> Result<EngineBlobsResponse, EpicAPIError> {
        let url = format!("https://www.unrealengine.com/api/blobs/{}", platform);
        debug!("Fetching engine versions from API: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .send()
            .await
            .map_err(|e| {
                error!("Failed engine versions: {:?}", e);
                EpicAPIError::NetworkError(e)
            })?;

        if response.status().is_success() {
            let body = response.text().await.map_err(|e| {
                error!("Failed to read engine versions response body: {:?}", e);
                EpicAPIError::DeserializationError(format!("{}", e))
            })?;
            debug!("engine_versions raw response ({} chars): {}", body.len(), &body[..500.min(body.len())]);

            if let Some(err_msg) = serde_json::from_str::<std::collections::HashMap<String, String>>(&body)
                .ok()
                .and_then(|m| m.get("error").cloned())
            {
                warn!("blobs/{} returned error: {}", platform, err_msg);
                return Err(EpicAPIError::InvalidCredentials);
            }

            serde_json::from_str::<EngineBlobsResponse>(&body).map_err(|e| {
                error!("Failed to parse engine versions response: {:?}", e);
                EpicAPIError::DeserializationError(format!("{}", e))
            })
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("blobs/{} failed: {} {}", platform, status, body);
            Err(EpicAPIError::HttpError { status, body })
        }
    }

    async fn engine_versions_from_page(&self) -> Result<EngineBlobsResponse, EpicAPIError> {
        let url = "https://www.unrealengine.com/en-US/linux";
        debug!("Fetching engine versions from page: {}", url);

        let response = self
            .client
            .get(url)
            .header("Accept", "text/html")
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch engine page: {:?}", e);
                EpicAPIError::NetworkError(e)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Engine page fetch failed: {} {}", status, &body[..200.min(body.len())]);
            return Err(EpicAPIError::HttpError { status, body });
        }

        let html = response.text().await.map_err(|e| {
            error!("Failed to read engine page body: {:?}", e);
            EpicAPIError::DeserializationError(format!("{}", e))
        })?;
        debug!("Engine page HTML: {} chars", html.len());

        let blobs = Self::extract_blobs_from_html(&html)?;
        debug!("Extracted {} blobs from page HTML", blobs.len());
        Ok(EngineBlobsResponse { blobs })
    }

    /// Extract the blob array from the page HTML.
    ///
    /// The page embeds blob data in React Server Component `__next_f` script
    /// tags as JS string literals (quotes escaped as `\"`). This method
    /// unescapes the JS strings first, then extracts the `"blobs":[...]`
    /// array using bracket counting.
    ///
    /// If Epic changes the page structure, update the `MARKER` constant.
    fn extract_blobs_from_html(
        html: &str,
    ) -> Result<Vec<crate::api::types::engine_blob::EngineBlob>, EpicAPIError> {
        const MARKER: &str = "\"blobs\":[{";

        let text = if html.contains(MARKER) {
            std::borrow::Cow::Borrowed(html)
        } else {
            std::borrow::Cow::Owned(html.replace("\\\"", "\""))
        };

        let marker_pos = text.find(MARKER).ok_or_else(|| {
            error!("Could not find blob data marker in page HTML");
            EpicAPIError::DeserializationError(
                "blob data not found in page HTML, Epic may have changed the page structure".into(),
            )
        })?;

        let array_start = marker_pos + "\"blobs\":".len();
        let text_bytes = text.as_bytes();

        let mut depth: u32 = 0;
        let mut array_end = array_start;
        let mut in_string = false;
        let mut escape_next = false;

        for (i, &byte) in text_bytes[array_start..].iter().enumerate() {
            if escape_next {
                escape_next = false;
                continue;
            }
            match byte {
                b'\\' if in_string => escape_next = true,
                b'"' => in_string = !in_string,
                b'[' if !in_string => depth += 1,
                b']' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        array_end = array_start + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        if depth != 0 {
            error!("Unmatched brackets while parsing blob array from HTML");
            return Err(EpicAPIError::DeserializationError(
                "malformed blob array in page HTML".into(),
            ));
        }

        let array_json = &text[array_start..array_end];
        let clean = array_json
            .replace("\\u0026", "&")
            .replace("\\u003c", "<")
            .replace("\\u003e", ">");

        serde_json::from_str(&clean).map_err(|e| {
            error!(
                "Failed to parse blobs from HTML (first 200 chars): {}",
                &clean[..200.min(clean.len())]
            );
            EpicAPIError::DeserializationError(format!("blob JSON parse error: {}", e))
        })
    }

    /// Search unrealengine.com content. Requires an active Cosmos session.
    pub async fn cosmos_search(
        &self,
        query: &str,
        slug: Option<&str>,
        locale: Option<&str>,
        filter: Option<&str>,
    ) -> Result<CosmosSearchResults, EpicAPIError> {
        let mut url = format!(
            "https://www.unrealengine.com/api/cosmos/search?query={}",
            query
        );
        if let Some(s) = slug {
            url.push_str(&format!("&slug={}", s));
        }
        if let Some(l) = locale {
            url.push_str(&format!("&locale={}", l));
        }
        if let Some(f) = filter {
            url.push_str(&format!("&filter={}", f));
        }
        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                error!("Failed cosmos search: {:?}", e);
                EpicAPIError::NetworkError(e)
            })?;

        if response.status().is_success() {
            response.json::<CosmosSearchResults>().await.map_err(|e| {
                error!("Failed to parse cosmos search response: {:?}", e);
                EpicAPIError::DeserializationError(format!("{}", e))
            })
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("cosmos/search failed: {} {}", status, body);
            Err(EpicAPIError::HttpError { status, body })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_blobs_from_rsc_payload_escaped() {
        let html = r#"<script>self.__next_f.push([1,"stuff\"blobs\":[{\"name\":\"Linux_Unreal_Engine_5.7.4.zip\",\"createdAt\":\"2026-03-10T12:46:39.745Z\",\"size\":32073680226,\"downloadUrl\":\"https://ucs-blob-store.s3-accelerate.amazonaws.com/blobs/40/f8/test\",\"version\":\"\",\"semver\":\"\",\"operatingSystem\":\"\"},{\"name\":\"Linux_Fab_5.7.0_0.0.7.zip\",\"createdAt\":\"2025-11-12T14:16:21.834Z\",\"size\":26745370,\"downloadUrl\":\"https://example.com/fab\",\"version\":\"\",\"semver\":\"\",\"operatingSystem\":\"\"}]rest"])</script>"#;
        let blobs = EpicAPI::extract_blobs_from_html(html).unwrap();
        assert_eq!(blobs.len(), 2);
        assert_eq!(blobs[0].name, "Linux_Unreal_Engine_5.7.4.zip");
        assert_eq!(blobs[0].size, 32073680226);
        assert!(blobs[0].download_url.starts_with("https://"));
        assert_eq!(blobs[1].name, "Linux_Fab_5.7.0_0.0.7.zip");
    }

    #[test]
    fn extract_blobs_unescaped() {
        let html = r#"stuff "blobs":[{"name":"test.zip","createdAt":"2026-01-01T00:00:00Z","size":100,"downloadUrl":"https://example.com?foo=1\u0026bar=2","version":"","semver":"","operatingSystem":""}] more"#;
        let blobs = EpicAPI::extract_blobs_from_html(html).unwrap();
        assert_eq!(blobs.len(), 1);
        assert_eq!(blobs[0].download_url, "https://example.com?foo=1&bar=2");
    }

    #[test]
    fn extract_blobs_missing_marker() {
        let html = "<html><body>no blob data here</body></html>";
        assert!(EpicAPI::extract_blobs_from_html(html).is_err());
    }
}
