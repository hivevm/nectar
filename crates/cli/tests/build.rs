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
fn two_builds_of_the_demo_are_byte_identical() {
    let demo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/demo");
    let out = tempfile::tempdir().unwrap();
    let builds: Vec<_> = ["first", "second"]
        .iter()
        .map(|name| {
            let dir = out.path().join(name);
            let output = nectar(&demo, &["build", "--output", dir.to_str().unwrap()]);
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

    let output = nectar(&root, &["build"]);
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
        let output = nectar(&root, &["build", "--output", "linked"]);
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
fn every_block_of_the_demo_website_is_in_the_pdf_or_reported() {
    let demo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/demo");
    let out = tempfile::tempdir().unwrap();
    let output = nectar(&demo, &["build", "--output", out.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    let reported = |message: &str| stderr.lines().filter(|l| l.contains(message)).count();
    let images_left_out = reported("not embedded in the PDF");
    let html_left_out = reported("raw HTML is left out of the PDF");

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
    let pdf = String::from_utf8_lossy(&files(&out.path().join("print"))[Path::new("manual.pdf")])
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
        ("figures", &["<figure"], &["/S/Figure"], images_left_out),
        // Code blocks are highlighted text in the PDF, without a tag; inline code is tagged.
        ("inline code", &["<code>"], &["/S/Code"], 0),
    ];
    for (kind, in_web, in_pdf, left_out) in exact {
        let (web, pdf) = (count(&web, in_web), count(&pdf, in_pdf));
        assert!(web > 0, "the demo has no {kind}");
        assert_eq!(
            pdf + left_out,
            web,
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
