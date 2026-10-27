use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::config::{CONFIG_FILE, Config};
use crate::diagnostic::{Diagnostic, Severity};
use crate::download::Downloader;
use crate::engine::CodeEngine;
use crate::fetch::Fetcher;
use crate::files::Files;
use crate::load::Loader;
use crate::model::Document;
use crate::passes::{self, CodeEngines, EngineCache};
use crate::render::{RenderContext, Renderer};
use crate::workers::{InTurn, Workers};

/// A project: its files, its configuration and the code engines to use.
pub struct Project {
    files: Box<dyn Files>,
    config: Config,
    engines: Vec<Box<dyn CodeEngine>>,
    workers: Box<dyn Workers>,
    environment: Vec<(String, String)>,
    include_drafts: bool,
    cache: EngineCache,
    downloader: Downloader,
}

/// The documents of a project after all transform passes.
pub struct Documents {
    pub documents: Vec<Document>,
    pub sources: BTreeSet<PathBuf>,
    pub diagnostics: Vec<Diagnostic>,
}

/// The result of a build, held in memory.
#[derive(Debug, Default)]
pub struct Build {
    /// Output files, relative to the output root.
    pub files: BTreeMap<PathBuf, Vec<u8>>,
    /// Every project file the build read, relative to the project root.
    pub sources: BTreeSet<PathBuf>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Project {
    /// Opens the project whose files `files` gives, reading its `nectar.toml`.
    pub fn load(files: Box<dyn Files>) -> Result<Self, Diagnostic> {
        let config = Config::load(files.as_ref())?;
        Ok(Self {
            files,
            config,
            engines: Vec::new(),
            workers: Box::new(InTurn),
            environment: Vec::new(),
            include_drafts: false,
            cache: Mutex::new(HashMap::new()),
            downloader: Downloader::new(),
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Adds a code engine. Engines are asked in the order they were added.
    pub fn add_engine(&mut self, engine: Box<dyn CodeEngine>) {
        self.engines.push(engine);
    }

    /// Sets the adapter that runs the code engines side by side. Without one they run in turn.
    pub fn set_workers(&mut self, workers: Box<dyn Workers>) {
        self.workers = workers;
    }

    /// Sets the environment variables of the host. Those named `NECTAR_<name>` override the
    /// variable `<name>` of `nectar.toml`; without any, the configuration alone counts.
    pub fn set_environment(&mut self, environment: impl IntoIterator<Item = (String, String)>) {
        self.environment = environment.into_iter().collect();
    }

    /// Whether files with `draft = true` are part of the documents.
    pub fn set_include_drafts(&mut self, include: bool) {
        self.include_drafts = include;
    }

    /// Sets the adapter that fetches images from the web. Without one they stay links.
    pub fn set_fetcher(&mut self, fetcher: Box<dyn Fetcher>) {
        self.downloader.set_fetcher(fetcher);
    }

    /// Whether images from the web stay links instead of being downloaded.
    pub fn set_offline(&mut self, offline: bool) {
        self.downloader.set_offline(offline);
    }

    /// Loads every start document and runs the transform passes.
    pub fn documents(&self) -> Documents {
        let vars = self
            .config
            .variables_with_environment(self.environment.iter().cloned());
        let mut loader = Loader::new(self.files.as_ref(), &vars, self.include_drafts);
        let mut documents = Vec::new();
        let mut diagnostics = Vec::new();

        for (index, document) in self.config.documents.iter().enumerate() {
            let Some(root) = loader.chapter(&document.source, "", None) else {
                continue;
            };
            let mut document = Document {
                title: root.title.clone(),
                path: self.config.document_path(index),
                split: document.split,
                numbered: document.numbered,
                contents: document.contents,
                print: document.print.clone(),
                root,
                footnotes: Vec::new(),
                assets: BTreeMap::new(),
                downloads: BTreeMap::new(),
            };
            passes::run(
                &mut document,
                self.files.as_ref(),
                &CodeEngines {
                    engines: &self.engines,
                    workers: self.workers.as_ref(),
                    cache: &self.cache,
                },
                &self.downloader,
                self.config.html,
                &mut diagnostics,
            );
            documents.push(document);
        }

        let mut sources = loader.sources;
        sources.insert(PathBuf::from(CONFIG_FILE));
        loader.diagnostics.extend(diagnostics);
        Documents {
            documents,
            sources,
            diagnostics: loader.diagnostics,
        }
    }

    /// Builds the whole project with every renderer. Renderers may write the same file only
    /// with the same content, such as shared theme files.
    pub fn build(&self, renderers: &[&dyn Renderer]) -> Build {
        let Documents {
            documents,
            mut sources,
            mut diagnostics,
        } = self.documents();
        let context = RenderContext {
            files: self.files.as_ref(),
            config: &self.config,
        };
        let mut files: BTreeMap<PathBuf, Vec<u8>> = BTreeMap::new();
        for renderer in renderers {
            let output = renderer.render(&documents, &context);
            sources.extend(output.sources);
            diagnostics.extend(output.diagnostics);
            for (path, bytes) in output.files {
                match files.get(&path) {
                    Some(existing) if *existing != bytes => diagnostics.push(Diagnostic::error(
                        format!(
                            "two renderers write different content to {}",
                            path.display()
                        ),
                        None,
                    )),
                    _ => {
                        files.insert(path, bytes);
                    }
                }
            }
        }
        for document in &documents {
            for (source, target) in &document.assets {
                if let Some(bytes) = document.downloads.get(source) {
                    files.insert(target.clone(), bytes.clone());
                    continue;
                }
                match self.files.read(source) {
                    Ok(bytes) => {
                        files.insert(target.clone(), bytes);
                        sources.insert(source.clone());
                    }
                    Err(e) => diagnostics.push(Diagnostic::error(
                        format!("cannot read {}: {e}", source.display()),
                        None,
                    )),
                }
            }
        }
        Build {
            files,
            sources,
            diagnostics,
        }
    }
}

impl Build {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }
}
