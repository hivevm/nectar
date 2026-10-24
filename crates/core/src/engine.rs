use crate::model::{Graphic, HighlightedCode};

pub type EngineError = Box<dyn std::error::Error + Send + Sync>;

/// What a code engine turns a code block into.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineOutput {
    Highlighted(HighlightedCode),
    Graphic(Graphic),
}

/// The port for highlighting and diagrams. Adapters implement it; the core knows no adapter.
pub trait CodeEngine: Send + Sync {
    /// A short name for diagnostics and the result cache.
    fn name(&self) -> &str;

    /// Whether the engine handles the language given in a code block's info string.
    fn supports(&self, language: &str) -> bool;

    fn render(&self, language: &str, code: &str) -> Result<EngineOutput, EngineError>;
}
