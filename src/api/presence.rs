use crate::api::EpicAPI;
use crate::api::error::EpicAPIError;
use crate::api::types::presence::PresenceUpdate;

impl EpicAPI {
    /// Update user presence status.
    pub async fn update_presence(
        &self,
        session_id: &str,
        body: &PresenceUpdate,
    ) -> Result<(), EpicAPIError> {
        let url = match &self.user_data.account_id {
            Some(id) => format!(
                "https://presence-public-service-prod.ol.epicgames.com/presence/api/v1/_/{}/presence/{}",
                id, session_id
            ),
            None => return Err(EpicAPIError::InvalidCredentials),
        };
        let parsed_url = url::Url::parse(&url).map_err(|_| EpicAPIError::InvalidParams)?;
        let response = self
            .set_authorization_header(self.client.patch(parsed_url))
            .json(body)
            .send()
            .await
            .map_err(|e| {
                log::error!("{:?}", e);
                EpicAPIError::NetworkError(e)
            })?;
        if response.status() == reqwest::StatusCode::OK
            || response.status() == reqwest::StatusCode::NO_CONTENT
        {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            log::warn!("{} result: {}", status, body);
            Err(EpicAPIError::HttpError { status, body })
        }
    }
}
