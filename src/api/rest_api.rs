use serde::{de, Deserialize};
use tracing::trace;

use crate::Result;

use super::errors::MissingUserIdError;
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

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
// Get the list of courses where a user is enrolled in
pub struct CoreEnrolGetUsersCourses {
    /// id of course
    pub id: i32,
    /// Short name of course
    pub shortname: String,
    /// Full name of course
    pub fullname: String,
    /// Course display name for lists.
    pub displayname: Option<String>,
    /// Number of enrolled users in this course
    pub enrolledusercount: Option<i32>,
    /// id number of course
    pub idnumber: String,
    /// true means visible, false means not yet visible course
    pub visible: bool,
    /// summary
    pub summary: Option<String>,
    /// summary format (1 = HTML, 0 = MOODLE, 2 = PLAIN, or 4 = MARKDOWN)
    pub summaryformat: Option<i32>,
    /// course format: weeks, topics, social, site
    pub courseformat: Option<String>,
    /// The course image URL
    pub courseimage: Option<String>,
    /// true if grades are shown, otherwise false
    pub showgrades: Option<bool>,
    /// forced course language
    pub lang: Option<String>,
    /// true if completion is enabled, otherwise false
    pub enablecompletion: Option<bool>,
    /// If completion criteria is set.
    pub completionhascriteria: Option<bool>,
    /// If the user is completion tracked.
    pub completionusertracked: Option<bool>,
    /// course category id
    pub category: Option<i32>,
    /// Progress percentage
    pub progress: Option<f64>,
    /// Whether the course is completed.
    pub completed: Option<bool>,
    /// Timestamp when the course start
    pub startdate: Option<u64>,
    /// Timestamp when the course end
    pub enddate: Option<u64>,
    /// Course section marker.
    pub marker: Option<i32>,
    /// Last access to the course (timestamp).
    pub lastaccess: Option<u64>,
    /// If the user marked this course a favourite.
    pub isfavourite: Option<bool>,
    /// If the user hide the course from the dashboard.
    pub hidden: Option<bool>,
    /// Overview files attached to this course.
    pub overviewfiles: Option<Vec<CoreEnrolGetUsersCoursesOverviewFile>>,
    /// Whether the activity dates are shown or not
    // #[serde(deserialize_with = "deserialize_bool_from_int")]
    pub showactivitydates: Option<bool>,
    /// Whether the activity completion conditions are shown or not
    pub showcompletionconditions: Option<bool>,
    /// Last time course settings were updated (timestamp).
    pub timemodified: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
/// File.
pub struct CoreEnrolGetUsersCoursesOverviewFile {
    /// File name.
    pub filename: Option<String>,
    /// File path.
    pub filepath: Option<String>,
    /// File size.
    pub filesize: Option<i32>,
    /// Downloadable file url.
    pub fileurl: Option<String>,
    /// Time modified.
    pub timemodified: Option<u64>,
    /// File mime type.
    pub mimetype: Option<String>,
    /// Whether is an external file.
    pub isexternalfile: Option<bool>,
    /// The repository type for external files.
    pub repositorytype: Option<String>,
}

impl Api {
    /// Generic function to make a rest api request
    /// # Arguments
    /// * `query` - query parameters
    /// * `T` - type to deserialize to
    pub async fn rest_api_request_json<T>(&self, query: &[(&str, &str)]) -> Result<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        trace!("Rest api request: {:?}", query);
        let response = self
            .client
            .get(format!(
                "{}/webservice/rest/server.php",
                self.credential.instance_url
            ))
            .query(&[
                ("moodlewsrestformat", "json"),
                ("wstoken", self.credential.wstoken.as_str()),
            ])
            .query(query)
            .send()
            .await?;
        Ok(response.json::<T>().await?)
    }

    pub async fn get_core_webservice_get_site_info(&self) -> Result<CoreWebserviceGetSiteInfo> {
        Ok(self
            .rest_api_request_json::<CoreWebserviceGetSiteInfo>(&[(
                "wsfunction",
                "core_webservice_get_site_info",
            )])
            .await?)
    }

    pub async fn core_enrol_get_users_courses(&self) -> Result<Vec<CoreEnrolGetUsersCourses>> {
        Ok(self
            .rest_api_request_json::<Vec<CoreEnrolGetUsersCourses>>(&[
                ("wsfunction", "core_enrol_get_users_courses"),
                (
                    "userid",
                    &self.user_id.ok_or(MissingUserIdError)?.to_string(),
                ),
            ])
            .await?)
    }
}
