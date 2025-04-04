use super::*;

// content_types, that are subtypes from modules
pub(super) mod content_types;
// modules
mod apiurl;
mod assign;
mod feedback;
mod folder;
mod forum;
mod glossary;
mod grouptool;
mod hsuforum;
mod label;
mod lti;
mod page;
mod pdfannotator;
mod quiz;
mod resource;
mod vpl;

// Reexport
use apiurl::*;
use assign::*;
use feedback::*;
use folder::*;
use forum::*;
use glossary::*;
use grouptool::*;
use hsuforum::*;
use label::*;
use lti::*;
use page::*;
use pdfannotator::*;
use quiz::*;
use resource::*;
use vpl::*;

#[derive(Debug, Deserialize)]
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
    ApiUrl(ApiUrl),
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
    async fn download(&self, config: Arc<Config>, path: &Path) -> Result<()> {
        match &self {
            Module::Resource(a) => a.download(config, path).await?,
            Module::ApiUrl(a) => a.download(config, path).await?,
            Module::Folder(a) => a.download(config, path).await?,
            Module::Label(a) => a.download(config, path).await?,
            Module::Quiz(a) => a.download(config, path).await?,
            Module::Lti(a) => a.download(config, path).await?,
            Module::Page(a) => a.download(config, path).await?,
            Module::Glossary(a) => a.download(config, path).await?,
            Module::Vpl(a) => a.download(config, path).await?,
            Module::Assign(a) => a.download(config, path).await?,
            // TODO
            _ => {}
        }
        Ok(())
    }
}
