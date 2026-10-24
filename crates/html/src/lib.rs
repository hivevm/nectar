//! HTML renderer with a built-in default theme.
//!
//! A document becomes one page, or one page per chapter when it is split. Pages are rendered
//! through the theme's `layouts/page.html`, a minijinja template; files in the theme's `static/`
//! directory are copied to `_theme/` in the output.

mod print;
mod write;

use std::collections::{BTreeSet, HashMap};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use minijinja::value::Serde;
use minijinja::{Environment, Value, context};
use nectar_core::model::{Block, BlockKind, Chapter, Document, Heading, Inline};
use nectar_core::{
    Diagnostic, Origin, Output, PrintSettings, RenderContext, Renderer, Split, pdf_file,
};
use serde::Serialize;

pub use crate::print::PrintRenderer;
use crate::write::{PageWriter, escape};

const DEFAULT_LAYOUT: &str = include_str!("../theme/layouts/page.html");
const DEFAULT_STATIC: &[(&str, &[u8])] = &[
    ("style.css", include_bytes!("../theme/static/style.css")),
    ("print.css", include_bytes!("../theme/static/print.css")),
    (
        "print/paged.polyfill.min.js",
        include_bytes!("../theme/static/print/paged.polyfill.min.js"),
    ),
    (
        "print/LICENSE-pagedjs.md",
        include_bytes!("../theme/static/print/LICENSE-pagedjs.md"),
    ),
    (
        "print/draft.png",
        include_bytes!("../theme/static/print/draft.png"),
    ),
    (
        "print/confidential.png",
        include_bytes!("../theme/static/print/confidential.png"),
    ),
    (
        "fonts/MaterialIcons-Regular.ttf",
        include_bytes!("../theme/static/fonts/MaterialIcons-Regular.ttf"),
    ),
    (
        "fonts/UbuntuSans-Regular.woff2",
        include_bytes!("../theme/static/fonts/UbuntuSans-Regular.woff2"),
    ),
    (
        "fonts/UbuntuSans-Italic.woff2",
        include_bytes!("../theme/static/fonts/UbuntuSans-Italic.woff2"),
    ),
    (
        "fonts/UbuntuSans-Bold.woff2",
        include_bytes!("../theme/static/fonts/UbuntuSans-Bold.woff2"),
    ),
    (
        "fonts/UbuntuSans-BoldItalic.woff2",
        include_bytes!("../theme/static/fonts/UbuntuSans-BoldItalic.woff2"),
    ),
    (
        "fonts/VictorMono-Medium.woff2",
        include_bytes!("../theme/static/fonts/VictorMono-Medium.woff2"),
    ),
    (
        "fonts/VictorMono-MediumItalic.woff2",
        include_bytes!("../theme/static/fonts/VictorMono-MediumItalic.woff2"),
    ),
    (
        "fonts/VictorMono-Bold.woff2",
        include_bytes!("../theme/static/fonts/VictorMono-Bold.woff2"),
    ),
    (
        "fonts/VictorMono-BoldItalic.woff2",
        include_bytes!("../theme/static/fonts/VictorMono-BoldItalic.woff2"),
    ),
    (
        "fonts/LICENCE-UbuntuSans.txt",
        include_bytes!("../theme/static/fonts/LICENCE-UbuntuSans.txt"),
    ),
    (
        "fonts/LICENSE-VictorMono.txt",
        include_bytes!("../theme/static/fonts/LICENSE-VictorMono.txt"),
    ),
];
pub(crate) const THEME_OUTPUT: &str = "_theme";
const LAYOUT: &str = "page.html";

#[derive(Debug, Default)]
pub struct HtmlRenderer {
    /// Link every page to the print view of its document, written by [`PrintRenderer`].
    pub print_view: bool,
    /// Link every page to the PDF file of its document, written by a PDF renderer.
    pub pdf: bool,
}

struct Page<'d> {
    chapter: &'d Chapter,
    /// Output file, relative to the output root.
    file: String,
    depth: usize,
}

/// Which print outputs the pages of a document link to.
#[derive(Clone, Copy)]
struct PrintLinks {
    view: bool,
    pdf: bool,
}

/// An entry of the menu tree in the sidebar.
#[derive(Serialize)]
struct MenuItem {
    /// Id of the checkbox that opens and closes the entry's children.
    id: String,
    title: String,
    url: String,
    current: bool,
    /// The children are shown when the page loads.
    open: bool,
    children: Vec<MenuItem>,
}

