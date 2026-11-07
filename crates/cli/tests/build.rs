//! Runs the `nectar` command line on projects in temporary directories, and compares it with a
//! build from files in memory, as a WebAssembly host gives them.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use nectar_core::{MemoryFiles, Project};
use nectar_html::{HtmlRenderer, PrintRenderer};
use nectar_pdf::PdfRenderer;
use nectar_railroad::Railroad;
use nectar_textmate::{DEFAULT_DARK_THEME, DEFAULT_THEME, TextMate};

fn write(root: &Path, file: &str, text: &str) {
    let path = root.join(file);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// An `[engine.process.<name>]` table whose program runs everywhere the tests do: `nectar` itself,
/// which reads nothing from standard input and writes no SVG, so every block it gets fails.
fn failing_engine(name: &str, language: &str) -> String {
    format!(
        "[engine.process.{name}]\ncommand = '{}'\nargs = [\"--version\"]\nlanguages = [\"{language}\"]\n",
        env!("CARGO_BIN_EXE_nectar")
    )
}

fn nectar(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_nectar"))
        .current_dir(root)
        .arg("-C")
        .arg(root)
        .arg("--offline")
        .args(args)
        .output()
        .unwrap()
}

// Verifies: G-1
#[test]
fn build_writes_every_document_and_fails_exactly_on_errors() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(
        root,
        "nectar.toml",
        "[[document]]\nsource = \"guide.md\"\n\n[[document]]\nsource = \"manual.md\"\n",
    );
    write(root, "guide.md", "# Guide\n");
    write(root, "manual.md", "# Manual\n\n[Gone](gone.md)\n");

    let output = nectar(root, &["build"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "a warning is no error: {stderr}");
    assert!(stderr.contains("gone.md does not exist"), "{stderr}");
    for page in ["public/guide/index.html", "public/manual/index.html"] {
        assert!(root.join(page).is_file(), "{page} is missing");
    }

    write(root, "manual.md", "# Manual\n\n[Itself](manual.md)\n");
    let output = nectar(root, &["build", "--output", "again"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "an include cycle is an error");
    assert!(stderr.contains("include cycle"), "{stderr}");
    assert!(
        !root.join("again").exists(),
        "a failed build writes nothing"
    );
}

// Verifies: G-5
#[test]
fn code_is_highlighted_or_drawn_in_every_output_and_kept_when_a_program_is_missing() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(
        root,
        "nectar.toml",
        &format!(
            "[[document]]\nsource = \"main.md\"\n\n[engine.process.missing]\n\
             command = \"nectar-test-no-such-program\"\nlanguages = [\"boxes\"]\n\n{}",
            failing_engine("failing", "lines")
        ),
    );
    write(
        root,
        "main.md",
        "# Main\n\n```rust\nfn main() {}\n```\n\n```railroad\nrule = \"a\" , { \"b\" } ;\n```\n\n\
         ```boxes\na -> b\n```\n\n```lines\nc -> d\n```\n",
    );

    let output = nectar(root, &["build"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    let warnings: Vec<_> = stderr
        .lines()
        .filter(|l| l.starts_with("warning:"))
        .collect();
    assert_eq!(
        warnings.len(),
        2,
        "only the missing and the failing program, the PDF shows the rest: {stderr}"
    );
    assert!(
        warnings[0].contains("main.md:11")
            && warnings[0].contains("could not render the boxes block"),
        "{stderr}"
    );
    assert!(
        warnings[1].contains("main.md:15") && warnings[1].contains("produced no SVG"),
        "{stderr}"
    );

    for page in ["public/index.html", "public/print/index.html"] {
        let html = fs::read_to_string(root.join(page)).unwrap();
        assert!(
            html.contains("<pre class=\"code highlighted\""),
            "{page} highlights: {html}"
        );
        assert!(
            html.contains("<figure class=\"graphic graphic-railroad\">"),
            "{page} draws: {html}"
        );
        for kept in ["language-boxes\">a -&gt; b", "language-lines\">c -&gt; d"] {
            assert!(html.contains(kept), "{page} keeps the code: {html}");
        }
    }
    assert!(root.join("public/print/main.pdf").is_file());
}

// Verifies: G-5, G-12
#[test]
fn a_rough_block_is_drawn_in_every_output_and_kept_when_it_is_no_svg() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "nectar.toml", "[[document]]\nsource = \"main.md\"\n");
    write(
        root,
        "main.md",
        "# Main\n\n```rough\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"60\" height=\"40\">\
         <rect x=\"5\" y=\"5\" width=\"50\" height=\"30\" fill=\"red\"/></svg>\n```\n\n\
         ```rough\nnot an svg\n```\n",
    );

    let output = nectar(root, &["build"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    let warnings: Vec<_> = stderr
        .lines()
        .filter(|l| l.starts_with("warning:"))
        .collect();
    assert_eq!(warnings.len(), 1, "{stderr}");
    assert!(
        warnings[0].contains("main.md:7") && warnings[0].contains("rough"),
        "{stderr}"
    );
    for page in ["public/index.html", "public/print/index.html"] {
        let html = fs::read_to_string(root.join(page)).unwrap();
        assert!(
            html.contains("<figure class=\"graphic graphic-rough\">"),
            "{page} draws: {html}"
        );
        assert!(
            html.contains("language-rough\">not an svg"),
            "{page} keeps: {html}"
        );
    }
    assert!(root.join("public/print/main.pdf").is_file());
}

/// Every file below `dir`, relative to it, with its content.
fn files(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(next) = pending.pop() {
        for entry in fs::read_dir(next).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let bytes = fs::read(&path).unwrap();
                found.insert(path.strip_prefix(dir).unwrap().to_path_buf(), bytes);
            }
        }
    }
    found
}

// Verifies: Q-3
#[test]
fn two_builds_of_the_guide_are_byte_identical() {
    let guide = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/guide");
    let out = tempfile::tempdir().unwrap();
    let builds: Vec<_> = ["first", "second"]
        .iter()
        .map(|name| {
            let dir = out.path().join(name);
            let output = nectar(&guide, &["build", "--output", dir.to_str().unwrap()]);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            files(&dir)
        })
        .collect();
    assert!(builds[0].len() > 10, "{:?}", builds[0].keys());
    assert_eq!(
        builds[0].keys().collect::<Vec<_>>(),
        builds[1].keys().collect::<Vec<_>>()
    );
    for (path, bytes) in &builds[0] {
        assert!(builds[1][path] == *bytes, "{} differs", path.display());
    }
}

/// What a build may touch outside its project on Linux besides the root's own parents: the
/// dynamic loader and the libraries it finds, under `/lib` or, where `/lib` is merged into it,
/// `/usr/lib`; what the standard library reads to count the cores; and what glibc's allocator
/// reads to count them when it sizes its arenas (get_nprocs, called from free in glibc 2.41).
const SYSTEM: &[&str] = &[
    "/etc/ld.so.cache",
    "/etc/ld.so.preload",
    "/lib/",
    "/usr/lib/",
    "/proc/self/cgroup",
    "/proc/self/maps",
    "/proc/self/mountinfo",
    "/sys/devices/system/cpu/online",
    "/sys/fs/cgroup/",
];

/// The string arguments of a call as strace writes them, escapes undone, each with the argument
/// before it, which names the directory descriptor a relative path is resolved against.
fn strings(arguments: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    let (mut previous, mut current) = (String::new(), String::new());
    let mut chars = arguments.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                let mut text = String::new();
                while let Some(c) = chars.next() {
                    match c {
                        '\\' => text.extend(chars.next()),
                        '"' => break,
                        c => text.push(c),
                    }
                }
                found.push((text, previous.trim().to_string()));
                current.push_str("\"\"");
            }
            ',' => previous = std::mem::take(&mut current),
            c => current.push(c),
        }
    }
    found
}

