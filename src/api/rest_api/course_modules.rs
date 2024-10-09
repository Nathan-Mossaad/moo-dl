use std::fmt::Debug;
use std::path::Path;

use futures::future::join_all;
use regex::Regex;
use select::{document::Document, predicate::Name};
use serde::Deserialize;
use tracing::{info, trace};

use crate::api::login::Credential;
use crate::api::Api;
use crate::downloader::check_for_updated_contents;
use crate::Result;

mod content_types;

use content_types::Content;

pub trait Download: Debug + Clone {
    async fn download(&self, api: &Api, path: &Path) -> Result<()>;
}

// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "modname")]
pub enum Module {
    #[serde(rename = "resource")]
    Resource(Resource),
    #[serde(rename = "folder")]
    Folder(Folder),
    #[serde(rename = "pdfannotator")]
    Pdfannotator(Pdfannotator),
    #[serde(rename = "assign")]
    Assign(Assign),
    #[serde(rename = "label")]
    Label(Label),
    #[serde(rename = "url")]
    Url(Url),
    #[serde(rename = "page")]
    Page(Page),
    #[serde(rename = "quiz")]
    Quiz(Quiz),
    #[serde(rename = "feedback")]
    Feedback(Feedback),
    #[serde(rename = "glossary")]
    Glossary(Glossary),
    #[serde(rename = "vpl")]
    Vpl(Vpl),
    #[serde(rename = "lti")]
    Lti(Lti),
    #[serde(rename = "forum")]
    Forum(Forum),
    #[serde(rename = "hsuforum")]
    HsuForum(HsuForum),
    #[serde(rename = "grouptool")]
    Grouptool(Grouptool),
    #[serde(other)]
    Unknown,
}

impl Download for Module {
    async fn download(&self, api: &Api, path: &Path) -> Result<()> {
        match self {
            Module::Resource(resource) => resource.download(api, path).await,
            Module::Folder(folder) => folder.download(api, path).await,
            Module::Pdfannotator(pdfannotator) => pdfannotator.download(api, path).await,
            Module::Assign(assign) => assign.download(api, path).await,
            Module::Label(label) => label.download(api, path).await,
            Module::Url(url) => url.download(api, path).await,
            Module::Page(page) => page.download(api, path).await,
            Module::Quiz(quiz) => quiz.download(api, path).await,
            Module::Glossary(glossary) => glossary.download(api, path).await,
            Module::Vpl(vpl) => vpl.download(api, path).await,
            _ => {
                // TODO add missing module downloaders
                Ok(())
            }
        }
    }
}

// Files
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Resource {
    pub id: u64,
    pub name: String,
    pub contents: Option<Vec<Content>>,
}
impl Download for Resource {
    async fn download(&self, api: &Api, path: &Path) -> Result<()> {
        let contents = match &self.contents {
            Some(contents) => contents,
            None => return Ok(()),
        };

        let download_path = path.join(&self.name);

        let file_futures = contents
            .iter()
            .map(|content| content.download(api, &download_path));

        let downloads = join_all(file_futures).await;
        // Return error if any download fails
        for download in downloads {
            download?;
        }

        Ok(())
    }
}
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Folder {
    pub id: u64,
    pub name: String,
    pub contents: Option<Vec<Content>>,
}
impl Download for Folder {
    async fn download(&self, api: &Api, path: &Path) -> Result<()> {
        let contents = match &self.contents {
            Some(contents) => contents,
            None => return Ok(()),
        };

        let download_path = path.join(&self.name);

        let file_futures = contents
            .iter()
            .map(|content| content.download(api, &download_path));

        let downloads = join_all(file_futures).await;
        // Return error if any download fails
        for download in downloads {
            download?;
        }

        Ok(())
    }
}

// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Pdfannotator {
    pub id: u64,
    pub name: String,
    pub contents: Option<Vec<Content>>,
}
impl Download for Pdfannotator {
    async fn download(&self, api: &Api, path: &Path) -> Result<()> {
        let contents = match &self.contents {
            Some(contents) => contents,
            None => return Ok(()),
        };

        let download_path = path.join(&self.name);

        let file_futures = contents
            .iter()
            .map(|content| content.download(api, &download_path));

        let downloads = join_all(file_futures).await;
        // Return error if any download fails
        for download in downloads {
            download?;
        }

        Ok(())
    }
}

// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Assign {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub description: Option<String>,
}
impl Download for Assign {
    async fn download(&self, api: &Api, path: &Path) -> Result<()> {
        let path = path.join(&self.name);
        if let Some(description) = &self.description {
            if (api.download_options.force_update)
                || (check_for_updated_contents(
                    &description,
                    &path.join(".moo-dl.description.html"),
                )
                .await?)
            {
                let pdf_path = path.join("description.pdf");
                // We can ignore errors as these happen if the file doesn't exist
                let _ = api
                    .download_options
                    .file_update_strategy
                    .force_archive_file(&pdf_path, false)
                    .await;
                api.save_page(self.url.to_string(), &pdf_path, None).await?;
            }
        }

        // TODO add extra request etc., as Assignments do not provide most information via core_course_get_contents
        // Use: mod_assign_get_submission_status with assignid=<assignmentid/instance>

        Ok(())
    }
}

