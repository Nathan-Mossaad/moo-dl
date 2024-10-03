use serde::{de, Deserialize};

use crate::Result;

use super::Api;

// Helper functions
fn deserialize_bool_from_int<'de, D>(
    deserializer: D,
) -> core::result::Result<Option<bool>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let value: Option<i64> = Option::deserialize(deserializer)?;
    match value {
        Some(value) => Ok(Some(value != 0)),
        None => Ok(None),
    }
}
fn force_deserialize_bool_from_int<'de, D>(deserializer: D) -> core::result::Result<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let value: Option<i64> = Option::deserialize(deserializer)?;
    match value {
        Some(value) => Ok(value != 0),
        None => Ok(false),
    }
}

// Descriptions taken from generated moodle docs these can be accessed on any moodle instance with administrator rights via: http://example.com/admin/webservice/documentation.php
// We always allow deadcode, as not all fields are used

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
/// Return some site info / user info / list web service functions
pub struct CoreWebserviceGetSiteInfo {
    /// Site name
    pub sitename: String,
    /// Username
    pub username: String,
    /// First name
    pub firstname: String,
    /// Last name
    pub lastname: String,
    /// User full name
    pub fullname: String,
    /// Current language
    pub lang: String,
    /// User id
    pub userid: i32,
    /// Site url
    pub siteurl: String,
    /// The user profile picture.
    /// Warning: this url is the public URL that only works when forcelogin is set to NO and guestaccess is set to YES.
    /// In order to retrieve user profile pictures independently of the Moodle config, replace "pluginfile.php" by
    /// "webservice/pluginfile.php?token=WSTOKEN&file="
    /// Of course the user can only see profile picture depending
    /// on his/her permissions. Moreover it is recommended to use HTTPS too.
    pub userpictureurl: String,
    /// Functions that are available
    pub functions: Vec<CoreWebserviceGetSiteInfoFunction>,
    /// true if users are allowd to download files, false if not
    #[serde(deserialize_with = "deserialize_bool_from_int")]
    pub downloadfiles: Option<bool>,
    /// true if users are allowd to upload files, false if not
    #[serde(deserialize_with = "deserialize_bool_from_int")]
    pub uploadfiles: Option<bool>,
    /// Moodle release number
    pub release: Option<String>,
    /// Moodle version number
    pub version: Option<String>,
    /// Mobile custom CSS theme
    pub mobilecssurl: Option<String>,
    /// Advanced features availability
    pub advancedfeatures: Option<Vec<CoreWebserviceGetSiteInfoAdvancedFeature>>,
    /// true if the user can manage his own files
    pub usercanmanageownfiles: Option<bool>,
    /// user quota (bytes). 0 means user can ignore the quota
    pub userquota: Option<i64>,
    /// user max upload file size (bytes). -1 means the user can ignore the upload file size
    pub usermaxuploadfilesize: Option<i64>,
    /// the default home page for the user: 0 for the site home, 1 for dashboard
    pub userhomepage: Option<i32>,
    /// Private user access key for fetching files.
    pub userprivateaccesskey: Option<String>,
    /// Site course ID
    pub siteid: Option<i32>,
    /// Calendar type set in the site.
    pub sitecalendartype: Option<String>,
    /// Calendar typed used by the user.
    pub usercalendartype: Option<String>,
    /// Whether the user is a site admin or not.
    pub userissiteadmin: Option<bool>,
    /// Current theme for the user.
    pub theme: Option<String>,
    /// Number of concurrent sessions allowed
    pub limitconcurrentlogins: Option<i32>,
    /// Number of active sessions for current user.
    /// Only returned when limitconcurrentlogins is used.
    pub usersessionscount: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
/// Available Function
pub struct CoreWebserviceGetSiteInfoFunction {
    /// Function name
    pub name: String,
    /// The version number of the component to which the function belongs
    pub version: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
/// Advanced feature availability
pub struct CoreWebserviceGetSiteInfoAdvancedFeature {
    /// Feature name
    pub name: String,
    /// Feature value. Usually true means enabled.
    /// May be broken if 1 does not mean enabled or values larger than 1 are used
    #[serde(deserialize_with = "force_deserialize_bool_from_int")]
    pub value: bool,
}

impl Api {
    pub async fn get_core_webservice_get_site_info(&self) -> Result<CoreWebserviceGetSiteInfo> {
        let response = self
            .client
            .get(format!(
                "{}/webservice/rest/server.php",
                self.credential.instance_url
            ))
            .query(&[
                ("moodlewsrestformat", "json"),
                ("wstoken", self.credential.wstoken.as_str()),
                ("wsfunction", "core_webservice_get_site_info"),
            ])
            .send()
            .await?;
        Ok(response.json::<CoreWebserviceGetSiteInfo>().await?)
    }
}