/// What an `strace -f` of a build in `root` shows that Q-5 forbids: a path outside the root, its
/// parents and the system's, or one the check cannot place; a program other than `program`; and
/// any use of the network.
fn forbidden_in_trace(trace: &str, root: &Path, program: &Path) -> Vec<String> {
    let mut forbidden = Vec::new();
    for line in trace.lines() {
        // `<pid> <call>(<arguments>) = <result>`
        let call = line
            .split_once(' ')
            .map_or(line, |(_, call)| call.trim_start());
        let Some((name, arguments)) = call.split_once('(') else {
            continue;
        };
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        if ["socket", "connect", "sendto", "sendmsg", "bind"].contains(&name) {
            forbidden.push(line.to_string());
            continue;
        }
        let mut paths = strings(arguments);
        // The program's arguments and a link's content are no paths the call touches.
        if ["execve", "readlink", "readlinkat"].contains(&name) {
            paths.truncate(1);
        }
        let allowed = paths
            .iter()
            .filter(|(path, _)| !path.is_empty())
            .all(|(path, before)| {
                let path = Path::new(path);
                let path = if path.is_absolute() {
                    path.to_path_buf()
                } else if before.is_empty() || before == "AT_FDCWD" {
                    root.join(path)
                } else {
                    // Relative to a directory descriptor the check cannot follow.
                    return false;
                };
                // `..` after a link leads where the link points, which the check cannot see.
                if path
                    .components()
                    .any(|c| c == std::path::Component::ParentDir)
                {
                    return false;
                }
                if name == "execve" {
                    return path == program;
                }
                path.starts_with(root)
                    || root.ancestors().any(|parent| parent == path)
                    || path == program
                    || SYSTEM.iter().any(|system| {
                        path.starts_with(system) || path == Path::new(system.trim_end_matches('/'))
                    })
            });
        if !allowed {
            forbidden.push(line.to_string());
        }
    }
    forbidden
}