// Basic elements (may need to be converted)
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Label {
    pub id: u64,
    pub name: String,
    pub description: String,
}
impl Download for Label {
    async fn download(&self, api: &Api, path: &Path) -> Result<()> {
        let path = path.join(format!("{}.html", self.name));

        api.save_text(&self.description, &path).await?;

        Ok(())
    }
}

// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Url {
    pub id: u64,
    pub name: String,
    pub contents: Vec<Content>,
}
impl Download for Url {
    async fn download(&self, api: &Api, path: &Path) -> Result<()> {
        let download_path = path.join(&self.name);

        let file_futures = self
            .contents
            .iter()
            .map(|content| content.download(api, &download_path));

        let downloads = join_all(file_futures).await;
        // Return error if any download fails
        for download in downloads {
            download?;
        }

        Ok(())
    }
}

// Pages that need to be converted
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Page {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub contents: Option<Vec<Content>>,
}
impl Download for Page {
    async fn download(&self, api: &Api, path: &Path) -> Result<()> {
        let contents = match &self.contents {
            Some(contents) => contents,
            None => return Ok(()),
        };

        let download_path = path.join(&self.name);

        // Create a PDF version from the website, in case of extra content, e.g. images that aren't in the pure html
        let lowest_last_modified = contents
            .iter()
            .map(|content| match content {
                Content::File(file) => Some(file.timemodified),
                Content::Url(url) => Some(url.timemodified),
                _ => None,
            })
            .flatten()
            .min();
        if let Some(last_modified) = lowest_last_modified {
            let pdf_path = download_path.join("page.pdf");
            // Generate_pdf
            api.save_page(&self.url, &pdf_path, Some(last_modified))
                .await?;
        }

        let file_futures = contents
            .iter()
            .map(|content| content.download(api, &download_path));

        let downloads = join_all(file_futures).await;
        // Return error if any download fails
        for download in downloads {
            download?;
        }

        Ok(())
    }
}

// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Quiz {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub instance: u64,
}
impl Download for Quiz {
    async fn download(&self, api: &Api, path: &Path) -> Result<()> {
        let download_path = path.join(&self.name);
        // We need a valid session cookie for the following (but we don't actually need the credential)
        let credential_guard = api.acuire_credential().await?;
        let credential = credential_guard
            .as_ref()
            .ok_or("Could not get Credential for session cookie")?;

        trace!(
            "Attempting to get available quiz attempts for quiz id: {} name: {}",
            self.id,
            self.name
        );
        info!("Checking for quiz attempts for quiz: {}", self.name);
        let response = api
            .client
            .get(&self.url)
            .header(
                "Cookie",
                "MoodleSession=".to_string() + credential.session_cookie.as_str(),
            )
            .send()
            .await?;

        let html = response.text().await?;
        let document = Document::from(html.as_str());

        let url_start = credential.instance_url.to_string() + "mod/quiz/review.php";
        let url_contains = "cmid=".to_string() + self.id.to_string().as_str();

        let response_url: Vec<_> = document
            .find(Name("a"))
            .map(|element| element.attr("href"))
            .flatten()
            .filter(|href| href.starts_with(&url_start) && href.contains(&url_contains))
            .collect();
        trace!("Quiz: Response token: {:?}", response_url);

        let page_futures = response_url.into_iter().map(|url| {
            let download_path = download_path.clone();
            async move {
                let regex = Regex::new(r"attempt=(\d+)").unwrap();
                let attemptnr = regex
                    .captures(url)
                    .and_then(|captures| captures.get(1).map(|match_| match_.as_str()))
                    .ok_or("Could not extract attemptnr from url")?;
                api.save_page(
                    url,
                    &download_path.join(attemptnr.to_string() + ".pdf"),
                    None,
                )
                .await
            }
        });

        let page_saves = join_all(page_futures).await;
        // Return error if any download fails
        for page_save in page_saves {
            page_save?;
        }

        Ok(())
    }
}

// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Feedback {
    pub id: u64,
    pub name: String,
}

// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Glossary {
    pub id: u64,
    pub name: String,
}
impl Download for Glossary {
    async fn download(&self, api: &Api, path: &Path) -> Result<()> {
        let pdf_path = path.join(self.name.to_string() + ".pdf");
        let mut glossary_url = api.api_credential.instance_url.to_string();
        glossary_url.push_str("mod/glossary/print.php?id=");
        glossary_url.push_str(&self.id.to_string());
        glossary_url.push_str("&mode&hook=ALL&sortkey&sortorder&offset=0&pagelimit=0");

        if api.download_options.force_update {
            // We can ignore errors as these happen if the file doesn't exist
            let _ = api
                .download_options
                .file_update_strategy
                .force_archive_file(&pdf_path, false)
                .await;
        }

        api.save_page(glossary_url, &pdf_path, None).await?;
        Ok(())
    }
}

