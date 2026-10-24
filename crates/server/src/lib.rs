//! Nectar development server: builds the project in memory, serves the build over HTTP, watches
//! the project directory and tells connected browsers to reload after every rebuild.
//!
//! The server knows neither engines nor renderer details: the shell hands it a function that
//! opens the project and the renderers to use.

use std::collections::BTreeSet;
use std::convert::Infallible;
use std::fmt::Write as _;
use std::net::SocketAddr;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use axum::Router;
use axum::extract::State;
use axum::http::{StatusCode, Uri, header};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use nectar_core::{Build, CONFIG_FILE, Diagnostic, Project, Renderer, Severity};
use notify::event::ModifyKind;
use notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, new_debouncer};
use percent_encoding::percent_decode_str;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

/// Path of the server-sent events stream that announces new builds.
const RELOAD_PATH: &str = "/_nectar/reload";

/// The renderers of every build, shared with the build threads.
pub type Renderers = Arc<Vec<Box<dyn Renderer + Send + Sync>>>;

/// Opens the project; called at start and whenever `nectar.toml` changes.
pub type OpenProject = Box<dyn Fn() -> Result<Project, Diagnostic> + Send + Sync>;

pub struct ServeOptions {
    pub root: PathBuf,
    pub address: SocketAddr,
    /// Directories below the root whose changes never trigger a rebuild, such as the output of
    /// `nectar build`.
    pub ignore: Vec<PathBuf>,
}

/// One build as served, with the version announced to browsers.
struct Snapshot {
    build: Build,
    version: String,
}

struct Shared {
    snapshot: RwLock<Arc<Snapshot>>,
    reload: broadcast::Sender<String>,
}

/// Builds, serves and watches until Ctrl+C.
pub async fn serve(open: OpenProject, renderers: Renderers, options: ServeOptions) -> Result<()> {
    let root = options
        .root
        .canonicalize()
        .with_context(|| format!("cannot open {}", options.root.display()))?;
    let open = Arc::new(open);
    // Versions restart with every server start; the epoch keeps them distinct for open pages.
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());

    let started = Instant::now();
    let (project, build) = {
        let open = open.clone();
        let renderers = renderers.clone();
        tokio::task::spawn_blocking(move || build_project(None, &open, &renderers, true)).await?
    };
    report(&build, started, &[]);

    let (reload, _) = broadcast::channel(16);
    let snapshot = Snapshot {
        build,
        version: format!("{epoch}-0"),
    };
    let shared = Arc::new(Shared {
        snapshot: RwLock::new(Arc::new(snapshot)),
        reload,
    });

    let (changes_tx, changes) = mpsc::unbounded_channel::<Vec<PathBuf>>();
    let mut debouncer = new_debouncer(
        Duration::from_millis(150),
        None,
        move |result: DebounceEventResult| {
            if let Ok(events) = result {
                let paths: Vec<PathBuf> = events
                    .into_iter()
                    .filter(|e| changes_content(&e.event.kind))
                    .flat_map(|e| e.event.paths)
                    .collect();
                if !paths.is_empty() {
                    let _ = changes_tx.send(paths);
                }
            }
        },
    )?;
    debouncer.watch(&root, RecursiveMode::Recursive)?;
    let watch = Watch {
        root,
        ignore: options.ignore,
        epoch,
    };
    tokio::spawn(rebuild_loop(
        project,
        open,
        renderers,
        watch,
        changes,
        shared.clone(),
    ));

    let app = Router::new()
        .route(RELOAD_PATH, get(reload_events))
        .fallback(get(serve_file))
        .with_state(shared);
    let listener = tokio::net::TcpListener::bind(options.address)
        .await
        .with_context(|| format!("cannot listen on {}", options.address))?;
    eprintln!(
        "serving on http://{}/ (Ctrl+C to stop)",
        listener.local_addr()?
    );

    // Open event streams never end, so the server stops without waiting for connections.
    tokio::select! {
        result = async move { axum::serve(listener, app).await } => result?,
        _ = tokio::signal::ctrl_c() => eprintln!("stopped"),
    }
    drop(debouncer);
    Ok(())
}

struct Watch {
    root: PathBuf,
    ignore: Vec<PathBuf>,
    epoch: u128,
}