#[test]
fn the_trace_check_finds_what_q5_forbids() {
    let root = Path::new("/work/project");
    let program = Path::new("/bin/nectar");
    let trace = r#"7 execve("/bin/nectar", ["/bin/nectar"], 0x0 /* 3 vars */) = 0
7 openat(AT_FDCWD, "/etc/ld.so.cache", O_RDONLY|O_CLOEXEC) = 3
7 readlink("/work", 0x7ffe, 1023) = -1 EINVAL (Invalid argument)
7 openat(AT_FDCWD, "/work/project/main.md", O_RDONLY|O_CLOEXEC) = 3
7 statx(3, "", AT_EMPTY_PATH, STATX_ALL, {}) = 0
7 readlink("/work/project/link.md", "/work/secret.md", 1023) = 15
7 +++ exited with 0 +++
7 openat(AT_FDCWD, "/work/project/odd\".md", O_RDONLY|O_CLOEXEC) = 3
7 openat(AT_FDCWD, "/proc/self/cgroup", O_RDONLY|O_CLOEXEC) = 3
8 openat(AT_FDCWD, "/work/project/../secret.md", O_RDONLY|O_CLOEXEC) = 3
8 newfstatat(AT_FDCWD, "notes/../../../etc/passwd", {}, 0) = 0
8 execve("/bin/sh", ["sh", "-c", "touch x"], 0x0 /* 3 vars */) = 0
8 connect(3, {sa_family=AF_INET, sin_port=htons(80)}, 16) = 0
8 openat(5, "secret.md", O_RDONLY|O_CLOEXEC) = 3
8 renameat2(AT_FDCWD, "/work/project/a.md", AT_FDCWD, "/work/b.md", 0) = 0
8 openat(AT_FDCWD, "/proc/self/root/etc/passwd", O_RDONLY|O_CLOEXEC) = 3"#;
    let forbidden = forbidden_in_trace(trace, root, program);
    let expected = [
        "../secret.md",
        "passwd",
        "/bin/sh",
        "connect",
        "openat(5,",
        "/work/b.md",
        "/proc/self/root",
    ];
    assert_eq!(forbidden.len(), expected.len(), "{forbidden:#?}");
    for (line, what) in forbidden.iter().zip(expected) {
        assert!(line.contains(what), "{line} for {what}");
    }
}

