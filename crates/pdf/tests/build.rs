//! Typesets small projects into PDF through the core and the PDF renderer.

use std::path::Path;

use nectar_core::model::{HighlightedCode, Token, TokenStyle};
use nectar_core::{CodeEngine, EngineError, EngineOutput, MemoryFiles, Project, Severity};
use nectar_pdf::PdfRenderer;

fn write(root: &MemoryFiles, file: &str, bytes: &[u8]) {
    root.insert(file, bytes);
}

const PNG: &[u8] = include_bytes!("../images/draft.png");

#[test]
fn typesets_every_kind_of_block() {
    let root = &MemoryFiles::new();
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

    let build = Project::load(Box::new(root.clone()))
        .unwrap()
        .build(&[&PdfRenderer]);
    assert!(build.diagnostics.is_empty(), "{:?}", build.diagnostics);
    let pdf = &build.files[Path::new("print/main.pdf")];
    assert!(pdf.starts_with(b"%PDF-"), "not a PDF");
    assert!(pdf.len() > 10_000, "fonts are embedded");
}

#[test]
fn pdf_can_be_turned_off_and_reports_bad_settings() {
    let root = &MemoryFiles::new();
    write(root, "main.md", b"# Manual\n");

    write(
        root,
        "nectar.toml",
        b"[[document]]\nsource = \"main.md\"\n[document.print]\npdf = false\n",
    );
    let build = Project::load(Box::new(root.clone()))
        .unwrap()
        .build(&[&PdfRenderer]);
    assert!(build.files.is_empty());

    write(
        root,
        "nectar.toml",
        b"[[document]]\nsource = \"main.md\"\n[document.print]\nsize = \"huge\"\n",
    );
    let build = Project::load(Box::new(root.clone()))
        .unwrap()
        .build(&[&PdfRenderer]);
    assert!(build.has_errors());
    assert!(
        build.diagnostics[0].message.contains("page size"),
        "{:?}",
        build.diagnostics
    );
}

// Verifies: G-4
#[test]
fn embeds_data_uri_images_and_reports_what_is_left_out() {
    let root = &MemoryFiles::new();
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

    let mut project = Project::load(Box::new(root.clone())).unwrap();
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
    assert!(build.diagnostics.iter().all(|d| {
        d.severity == Severity::Warning
            && d.origin
                .as_ref()
                .is_some_and(|o| o.file == Path::new("main.md") && o.line > 0)
    }));
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
    let root = &MemoryFiles::new();
    write(
        root,
        "nectar.toml",
        b"[[document]]\nsource = \"main.md\"\ncontents = false\n[document.print]\ncover = false\n",
    );
    write(root, "main.md", b"# Manual\n\nBefore.\n\nAfter.\n");
    let build = Project::load(Box::new(root.clone()))
        .unwrap()
        .build(&[&PdfRenderer]);
    let without = pages(&build.files[Path::new("print/main.pdf")]);

    write(root, "main.md", b"# Manual\n\nBefore.\n\n---\n\nAfter.\n");
    let build = Project::load(Box::new(root.clone()))
        .unwrap()
        .build(&[&PdfRenderer]);
    assert!(build.diagnostics.is_empty(), "{:?}", build.diagnostics);
    assert_eq!(
        pages(&build.files[Path::new("print/main.pdf")]),
        without + 1
    );
}

// Verifies: G-3
#[test]
fn the_pdf_has_a_cover_contents_and_footnotes() {
    let root = &MemoryFiles::new();
    write(
        root,
        "main.md",
        b"# Manual\n\nIntro[^n].\n\n[^n]: The note.\n\n[Part](part.md)\n",
    );
    write(root, "part.md", b"# Part\n\n## Details\n");
    let pdf = |cover: bool| {
        write(
            root,
            "nectar.toml",
            format!("[[document]]\nsource = \"main.md\"\n[document.print]\ncover = {cover}\n")
                .as_bytes(),
        );
        let build = Project::load(Box::new(root.clone()))
            .unwrap()
            .build(&[&PdfRenderer]);
        assert!(build.diagnostics.is_empty(), "{:?}", build.diagnostics);
        build.files[Path::new("print/main.pdf")].clone()
    };

    let with_cover = pdf(true);
    assert_eq!(
        pages(&with_cover),
        pages(&pdf(false)) + 1,
        "the cover is a page"
    );
    // The PDF is tagged: the contents and the footnotes are structure elements of their own.
    let text = String::from_utf8_lossy(&with_cover);
    assert!(text.contains("/S/TOC"), "contents");
    assert!(text.matches("/S/TOCI").count() >= 2, "an entry per chapter");
    assert!(text.contains("/S/Note"), "footnotes");
}

/// Highlights every `rust` block: keywords red, the rest as it is.
struct Colours;

impl CodeEngine for Colours {
    fn name(&self) -> &str {
        "colours"
    }

    fn supports(&self, language: &str) -> bool {
        language == "rust"
    }

    fn render(&self, language: &str, code: &str) -> Result<EngineOutput, EngineError> {
        let token = |text: &str, color: Option<&str>| Token {
            text: text.to_string(),
            style: TokenStyle {
                color: color.map(str::to_string),
                ..TokenStyle::default()
            },
            dark: None,
        };
        let lines = code
            .lines()
            .map(|line| match line.split_once(' ') {
                Some((keyword, rest)) => vec![
                    token(keyword, Some("#d73a49")),
                    token(&format!(" {rest}"), None),
                ],
                None => vec![token(line, None)],
            })
            .collect();
        Ok(EngineOutput::Highlighted(HighlightedCode {
            language: language.to_string(),
            foreground: None,
            background: None,
            dark_foreground: None,
            dark_background: None,
            lines,
        }))
    }
}

#[test]
fn highlighted_code_blocks_are_code_in_the_pdf() {
    let root = &MemoryFiles::new();
    write(root, "nectar.toml", b"[[document]]\nsource = \"main.md\"\n");
    write(
        root,
        "main.md",
        b"# Manual\n\nCall `main` first.\n\n```rust\nfn main() {}\nlet x = 1;\n```\n",
    );
    let mut project = Project::load(Box::new(root.clone())).unwrap();
    project.add_engine(Box::new(Colours));
    let build = project.build(&[&PdfRenderer]);
    assert!(build.diagnostics.is_empty(), "{:?}", build.diagnostics);

    let pdf = &build.files[Path::new("print/main.pdf")];
    let text = String::from_utf8_lossy(pdf);
    assert_eq!(
        text.matches("/S/Code").count(),
        2,
        "the inline code and the block"
    );
    // #d73a49, the engine's colour of `fn` and `let`, as the fill of the page's text.
    assert!(
        contents(pdf).contains("0.84313726 0.22745098 0.28627452"),
        "the engine's colours reach the page"
    );
}

/// The content streams of a PDF, unpacked, one after the other.
fn contents(pdf: &[u8]) -> String {
    let mut text = String::new();
    let mut rest = pdf;
    while let Some(start) = rest.windows(6).position(|w| w == b"stream") {
        let body = &rest[start + 6..];
        let body = body
            .strip_prefix(b"\r\n")
            .or(body.strip_prefix(b"\n"))
            .unwrap_or(body);
        let end = body.windows(9).position(|w| w == b"endstream").unwrap();
        if let Ok(unpacked) = miniz_oxide::inflate::decompress_to_vec_zlib(&body[..end]) {
            text.push_str(&String::from_utf8_lossy(&unpacked));
        }
        rest = &body[end + 9..];
    }
    text
}
