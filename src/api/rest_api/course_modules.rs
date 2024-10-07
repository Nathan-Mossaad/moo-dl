use std::fmt::Debug;
use std::path::Path;

use futures::future::join_all;
use serde::Deserialize;

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
            Module::Url(url) => url.download(api, path).await,
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
                    .force_archive_file(&pdf_path)
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
}

// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Quiz {
    pub id: u64,
    pub name: String,
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

// Extra
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Vpl {
    pub id: u64,
    pub name: String,
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