/// Runs `nectar` like [`nectar`]. On Linux it runs under `strace -f` and fails on whatever Q-5
/// forbids; elsewhere no tracer is at hand, and the checks of the output alone hold.
fn confined(root: &Path, args: &[&str]) -> Output {
    #[cfg(target_os = "linux")]
    {
        let program = Path::new(env!("CARGO_BIN_EXE_nectar"));
        let trace = root.parent().unwrap().join("trace.txt");
        let output = Command::new("strace")
            .args([
                "-f",
                "-qq",
                "-s",
                "4096",
                "-e",
                "trace=%file,%process,%network",
                "-o",
            ])
            .arg(&trace)
            .arg(program)
            // Cargo points the loader at its build directories for tests; nectar needs none.
            .env_remove("LD_LIBRARY_PATH")
            .current_dir(root)
            .arg("-C")
            .arg(root)
            .arg("--offline")
            .args(args)
            .output()
            .unwrap_or_else(|e| panic!("Q-5 traces the build with strace on Linux: {e}"));
        let root = root.canonicalize().unwrap();
        let forbidden = forbidden_in_trace(&fs::read_to_string(trace).unwrap(), &root, program);
        assert!(forbidden.is_empty(), "outside the project: {forbidden:#?}");
        output
    }
    #[cfg(not(target_os = "linux"))]
    nectar(root, args)
}

// Verifies: Q-5
#[test]
fn a_project_reaches_nothing_outside_its_root_runs_no_injected_command_and_stays_offline() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("project");
    write(dir.path(), "secret.md", "# Secret\n\nTOPSECRET\n");
    write(dir.path(), "secret.png", "TOPSECRET");
    let marker = dir.path().join("injected");
    let web = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    web.set_nonblocking(true).unwrap();
    let port = web.local_addr().unwrap().port();

    write(
        &root,
        "nectar.toml",
        &format!(
            "[[document]]\nsource = \"main.md\"\n\n{}",
            failing_engine("shell", "sh")
        ),
    );
    let touch = format!("touch {}", marker.display());
    write(
        &root,
        "main.md",
        &format!(
            "# Main\n\n[Secret](../secret.md)\n\n![Secret](../secret.png)\n\n\
             Read [it](../secret.md).\n\n![Web](http://127.0.0.1:{port}/a.png)\n\n\
             ```sh\n$({touch})\n`{touch}`\n; {touch}\n```\n"
        ),
    );

    let output = confined(&root, &["build"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert_eq!(
        stderr.matches("points outside the project").count(),
        3,
        "{stderr}"
    );
    for (path, bytes) in files(&root.join("public")) {
        let text = String::from_utf8_lossy(&bytes);
        assert!(!text.contains("TOPSECRET"), "{} leaks", path.display());
    }
    assert!(
        stderr.contains("produced no SVG"),
        "the program got the code block: {stderr}"
    );
    assert!(!marker.exists(), "a code block ran a command");
    assert!(
        web.accept()
            .is_err_and(|e| e.kind() == std::io::ErrorKind::WouldBlock),
        "a request went out despite --offline"
    );

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(dir.path().join("secret.md"), root.join("linked.md")).unwrap();
        write(&root, "main.md", "# Main\n\n[Linked](linked.md)\n");
        let output = confined(&root, &["build", "--output", "linked"]);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{stderr}");
        assert!(
            stderr.contains("the file lies outside the project"),
            "{stderr}"
        );
        assert!(!stderr.contains("TOPSECRET"), "{stderr}");
        assert!(!root.join("linked").exists());
    }
}

