use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::load::read_bytes_inside;
use crate::model::Document;

/// What a renderer gets besides the documents.
pub struct RenderContext<'a> {
    pub root: &'a Path,
    pub config: &'a Config,
}

impl RenderContext<'_> {
    /// Reads a project file, such as a theme file, relative to the project root.
    pub fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        read_bytes_inside(self.root, path)
    }
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
