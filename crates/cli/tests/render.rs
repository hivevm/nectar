//! Runs `nectar render` on texts from standard input and from files, in temporary directories.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

/// Runs `nectar` in `dir` with `args`, `input` on standard input.
fn nectar(dir: &Path, args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nectar"))
        .current_dir(dir)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

/// `text` without its escape sequences.
fn plain(text: &str) -> String {
    let mut plain = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('\x1b') {
        plain.push_str(&rest[..start]);
        let end = rest[start..]
            .find(|c: char| c.is_ascii_alphabetic())
            .unwrap();
        rest = &rest[start + end + 1..];
    }
    plain.push_str(rest);
    plain
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// A `nectar.toml` with a process engine for `rr` that runs on every host the tests do: `nectar
/// render railroad` itself, which reads a grammar on standard input and writes SVG.
fn project_with_process_engine(dir: &Path) {
    fs::write(
        dir.join("nectar.toml"),
        format!(
            "[[document]]\nsource = \"index.md\"\n\n[engine.process.rr]\ncommand = '{}'\n\
             args = [\"render\", \"railroad\"]\nlanguages = [\"rr\"]\n",
            env!("CARGO_BIN_EXE_nectar")
        ),
    )
    .unwrap();
    fs::write(dir.join("index.md"), "# Index\n").unwrap();
}

// Verifies: G-11, ADR-0018
#[test]
fn a_graphic_from_standard_input_goes_to_standard_output_as_svg() {
    let dir = tempfile::tempdir().unwrap();
    let output = nectar(
        dir.path(),
        &["render", "railroad"],
        "rule ::= \"a\" | \"b\" ;\n",
    );
    assert!(output.status.success(), "{}", stderr(&output));
    let svg = String::from_utf8(output.stdout).unwrap();
    assert!(svg.starts_with("<svg"), "{svg}");
    assert!(fs::read_dir(dir.path()).unwrap().next().is_none());
}

// Verifies: G-11, ADR-0018
#[test]
fn a_file_is_read_and_the_extension_of_output_chooses_the_format() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("grammar.txt"), "rule ::= \"a\" ;\n").unwrap();
    let output = nectar(
        dir.path(),
        &["render", "railroad", "grammar.txt", "-o", "out/rule.png"],
        "",
    );
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(output.stdout.is_empty());
    let png = fs::read(dir.path().join("out/rule.png")).unwrap();
    assert!(png.starts_with(PNG_SIGNATURE));
}

// Verifies: G-11, ADR-0018
#[test]
fn highlighted_code_is_html_by_default_and_ansi_on_request() {
    let dir = tempfile::tempdir().unwrap();
    let code = "fn main() {\n\n    let x = 1 < 2;\n}\n";

    let html = nectar(dir.path(), &["render", "rust"], code);
    assert!(html.status.success(), "{}", stderr(&html));
    let html = String::from_utf8(html.stdout).unwrap();
    assert!(html.starts_with("<pre style=\""), "{html}");
    assert!(html.contains("<span style=\"color:"), "{html}");
    assert!(html.contains("&lt;"), "{html}");

    let explicit = nectar(dir.path(), &["render", "rust", "-f", "html"], code);
    assert!(explicit.status.success(), "{}", stderr(&explicit));
    assert_eq!(String::from_utf8(explicit.stdout).unwrap(), html);

    let ansi = nectar(dir.path(), &["render", "rust", "-f", "ansi"], code);
    assert!(ansi.status.success(), "{}", stderr(&ansi));
    let ansi = String::from_utf8(ansi.stdout).unwrap();
    assert!(ansi.contains("\x1b[0;38;2;"), "{ansi:?}");
    // Every line, the empty one included, is filled to the edge before its reset.
    assert_eq!(
        ansi.matches("\x1b[K\x1b[0m\n").count(),
        code.lines().count(),
        "{ansi:?}"
    );
    assert_eq!(plain(&ansi), code);
}

// Verifies: G-11, ADR-0018
#[test]
fn an_extension_that_names_no_format_gives_the_default() {
    let dir = tempfile::tempdir().unwrap();
    let output = nectar(
        dir.path(),
        &["render", "rust", "-o", "x.txt"],
        "fn f() {}\n",
    );
    assert!(output.status.success(), "{}", stderr(&output));
    let written = fs::read_to_string(dir.path().join("x.txt")).unwrap();
    assert!(written.starts_with("<pre"), "{written}");
}