#[derive(Serialize)]
struct TocItem {
    level: u8,
    id: String,
    title: String,
}

#[derive(Serialize)]
pub(crate) struct ContentsItem {
    level: u8,
    title: String,
    url: String,
}

#[derive(Serialize)]
struct LinkItem {
    title: String,
    url: String,
}

impl Renderer for HtmlRenderer {
    fn render(&self, documents: &[Document], ctx: &RenderContext<'_>) -> Output {
        let mut output = Output::default();
        let mut env = Environment::new();
        let layout = layout(ctx, &mut output, LAYOUT, DEFAULT_LAYOUT);
        if let Err(e) = env.add_template_owned(LAYOUT, layout) {
            output
                .diagnostics
                .push(Diagnostic::error(format!("invalid layout: {e}"), None));
            return output;
        }
        copy_static(ctx, &mut output);

        let index_files: Vec<String> = documents
            .iter()
            .map(|d| join(&d.path, "index.html"))
            .collect();
        for document in documents {
            let settings = PrintSettings::of(document).ok();
            let links = PrintLinks {
                view: self.print_view && settings.as_ref().is_some_and(|s| s.enabled),
                pdf: self.pdf && settings.as_ref().is_some_and(|s| s.pdf),
            };
            render_document(
                document,
                documents,
                &index_files,
                links,
                &env,
                ctx,
                &mut output,
            );
        }
        if !documents.is_empty() && documents.iter().all(|d| !d.path.is_empty()) {
            render_index(documents, &index_files, &env, ctx, &mut output);
        }
        output
    }
}

fn render_document(
    document: &Document,
    documents: &[Document],
    index_files: &[String],
    links: PrintLinks,
    env: &Environment<'_>,
    ctx: &RenderContext<'_>,
    output: &mut Output,
) {
    let nested = document.split == Split::None;
    let pages = pages(document);
    let mut anchors = HashMap::new();
    for page in &pages {
        add_anchors(page.chapter, &page.file, nested, &mut anchors);
    }

    for (index, page) in pages.iter().enumerate() {
        let mut writer = PageWriter {
            out: String::new(),
            page: &page.file,
            anchors: &anchors,
            assets: &document.assets,
            nested_chapters: nested,
            files: &mut output.files,
            footnotes: BTreeSet::new(),
            inline_notes: None,
            heading_anchors: true,
        };
        writer.blocks(&page.chapter.blocks, false);
        writer.footnotes(&document.footnotes);
        let content = writer.out;

        let link = |other: &Page<'_>| LinkItem {
            title: chapter_title(other.chapter),
            url: relative_url(&page.file, &other.file),
        };
        let parents = parents(&pages, index);
        let menu = menu(document, &pages, index, &parents);
        let breadcrumb: Vec<LinkItem> = parents.iter().rev().map(|&i| link(&pages[i])).collect();
        let prev = index.checked_sub(1).map(|i| link(&pages[i]));
        let next = pages.get(index + 1).map(link);
        let documents: Vec<LinkItem> = documents
            .iter()
            .zip(index_files)
            .map(|(d, file)| LinkItem {
                title: d.title.clone(),
                url: relative_url(&page.file, file),
            })
            .collect();

        let page_contents = if index == 0 && document.contents {
            contents(document, &page.file, &anchors)
        } else {
            Vec::new()
        };
        let root = root_prefix(&page.file);
        let context = context! {
            site => context! { title => site_title(ctx, document) },
            document => context! { title => document.title.clone(), url => relative_url(&page.file, &pages[0].file) },
            page => context! { title => page.chapter.title.clone(), url => relative_url(&page.file, &page.file) },
            menu => Value::from(Serde(&menu)),
            breadcrumb => Value::from(Serde(&breadcrumb)),
            print => links.view.then(|| relative_url(&page.file, &print::print_file(document))),
            pdf => links.pdf.then(|| relative_url(&page.file, &pdf_file(document))),
            toc => Value::from(Serde(&toc(page.chapter, nested))),
            contents => Value::from(Serde(&page_contents)),
            prev => Value::from(Serde(&prev)),
            next => Value::from(Serde(&next)),
            documents => Value::from(Serde(&documents)),
            root => root.clone(),
            theme => format!("{root}{THEME_OUTPUT}/"),
            content => Value::from_safe_string(content),
        };
        render_page(
            env,
            &page.file,
            context,
            Some(Origin::new(&page.chapter.source, 0)),
            output,
        );
    }
}

