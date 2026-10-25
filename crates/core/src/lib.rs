//! Nectar core: loads Markdown sources into a format-neutral document, enriches it with transform
//! passes and hands it to a [`Renderer`].
//!
//! The core is synchronous and works in memory. It knows no code engine, no output format and no
//! network; each is an adapter behind [`CodeEngine`], [`Renderer`] and [`Fetcher`].

mod config;
mod diagnostic;
mod download;
mod engine;
mod fetch;
mod load;
pub mod model;
mod parse;
mod passes;
mod print;
mod project;
mod render;

pub use config::{CONFIG_FILE, Config, DocumentConfig, Split};
pub use diagnostic::{Diagnostic, Origin, Severity};
pub use engine::{CodeEngine, EngineError, EngineOutput};
pub use fetch::{FetchError, Fetched, Fetcher};
pub use passes::slugify;
pub use print::{PrintSettings, pdf_file};
pub use project::{Build, Documents, Project};
pub use render::{Output, RenderContext, Renderer};