/// Rebuilds after every batch of relevant changes and announces the new build. A change to
/// `nectar.toml` opens the project again, with its engines.
async fn rebuild_loop(
    mut project: Option<Project>,
    open: Arc<OpenProject>,
    renderers: Renderers,
    watch: Watch,
    mut changes: mpsc::UnboundedReceiver<Vec<PathBuf>>,
    shared: Arc<Shared>,
) {
    let mut version = 0u64;
    while let Some(mut paths) = changes.recv().await {
        while let Ok(more) = changes.try_recv() {
            paths.extend(more);
        }
        let changed: BTreeSet<PathBuf> = paths
            .iter()
            .filter_map(|path| path.strip_prefix(&watch.root).ok())
            .filter(|path| is_relevant(path, &watch.ignore))
            .map(Path::to_path_buf)
            .collect();
        if changed.is_empty() {
            continue;
        }

        let reopen = project.is_none() || changed.contains(Path::new(CONFIG_FILE));
        let started = Instant::now();
        let previous = project.take();
        let open = open.clone();
        let renderers_for_build = renderers.clone();
        let result = tokio::task::spawn_blocking(move || {
            build_project(previous, &open, &renderers_for_build, reopen)
        })
        .await;
        let Ok((next, build)) = result else {
            eprintln!(
                "error: the build stopped unexpectedly; the project is opened again on the next change"
            );
            continue;
        };
        project = next;
        let changed: Vec<PathBuf> = changed.into_iter().collect();
        report(&build, started, &changed);

        version += 1;
        let version = format!("{}-{version}", watch.epoch);
        *shared.snapshot.write().unwrap() = Arc::new(Snapshot {
            build,
            version: version.clone(),
        });
        let _ = shared.reload.send(version);
    }
}

/// Announces the current build on connect and every new build afterwards.
async fn reload_events(
    State(shared): State<Arc<Shared>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let current = shared.snapshot.read().unwrap().version.clone();
    let updates = BroadcastStream::new(shared.reload.subscribe()).filter_map(Result::ok);
    let stream = tokio_stream::once(current)
        .chain(updates)
        .map(|version| Ok(Event::default().data(version)));
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn serve_file(State(shared): State<Arc<Shared>>, uri: Uri) -> Response {
    let snapshot = shared.snapshot.read().unwrap().clone();
    let path = uri.path();
    let Some(file) = file_for(path) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    if let Some(bytes) = snapshot.build.files.get(&file) {
        let content_type = content_type(&file);
        let body = if content_type.starts_with("text/html") {
            inject(bytes, &snapshot.build.diagnostics)
        } else {
            bytes.clone()
        };
        let headers = [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, "no-store"),
        ];
        return (headers, body).into_response();
    }
    // Pages link relative to their directory, so a directory needs its trailing slash.
    if !path.ends_with('/') && snapshot.build.files.contains_key(&file.join("index.html")) {
        return Redirect::to(&format!("/{}/", path.trim_start_matches('/'))).into_response();
    }
    missing_page(path, &snapshot.build.diagnostics)
}

/// Opens the project when needed and builds it. Returns the project for the next rebuild.
fn build_project(
    project: Option<Project>,
    open: &OpenProject,
    renderers: &[Box<dyn Renderer + Send + Sync>],
    reopen: bool,
) -> (Option<Project>, Build) {
    let project = match project {
        Some(project) if !reopen => project,
        _ => match open() {
            Ok(project) => project,
            Err(diagnostic) => {
                let build = Build {
                    diagnostics: vec![diagnostic],
                    ..Build::default()
                };
                return (None, build);
            }
        },
    };
    let renderers: Vec<&dyn Renderer> = renderers
        .iter()
        .map(|r| r.as_ref() as &dyn Renderer)
        .collect();
    let build = project.build(&renderers);
    (Some(project), build)
}

fn report(build: &Build, started: Instant, changed: &[PathBuf]) {
    for diagnostic in &build.diagnostics {
        eprintln!("{diagnostic}");
    }
    let errors = build
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warnings = build.diagnostics.len() - errors;
    let cause = match changed {
        [] => String::new(),
        [one] => format!(" after a change to {}", one.display()),
        many => format!(" after changes to {} files", many.len()),
    };
    eprintln!(
        "built {} files in {} ms{cause} ({errors} errors, {warnings} warnings)",
        build.files.len(),
        started.elapsed().as_millis()
    );
}

