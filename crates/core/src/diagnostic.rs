use std::fmt;
use std::path::PathBuf;

/// How serious a diagnostic is. Errors make `nectar build` fail; warnings do not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Warning,
    Error,
}

/// Where a node or a diagnostic comes from: a file relative to the project root and a 1-based
/// line. Line 0 stands for the file as a whole.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Origin {
    pub file: PathBuf,
    pub line: usize,
}

impl Origin {
    pub fn new(file: impl Into<PathBuf>, line: usize) -> Self {
        Self {
            file: file.into(),
            line,
        }
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "{}", self.file.display())
        } else {
            write!(f, "{}:{}", self.file.display(), self.line)
        }
    }
}

/// A problem found while loading, transforming or rendering. Diagnostics are data; the shells
/// decide how to show them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub origin: Option<Origin>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>, origin: Option<Origin>) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            origin,
        }
    }

    pub fn warning(message: impl Into<String>, origin: Option<Origin>) -> Self {
        Self {
            severity: Severity::Warning,
            message: message.into(),
            origin,
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity = match self.severity {
            Severity::Warning => "warning",
            Severity::Error => "error",
        };
        match &self.origin {
            Some(origin) => write!(f, "{severity}: {origin}: {}", self.message),
            None => write!(f, "{severity}: {}", self.message),
        }
    }
}
