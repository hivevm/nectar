//! Print renderer: one paged HTML file per document, laid out into pages in the browser by
//! Paged.js. The same file is the print preview on screen and the source for printing or saving
//! as PDF.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use minijinja::value::Serde;
use minijinja::{Environment, Value, context};
use nectar_core::model::Document;
use nectar_core::{Diagnostic, Origin, Output, PrintSettings, RenderContext, Renderer};

use crate::write::PageWriter;
use crate::{
    THEME_OUTPUT, add_anchors, contents, copy_static, join, layout, relative_url, root_prefix,
    site_title,
};

const DEFAULT_LAYOUT: &str = include_str!("../theme/layouts/print.html");
const LAYOUT: &str = "print.html";

#[derive(Debug, Default)]
pub struct PrintRenderer;

/// The print file of a document, relative to the output root.
pub(crate) fn print_file(document: &Document) -> String {
    join(&document.path, "print/index.html")
}

impl Renderer for PrintRenderer {
    fn render(&self, documents: &[Document], ctx: &RenderContext<'_>) -> Output {
        let mut output = Output::default();
        let mut env = Environment::new();
        let source = layout(ctx, &mut output, LAYOUT, DEFAULT_LAYOUT);
        if let Err(e) = env.add_template_owned(LAYOUT, source) {
            output.diagnostics.push(Diagnostic::error(
                format!("invalid print layout: {e}"),
                None,
            ));
            return output;
        }
        copy_static(ctx, &mut output);

        for document in documents {
            match PrintSettings::of(document) {
                Ok(settings) if settings.enabled => {
                    render_document(document, &settings, &env, ctx, &mut output)
                }
                Ok(_) => {}
                Err(e) => output.diagnostics.push(Diagnostic::error(
                    format!("invalid [document.print]: {e}"),
                    Some(Origin::new("nectar.toml", 0)),
                )),
            }
        }
        output
    }
}

fn render_document(
    document: &Document,
    settings: &PrintSettings,
    env: &Environment<'_>,
    ctx: &RenderContext<'_>,
    output: &mut Output,
) {
    let file = print_file(document);
    let mut anchors = HashMap::new();
    add_anchors(&document.root, &file, true, &mut anchors);

    let mut writer = PageWriter {
        out: String::new(),
        page: &file,
        anchors: &anchors,
        assets: &document.assets,
        nested_chapters: true,
        files: &mut output.files,
        footnotes: BTreeSet::new(),
        inline_notes: Some(&document.footnotes),
        heading_anchors: false,
    };
    writer.blocks(&document.root.blocks, false);
    let content = writer.out;

    let theme = format!("{}{THEME_OUTPUT}/", root_prefix(&file));
    let watermark = image(
        settings.watermark.as_deref(),
        document,
        &file,
        &theme,
        ctx,
        output,
    );
    let cover_image = image(
        settings.cover_image.as_deref(),
        document,
        &file,
        &theme,
        ctx,
        output,
    );
    let contents = if document.contents {
        contents(document, &file, &anchors)
    } else {
        Vec::new()
    };

    let context = context! {
        site => context! { title => site_title(ctx, document) },
        document => context! {
            title => document.title.clone(),
            url => relative_url(&file, &join(&document.path, "index.html")),
        },
        print => context! {
            size => settings.size.clone(),
            margin => settings.margin.clone(),
            cover => settings.cover,
            subtitle => settings.subtitle.clone(),
            chapter_pages => settings.chapter_pages,
        },
        watermark => watermark,
        cover_image => cover_image,
        contents => Value::from(Serde(&contents)),
        theme => theme,
        content => Value::from_safe_string(content),
    };
    let origin = Some(Origin::new(&document.root.source, 0));
    match env
        .get_template(LAYOUT)
        .and_then(|template| template.render(context))
    {
        Ok(html) => {
            output.files.insert(PathBuf::from(&file), html.into_bytes());
        }
        Err(e) => output.diagnostics.push(Diagnostic::error(
            format!("cannot render {file}: {e}"),
            origin,
        )),
    }
}

/// The URL of a built-in image (`draft`, `confidential`) or of a project image, which is copied
/// next to the print file.
fn image(
    value: Option<&str>,
    document: &Document,
    file: &str,
    theme: &str,
    ctx: &RenderContext<'_>,
    output: &mut Output,
) -> Option<String> {
    match value? {
        name @ ("draft" | "confidential") => Some(format!("{theme}print/{name}.png")),
        path => {
            let source = PathBuf::from(path);
            match ctx.files.read(&source) {
                Ok(bytes) => {
                    let name = source.file_name().unwrap_or_default().to_string_lossy();
                    let target = join(&document.path, &format!("print/{name}"));
                    output.files.insert(PathBuf::from(&target), bytes);
                    output.sources.insert(source.clone());
                    Some(relative_url(file, &target))
                }
                Err(e) => {
                    output.diagnostics.push(Diagnostic::warning(
                        format!("cannot read {}: {e}", Path::new(path).display()),
                        Some(Origin::new("nectar.toml", 0)),
                    ));
                    None
                }
            }
        }
    }
}