// Extra
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Vpl {
    pub id: u64,
    pub name: String,
    pub url: String,
}
impl Download for Vpl {
    async fn download(&self, api: &Api, path: &Path) -> Result<()> {
        let vpl_folder = path.join(self.name.to_string() + ".vpl");
        // Support force update
        if api.download_options.force_update {
            // We can ignore errors as these happen if the file doesn't exist
            let _ = api
                .download_options
                .file_update_strategy
                .force_archive_file(&vpl_folder, true)
                .await;
        }

        let mut description_page_url = api.api_credential.instance_url.to_string();
        description_page_url.push_str("mod/vpl/view.php?id=");
        description_page_url.push_str(&self.id.to_string());

        // https://moodle.rwth-aachen.de/mod/vpl/views/downloadrequiredfiles.php?id=1643546
        let description_files_url = api
            .api_credential
            .instance_url
            .join("mod/vpl/views/downloadrequiredfiles.php")?
            .join(&("?id=".to_string() + &self.id.to_string()))?;

        let mut submission_page_url = api.api_credential.instance_url.to_string();
        submission_page_url.push_str("mod/vpl/forms/submissionview.php?id=");
        submission_page_url.push_str(&self.id.to_string());

        // trace!("VPL: description page url: {}", description_page_url);
        // trace!("VPL: description files url: {}", description_files_url);
        // trace!("VPL: submission page url: {}", submission_page_url);

        let credential_guard = api.acuire_credential().await?;
        let credential = credential_guard
            .as_ref()
            .ok_or("Could not get Credential for session cookie")?;

        // Download files
        let description_file_request = api.client.get(description_files_url).header(
            "Cookie",
            "MoodleSession=".to_string() + credential.session_cookie.as_str(),
        );
        api.download_options
            .file_update_strategy
            .download_from_requestbuilder(
                description_file_request,
                &vpl_folder.join("description_files.zip"),
                None,
            )
            .await?;

        // Save pages
        api.save_page(
            description_page_url,
            &vpl_folder.join("description.pdf"),
            None,
        )
        .await?;
        api.save_page(
            &submission_page_url,
            &vpl_folder.join("submission.pdf"),
            None,
        )
        .await?;

        // // Check if submission_files_url exists
        // let submission_files_path = vpl_folder.join("submission_files.zip");
        // // Check if submission_files are downloaded
        // if api
        //     .download_options
        //     .file_update_strategy
        //     .archive_file(&submission_files_path, None)
        //     .await?
        // {
        let response = api
            .client
            .get(submission_page_url)
            .header(
                "Cookie",
                "MoodleSession=".to_string() + credential.session_cookie.as_str(),
            )
            .send()
            .await?;

        let html = response.text().await?;
        let document = Document::from(html.as_str());

        let url_start =
            credential.instance_url.to_string() + "mod/vpl/views/downloadsubmission.php";
        let url_contains = "id=".to_string() + self.id.to_string().as_str();

        let response_url: Vec<_> = document
            .find(Name("a"))
            .map(|element| element.attr("href"))
            .flatten()
            .filter(|href| href.starts_with(&url_start) && href.contains(&url_contains))
            .collect();

        let page_futures = response_url.into_iter().map(|url| {
            let download_path = vpl_folder.clone();
            async move {
                let regex = Regex::new(r"submissionid=(\d+)").unwrap();
                let submissionid = regex
                    .captures(url)
                    .and_then(|captures| captures.get(1).map(|match_| match_.as_str()))
                    .ok_or("Could not extract submissionid from url")?;
                let request = api.client.get(url).header(
                    "Cookie",
                    "MoodleSession=".to_string() + credential.session_cookie.as_str(),
                );
                api.download_options
                    .file_update_strategy
                    .download_from_requestbuilder(
                        request,
                        &download_path.join(
                            "submission-files_submissionid-".to_string() + submissionid + ".zip",
                        ),
                        None,
                    )
                    .await
            }
        });

        let page_saves = join_all(page_futures).await;
        // Return error if any download fails
        for page_save in page_saves {
            page_save?;
        }
        // }

        Ok(())
    }
}

// At RWTH mainly OpenCast
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Lti {
    pub id: u64,
    pub name: String,
}

// Unsupported for now
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Forum {
    pub id: u64,
    pub name: String,
}
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct HsuForum {
    pub id: u64,
    pub name: String,
}
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Grouptool {
    pub id: u64,
    pub name: String,
}
