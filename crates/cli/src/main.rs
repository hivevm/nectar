//! Nectar command line. This is the one place that composes the core with the renderers, the code
//! engines and the adapters, so every command uses the same set.

use std::io;
use std::net::{IpAddr, SocketAddr};
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand};
use nectar_core::{Build, CodeEngine, Config, Diagnostic, Project, Severity};
use nectar_fetch::Http;
use nectar_html::{HtmlRenderer, PrintRenderer};
use nectar_native::{Directory, Threads};
use nectar_pdf::PdfRenderer;
use nectar_process::Process;
use nectar_railroad::Railroad;
use nectar_server::{OpenProject, ServeOptions};
use nectar_textmate::{DEFAULT_DARK_THEME, DEFAULT_THEME, TextMate};

const DEFAULT_PORT: u16 = 1313;

#[derive(Parser)]
#[command(name = "nectar", version, about = "Turns Markdown sources into HTML")]
struct Cli {
    /// Project root, the directory that holds nectar.toml
    #[arg(long, short = 'C', default_value = ".", global = true)]
    root: PathBuf,

    /// Keep images from the web as links instead of downloading them
    #[arg(long, global = true)]
    offline: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build the site into the output directory
    Build {
        /// Output directory; overrides `output` in nectar.toml
        #[arg(long, short)]
        output: Option<PathBuf>,

        /// Include files marked as draft
        #[arg(long)]
        drafts: bool,
    },

    /// Serve the site over local HTTP, rebuild on changes and reload the browser
    Serve {
        /// Port; overrides `port` under [serve] in nectar.toml (default 1313)
        #[arg(long, short)]
        port: Option<u16>,

        /// Address to listen on; overrides `bind` under [serve] (default 127.0.0.1)
        #[arg(long)]
        bind: Option<IpAddr>,

        /// Leave out files marked as draft
        #[arg(long)]
        no_drafts: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Build { output, drafts } => build(&cli.root, output, drafts, cli.offline),
        Command::Serve {
            port,
            bind,
            no_drafts,
        } => serve(&cli.root, port, bind, !no_drafts, cli.offline),
    };
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn build(root: &Path, output: Option<PathBuf>, drafts: bool, offline: bool) -> Result<ExitCode> {
    let started = Instant::now();
    let project = open(root, drafts, offline)?;
    let web = HtmlRenderer {
        print_view: true,
        pdf: true,
    };
    let build = project.build(&[&web, &PrintRenderer, &PdfRenderer]);
    report(&build.diagnostics);
    if build.has_errors() {
        eprintln!("build failed");
        return Ok(ExitCode::FAILURE);
    }

    let dir = output.unwrap_or_else(|| root.join(&project.config().output));
    write(&build, &dir).with_context(|| format!("cannot write to {}", dir.display()))?;
    eprintln!(
        "{} files written to {} in {} ms",
        build.files.len(),
        dir.display(),
        started.elapsed().as_millis()
    );
    Ok(ExitCode::SUCCESS)
}

fn serve(
    root: &Path,
    port: Option<u16>,
    bind: Option<IpAddr>,
    drafts: bool,
    offline: bool,
) -> Result<ExitCode> {
    // The configuration may be broken at start; the server then shows why and keeps watching.
    let config = Config::load(&Directory::new(root)).ok();
    let setting = |name: &str| config.as_ref().and_then(|c| c.serve.get(name).cloned());
    let port = match port {
        Some(port) => port,
        None => match setting("port").and_then(|v| v.as_integer()) {
            Some(port) => u16::try_from(port).context("[serve] port is out of range")?,
            None => DEFAULT_PORT,
        },
    };
    let bind = match bind {
        Some(bind) => bind,
        None => match setting("bind") {
            Some(value) => value
                .as_str()
                .and_then(|text| text.parse().ok())
                .context("[serve] bind must be an IP address")?,
            None => IpAddr::from([127, 0, 0, 1]),
        },
    };
    let ignore = config.map(|c| vec![c.output]).unwrap_or_default();

    let project_root = root.to_path_buf();
    let open: OpenProject = Box::new(move || {
        open(&project_root, drafts, offline).map_err(|e| Diagnostic::error(format!("{e:#}"), None))
    });
    let options = ServeOptions {
        root: root.to_path_buf(),
        address: SocketAddr::new(bind, port),
        ignore,
    };
    let runtime = tokio::runtime::Runtime::new()?;
    let renderers: nectar_server::Renderers = Arc::new(vec![
        Box::new(HtmlRenderer {
            print_view: true,
            pdf: true,
        }),
        Box::new(PrintRenderer),
        Box::new(PdfRenderer),
    ]);
    runtime.block_on(nectar_server::serve(open, renderers, options))?;
    Ok(ExitCode::SUCCESS)
}

fn open(root: &Path, drafts: bool, offline: bool) -> Result<Project> {
    let mut project = Project::load(Box::new(Directory::new(root))).map_err(|d| anyhow!("{d}"))?;
    project.set_workers(Box::new(Threads));
    project.set_include_drafts(drafts);
    project.set_offline(offline);
    project.set_fetcher(Box::new(Http::new()));
    for engine in engines(project.config())? {
        project.add_engine(engine);
    }
    Ok(project)
}

/// Writes every output file of `build` below `dir`. Files from earlier builds are left in place.
fn write(build: &Build, dir: &Path) -> io::Result<()> {
    for (path, bytes) in &build.files {
        if !path.components().all(|c| matches!(c, Component::Normal(_))) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("output path {} leaves the output directory", path.display()),
            ));
        }
        let target = dir.join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(target, bytes)?;
    }
    Ok(())
}

