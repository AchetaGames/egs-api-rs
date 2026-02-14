use crate::api::error::EpicAPIError;
use crate::api::types::presence::PresenceUpdate;
use crate::api::EpicAPI;

impl EpicAPI {
    /// Update user presence status.
    pub async fn update_presence(
        &self,
        session_id: &str,
        body: &PresenceUpdate,
    ) -> Result<(), EpicAPIError> {
        let id = match &self.user_data.account_id {
            Some(id) => id.clone(),
            None => return Err(EpicAPIError::InvalidCredentials),
        };
        let url = format!(
            "https://presence-public-service-prod.ol.epicgames.com/presence/api/v1/_/{}/presence/{}",
            id, session_id
        );
        let parsed_url = url::Url::parse(&url).map_err(|_| EpicAPIError::InvalidParams)?;
        let response = self
            .set_authorization_header(self.client.patch(parsed_url))
            .json(body)
            .send()
            .await
            .map_err(|e| {
                log::error!("{:?}", e);
                EpicAPIError::Unknown
            })?;
        if response.status() == reqwest::StatusCode::OK
            || response.status() == reqwest::StatusCode::NO_CONTENT
        {
            Ok(())
        } else {
            log::warn!(
                "{} result: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
            Err(EpicAPIError::Unknown)
        }
    }
}