// Verifies: Q-2
#[test]
fn every_block_of_the_guide_website_is_in_the_pdf_or_reported() {
    let guide = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/guide");
    let out = tempfile::tempdir().unwrap();
    let output = nectar(&guide, &["build", "--output", out.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    let reported = |message: &str| stderr.lines().filter(|l| l.contains(message)).count();
    let images_left_out = reported("not embedded in the PDF");
    let html_left_out = reported("raw HTML is left out of the PDF");
    // A cover image is a figure of the PDF that no web page has.
    let cover = usize::from(
        fs::read_to_string(guide.join("nectar.toml"))
            .unwrap()
            .contains("cover_image"),
    );

    // The content of every web page, without menu, header and footer.
    let mut web = String::new();
    for (path, bytes) in files(out.path()) {
        if path.starts_with("print") || path.extension().is_none_or(|e| e != "html") {
            continue;
        }
        let page = String::from_utf8(bytes).unwrap();
        let start = page.find("<main class=\"markdown\">").expect("content");
        let end = page[start..].find("</main>").unwrap() + start;
        web.push_str(&page[start..end]);
    }
    // The PDF is tagged: every element is a structure element named by its kind.
    let pdf = String::from_utf8_lossy(&files(&out.path().join("print"))[Path::new("GUIDE.pdf")])
        .into_owned();
    let count = |text: &str, patterns: &[&str]| -> usize {
        patterns.iter().map(|p| text.matches(p).count()).sum()
    };

    // Each kind as the website and the PDF mark it, and what the PDF reports as left out of it.
    let exact: [(&str, &[&str], &[&str], usize); 4] = [
        (
            "headings",
            &["<h1", "<h2", "<h3", "<h4", "<h5", "<h6"],
            &["/S/H1", "/S/H2", "/S/H3", "/S/H4", "/S/H5", "/S/H6"],
            0,
        ),
        ("tables", &["<table"], &["/S/Table"], 0),
        // The PDF reserves figures for pictures, a caption around one included: a listing, code
        // with a caption, counts as code.
        ("pictures", &["<img"], &["/S/Figure"], images_left_out),
        // Code blocks, highlighted or not, and inline code are code elements alike in the PDF.
        ("code", &["<pre", "<code>"], &["/S/Code"], 0),
    ];
    for (kind, in_web, in_pdf, left_out) in exact {
        let (mut on_web, mut in_pdf) = (count(&web, in_web), count(&pdf, in_pdf));
        if kind == "pictures" {
            in_pdf -= cover;
        }
        if kind == "code" {
            // A block without a language is `<pre class="code"><code>`: one block, not two.
            on_web -= count(&web, &["<pre class=\"code\"><code>"]);
        }
        assert!(on_web > 0, "the guide has no {kind}");
        assert_eq!(
            in_pdf + left_out,
            on_web,
            "{kind}: {left_out} reported as left out"
        );
    }
    // The cover and the contents add paragraphs of their own.
    let (web_paragraphs, pdf_paragraphs) = (count(&web, &["<p>", "<p "]), count(&pdf, &["/S/P/"]));
    assert!(
        pdf_paragraphs + html_left_out >= web_paragraphs,
        "{web_paragraphs} paragraphs on the website, {pdf_paragraphs} in the PDF"
    );
}

/// The lines of every Markdown file below `dir` that Nectar reads as Markdown, with inline code
/// cut out, the info strings of the fenced code blocks among them, and the lines of the front
/// matter that opens a file: what a guide uses itself rather than shows as a sample.
fn live_markdown(dir: &Path) -> (Vec<String>, Vec<String>, Vec<String>) {
    let (mut lines, mut fences, mut front) = (Vec::new(), Vec::new(), Vec::new());
    let mut paths: Vec<_> = files(dir)
        .into_keys()
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    paths.sort();
    for path in paths {
        let text = fs::read_to_string(dir.join(&path)).unwrap();
        let mut body = text.lines();
        if text.starts_with("+++\n") {
            body.next();
            front.extend(
                body.by_ref()
                    .take_while(|l| *l != "+++")
                    .map(str::to_string),
            );
        }
        let mut open: Option<(char, usize)> = None;
        for line in body {
            let trimmed = line.trim_start();
            let mark = trimmed.chars().next().filter(|c| *c == '`' || *c == '~');
            let run = mark.map_or(0, |m| trimmed.chars().take_while(|c| *c == m).count());
            if let (Some(m), true) = (mark, run >= 3) {
                match open {
                    None => {
                        open = Some((m, run));
                        fences.push(trimmed[run..].trim().to_string());
                    }
                    Some((o, n)) if o == m && run >= n && trimmed[run..].trim().is_empty() => {
                        open = None;
                    }
                    Some(_) => {}
                }
                continue;
            }
            if open.is_none() {
                // Inline code is a sample too: drop every span between backticks.
                let live: String = line.split('`').step_by(2).collect();
                lines.push(live);
            }
        }
    }
    (lines, fences, front)
}

// Verifies: ADR-0014
#[test]
fn the_guide_uses_every_feature_it_explains() {
    let guide = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/guide");
    let (lines, fences, front) = live_markdown(&guide);
    let config = fs::read_to_string(guide.join("nectar.toml")).unwrap();
    let any = |test: &dyn Fn(&str) -> bool| lines.iter().any(|l| test(l));
    let image = |l: &str, ends: &[&str]| {
        l.starts_with("![")
            && ends
                .iter()
                .any(|e| l.split("](").nth(1).is_some_and(|t| t.contains(e)))
    };
    let symbol = |l: &str| {
        l.split('@')
            .skip(1)
            .step_by(2)
            .any(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase() || c == '_'))
            && l.matches('@').count() >= 2
    };
    let features: [(&str, bool); 17] = [
        (
            "an included chapter",
            any(&|l| l.starts_with('[') && l.contains("](") && l.ends_with(".md)")),
        ),
        (
            "a slug in front matter",
            front.iter().any(|l| l.starts_with("slug ")),
        ),
        (
            "a title in front matter",
            front.iter().any(|l| l.starts_with("title ")),
        ),
        (
            "variables in front matter",
            front.iter().any(|l| l == "[vars]"),
        ),
        (
            "a page break",
            any(&|l| ["---", "___", "***"].contains(&l.trim_end())),
        ),
        (
            "an alert in GitHub's notation",
            any(&|l| l.starts_with("> [!")),
        ),
        (
            "print settings with a cover image",
            config.contains("[document.print]") && config.contains("cover_image"),
        ),
        ("an embedded source file", any(&|l| image(l, &[".java)"]))),
        (
            "an embedded grammar file",
            any(&|l| image(l, &[".railroad)"])),
        ),
        (
            "a raster image with attributes",
            any(&|l| image(l, &[".png){"])),
        ),
        ("a railroad diagram", fences.iter().any(|f| f == "railroad")),
        ("a Graphviz diagram", fences.iter().any(|f| f == "dot")),
        ("a PlantUML diagram", fences.iter().any(|f| f == "uml")),
        (
            "an alert",
            any(&|l| {
                ["!n ", "!s ", "!w ", "!e ", "!!"]
                    .iter()
                    .any(|a| l.starts_with(a))
            }),
        ),
        ("a footnote", any(&|l| l.starts_with("[^"))),
        ("a symbol", any(&symbol)),
        ("a variable", any(&|l| l.contains("{{"))),
    ];
    for (feature, used) in features {
        assert!(used, "the guide no longer uses {feature} itself");
    }
    assert!(
        any(&|l| l.starts_with("- [x]") || l.starts_with("- [ ]")),
        "no task list"
    );
    assert!(
        any(&|l| l.starts_with("|--") || l.starts_with("|:-")),
        "no table"
    );
}

/// Runs `nectar` in `dir` without `-C`, as a writer calls it on a single file.
fn nectar_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_nectar"))
        .current_dir(dir)
        .arg("--offline")
        .args(args)
        .output()
        .unwrap()
}

