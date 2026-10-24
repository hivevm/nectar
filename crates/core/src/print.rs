use serde::Deserialize;

use crate::model::Document;

/// Settings from `[document.print]`, shared by the renderers for print: the print view in the
/// browser and the PDF file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct PrintSettings {
    /// The print view in the browser.
    pub enabled: bool,
    /// The PDF file.
    pub pdf: bool,
    /// CSS page size, such as `A4`, `letter` or `A5 landscape`.
    pub size: String,
    /// CSS page margins.
    pub margin: String,
    pub cover: bool,
    pub subtitle: Option<String>,
    /// An image on the cover, relative to the project root.
    pub cover_image: Option<String>,
    /// Every top-level chapter starts with a page of its own that shows its number and title.
    pub chapter_pages: bool,
    /// `draft`, `confidential` or an image relative to the project root, behind every page.
    pub watermark: Option<String>,
}

impl Default for PrintSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            pdf: true,
            size: "A4".to_string(),
            margin: "20mm 18mm".to_string(),
            cover: true,
            subtitle: None,
            cover_image: None,
            chapter_pages: false,
            watermark: None,
        }
    }
}

impl PrintSettings {
    /// The settings of a document; an error names what is wrong in its table.
    pub fn of(document: &Document) -> Result<Self, String> {
        toml::Value::Table(document.print.clone())
            .try_into()
            .map_err(|e: toml::de::Error| e.message().to_string())
    }
}

/// The PDF file of a document, relative to the output root: `<document>/print/<stem>.pdf`.
pub fn pdf_file(document: &Document) -> String {
    let stem = document
        .root
        .source
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "document".to_string());
    if document.path.is_empty() {
        format!("print/{stem}.pdf")
    } else {
        format!("{}/print/{stem}.pdf", document.path)
    }
}
