use futures::future::BoxFuture;

use super::{modules::content_types::ContentFile, *};

#[derive(Debug, Deserialize)]
pub(super) struct ModAssignGetSubmissionStatus {
    lastattempt: Option<Lastattempt>,
    feedback: Option<Feedback>,
    assignmentdata: Option<AssignmentData>,
}
impl Download for ModAssignGetSubmissionStatus {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        tracing::trace!("\n{:?}", self);

        // Use a lifetime dependent BoxFuture instead of 'static.
        let mut tasks: Vec<BoxFuture<Result<()>>> = Vec::new();

        if let Some(lastattempt) = &self.lastattempt {
            tasks.push(Box::pin(lastattempt.download(config.clone(), path)));
        }
        if let Some(feedback) = &self.feedback {
            tasks.push(Box::pin(feedback.download(config.clone(), path)));
        }
        if let Some(assignmentdata) = &self.assignmentdata {
            tasks.push(Box::pin(assignmentdata.download(config.clone(), path)));
        }

        // Return an error if one occured
        for res in join_all(tasks).await {
            res.context("Failed Downloading sub-ressource")?;
        }
        Ok(())
    }
}

// lastattempt
#[derive(Deserialize, Debug)]
pub(super) struct Lastattempt {
    submission: Option<Submission>,
    teamsubmission: Option<Submission>,
}
impl Download for Lastattempt {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        let path = &path.join("last_attempt");
        // We usually only have one of the two submission types, therefore additional threads are unnecessary
        let mut res = Vec::new();
        if let Some(submission) = &self.submission {
            res.push(submission.download(config.clone(), path).await);
        }
        if let Some(teamsubmission) = &self.teamsubmission {
            res.push(teamsubmission.download(config.clone(), path).await);
        }
        // Return an error if one occured
        for res in res {
            res.context("Failed Downloading sub-ressource")?;
        }
        Ok(())
    }
}
#[derive(Deserialize, Debug)]
pub(super) struct Submission {
    plugins: Vec<Plugin>,
}
impl Download for Submission {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        // Create a task for each content
        let tasks = self
            .plugins
            .iter()
            .map(|r| r.download(config.clone(), &path));
        // Return an error if one occured
        for res in join_all(tasks).await {
            res.context("Failed Resource")?;
        }
        Ok(())
    }
}

// feedback
#[derive(Deserialize, Debug)]
pub(super) struct Feedback {
    plugins: Vec<Plugin>,
}
impl Download for Feedback {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        let path = &path.join("feedback");
        // Create a task for each content
        let tasks = self
            .plugins
            .iter()
            .map(|r| r.download(config.clone(), &path));
        // Return an error if one occured
        for res in join_all(tasks).await {
            res.context("Failed Resource")?;
        }
        Ok(())
    }
}

// assignmentdata
#[derive(Deserialize, Debug)]
pub(super) struct AssignmentData {
    attachments: Attachments,
}
impl Download for AssignmentData {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        self.attachments.download(config, path).await?;
        Ok(())
    }
}
#[derive(Deserialize, Debug)]
pub(super) struct Attachments {
    intro: Vec<ContentFile>,
}
impl Download for Attachments {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        // Create a task for each content
        let tasks = self.intro.iter().map(|r| r.download(config.clone(), &path));
        // Return an error if one occured
        for res in join_all(tasks).await {
            res.context("Failed Resource")?;
        }
        Ok(())
    }
}

// For all: Universal
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub(super) enum Plugin {
    #[serde(rename = "file")]
    File(PluginFile),
    #[serde(rename = "editpdf")]
    EditPdf(EditPdf),
    #[serde(other)]
    Unknown,
}
impl Download for Plugin {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        match &self {
            Plugin::File(plugin_file) => plugin_file.download(config, path).await?,
            Plugin::EditPdf(edit_pdf) => edit_pdf.download(config, path).await?,
            Plugin::Unknown => {}
        }
        Ok(())
    }
}
// file
#[derive(Deserialize, Debug)]
pub(super) struct PluginFile {
    fileareas: Vec<PluginFileFiles>,
}
impl Download for PluginFile {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        // Create a task for each content
        let tasks = self
            .fileareas
            .iter()
            .map(|r| r.download(config.clone(), &path));
        // Return an error if one occured
        for res in join_all(tasks).await {
            res.context("Failed Resource")?;
        }
        Ok(())
    }
}
#[derive(Deserialize, Debug)]
pub(super) struct PluginFileFiles {
    files: Vec<ContentFile>,
}
impl Download for PluginFileFiles {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        // Create a task for each content
        let tasks = self.files.iter().map(|r| r.download(config.clone(), &path));
        // Return an error if one occured
        for res in join_all(tasks).await {
            res.context("Failed Resource")?;
        }
        Ok(())
    }
}
// editpdf
#[derive(Deserialize, Debug)]
pub(super) struct EditPdf {
    fileareas: Vec<EditPdfArea>,
}
impl Download for EditPdf {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        // Create a task for each content
        let tasks = self
            .fileareas
            .iter()
            .map(|r| r.download(config.clone(), &path));
        // Return an error if one occured
        for res in join_all(tasks).await {
            res.context("Failed Resource")?;
        }
        Ok(())
    }
}
#[derive(Deserialize, Debug)]
#[serde(tag = "area")]
pub(super) enum EditPdfArea {
    #[serde(rename = "download", alias = "combined")]
    Download(EditPdfAreaDownload),
    #[serde(other)]
    Unknown,
}
impl Download for EditPdfArea {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        match &self {
            EditPdfArea::Download(edit_pdf_area_download) => {
                edit_pdf_area_download.download(config, path).await?;
            }
            EditPdfArea::Unknown => {}
        }
        Ok(())
    }
}
#[derive(Deserialize, Debug)]
pub(super) struct EditPdfAreaDownload {
    files: Vec<ContentFile>,
}
impl Download for EditPdfAreaDownload {
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        // Create a task for each content
        let tasks = self.files.iter().map(|r| r.download(config.clone(), &path));
        // Return an error if one occured
        for res in join_all(tasks).await {
            res.context("Failed Resource")?;
        }
        Ok(())
    }
}
