use crate::api::error::EpicAPIError;
use crate::api::types::cloud_save::CloudSaveResponse;
use crate::api::types::presence::PresenceUpdate;
use crate::api::types::service_status::ServiceStatus;
use crate::EpicGames;

impl EpicGames {
    /// Like [`service_status`](Self::service_status), but returns a `Result` instead of swallowing errors.
    pub async fn try_service_status(
        &self,
        service_id: &str,
    ) -> Result<Vec<ServiceStatus>, EpicAPIError> {
        self.egs.service_status(service_id).await
    }

    /// Fetch service status from Epic's lightswitch API.
    ///
    /// Returns the operational status of an Epic online service (e.g., a game's
    /// backend). The response includes whether the service is UP/DOWN, any
    /// maintenance message, and whether the current user is banned.
    ///
    /// Returns `None` on API errors.
    pub async fn service_status(&self, service_id: &str) -> Option<Vec<ServiceStatus>> {
        self.try_service_status(service_id).await.ok()
    }

    /// Update the user's presence status.
    ///
    /// Sends a PATCH request to update the user's online presence (e.g.,
    /// "online", "away") and optionally set an activity with custom properties.
    /// The `session_id` is the OAuth session token from login. Returns `Ok(())`
    /// on success (204 No Content) or an [`EpicAPIError`] on failure.
    pub async fn update_presence(
        &self,
        session_id: &str,
        body: &PresenceUpdate,
    ) -> Result<(), EpicAPIError> {
        self.egs.update_presence(session_id, body).await
    }

    // ── Cloud Saves ──

    /// List cloud save files for the logged-in user.
    ///
    /// If `app_name` is provided, lists saves for that specific game.
    /// If `manifests` is true (only relevant when `app_name` is set), lists manifest files.
    pub async fn cloud_save_list(
        &self,
        app_name: Option<&str>,
        manifests: bool,
    ) -> Result<CloudSaveResponse, EpicAPIError> {
        self.egs.cloud_save_list(app_name, manifests).await
    }

    /// Query cloud save files by specific filenames.
    ///
    /// Returns metadata including read/write links for the specified files.
    pub async fn cloud_save_query(
        &self,
        app_name: &str,
        filenames: &[String],
    ) -> Result<CloudSaveResponse, EpicAPIError> {
        self.egs.cloud_save_query(app_name, filenames).await
    }

    /// Delete a cloud save file by its storage path.
    pub async fn cloud_save_delete(&self, path: &str) -> Result<(), EpicAPIError> {
        self.egs.cloud_save_delete(path).await
    }
}
