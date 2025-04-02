use super::*;

impl Config {
    pub(super) async fn api_core_webservice_get_site_info(
        &self,
    ) -> Result<CoreWebserviceGetSiteInfo> {
        Ok(self
            .api_request_json::<CoreWebserviceGetSiteInfo>(&[(
                "wsfunction",
                "core_webservice_get_site_info",
            )])
            .await?)
    }
}

// Descriptions taken from generated moodle docs these can be accessed on any moodle instance with administrator rights via: http://example.com/admin/webservice/documentation.php
#[derive(Debug, Deserialize)]
/// Return some site info / user info / list web service functions
pub struct CoreWebserviceGetSiteInfo {
    /// User id
    pub userid: u64,
}