// Verifies: G-11, ADR-0018
#[test]
fn an_extension_the_output_does_not_allow_fails_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let output = nectar(
        dir.path(),
        &["render", "rust", "-o", "x.png"],
        "fn f() {}\n",
    );
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("choose html or ansi"),
        "{}",
        stderr(&output)
    );
    assert!(!dir.path().join("x.png").exists());
}

// Verifies: G-11, ADR-0018
#[test]
fn a_format_the_output_does_not_allow_fails_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let output = nectar(
        dir.path(),
        &["render", "rust", "-f", "png", "-o", "x.png"],
        "fn f() {}\n",
    );
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("choose html or ansi"),
        "{}",
        stderr(&output)
    );
    assert!(!dir.path().join("x.png").exists());
}

// Verifies: G-11, ADR-0018
#[test]
fn an_unknown_language_fails_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let output = nectar(dir.path(), &["render", "nope", "-o", "x.svg"], "x\n");
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("no engine supports 'nope'"),
        "{}",
        stderr(&output)
    );
    assert!(!dir.path().join("x.svg").exists());
}

// Verifies: ADR-0018
#[test]
fn an_engine_that_fails_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let output = nectar(dir.path(), &["render", "railroad", "-o", "x.svg"], "((((\n");
    assert!(!output.status.success());
    assert!(stderr(&output).contains("railroad"), "{}", stderr(&output));
    assert!(!dir.path().join("x.svg").exists());
}

// Verifies: ADR-0018
#[test]
fn c_needs_a_complete_project_configuration() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("nectar.toml"),
        "[engine.railroad]\nenabled = true\n",
    )
    .unwrap();
    let output = nectar(
        dir.path(),
        &["-C", ".", "render", "railroad"],
        "rule ::= \"a\" ;\n",
    );
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("[[document]]"),
        "{}",
        stderr(&output)
    );
}

// Verifies: G-11, ADR-0018
#[test]
fn process_engines_come_only_from_the_nectar_toml_that_c_names() {
    let dir = tempfile::tempdir().unwrap();
    project_with_process_engine(dir.path());
    let grammar = "rule ::= \"a\" ;\n";

    let without = nectar(dir.path(), &["render", "rr"], grammar);
    assert!(!without.status.success());
    assert!(stderr(&without).contains("-C"), "{}", stderr(&without));

    let with = nectar(dir.path(), &["-C", ".", "render", "rr"], grammar);
    assert!(with.status.success(), "{}", stderr(&with));
    assert!(String::from_utf8(with.stdout).unwrap().starts_with("<svg"));
}

// Verifies: ADR-0017
#[test]
fn a_png_is_the_same_every_time_and_draws_its_text() {
    let dir = tempfile::tempdir().unwrap();
    let render = |grammar: &str| {
        let output = nectar(dir.path(), &["render", "railroad", "-f", "png"], grammar);
        assert!(output.status.success(), "{}", stderr(&output));
        output.stdout
    };

    let first = render("rule ::= \"ab\" ;\n");
    assert!(first.starts_with(PNG_SIGNATURE));
    assert_eq!(first, render("rule ::= \"ab\" ;\n"));
    // The same boxes with other glyphs: without a font for the text, both pictures are equal.
    assert_ne!(first, render("rule ::= \"ba\" ;\n"));
}

// Verifies: G-12, ADR-0019
#[test]
fn an_svg_becomes_a_sketch_as_svg_and_as_png() {
    let dir = tempfile::tempdir().unwrap();
    let input = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"120\" height=\"80\">\
                 <rect x=\"10\" y=\"10\" width=\"100\" height=\"60\" fill=\"red\" stroke=\"black\" \
                 data-rough-fill-style=\"cross-hatch\"/></svg>";
    fs::write(dir.path().join("in.svg"), input).unwrap();

    let output = nectar(dir.path(), &["render", "rough"], input);
    assert!(output.status.success(), "{}", stderr(&output));
    let svg = String::from_utf8(output.stdout).unwrap();
    assert!(svg.starts_with("<svg"), "{svg}");
    assert!(svg.contains("stroke=\"#ff0000\""), "{svg}");
    assert!(!svg.contains("<rect"), "{svg}");

    let output = nectar(
        dir.path(),
        &["render", "rough", "in.svg", "-o", "out.png"],
        "",
    );
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        fs::read(dir.path().join("out.png"))
            .unwrap()
            .starts_with(PNG_SIGNATURE)
    );

    let output = nectar(dir.path(), &["render", "rough"], "rule ::= 'a' ;");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}