/// A list of all documents at the output root, for projects whose documents all have their own
/// directory.
fn render_index(
    documents: &[Document],
    index_files: &[String],
    env: &Environment<'_>,
    ctx: &RenderContext<'_>,
    output: &mut Output,
) {
    let mut content = String::from("<ul class=\"documents\">\n");
    let mut links = Vec::new();
    for (document, file) in documents.iter().zip(index_files) {
        let _ = writeln!(
            content,
            "<li><a href=\"{}\">{}</a></li>",
            escape(file),
            escape(&document.title)
        );
        links.push(LinkItem {
            title: document.title.clone(),
            url: file.clone(),
        });
    }
    content.push_str("</ul>\n");
    let title = if ctx.config.title.is_empty() {
        "Documents".to_string()
    } else {
        ctx.config.title.clone()
    };
    let context = context! {
        site => context! { title => title.clone() },
        document => context! { title => title.clone(), url => "index.html" },
        page => context! { title => title, url => "index.html" },
        menu => Value::from(Serde(&Vec::<MenuItem>::new())),
        breadcrumb => Value::from(Serde(&Vec::<LinkItem>::new())),
        toc => Value::from(Serde(&Vec::<TocItem>::new())),
        documents => Value::from(Serde(&links)),
        root => "",
        theme => format!("{THEME_OUTPUT}/"),
        content => Value::from_safe_string(content),
    };
    render_page(env, "index.html", context, None, output);
}

fn render_page(
    env: &Environment<'_>,
    file: &str,
    context: Value,
    origin: Option<Origin>,
    output: &mut Output,
) {
    match env
        .get_template(LAYOUT)
        .and_then(|template| template.render(context))
    {
        Ok(html) => {
            output.files.insert(PathBuf::from(file), html.into_bytes());
        }
        Err(e) => output.diagnostics.push(Diagnostic::error(
            format!("cannot render {file}: {e}"),
            origin,
        )),
    }
}

pub(crate) fn site_title(ctx: &RenderContext<'_>, document: &Document) -> String {
    if ctx.config.title.is_empty() {
        document.title.clone()
    } else {
        ctx.config.title.clone()
    }
}

/// The root chapter is the document's index page; with `split = "chapter"` every nested chapter
/// gets a page of its own below the document directory.
fn pages(document: &Document) -> Vec<Page<'_>> {
    let mut pages = vec![Page {
        chapter: &document.root,
        file: join(&document.path, "index.html"),
        depth: 0,
    }];
    if document.split == Split::Chapter {
        collect_pages(&document.root, 1, &document.path, &mut pages);
    }
    pages
}

