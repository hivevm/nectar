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
