//! Nectar command line. This is the one place that composes the core with the renderers, the code
//! engines and the adapters, so every command uses the same set.

mod render;

use std::io::{self, Read, Write};
use std::net::{IpAddr, SocketAddr};
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand};
use nectar_core::{Build, CodeEngine, Config, Diagnostic, Project, Renderer, Severity};
use nectar_fetch::Http;
use nectar_html::{HtmlRenderer, PrintRenderer};
use nectar_native::{Directory, Threads};
use nectar_pdf::PdfRenderer;
use nectar_process::Process;
use nectar_railroad::Railroad;
use nectar_rough::{Rough, Style as RoughStyle};
use nectar_server::{OpenProject, Renderers, ServeOptions};
use nectar_textmate::{DEFAULT_DARK_THEME, DEFAULT_THEME, TextMate};

const DEFAULT_PORT: u16 = 1313;

#[derive(Parser)]
#[command(name = "nectar", version, about = "Turns Markdown sources into HTML")]
struct Cli {
    /// Project root, the directory that holds nectar.toml [default: .]; for render, the directory
    /// whose nectar.toml configures the engines, without a default
    #[arg(long, short = 'C', global = true)]
    root: Option<PathBuf>,

    /// Keep images from the web as links instead of downloading them
    #[arg(long, global = true)]
    offline: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build the site into the output directory, or the PDF of one Markdown file
    Build {
        /// A Markdown file to build alone, without nectar.toml: its PDF is written next to it
        file: Option<PathBuf>,

        /// Output directory; overrides `output` in nectar.toml. With a file, the PDF file to write
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

    /// Run the code engine of one language on one text and write its output
    Render {
        /// The language, as in a code block's info string, such as railroad or rust
        language: String,

        /// The file to read; standard input when it is missing or -
        file: Option<PathBuf>,

        /// The file to write; standard output when it is missing or -
        #[arg(long, short)]
        output: Option<PathBuf>,

        /// svg or png for a graphic, html or ansi for highlighted code; without it, the extension
        /// of --output, or svg and html
        #[arg(long, short, value_enum)]
        format: Option<render::Format>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let root = cli.root.clone().unwrap_or_else(|| PathBuf::from("."));
    let result = match cli.command {
        Command::Build {
            file: Some(file),
            output,
            drafts,
        } => match cli.root {
            Some(_) => Err(anyhow!("give either -C <dir> or a Markdown file, not both")),
            None => build_file(&file, output, drafts, cli.offline),
        },
        Command::Build {
            file: None,
            output,
            drafts,
        } => build(&root, output, drafts, cli.offline),
        Command::Serve {
            port,
            bind,
            no_drafts,
        } => serve(&root, port, bind, !no_drafts, cli.offline),
        Command::Render {
            language,
            file,
            output,
            format,
        } => render_text(cli.root.as_deref(), &language, file, output, format),
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
    let renderers = renderers();
    let renderers: Vec<&dyn Renderer> = renderers.iter().map(|r| r.as_ref() as _).collect();
    let build = project.build(&renderers);
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

/// Builds one Markdown file without a `nectar.toml`: its directory is the project root, and only
/// its PDF is written, next to it or to `output`.
fn build_file(
    file: &Path,
    output: Option<PathBuf>,
    drafts: bool,
    offline: bool,
) -> Result<ExitCode> {
    let started = Instant::now();
    if !file.is_file() {
        bail!("{} is not a file", file.display());
    }
    let name = file.file_name().context("the file has no name")?;
    let root = match file.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };

    // Settings come from the file's front matter alone: no nectar.toml, no NECTAR_ variables.
    let config = Config::single(PathBuf::from(name));
    let project = prepare(
        Project::new(Box::new(Directory::new(root)), config),
        drafts,
        offline,
    )?;

    let build = project.build(&[&PdfRenderer]);
    report(&build.diagnostics);
    if build.has_errors() {
        eprintln!("build failed");
        return Ok(ExitCode::FAILURE);
    }
    let pdf = build
        .files
        .iter()
        .find(|(path, _)| path.extension().is_some_and(|e| e == "pdf"))
        .map(|(_, bytes)| bytes)
        .with_context(|| {
            format!(
                "{} gave no PDF; a file whose front matter says draft = true needs --drafts",
                file.display()
            )
        })?;

    let pdf_name = Path::new(name).with_extension("pdf");
    let target = match output {
        // An existing directory takes the PDF under its usual name, as -o takes a directory for a
        // project.
        Some(dir) if dir.is_dir() => dir.join(&pdf_name),
        Some(path) => path,
        None => root.join(&pdf_name),
    };
    if std::fs::canonicalize(&target).ok() == std::fs::canonicalize(file).ok() {
        bail!("{} would overwrite the source", target.display());
    }
    if let Some(parent) = target.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("cannot create {}", parent.display()))?;
    }
    std::fs::write(&target, pdf).with_context(|| format!("cannot write {}", target.display()))?;
    eprintln!(
        "{} written in {} ms",
        target.display(),
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
    runtime.block_on(nectar_server::serve(open, renderers(), options))?;
    Ok(ExitCode::SUCCESS)
}

/// Runs `nectar render`. Without `-C` the built-in engines run with their defaults; with it,
/// `[engine]` of that directory's `nectar.toml` configures them, process engines included.
fn render_text(
    root: Option<&Path>,
    language: &str,
    file: Option<PathBuf>,
    output: Option<PathBuf>,
    format: Option<render::Format>,
) -> Result<ExitCode> {
    let table = match root {
        Some(root) => {
            Config::load(&Directory::new(root))
                .map_err(|d| anyhow!("{d}"))?
                .engine
        }
        None => toml::Table::new(),
    };
    let engines = engines(&table)?;
    let engine = engines
        .iter()
        .find(|engine| engine.supports(language))
        .ok_or_else(|| match root {
            None => anyhow!(
                "no engine supports '{language}'; without -C <dir>, only railroad, rough and \
                 TextMate grammars are available"
            ),
            Some(_) => anyhow!("no engine supports '{language}'"),
        })?;

    let mut code = String::new();
    match file.as_deref() {
        Some(path) if path != Path::new("-") => {
            code = std::fs::read_to_string(path)
                .with_context(|| format!("cannot read {}", path.display()))?;
        }
        _ => {
            io::stdin()
                .read_to_string(&mut code)
                .context("cannot read standard input")?;
        }
    }

    // `-o -` is standard output, as `-` for the file is standard input.
    let output = output.filter(|path| path != Path::new("-"));
    let bytes = render::render(engine.as_ref(), language, &code, format, output.as_deref())?;

    match output {
        Some(path) => {
            if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("cannot create {}", parent.display()))?;
            }
            std::fs::write(&path, bytes)
                .with_context(|| format!("cannot write {}", path.display()))?;
        }
        None => {
            let mut stdout = io::stdout().lock();
            stdout
                .write_all(&bytes)
                .and_then(|()| stdout.flush())
                .context("cannot write standard output")?;
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// The renderers of every build: the website with its print view and PDF links, the print view
/// and the PDF. `build` and `serve` share them, so both write the same site.
fn renderers() -> Renderers {
    Arc::new(vec![
        Box::new(HtmlRenderer {
            print_view: true,
            pdf: true,
        }),
        Box::new(PrintRenderer),
        Box::new(PdfRenderer),
    ])
}

fn open(root: &Path, drafts: bool, offline: bool) -> Result<Project> {
    let mut project = Project::load(Box::new(Directory::new(root))).map_err(|d| anyhow!("{d}"))?;
    project.set_environment(std::env::vars());
    prepare(project, drafts, offline)
}

/// Plugs the workers, the fetcher and the configured code engines into `project`.
fn prepare(mut project: Project, drafts: bool, offline: bool) -> Result<Project> {
    project.set_workers(Box::new(Threads));
    project.set_include_drafts(drafts);
    project.set_offline(offline);
    project.set_fetcher(Box::new(Http::new()));
    for engine in engines(&project.config().engine)? {
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

/// The defaults of the rough engine from `[engine.rough]`, every key but `enabled` an option of
/// its `data-rough-*` attributes in snake case (ADR-0015).
fn rough_defaults(config: &toml::Table) -> Result<RoughStyle> {
    let mut defaults = RoughStyle::default();
    let Some(table) = config.get("rough") else {
        return Ok(defaults);
    };
    let table = table.as_table().context("[engine.rough] must be a table")?;
    for (key, value) in table {
        if key == "enabled" {
            continue;
        }
        let text = match value {
            toml::Value::String(text) => text.clone(),
            toml::Value::Integer(_) | toml::Value::Float(_) | toml::Value::Boolean(_) => {
                value.to_string()
            }
            _ => bail!("[engine.rough]: {key} must be a string, a number or a boolean"),
        };
        defaults
            .set_default(key, &text, &value.to_string())
            .map_err(|e| anyhow!("[engine.rough]: {e}"))?;
    }
    Ok(defaults)
}

/// The code engines configured under `[engine]`: every `[engine.process.<name>]` first, then
/// the built-in railroad and rough engines and TextMate highlighting, each on unless
/// `enabled = false`.
fn engines(config: &toml::Table) -> Result<Vec<Box<dyn CodeEngine>>> {
    let mut engines: Vec<Box<dyn CodeEngine>> = Vec::new();
    for key in config.keys() {
        if !["process", "railroad", "rough", "textmate"].contains(&key.as_str()) {
            bail!("unknown engine [engine.{key}]");
        }
    }
    let enabled = |name: &str| {
        config
            .get(name)
            .and_then(|table| table.get("enabled"))
            .and_then(toml::Value::as_bool)
            .unwrap_or(true)
    };

    if let Some(processes) = config.get("process") {
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

    // Checked even when the engine is off, so a mistyped key is not kept silently.
    let rough = rough_defaults(config)?;
    if enabled("rough") {
        engines.push(Box::new(
            Rough::new(render::bundled_fonts()).with_defaults(rough),
        ));
    }

    if enabled("textmate") {
        let setting = |name: &str| {
            config
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
