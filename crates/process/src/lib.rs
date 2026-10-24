//! Code engine that renders diagrams by running a local command: the code goes to the command's
//! standard input, SVG is expected on its standard output.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use nectar_core::model::Graphic;
use nectar_core::{CodeEngine, EngineError, EngineOutput};
use serde::Deserialize;

/// Settings of one process engine, from `[engine.process.<name>]`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    /// The info-string languages the engine handles. A table maps each language to the name
    /// used for `{lang}` in `input`.
    pub languages: Languages,
    /// Template for the standard input; `{code}` and `{lang}` are replaced. Defaults to the code.
    pub input: Option<String>,
    /// Seconds before the command is stopped.
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Languages {
    List(Vec<String>),
    Map(BTreeMap<String, String>),
}

fn default_timeout() -> u64 {
    30
}

pub struct Process {
    name: String,
    config: ProcessConfig,
    languages: BTreeMap<String, String>,
}

impl Process {
    pub fn new(name: &str, config: ProcessConfig) -> Self {
        let languages = match &config.languages {
            Languages::List(list) => list.iter().map(|l| (l.clone(), l.clone())).collect(),
            Languages::Map(map) => map.clone(),
        };
        Self {
            name: name.to_string(),
            config,
            languages,
        }
    }

    /// Reads the settings from a `[engine.process.<name>]` table.
    pub fn from_table(name: &str, table: &toml::Table) -> Result<Self, EngineError> {
        let config: ProcessConfig = table.clone().try_into()?;
        Ok(Self::new(name, config))
    }

    fn run(&self, input: String) -> Result<Vec<u8>, EngineError> {
        let mut child = Command::new(&self.config.command)
            .args(&self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("cannot run '{}': {e}", self.config.command))?;

        let mut stdin = child.stdin.take().unwrap();
        let writer = thread::spawn(move || stdin.write_all(input.as_bytes()));
        let mut stdout = child.stdout.take().unwrap();
        let reader = thread::spawn(move || {
            let mut bytes = Vec::new();
            stdout.read_to_end(&mut bytes).map(|_| bytes)
        });
        let mut stderr = child.stderr.take().unwrap();
        let errors = thread::spawn(move || {
            let mut text = String::new();
            let _ = stderr.read_to_string(&mut text);
            text
        });

        let deadline = Instant::now() + Duration::from_secs(self.config.timeout);
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "'{}' did not finish within {} s",
                    self.config.command, self.config.timeout
                )
                .into());
            }
            thread::sleep(Duration::from_millis(10));
        };

        let _ = writer.join();
        let output = reader.join().map_err(|_| "reading the output failed")??;
        let errors = errors.join().unwrap_or_default();
        if !status.success() {
            return Err(format!(
                "'{}' failed ({status}): {}",
                self.config.command,
                errors.trim()
            )
            .into());
        }
        Ok(output)
    }
}

impl CodeEngine for Process {
    fn name(&self) -> &str {
        &self.name
    }

    fn supports(&self, language: &str) -> bool {
        self.languages.contains_key(language)
    }

    fn render(&self, language: &str, code: &str) -> Result<EngineOutput, EngineError> {
        let lang = self
            .languages
            .get(language)
            .map_or(language, String::as_str);
        let input = match &self.config.input {
            Some(template) => template.replace("{lang}", lang).replace("{code}", code),
            None => code.to_string(),
        };
        let output = String::from_utf8(self.run(input)?)?;
        let Some(start) = output.find("<svg") else {
            return Err(format!("'{}' produced no SVG", self.config.command).into());
        };
        Ok(EngineOutput::Graphic(Graphic {
            language: language.to_string(),
            svg: output[start..].trim_end().to_string(),
            alt: format!("{language} diagram"),
            code: code.to_string(),
        }))
    }
}

// The tests run `cat` and `sleep`.
#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn engine(command: &str, args: &[&str], input: Option<&str>) -> Process {
        Process::new(
            "test",
            ProcessConfig {
                command: command.to_string(),
                args: args.iter().map(|a| a.to_string()).collect(),
                languages: Languages::Map(BTreeMap::from([(
                    "svg".to_string(),
                    "picture".to_string(),
                )])),
                input: input.map(str::to_string),
                timeout: 5,
            },
        )
    }

    #[test]
    fn passes_the_code_through_the_command() {
        let engine = engine("cat", &[], Some("<!-- {lang} -->\n<svg>{code}</svg>"));
        assert!(engine.supports("svg"));
        let EngineOutput::Graphic(graphic) = engine.render("svg", "x").unwrap() else {
            panic!("expected a graphic")
        };
        assert_eq!(graphic.svg, "<svg>x</svg>");
    }

    #[test]
    fn reports_missing_commands_and_missing_svg() {
        assert!(
            engine("no-such-command-nectar", &[], None)
                .render("svg", "x")
                .is_err()
        );
        assert!(
            engine("cat", &[], None)
                .render("svg", "plain text")
                .is_err()
        );
    }

    #[test]
    fn stops_commands_that_do_not_finish() {
        let mut engine = engine("sleep", &["10"], None);
        engine.config.timeout = 1;
        let error = engine.render("svg", "").unwrap_err().to_string();
        assert!(error.contains("did not finish"), "{error}");
    }
}
