use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::files::Files;
use crate::model::Document;

/// What a renderer gets besides the documents: the project's files, such as theme files, and its
/// configuration.
pub struct RenderContext<'a> {
    pub files: &'a dyn Files,
    pub config: &'a Config,
}

/// The files a renderer produces, with the project files it read and its diagnostics.
#[derive(Debug, Default)]
pub struct Output {
    /// Output files, relative to the output root.
    pub files: BTreeMap<PathBuf, Vec<u8>>,
    /// Project files the output depends on, relative to the project root.
    pub sources: BTreeSet<PathBuf>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Maps the enriched documents to an output format.
pub trait Renderer {
    fn render(&self, documents: &[Document], ctx: &RenderContext<'_>) -> Output;
}
