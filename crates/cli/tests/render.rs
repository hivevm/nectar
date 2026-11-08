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
    assert_eq!(plain(&ansi), code);
}

/// The ANSI output fills every line with the theme's background, a choice of the
/// implementation, not of ADR-0018, and shows control characters instead of sending them.
#[test]
fn ansi_lines_are_filled_to_the_edge_and_control_characters_are_shown() {
    let dir = tempfile::tempdir().unwrap();
    let code = "fn main() {\n\n    let x = 1 < 2;\n}\n";
    let ansi = nectar(dir.path(), &["render", "rust", "-f", "ansi"], code);
    assert!(ansi.status.success(), "{}", stderr(&ansi));
    let ansi = String::from_utf8(ansi.stdout).unwrap();
    // Every line, the empty one included, is filled to the edge before its reset.
    assert_eq!(
        ansi.matches("\x1b[K\x1b[0m\n").count(),
        code.lines().count(),
        "{ansi:?}"
    );

    let hostile = "let s = \"\x1b]0;owned\x07\u{9b}2J\tend\";\n";
    let ansi = nectar(dir.path(), &["render", "rust", "-f", "ansi"], hostile);
    assert!(ansi.status.success(), "{}", stderr(&ansi));
    let ansi = String::from_utf8(ansi.stdout).unwrap();
    assert!(!ansi.contains("\x1b]0;"), "{ansi:?}");
    assert!(!ansi.contains('\x07'), "{ansi:?}");
    assert!(
        plain(&ansi).contains("\u{241b}]0;owned\u{2407}\u{fffd}2J\tend"),
        "C0 as pictures, C1 replaced, the tab kept: {ansi:?}"
    );
    assert!(!ansi.contains('\u{9b}'), "{ansi:?}");
}

// Verifies: G-11, ADR-0018
#[test]
fn a_failure_writes_nothing_to_standard_output() {
    let dir = tempfile::tempdir().unwrap();
    let grammar = "rule ::= \"a\" ;\n";
    for (args, input) in [
        (&["render", "no-such-language"][..], grammar),
        (&["render", "railroad", "-f", "ansi"][..], grammar),
        (&["render", "railroad"][..], "not a grammar"),
    ] {
        let output = nectar(dir.path(), args, input);
        assert!(!output.status.success(), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}: {:?}", output.stdout);
    }
}

/// `-o -` is standard output, as `-` for the file is standard input: a convention ADR-0018 is
/// silent on.
#[test]
fn dash_as_output_is_standard_output() {
    let dir = tempfile::tempdir().unwrap();
    let output = nectar(
        dir.path(),
        &["render", "railroad", "-o", "-"],
        "rule ::= \"a\" ;\n",
    );
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .starts_with("<svg")
    );
    assert!(!dir.path().join("-").exists());
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

// Verifies: G-11, ADR-0018
#[test]
fn an_engine_that_fails_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let output = nectar(dir.path(), &["render", "railroad", "-o", "x.svg"], "((((\n");
    assert!(!output.status.success());
    assert!(stderr(&output).contains("railroad"), "{}", stderr(&output));
    assert!(!dir.path().join("x.svg").exists());
}

/// `-C` reads `nectar.toml` as a build does, so it has to be a complete project configuration: a
/// consequence of reusing the configuration of a build, which ADR-0018 does not itself ask for.
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

// Verifies: G-12, ADR-0021
#[test]
fn engine_rough_in_the_nectar_toml_of_c_sets_the_defaults_of_the_rough_engine() {
    let dir = tempfile::tempdir().unwrap();
    let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"60\" height=\"60\">\
               <rect x=\"5\" y=\"5\" width=\"50\" height=\"50\" fill=\"red\"/></svg>";
    let configure = |rough: &str| {
        fs::write(
            dir.path().join("nectar.toml"),
            format!("[[document]]\nsource = \"index.md\"\n\n[engine.rough]\n{rough}"),
        )
        .unwrap();
        fs::write(dir.path().join("index.md"), "# Index\n").unwrap();
    };

    configure("fill_style = \"solid\"\nroughness = 1.5\nseed = 7\n");
    let output = nectar(dir.path(), &["-C", ".", "render", "rough"], svg);
    assert!(output.status.success(), "{}", stderr(&output));
    let sketch = String::from_utf8(output.stdout).unwrap();
    assert!(
        sketch.contains("fill=\"#ff0000\""),
        "solid by default: {sketch}"
    );

    // Without -C no nectar.toml is read, and Rough.js's hachure stays.
    let output = nectar(dir.path(), &["render", "rough"], svg);
    assert!(output.status.success(), "{}", stderr(&output));
    let sketch = String::from_utf8(output.stdout).unwrap();
    assert!(sketch.contains("stroke=\"#ff0000\""), "{sketch}");
    assert!(!sketch.contains("fill=\"#ff0000\""), "{sketch}");

    for (rough, named) in [
        ("colour = \"red\"\n", "colour"),
        ("enabled = false\nfill_stlye = \"dots\"\n", "fill_stlye"),
        ("fill_style = \"plaid\"\n", "fill_style"),
        ("roughness = [1]\n", "roughness"),
    ] {
        configure(rough);
        let output = nectar(
            dir.path(),
            &["-C", ".", "render", "rough", "-o", "out.svg"],
            svg,
        );
        assert!(!output.status.success(), "{rough}");
        assert!(
            stderr(&output).contains(named),
            "{rough}: {}",
            stderr(&output)
        );
        assert!(!dir.path().join("out.svg").exists());
    }
}

// Verifies: G-11, ADR-0018
#[test]
fn highlighted_code_with_crlf_line_ends_has_the_lines_of_its_text() {
    let dir = tempfile::tempdir().unwrap();
    let lf = "fn main() {\n    let x = 1;\n}\n";
    let crlf = lf.replace('\n', "\r\n");
    for format in ["html", "ansi"] {
        let render = |code: &str| {
            let output = nectar(dir.path(), &["render", "rust", "-f", format], code);
            assert!(output.status.success(), "{}", stderr(&output));
            String::from_utf8(output.stdout).unwrap()
        };
        let expected = render(lf);
        assert_eq!(render(&crlf), expected, "{format}");
        // A last line without a line end renders as before.
        assert_eq!(render(lf.trim_end()), expected, "{format}");
        assert_eq!(render(crlf.trim_end()), expected, "{format}");
    }
}

// Verifies: ADR-0020
#[test]
fn the_rough_engine_of_the_command_line_draws_a_family_the_fonts_lack_in_ubuntu_sans() {
    let dir = tempfile::tempdir().unwrap();
    let sketch = |family: &str| {
        let input = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"120\" height=\"40\">\
             <text x=\"4\" y=\"28\" font-size=\"20\" font-family=\"{family}\">Ag 1</text></svg>"
        );
        let output = nectar(dir.path(), &["render", "rough"], &input);
        assert!(output.status.success(), "{}", stderr(&output));
        output.stdout
    };
    assert_eq!(sketch("Nope"), sketch("Ubuntu Sans"));
    assert_ne!(
        sketch("Nope"),
        sketch("Victor Mono"),
        "the sketch shows its font"
    );
    assert_eq!(sketch("monospace"), sketch("Victor Mono"));
}