/// The code engines configured under `[engine]`: every `[engine.process.<name>]` first, then
/// the built-in railroad engine and TextMate highlighting, each on unless `enabled = false`.
fn engines(config: &Config) -> Result<Vec<Box<dyn CodeEngine>>> {
    let mut engines: Vec<Box<dyn CodeEngine>> = Vec::new();
    for key in config.engine.keys() {
        if !["process", "railroad", "textmate"].contains(&key.as_str()) {
            bail!("unknown engine [engine.{key}]");
        }
    }
    let enabled = |name: &str| {
        config
            .engine
            .get(name)
            .and_then(|table| table.get("enabled"))
            .and_then(toml::Value::as_bool)
            .unwrap_or(true)
    };

    if let Some(processes) = config.engine.get("process") {
        let processes = processes
            .as_table()
            .context("[engine.process] must be a table")?;
        for (name, settings) in processes {
            let settings = settings
                .as_table()
                .with_context(|| format!("[engine.process.{name}] must be a table"))?;
            let engine = Process::from_table(name, settings)
                .map_err(|e| anyhow!("[engine.process.{name}]: {e}"))?;
            engines.push(Box::new(engine));
        }
    }

    if enabled("railroad") {
        engines.push(Box::new(Railroad));
    }

    if enabled("textmate") {
        let setting = |name: &str| {
            config
                .engine
                .get("textmate")
                .and_then(|table| table.get(name))
                .and_then(toml::Value::as_str)
        };
        let theme = setting("theme").unwrap_or(DEFAULT_THEME);
        // An empty dark theme turns the colours for dark colour schemes off.
        let dark_theme =
            Some(setting("dark_theme").unwrap_or(DEFAULT_DARK_THEME)).filter(|t| !t.is_empty());
        let engine =
            TextMate::new(theme, dark_theme).map_err(|e| anyhow!("[engine.textmate]: {e}"))?;
        engines.push(Box::new(engine));
    }
    Ok(engines)
}

fn report(diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        eprintln!("{diagnostic}");
    }
    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warnings = diagnostics.len() - errors;
    if !diagnostics.is_empty() {
        eprintln!("{errors} errors, {warnings} warnings");
    }
}