/// Whether an event changes what a file holds. Reading files, as every build does, raises access
/// and metadata events; reacting to them would rebuild forever.
fn changes_content(kind: &EventKind) -> bool {
    match kind {
        EventKind::Create(_) | EventKind::Remove(_) => true,
        EventKind::Modify(modify) => !matches!(modify, ModifyKind::Metadata(_)),
        EventKind::Access(_) | EventKind::Any | EventKind::Other => false,
    }
}

/// Whether a change below the root may affect the build: hidden files, editor backups and the
/// ignored directories never do.
fn is_relevant(path: &Path, ignore: &[PathBuf]) -> bool {
    if ignore.iter().any(|dir| path.starts_with(dir)) {
        return false;
    }
    let hidden = path.components().any(|c| match c {
        Component::Normal(part) => part.to_string_lossy().starts_with('.'),
        _ => false,
    });
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();
    !hidden && !name.ends_with('~') && !name.ends_with(".swp") && !name.ends_with(".tmp")
}

/// Maps a request path to an output file. Directories map to their `index.html`.
fn file_for(path: &str) -> Option<PathBuf> {
    let decoded = percent_decode_str(path).decode_utf8().ok()?;
    let mut file = PathBuf::new();
    for part in decoded.split('/') {
        match part {
            "" | "." => {}
            ".." => return None,
            part => file.push(part),
        }
    }
    if decoded.is_empty() || decoded.ends_with('/') {
        file.push("index.html");
    }
    Some(file)
}

fn content_type(path: &Path) -> &'static str {
    let extension = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    match extension.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "pdf" => "application/pdf",
        "txt" | "md" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// The script that reloads the page when the server announces a different build.
///
/// The diagnostics panel goes in through the script, after the document has loaded and outside
/// `<body>`: scripts such as a print preview may rearrange the body, and the panel must stay out
/// of their way.
fn reload_script(panel: &str) -> String {
    format!(
        "<script>(() => {{ let seen; const events = new EventSource(\"{RELOAD_PATH}\"); \
         events.onmessage = (e) => {{ if (seen !== undefined && e.data !== seen) location.reload(); seen = e.data; }}; \
         const panel = {}; if (panel) addEventListener(\"load\", () => document.documentElement.insertAdjacentHTML(\"beforeend\", panel)); }})();</script>",
        js_string(panel)
    )
}

/// `text` as a JavaScript string literal that is safe inside a `<script>` element.
fn js_string(text: &str) -> String {
    let mut literal = String::with_capacity(text.len() + 2);
    literal.push('"');
    for c in text.chars() {
        match c {
            '"' => literal.push_str("\\\""),
            '\\' => literal.push_str("\\\\"),
            '\n' => literal.push_str("\\n"),
            '\r' => literal.push_str("\\r"),
            '<' => literal.push_str("\\u003c"),
            c => literal.push(c),
        }
    }
    literal.push('"');
    literal
}

/// A panel with the build's errors, and its warnings folded away.
fn diagnostics_panel(diagnostics: &[Diagnostic]) -> String {
    if diagnostics.is_empty() {
        return String::new();
    }
    let errors: Vec<&Diagnostic> = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    let warnings: Vec<&Diagnostic> = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .collect();
    let mut panel = String::from(
        "<div id=\"nectar-diagnostics\" style=\"position:fixed;right:1rem;bottom:1rem;z-index:2147483647;\
         max-width:min(48rem,calc(100vw - 2rem));max-height:60vh;overflow:auto;padding:0.75rem 1rem;\
         background:#1f1a17;color:#f3ead2;font:13px/1.5 ui-monospace,monospace;border-radius:6px;\
         box-shadow:0 4px 24px rgba(0,0,0,.35)\">",
    );
    let list = |panel: &mut String, items: &[&Diagnostic]| {
        panel.push_str("<ul style=\"margin:0.4rem 0 0;padding-left:1.2rem\">");
        for item in items {
            let _ = write!(panel, "<li>{}</li>", escape(&item.to_string()));
        }
        panel.push_str("</ul>");
    };
    if !errors.is_empty() {
        let _ = write!(
            panel,
            "<strong style=\"color:#f78082\">Build failed: {} errors</strong>",
            errors.len()
        );
        list(&mut panel, &errors);
    }
    if !warnings.is_empty() {
        let open = if errors.is_empty() { "" } else { " open" };
        let _ = write!(
            panel,
            "<details{open}><summary>{} warnings</summary>",
            warnings.len()
        );
        list(&mut panel, &warnings);
        panel.push_str("</details>");
    }
    panel.push_str("</div>");
    panel
}

