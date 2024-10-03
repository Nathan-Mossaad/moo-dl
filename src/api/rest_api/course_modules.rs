use serde::Deserialize;

// TODO remove dead_code warning
#[allow(dead_code)]
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

// Files
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Resource {
    pub id: u64,
    pub name: String,
}

// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Folder {
    pub id: u64,
    pub name: String,
}

// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Pdfannotator {
    pub id: u64,
    pub name: String,
}

// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Assign {
    pub id: u64,
    pub name: String,
}

// Basic elements (may need to be converted)
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Label {
    pub id: u64,
    pub name: String,
    pub description: String,
}

// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Url {
    pub id: u64,
    pub name: String,
}

// Pages that need to be converted
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Page {
    pub id: u64,
    pub name: String,
}

// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Quiz {
    pub id: u64,
    pub name: String,
}

// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Feedback {
    pub id: u64,
    pub name: String,
}

// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Glossary {
    pub id: u64,
    pub name: String,
}

// Extra
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Vpl {
    pub id: u64,
    pub name: String,
}

// At RWTH mainly OpenCast
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Lti {
    pub id: u64,
    pub name: String,
}

// Unsupported for now
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Forum {
    pub id: u64,
    pub name: String,
}
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct HsuForum {
    pub id: u64,
    pub name: String,
}
// TODO remove dead_code warning
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Grouptool {
    pub id: u64,
    pub name: String,
}
