use crate::api::error::EpicAPIError;
use crate::api::types::service_status::ServiceStatus;
use crate::api::EpicAPI;

impl EpicAPI {
    /// Fetch bulk service status from the lightswitch API.
    pub async fn service_status(
        &self,
        service_id: &str,
    ) -> Result<Vec<ServiceStatus>, EpicAPIError> {
        let url = format!(
            "https://lightswitch-public-service-prod06.ol.epicgames.com/lightswitch/api/service/bulk/status?serviceId={}",
            service_id
        );
        self.authorized_get_json(&url).await
    }
}