/// The menu as seen from page `index`: the document's first page, then the chapters as a tree
/// whose branches open along the path to the current page. A document on a single page lists
/// its headings instead, with every branch closed.
fn menu(document: &Document, pages: &[Page<'_>], index: usize, parents: &[usize]) -> Vec<MenuItem> {
    let from = &pages[index].file;
    let mut items = vec![MenuItem {
        id: format!("menu-{}", pages[0].chapter.id),
        title: chapter_title(pages[0].chapter),
        url: relative_url(from, &pages[0].file),
        current: index == 0,
        open: false,
        children: Vec::new(),
    }];
    if pages.len() > 1 {
        let mut next = 1;
        items.extend(page_tree(pages, &mut next, 1, index, parents));
    } else if document.split == Split::None {
        let mut headings = Vec::new();
        for_each_heading(&document.root.blocks, true, &mut |heading| {
            if (2..=3).contains(&heading.level) {
                headings.push(heading.clone());
            }
        });
        let mut next = 0;
        items.extend(heading_tree(&headings, &mut next, 2));
    }
    items
}

fn page_tree(
    pages: &[Page<'_>],
    next: &mut usize,
    depth: usize,
    current: usize,
    parents: &[usize],
) -> Vec<MenuItem> {
    let mut items = Vec::new();
    while *next < pages.len() && pages[*next].depth == depth {
        let index = *next;
        *next += 1;
        let children = page_tree(pages, next, depth + 1, current, parents);
        items.push(MenuItem {
            id: format!("menu-{}", pages[index].chapter.id),
            title: chapter_title(pages[index].chapter),
            url: relative_url(&pages[current].file, &pages[index].file),
            current: index == current,
            open: index == current || parents.contains(&index),
            children,
        });
    }
    items
}

fn heading_tree(headings: &[Heading], next: &mut usize, level: u8) -> Vec<MenuItem> {
    let mut items = Vec::new();
    while *next < headings.len() && headings[*next].level >= level {
        let heading = &headings[*next];
        *next += 1;
        let children = heading_tree(headings, next, heading.level + 1);
        items.push(MenuItem {
            id: format!("menu-{}", heading.id),
            title: heading_title(heading),
            url: format!("#{}", heading.id),
            current: false,
            open: false,
            children,
        });
    }
    items
}

/// The indexes of the pages above page `index`, nearest first.
fn parents(pages: &[Page<'_>], index: usize) -> Vec<usize> {
    let mut parents = Vec::new();
    let mut depth = pages[index].depth;
    for i in (0..index).rev() {
        if pages[i].depth < depth {
            parents.push(i);
            depth = pages[i].depth;
        }
    }
    parents
}

fn collect_pages<'d>(chapter: &'d Chapter, depth: usize, base: &str, pages: &mut Vec<Page<'d>>) {
    for block in &chapter.blocks {
        if let BlockKind::Chapter(child) = &block.kind {
            let file = join(&join(base, &child.slug), "index.html");
            pages.push(Page {
                chapter: child,
                file,
                depth,
            });
            collect_pages(child, depth + 1, base, pages);
        }
    }
}

pub(crate) fn add_anchors(
    chapter: &Chapter,
    file: &str,
    nested: bool,
    anchors: &mut HashMap<String, String>,
) {
    anchors
        .entry(chapter.id.clone())
        .or_insert_with(|| file.to_string());
    for_each_heading(&chapter.blocks, nested, &mut |heading| {
        anchors
            .entry(heading.id.clone())
            .or_insert_with(|| file.to_string());
    });
    if nested {
        for block in &chapter.blocks {
            if let BlockKind::Chapter(child) = &block.kind {
                add_anchors(child, file, nested, anchors);
            }
        }
    }
}

/// The headings of a page below its title, at most two levels deep.
/// A heading's text with its section number.
pub(crate) fn heading_title(heading: &Heading) -> String {
    let text = Inline::plain_text(&heading.content);
    if heading.number.is_empty() {
        text
    } else {
        format!("{} {text}", heading.number)
    }
}

/// A chapter's title with the section number of its first heading.
pub(crate) fn chapter_title(chapter: &Chapter) -> String {
    let number = chapter.blocks.iter().find_map(|block| match &block.kind {
        BlockKind::Heading(heading) => Some(heading.number.as_str()),
        _ => None,
    });
    match number {
        Some(number) if !number.is_empty() => format!("{number} {}", chapter.title),
        _ => chapter.title.clone(),
    }
}

/// The table of contents of the whole document, as seen from `page`: every heading below the
/// top level, at most two levels deep.
pub(crate) fn contents(
    document: &Document,
    page: &str,
    anchors: &HashMap<String, String>,
) -> Vec<ContentsItem> {
    let mut items = Vec::new();
    for_each_heading(&document.root.blocks, true, &mut |heading| {
        if !(2..=3).contains(&heading.level) {
            return;
        }
        let url = match anchors.get(&heading.id) {
            Some(file) if file != page => format!("{}#{}", relative_url(page, file), heading.id),
            _ => format!("#{}", heading.id),
        };
        items.push(ContentsItem {
            level: heading.level,
            title: heading_title(heading),
            url,
        });
    });
    items
}

fn toc(chapter: &Chapter, nested: bool) -> Vec<TocItem> {
    let mut headings = Vec::new();
    for_each_heading(&chapter.blocks, nested, &mut |heading| {
        headings.push(TocItem {
            level: heading.level,
            id: heading.id.clone(),
            title: heading_title(heading),
        });
    });
    let Some(top) = headings.iter().map(|h| h.level).min() else {
        return headings;
    };
    headings.retain(|h| h.level > top && h.level <= top + 2);
    headings
}

pub(crate) fn for_each_heading(blocks: &[Block], nested: bool, f: &mut dyn FnMut(&Heading)) {
    for block in blocks {
        match &block.kind {
            BlockKind::Heading(heading) => f(heading),
            BlockKind::Chapter(chapter) if nested => for_each_heading(&chapter.blocks, nested, f),
            BlockKind::Quote(children) | BlockKind::Alert(_, children) => {
                for_each_heading(children, nested, f)
            }
            _ => {}
        }
    }
}

/// The project theme's layout `name`, or the built-in one.
pub(crate) fn layout(
    ctx: &RenderContext<'_>,
    output: &mut Output,
    name: &str,
    default: &str,
) -> String {
    let Some(theme) = &ctx.config.theme else {
        return default.to_string();
    };
    let dir = Path::new("themes").join(theme);
    if !ctx.root.join(&dir).is_dir() {
        output.diagnostics.push(Diagnostic::error(
            format!("theme directory {} does not exist", dir.display()),
            None,
        ));
        return default.to_string();
    }
    let path = dir.join("layouts").join(name);
    if !ctx.root.join(&path).is_file() {
        return default.to_string();
    }
    output.sources.insert(path.clone());
    match ctx.read(&path).map(String::from_utf8) {
        Ok(Ok(text)) => text,
        _ => {
            output.diagnostics.push(Diagnostic::error(
                format!("cannot read {}", path.display()),
                None,
            ));
            default.to_string()
        }
    }
}

/// Copies the built-in static files, then the project theme's, which win on equal names.
pub(crate) fn copy_static(ctx: &RenderContext<'_>, output: &mut Output) {
    for (name, bytes) in DEFAULT_STATIC {
        output
            .files
            .insert(Path::new(THEME_OUTPUT).join(name), bytes.to_vec());
    }
    let Some(theme) = &ctx.config.theme else {
        return;
    };
    let dir = Path::new("themes").join(theme).join("static");
    let mut files = Vec::new();
    list_files(ctx.root, &dir, &mut files);
    for file in files {
        let Ok(relative) = file.strip_prefix(&dir) else {
            continue;
        };
        match ctx.read(&file) {
            Ok(bytes) => {
                output
                    .files
                    .insert(Path::new(THEME_OUTPUT).join(relative), bytes);
                output.sources.insert(file);
            }
            Err(e) => output.diagnostics.push(Diagnostic::error(
                format!("cannot read {}: {e}", file.display()),
                None,
            )),
        }
    }
}

fn list_files(root: &Path, dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root.join(dir)) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = dir.join(entry.file_name());
        match entry.file_type() {
            Ok(kind) if kind.is_dir() => list_files(root, &path, files),
            Ok(kind) if kind.is_file() => files.push(path),
            _ => {}
        }
    }
}

