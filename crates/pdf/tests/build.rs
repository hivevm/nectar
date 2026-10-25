//! Typesets small projects into PDF through the core and the PDF renderer.

use std::path::Path;

use nectar_core::Project;
use nectar_pdf::PdfRenderer;

fn write(root: &Path, file: &str, bytes: &[u8]) {
    let path = root.join(file);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}

const PNG: &[u8] = include_bytes!("../images/draft.png");

#[test]
fn typesets_every_kind_of_block() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(
        root,
        "nectar.toml",
        b"title = \"Site\"\n[[document]]\nsource = \"main.md\"\n[document.print]\nsubtitle = \"Version 1\"\nwatermark = \"confidential\"\nchapter_pages = true\nsize = \"A5 landscape\"\nmargin = \"15mm\"\n",
    );
    write(
        root,
        "main.md",
        "# Manual \"quoted\" #not-code [x]\n\nIntro with *em*, **strong**, //mark//, ____under____, ~~strike~~, `code`, @web@ and a note[^n].\n\n\
         [^n]: The *note*.\n\n[Part](part.md)\n"
            .as_bytes(),
    );
    write(
        root,
        "part.md",
        "# Part\n\n## Details {#details}\n\nSee [details](#details), [the web](https://example.org) and [ref][r].\n\n\
         [r]: https://ref.org \"Ref\"\n\n!w Careful.\n\n> Quoted.\n\n1. one\n2. two\n\n- [x] done\n\n\
         | a | b |\n| -- | ---- |\n| 1 | 2 |\n\n```\nplain $code\n```\n\n---\n\n![logo](img/logo.png){width=60 align=center}\n"
            .as_bytes(),
    );
    write(root, "img/logo.png", PNG);

    let build = Project::load(root).unwrap().build(&[&PdfRenderer]);
    assert!(build.diagnostics.is_empty(), "{:?}", build.diagnostics);
    let pdf = &build.files[Path::new("print/main.pdf")];
    assert!(pdf.starts_with(b"%PDF-"), "not a PDF");
    assert!(pdf.len() > 10_000, "fonts are embedded");
}

#[test]
fn pdf_can_be_turned_off_and_reports_bad_settings() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "main.md", b"# Manual\n");

    write(
        root,
        "nectar.toml",
        b"[[document]]\nsource = \"main.md\"\n[document.print]\npdf = false\n",
    );
    let build = Project::load(root).unwrap().build(&[&PdfRenderer]);
    assert!(build.files.is_empty());

    write(
        root,
        "nectar.toml",
        b"[[document]]\nsource = \"main.md\"\n[document.print]\nsize = \"huge\"\n",
    );
    let build = Project::load(root).unwrap().build(&[&PdfRenderer]);
    assert!(build.has_errors());
    assert!(
        build.diagnostics[0].message.contains("page size"),
        "{:?}",
        build.diagnostics
    );
}

#[test]
fn embeds_data_uri_images_and_reports_what_is_left_out() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    // With html = true raw HTML reaches the renderers, and the PDF reports it (G-4).
    write(
        root,
        "nectar.toml",
        b"html = true\n[[document]]\nsource = \"main.md\"\n",
    );
    let encoded = base64(PNG);
    write(
        root,
        "main.md",
        format!(
            "# Manual\n\n![logo](data:image/png;base64,{encoded})\n\n<div>raw</div>\n\n![web](https://example.org/a.png)\n\nGet [the file](files/data.zip).\n"
        )
        .as_bytes(),
    );
    write(root, "files/data.zip", b"zip");

    let mut project = Project::load(root).unwrap();
    project.set_offline(true);
    let build = project.build(&[&PdfRenderer]);
    let messages: Vec<&str> = build
        .diagnostics
        .iter()
        .map(|d| d.message.as_str())
        .collect();
    assert_eq!(messages.len(), 2, "{messages:?}");
    assert!(
        messages.iter().any(|m| m.contains("raw HTML is left out")),
        "{messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|m| m.contains("https://example.org/a.png")),
        "{messages:?}"
    );
    assert!(
        build
            .diagnostics
            .iter()
            .all(|d| d.origin.as_ref().is_some_and(|o| o.line > 0))
    );
    assert!(build.files[Path::new("print/main.pdf")].starts_with(b"%PDF-"));
}

fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut text = String::new();
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, b)| n | (u32::from(*b) << (16 - 8 * i)));
        for i in 0..4 {
            if i <= chunk.len() {
                text.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                text.push('=');
            }
        }
    }
    text
}

/// The pages of a PDF, counted by their page objects.
fn pages(pdf: &[u8]) -> usize {
    let text = String::from_utf8_lossy(pdf).replace("/Type /", "/Type/");
    text.matches("/Type/Page").count() - text.matches("/Type/Pages").count()
}

#[test]
fn a_thematic_break_starts_a_new_page() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(
        root,
        "nectar.toml",
        b"[[document]]\nsource = \"main.md\"\ncontents = false\n[document.print]\ncover = false\n",
    );
    write(root, "main.md", b"# Manual\n\nBefore.\n\nAfter.\n");
    let build = Project::load(root).unwrap().build(&[&PdfRenderer]);
    let without = pages(&build.files[Path::new("print/main.pdf")]);

    write(root, "main.md", b"# Manual\n\nBefore.\n\n---\n\nAfter.\n");
    let build = Project::load(root).unwrap().build(&[&PdfRenderer]);
    assert!(build.diagnostics.is_empty(), "{:?}", build.diagnostics);
    assert_eq!(
        pages(&build.files[Path::new("print/main.pdf")]),
        without + 1
    );
}