/// The names of the files directly in `dir`, sorted.
fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

// Verifies: G-10, ADR-0015
#[test]
fn a_markdown_file_builds_to_its_pdf_without_a_configuration() {
    let dir = tempfile::tempdir().unwrap();
    let notes = dir.path().join("notes");
    write(
        &notes,
        "readme.md",
        "# Notes\n\nSome *text*.\n\n```railroad\nrule = \"a\" { \"b\" } ;\n```\n",
    );

    // Run from the parent: the file's directory is the root, and the PDF lands next to the file.
    let output = nectar_in(dir.path(), &["build", "notes/readme.md"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert_eq!(names(&notes), ["readme.md", "readme.pdf"]);
    assert_eq!(names(dir.path()), ["notes"]);
    assert!(
        fs::read(notes.join("readme.pdf"))
            .unwrap()
            .starts_with(b"%PDF")
    );

    // -o names the PDF file instead, and nothing lands next to the source; an existing directory
    // takes the PDF under its usual name, and the source itself is never overwritten.
    fs::remove_file(notes.join("readme.pdf")).unwrap();
    let output = nectar_in(&notes, &["build", "readme.md", "-o", "readme.md"]);
    assert!(!output.status.success());
    assert!(
        fs::read_to_string(notes.join("readme.md"))
            .unwrap()
            .starts_with("# Notes")
    );
    let output = nectar_in(&notes, &["build", "readme.md", "-o", "out/manual.pdf"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(names(&notes), ["out", "readme.md"]);
    assert_eq!(names(&notes.join("out")), ["manual.pdf"]);
    let output = nectar_in(&notes, &["build", "readme.md", "-o", "out"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(names(&notes.join("out")), ["manual.pdf", "readme.pdf"]);
}

// Verifies: G-10, ADR-0015
#[test]
fn a_markdown_file_ignores_a_configuration_beside_it_and_stays_in_its_directory() {
    let dir = tempfile::tempdir().unwrap();
    let notes = dir.path().join("notes");
    write(dir.path(), "secret.md", "# Secret\n\nTOPSECRET\n");
    write(dir.path(), "secret.png", "TOPSECRET");
    write(&notes, "nectar.toml", "this is not a configuration");
    write(
        &notes,
        "readme.md",
        "# Notes\n\n[Secret](../secret.md)\n\n![Secret](../secret.png)\n",
    );

    // Run from the parent: were the current directory the root, ../secret.md from notes/ would lie
    // inside it and be read; with the file's directory as the root it lies outside.
    let output = nectar_in(dir.path(), &["build", "notes/readme.md"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert_eq!(
        stderr.matches("points outside the project").count(),
        2,
        "{stderr}"
    );
    assert_eq!(names(&notes), ["nectar.toml", "readme.md", "readme.pdf"]);

    // -C names a project; together with a file it is refused.
    let output = nectar_in(dir.path(), &["-C", "notes", "build", "notes/readme.md"]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("not both"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// Verifies: G-8
#[test]
fn files_in_memory_build_the_same_website_as_the_command_line() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("project");
    write(
        &root,
        "nectar.toml",
        "title = \"Site\"\n\n[[document]]\nsource = \"docs/main.md\"\n",
    );
    write(
        &root,
        "docs/main.md",
        "# Manual\n\nSee [the part](part.md#details)[^n].\n\n[^n]: A note.\n\n[Part](part.md)\n",
    );
    write(
        &root,
        "docs/part.md",
        "# Part\n\n## Details\n\n```rust\nfn main() {}\n```\n\n```railroad\nrule = \"a\" , { \"b\" } ;\n```\n\n\
          ![Logo](img/logo.svg)\n",
    );
    write(
        &root,
        "docs/img/logo.svg",
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"10\" height=\"10\"/>",
    );
    let in_memory: MemoryFiles = files(&root).into_iter().collect();

    let output = dir.path().join("public");
    let built = nectar(&root, &["build", "--output", output.to_str().unwrap()]);
    assert!(built.status.success());
    let from_disk = files(&output);

    let mut project = Project::load(Box::new(in_memory)).unwrap();
    project.set_offline(true);
    project.add_engine(Box::new(Railroad));
    project.add_engine(Box::new(
        TextMate::new(DEFAULT_THEME, Some(DEFAULT_DARK_THEME)).unwrap(),
    ));
    let web = HtmlRenderer {
        print_view: true,
        pdf: true,
    };
    let build = project.build(&[&web, &PrintRenderer, &PdfRenderer]);
    assert!(build.diagnostics.is_empty(), "{:?}", build.diagnostics);

    assert_eq!(
        build.files.keys().collect::<Vec<_>>(),
        from_disk.keys().collect::<Vec<_>>()
    );
    for (path, bytes) in &build.files {
        assert!(from_disk[path] == *bytes, "{} differs", path.display());
    }
}

#[test]
fn the_command_line_hands_nectar_variables_to_the_project() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(
        root,
        "nectar.toml",
        "[[document]]\nsource = \"main.md\"\n[vars]\nVERSION = \"1.0\"\n",
    );
    write(root, "main.md", "# Main\n\nVersion {{VERSION}}.\n");
    let output = Command::new(env!("CARGO_BIN_EXE_nectar"))
        .arg("-C")
        .arg(root)
        .args(["--offline", "build"])
        .env("NECTAR_VERSION", "2.0")
        .output()
        .unwrap();
    assert!(output.status.success());
    let page = fs::read_to_string(root.join("public/index.html")).unwrap();
    assert!(page.contains("Version 2.0."), "{page}");
}