pub(crate) fn join(base: &str, name: &str) -> String {
    if base.is_empty() {
        name.to_string()
    } else {
        format!("{base}/{name}")
    }
}

/// `../` for every directory between the output root and `file`.
pub(crate) fn root_prefix(file: &str) -> String {
    "../".repeat(file.matches('/').count())
}

/// The URL of output file `to`, relative to output file `from`. Both use `/`.
pub(crate) fn relative_url(from: &str, to: &str) -> String {
    let to = to.replace('\\', "/");
    let mut from_dirs: Vec<&str> = from.split('/').collect();
    from_dirs.pop();
    let to_parts: Vec<&str> = to.split('/').collect();
    let (to_dirs, to_file) = to_parts.split_at(to_parts.len() - 1);
    let common = from_dirs
        .iter()
        .zip(to_dirs)
        .take_while(|(a, b)| a == b)
        .count();
    let mut parts = vec![".."; from_dirs.len() - common];
    parts.extend(&to_dirs[common..]);
    parts.extend(to_file);
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_urls() {
        assert_eq!(
            relative_url("index.html", "setup/index.html"),
            "setup/index.html"
        );
        assert_eq!(
            relative_url("setup/index.html", "index.html"),
            "../index.html"
        );
        assert_eq!(
            relative_url("m/a/index.html", "m/b/index.html"),
            "../b/index.html"
        );
        assert_eq!(
            relative_url("m/a/index.html", "m/a/index.html"),
            "index.html"
        );
        assert_eq!(
            relative_url("m/index.html", "_graphics/x.svg"),
            "../_graphics/x.svg"
        );
    }

    #[test]
    fn root_prefixes() {
        assert_eq!(root_prefix("index.html"), "");
        assert_eq!(root_prefix("manual/setup/index.html"), "../../");
    }
}
