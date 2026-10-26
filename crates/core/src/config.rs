use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::diagnostic::{Diagnostic, Origin};
use crate::files::{Files, read_text};

/// Environment variables with this prefix are variables of the documents.
pub const ENVIRONMENT_PREFIX: &str = "NECTAR_";

/// The name of the project configuration file in the project root.
pub const CONFIG_FILE: &str = "nectar.toml";

/// The contents of `nectar.toml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub title: String,
    /// Name of a theme directory below `themes/`; the built-in theme is used when absent.
    pub theme: Option<String>,
    #[serde(default = "default_output")]
    pub output: PathBuf,
    #[serde(default, rename = "document")]
    pub documents: Vec<DocumentConfig>,
    #[serde(default)]
    pub vars: BTreeMap<String, toml::Value>,
    /// Whether raw HTML in the sources reaches the website; without it, it is shown as text.
    #[serde(default)]
    pub html: bool,
    /// Engine tables, read by the shell that composes the adapters.
    #[serde(default)]
    pub engine: toml::Table,
    /// Server settings, read by `nectar serve`.
    #[serde(default)]
    pub serve: toml::Table,
}

fn default_output() -> PathBuf {
    PathBuf::from("public")
}

/// One start document.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentConfig {
    /// The start document, relative to the project root.
    pub source: PathBuf,
    #[serde(default)]
    pub split: Split,
    /// Output directory of the document, relative to the output root. Defaults to the root for a
    /// single document and to the file stem otherwise.
    pub path: Option<String>,
    /// Section numbers in front of headings, such as `1.2.`.
    #[serde(default = "enabled")]
    pub numbered: bool,
    /// A table of contents of the whole document on its first page.
    #[serde(default = "enabled")]
    pub contents: bool,
    /// Settings of the print renderer, `[document.print]`.
    #[serde(default)]
    pub print: toml::Table,
}

fn enabled() -> bool {
    true
}

/// How the HTML output of a document is divided into pages.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Split {
    /// One page per chapter, that is per included file.
    #[default]
    Chapter,
    /// The whole document on one page.
    None,
}

impl Config {
    /// Reads `nectar.toml` from the project root.
    pub fn load(files: &dyn Files) -> Result<Self, Diagnostic> {
        let origin = Some(Origin::new(CONFIG_FILE, 0));
        let text = read_text(files, Path::new(CONFIG_FILE)).map_err(|e| {
            Diagnostic::error(
                format!("cannot read the configuration: {e}"),
                origin.clone(),
            )
        })?;
        let config: Config = toml::from_str(&text).map_err(|e| {
            Diagnostic::error(format!("invalid configuration: {e}"), origin.clone())
        })?;
        if config.documents.is_empty() {
            return Err(Diagnostic::error("no [[document]] configured", origin));
        }
        Ok(config)
    }

    /// The configured variables as text.
    pub fn variables(&self) -> BTreeMap<String, String> {
        to_strings(&self.vars)
    }

    /// The configured variables, overridden by environment variables with the prefix
    /// `NECTAR_`: `NECTAR_VERSION` sets `{{VERSION}}`. Other environment variables stay out of
    /// reach, so nothing secret ends up in a page by accident.
    pub fn variables_with_environment(
        &self,
        environment: impl IntoIterator<Item = (String, String)>,
    ) -> BTreeMap<String, String> {
        let mut vars = self.variables();
        for (name, value) in environment {
            if let Some(name) = name.strip_prefix(ENVIRONMENT_PREFIX)
                && !name.is_empty()
            {
                vars.insert(name.to_string(), value);
            }
        }
        vars
    }

    /// The output directory of a document, relative to the output root, without slashes at
    /// either end.
    pub fn document_path(&self, index: usize) -> String {
        let document = &self.documents[index];
        match &document.path {
            Some(path) => path.trim_matches('/').to_string(),
            None if self.documents.len() == 1 => String::new(),
            None => document
                .source
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_default(),
        }
    }
}

pub(crate) fn to_strings(values: &BTreeMap<String, toml::Value>) -> BTreeMap<String, String> {
    values
        .iter()
        .map(|(name, value)| {
            let text = match value {
                toml::Value::String(text) => text.clone(),
                other => other.to_string(),
            };
            (name.clone(), text)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_variables_with_the_prefix_override_the_configuration() {
        let config: Config = toml::from_str(
            "[[document]]\nsource = \"a.md\"\n[vars]\nVERSION = \"1.0\"\nname = \"Doc\"\n",
        )
        .unwrap();
        let environment = [
            ("NECTAR_VERSION".to_string(), "2.0".to_string()),
            ("NECTAR_".to_string(), "ignored".to_string()),
            ("HOME".to_string(), "/home/secret".to_string()),
        ];
        let vars = config.variables_with_environment(environment);
        assert_eq!(vars["VERSION"], "2.0");
        assert_eq!(vars["name"], "Doc");
        assert!(!vars.contains_key("HOME"));
        assert!(!vars.contains_key(""));
    }
}
