//! PDF renderer: typesets every document with Typst, as a library, into one PDF file.
//!
//! The document model becomes Typst source, which Typst compiles with the bundled fonts and the
//! document's images and graphics. The settings of `[document.print]` apply as for the print
//! view.

mod typeset;
mod world;

use std::collections::HashMap;
use std::path::PathBuf;

use nectar_core::model::Document;
use nectar_core::{Diagnostic, Origin, Output, PrintSettings, RenderContext, Renderer, pdf_file};
use typst::WorldExt;
use typst::diag::SourceDiagnostic;
use typst_layout::PagedDocument;
use typst_pdf::PdfOptions;

use crate::typeset::{Page, Typesetter, virtual_path};
use crate::world::DocumentWorld;

const WATERMARKS: &[(&str, &[u8])] = &[
    ("draft", include_bytes!("../images/draft.png")),
    ("confidential", include_bytes!("../images/confidential.png")),
];

#[derive(Debug, Default)]
pub struct PdfRenderer;

impl Renderer for PdfRenderer {
    fn render(&self, documents: &[Document], ctx: &RenderContext<'_>) -> Output {
        let mut output = Output::default();
        for document in documents {
            let origin = Some(Origin::new("nectar.toml", 0));
            let settings = match PrintSettings::of(document) {
                Ok(settings) => settings,
                Err(e) => {
                    output.diagnostics.push(Diagnostic::error(
                        format!("invalid [document.print]: {e}"),
                        origin,
                    ));
                    continue;
                }
            };
            if !settings.pdf {
                continue;
            }
            let page = match Page::from_settings(&settings) {
                Ok(page) => page,
                Err(e) => {
                    output.diagnostics.push(Diagnostic::error(
                        format!("invalid [document.print]: {e}"),
                        origin,
                    ));
                    continue;
                }
            };
            render_document(document, &settings, &page, ctx, &mut output);
        }
        output
    }
}

fn render_document(
    document: &Document,
    settings: &PrintSettings,
    page: &Page,
    ctx: &RenderContext<'_>,
    output: &mut Output,
) {
    let mut files: HashMap<String, Vec<u8>> = HashMap::new();
    let mut image = |value: Option<&str>, output: &mut Output| -> Option<String> {
        let value = value?;
        if let Some((name, bytes)) = WATERMARKS.iter().find(|(name, _)| *name == value) {
            let path = format!("/_nectar/{name}.png");
            files.insert(path.clone(), bytes.to_vec());
            return Some(path);
        }
        let source = PathBuf::from(value);
        match ctx.read(&source) {
            Ok(bytes) => {
                let path = virtual_path(&source);
                files.insert(path.clone(), bytes);
                output.sources.insert(source);
                Some(path)
            }
            Err(e) => {
                output.diagnostics.push(Diagnostic::warning(
                    format!("cannot read {value}: {e}"),
                    Some(Origin::new("nectar.toml", 0)),
                ));
                None
            }
        }
    };
    let cover_image = image(settings.cover_image.as_deref(), output);
    let watermark = image(settings.watermark.as_deref(), output);

    let site_title = if ctx.config.title.is_empty() {
        &document.title
    } else {
        &ctx.config.title
    };
    let mut typesetter = Typesetter::new(document, settings);
    typesetter.document(
        site_title,
        page,
        cover_image.as_deref(),
        watermark.as_deref(),
    );

    output.diagnostics.append(&mut typesetter.diagnostics);
    for (path, source) in std::mem::take(&mut typesetter.wanted) {
        if let Some(bytes) = document.downloads.get(&source) {
            files.insert(path, bytes.clone());
            continue;
        }
        match ctx.read(&source) {
            Ok(bytes) => {
                files.insert(path, bytes);
            }
            Err(e) => output.diagnostics.push(Diagnostic::warning(
                format!("cannot read {}: {e}", source.display()),
                None,
            )),
        }
    }
    files.extend(std::mem::take(&mut typesetter.files));

    let world = DocumentWorld::new(typesetter.out, files);
    let origin = Some(Origin::new(&document.root.source, 0));
    let compiled = typst::compile::<PagedDocument>(&world);
    let paged = match compiled.output {
        Ok(paged) => paged,
        Err(errors) => {
            for error in errors.iter().take(5) {
                output.diagnostics.push(Diagnostic::error(
                    format!("PDF: {}", describe(error, &world)),
                    origin.clone(),
                ));
            }
            return;
        }
    };
    match typst_pdf::pdf(&paged, &PdfOptions::default()) {
        Ok(bytes) => {
            output
                .files
                .insert(PathBuf::from(pdf_file(document)), bytes);
        }
        Err(errors) => {
            for error in errors.iter().take(5) {
                output.diagnostics.push(Diagnostic::error(
                    format!("PDF: {}", describe(error, &world)),
                    origin.clone(),
                ));
            }
        }
    }
}

/// A Typst error with the line of generated source it points to.
fn describe(error: &SourceDiagnostic, world: &DocumentWorld) -> String {
    let source = world.main_source();
    let line = world
        .range(error.span)
        .and_then(|range| source.lines().byte_to_line(range.start))
        .and_then(|line| source.text().lines().nth(line));
    match line {
        Some(line) => {
            let excerpt: String = line.chars().take(160).collect();
            format!("{} (in: {excerpt})", error.message)
        }
        None => error.message.to_string(),
    }
}
