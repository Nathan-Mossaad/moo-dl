use super::*;

// content_types, that are subtypes from modules
mod content_types;
// modules
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
mod url;
mod vpl;

// Reexport
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
use url::*;
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