/// Adds the reload script and the diagnostics panel before `</body>`.
fn inject(html: &[u8], diagnostics: &[Diagnostic]) -> Vec<u8> {
    let html = String::from_utf8_lossy(html);
    let addition = reload_script(&diagnostics_panel(diagnostics));
    let mut result = String::with_capacity(html.len() + addition.len());
    match html.find("</head>").or_else(|| html.rfind("</body>")) {
        Some(index) => {
            result.push_str(&html[..index]);
            result.push_str(&addition);
            result.push_str(&html[index..]);
        }
        None => {
            result.push_str(&html);
            result.push_str(&addition);
        }
    }
    result.into_bytes()
}

/// The page for requests without a file: the build's diagnostics when it failed, or not found.
fn missing_page(path: &str, diagnostics: &[Diagnostic]) -> Response {
    let failed = diagnostics.iter().any(|d| d.severity == Severity::Error);
    let (status, title) = if failed {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Build failed".to_string(),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            format!("Not found: {}", escape(path)),
        )
    };
    let html = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{title}</title></head>\
         <body style=\"font-family:system-ui,sans-serif;margin:2rem\"><h1>{title}</h1></body></html>"
    );
    let body = inject(html.as_bytes(), diagnostics);
    (
        status,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        body,
    )
        .into_response()
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_paths_map_to_output_files() {
        assert_eq!(file_for("/"), Some(PathBuf::from("index.html")));
        assert_eq!(
            file_for("/manual/"),
            Some(PathBuf::from("manual/index.html"))
        );
        assert_eq!(
            file_for("/_theme/style.css"),
            Some(PathBuf::from("_theme/style.css"))
        );
        assert_eq!(
            file_for("/my%20file.png"),
            Some(PathBuf::from("my file.png"))
        );
        assert_eq!(file_for("/../secret"), None);
    }

    #[test]
    fn reading_files_is_no_change() {
        use notify::event::{AccessKind, CreateKind, DataChange, MetadataKind};
        assert!(!changes_content(&EventKind::Access(AccessKind::Any)));
        assert!(!changes_content(&EventKind::Modify(ModifyKind::Metadata(
            MetadataKind::AccessTime
        ))));
        assert!(changes_content(&EventKind::Modify(ModifyKind::Data(
            DataChange::Content
        ))));
        assert!(changes_content(&EventKind::Create(CreateKind::File)));
    }

    #[test]
    fn ignores_hidden_backup_and_output_files() {
        let ignore = [PathBuf::from("public")];
        assert!(is_relevant(Path::new("content/a.md"), &ignore));
        assert!(!is_relevant(Path::new("public/index.html"), &ignore));
        assert!(!is_relevant(Path::new(".git/index"), &ignore));
        assert!(!is_relevant(Path::new("content/.a.md.swp"), &ignore));
        assert!(!is_relevant(Path::new("content/a.md~"), &ignore));
    }

    #[test]
    fn injects_into_the_head() {
        let html = inject(
            b"<html><head><title>t</title></head><body><p>x</p></body></html>",
            &[],
        );
        let html = String::from_utf8(html).unwrap();
        assert!(html.contains("EventSource"));
        assert!(html.contains("</script></head><body><p>x</p></body></html>"));
        assert!(html.contains("const panel = \"\";"));

        let errors = [Diagnostic::error("broken <tag> \"quoted\"", None)];
        let html = String::from_utf8(inject(b"<p>x</p>", &errors)).unwrap();
        assert!(html.contains("Build failed: 1 errors"));
        assert!(html.contains("broken &lt;tag&gt;"));
        assert!(
            !html.contains("</div>"),
            "markup is escaped inside the script: {html}"
        );
        assert!(html.starts_with("<p>x</p><script>"));
    }
}
