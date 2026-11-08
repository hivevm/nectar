//! A WebAssembly module of the portable crates, built by the WebAssembly step of
//! `scripts/check-build.sh` so that they link for `wasm32-unknown-unknown`, not only type-check
//! (ADR-0012). A link error, such as a second `getrandom` backend beside the one `nectar-rough`
//! registers (ADR-0014), fails that step. It is no product: nothing ships or calls it.

use std::sync::Arc;

use nectar_core::{MemoryFiles, Project};
use nectar_html::{EpubRenderer, HtmlRenderer};
use nectar_pdf::PdfRenderer;
use nectar_railroad::Railroad;
use nectar_rough::Rough;

/// Builds a project with a railroad diagram and a sketch into the website, the PDF and the EPUB.
#[unsafe(no_mangle)]
pub extern "C" fn build() {
    let files = MemoryFiles::new();
    files.insert(
        "nectar.toml",
        "[[document]]\nsource = \"main.md\"\n\n[document.epub]\nenabled = true\nlanguage = \"en\"\n",
    );
    files.insert(
        "main.md",
        "# Main\n\n```railroad\nrule ::= 'a' ;\n```\n\n```rough\n\
         <svg xmlns=\"http://www.w3.org/2000/svg\" width=\"10\" height=\"10\">\
         <rect width=\"5\" height=\"5\"/></svg>\n```\n",
    );
    let mut project = Project::load(Box::new(files)).expect("the project is valid");
    project.add_engine(Box::new(Railroad));
    project.add_engine(Box::new(Rough::new(
        Arc::new(usvg::fontdb::Database::new()),
    )));
    project.build(&[
        &HtmlRenderer::default(),
        &PdfRenderer,
        &EpubRenderer::default(),
    ]);
}
