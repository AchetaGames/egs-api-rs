use crate::api::EpicAPI;
use crate::api::error::EpicAPIError;
use crate::api::types::cloud_save::CloudSaveResponse;

impl EpicAPI {
    #[allow(dead_code)]
    /// List cloud save files for a user, optionally filtered by app name.
    ///
    /// If `app_name` is provided, lists saves for that specific game.
    /// If `manifests` is true (only relevant when `app_name` is set), lists manifest files.
    pub async fn cloud_save_list(
        &self,
        app_name: Option<&str>,
        manifests: bool,
    ) -> Result<CloudSaveResponse, EpicAPIError> {
        let user_id = self
            .user_data
            .account_id
            .as_deref()
            .ok_or(EpicAPIError::InvalidCredentials)?;
        let app_path = match app_name {
            Some(name) if manifests => format!("{}/manifests/", name),
            Some(name) => format!("{}/", name),
            None => String::new(),
        };
        let url = format!(
            "https://datastorage-public-service-liveegs.live.use1a.on.epicgames.com/api/v1/access/egstore/savesync/{}/{}",
            user_id, app_path
        );
        self.authorized_get_json(&url).await
    }

    #[allow(dead_code)]
    /// Query cloud save files by specific filenames (POST with filenames body).
    ///
    /// Returns metadata including read/write links for the specified files.
    pub async fn cloud_save_query(
        &self,
        app_name: &str,
        filenames: &[String],
    ) -> Result<CloudSaveResponse, EpicAPIError> {
        let user_id = self
            .user_data
            .account_id
            .as_deref()
            .ok_or(EpicAPIError::InvalidCredentials)?;
        let url = format!(
            "https://datastorage-public-service-liveegs.live.use1a.on.epicgames.com/api/v1/access/egstore/savesync/{}/{}/",
            user_id, app_name
        );
        let body = serde_json::json!({ "files": filenames });
        self.authorized_post_json(&url, &body).await
    }

    #[allow(dead_code)]
    /// Cloud save deletion endpoint.
    pub async fn cloud_save_delete(&self, path: &str) -> Result<(), EpicAPIError> {
        let url = format!(
            "https://datastorage-public-service-liveegs.live.use1a.on.epicgames.com/api/v1/data/egstore/{}",
            path
        );
        self.authorized_delete(&url).await
    }
}
