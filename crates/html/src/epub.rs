//! EPUB renderer: one EPUB 3.3 publication per document that enables it under `[document.epub]`.
//! Each top-level chapter is one XHTML content document, written by the block writer of the
//! website in its mode for XML (ADR-draft-epub-renderer).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use nectar_core::model::{Block, BlockKind, Chapter, Document, Heading};
use nectar_core::{Diagnostic, Origin, Output, RenderContext, Renderer};
use serde::Deserialize;

use crate::write::{
    EpubNotes, PageWriter, escape, has_own_heading, id_fits_link, is_url, xml_text,
};
use crate::{heading_title, join, site_title, zip};

const STYLE: &[u8] = include_bytes!("../theme/epub.css");
const STYLE_FILE: &str = "style.css";
const NAV_FILE: &str = "nav.xhtml";
const COVER_FILE: &str = "text/cover.xhtml";
/// The container's root of the publication's files.
const ROOT: &str = "OEBPS";
const CONTAINER_XML: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<container version=\"1.0\" xmlns=\"urn:oasis:names:tc:opendocument:xmlns:container\">\n\
<rootfiles><rootfile full-path=\"OEBPS/content.opf\" media-type=\"application/oebps-package+xml\"/>\
</rootfiles>\n</container>\n";

/// `dcterms:modified` of a build the shell gives no time for: the earliest date ZIP can hold.
pub const DEFAULT_MODIFIED: &str = "1980-01-01T00:00:00Z";

#[derive(Debug)]
pub struct EpubRenderer {
    /// `dcterms:modified` of every publication, in the form `YYYY-MM-DDThh:mm:ssZ`, or what is
    /// wrong with the value the host gave, which only a document that enables the output has to
    /// answer for. A shell hands it over, since a clock of its own would break Q-3.
    pub modified: Result<String, String>,
}

impl Default for EpubRenderer {
    fn default() -> Self {
        Self {
            modified: Ok(DEFAULT_MODIFIED.to_string()),
        }
    }
}

/// Settings from `[document.epub]`.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub(crate) struct EpubSettings {
    pub enabled: bool,
    /// BCP 47 tag; EPUB 3.3 requires one, and Nectar has no other setting for it.
    language: Option<String>,
    identifier: Option<String>,
    /// An image, relative to the project root.
    cover_image: Option<String>,
}

impl EpubSettings {
    pub fn of(document: &Document) -> Result<Self, String> {
        toml::Value::Table(document.epub.clone())
            .try_into()
            .map_err(|e: toml::de::Error| e.message().to_string())
    }
}

/// The EPUB of a document, relative to the output root.
pub(crate) fn epub_file(document: &Document) -> String {
    let stem = document
        .root
        .source
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "document".to_string());
    join(&document.path, &format!("epub/{stem}.epub"))
}

impl Renderer for EpubRenderer {
    fn render(&self, documents: &[Document], ctx: &RenderContext<'_>) -> Output {
        let mut output = Output::default();
        for document in documents {
            match enabled_settings(document) {
                Ok(Some((settings, language))) => match &self.modified {
                    Ok(modified) => {
                        render_document(document, &settings, &language, modified, ctx, &mut output)
                    }
                    Err(e) => output.diagnostics.push(Diagnostic::error(
                        e.clone(),
                        Some(Origin::new(&document.root.source, 0)),
                    )),
                },
                Ok(None) => {}
                Err(e) => output.diagnostics.push(Diagnostic::error(
                    format!("invalid [document.epub]: {e}"),
                    Some(Origin::new("nectar.toml", 0)),
                )),
            }
        }
        output
    }
}

/// The settings of a document that enables the output, with its language; what is wrong with them.
fn enabled_settings(document: &Document) -> Result<Option<(EpubSettings, String)>, String> {
    let settings = EpubSettings::of(document)?;
    if !settings.enabled {
        return Ok(None);
    }
    let Some(language) = settings.language.clone() else {
        return Err("`enabled = true` needs a `language`, such as \"en\"".to_string());
    };
    if !is_language_tag(&language) {
        return Err(format!("`language` is no language tag: {language}"));
    }
    if settings
        .identifier
        .as_deref()
        .is_some_and(|identifier| xml_text(identifier).trim().is_empty())
    {
        return Err("`identifier` is empty".to_string());
    }
    Ok(Some((settings, language)))
}

/// Whether a tag has the shape of a BCP 47 tag: language, extended languages, script, region,
/// variants, extensions, private use, in that order. Whether a subtag is registered is for a
/// validator to say.
fn is_language_tag(tag: &str) -> bool {
    let alpha = |s: &str, n: std::ops::RangeInclusive<usize>| {
        n.contains(&s.len()) && s.chars().all(|c| c.is_ascii_alphabetic())
    };
    let alnum = |s: &str, n: std::ops::RangeInclusive<usize>| {
        n.contains(&s.len()) && s.chars().all(|c| c.is_ascii_alphanumeric())
    };
    let mut subtags = tag.split('-').peekable();
    if !subtags.next().is_some_and(|s| alpha(s, 2..=8)) {
        return false;
    }
    for _ in 0..3 {
        if subtags.peek().is_some_and(|s| alpha(s, 3..=3)) {
            subtags.next();
        }
    }
    if subtags.peek().is_some_and(|s| alpha(s, 4..=4)) {
        subtags.next();
    }
    if subtags
        .peek()
        .is_some_and(|s| alpha(s, 2..=2) || (s.len() == 3 && s.chars().all(|c| c.is_ascii_digit())))
    {
        subtags.next();
    }
    while subtags.peek().is_some_and(|s| {
        alnum(s, 5..=8)
            || (s.len() == 4 && s.starts_with(|c: char| c.is_ascii_digit()) && alnum(s, 4..=4))
    }) {
        subtags.next();
    }
    while subtags
        .peek()
        .is_some_and(|s| alnum(s, 1..=1) && !s.eq_ignore_ascii_case("x"))
    {
        subtags.next();
        if !subtags.peek().is_some_and(|s| alnum(s, 2..=8)) {
            return false;
        }
        while subtags.peek().is_some_and(|s| alnum(s, 2..=8)) {
            subtags.next();
        }
    }
    if subtags.peek().is_some_and(|s| s.eq_ignore_ascii_case("x")) {
        subtags.next();
        if !subtags.peek().is_some_and(|s| alnum(s, 1..=8)) {
            return false;
        }
        while subtags.peek().is_some_and(|s| alnum(s, 1..=8)) {
            subtags.next();
        }
    }
    subtags.next().is_none()
}

/// One content document: a chapter and whether the chapters below it are part of it.
struct ContentDocument<'d> {
    chapter: &'d Chapter,
    nested: bool,
    file: String,
}

impl ContentDocument<'_> {
    /// The level of its chapter in the tree of chapters: the root is the first.
    fn level(&self) -> u8 {
        if self.nested { 2 } else { 1 }
    }
}

fn content_documents(document: &Document) -> Vec<ContentDocument<'_>> {
    let mut pages = vec![ContentDocument {
        chapter: &document.root,
        nested: false,
        file: "text/c000.xhtml".to_string(),
    }];
    for block in &document.root.blocks {
        if let BlockKind::Chapter(chapter) = &block.kind {
            let file = format!("text/c{:03}.xhtml", pages.len());
            pages.push(ContentDocument {
                chapter,
                nested: true,
                file,
            });
        }
    }
    pages
}

fn render_document(
    document: &Document,
    settings: &EpubSettings,
    language: &str,
    modified: &str,
    ctx: &RenderContext<'_>,
    output: &mut Output,
) {
    // Images and graphics, by their paths below the publication's root.
    let mut resources: BTreeMap<PathBuf, Vec<u8>> = BTreeMap::new();
    let (assets, refused) = embed_assets(document, ctx, &mut resources, output);
    let pages = content_documents(document);
    let title = publication_title(document);

    // The content document of every heading and every chapter.
    let (mut anchors, mut chapters) = (HashMap::new(), HashMap::new());
    for page in &pages {
        visit_chapter(
            page.chapter,
            page.nested,
            page.level(),
            &mut |found| match found {
                Found::Heading(heading) if id_fits_link(&heading.id) => {
                    anchors
                        .entry(heading.id.clone())
                        .or_insert_with(|| page.file.clone());
                }
                Found::Heading(_) => {}
                Found::Chapter(chapter, level) => {
                    // Below a page's own chapter, its `<section>` or heading carries the id, if
                    // that can stand in a link. Otherwise a link finds the content document.
                    let map = if level > page.level() && id_fits_link(&chapter.id) {
                        &mut anchors
                    } else {
                        &mut chapters
                    };
                    map.entry(chapter.id.clone())
                        .or_insert_with(|| page.file.clone());
                }
            },
        );
    }

    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for page in &pages {
        let mut writer = PageWriter {
            out: String::new(),
            page: &page.file,
            anchors: &anchors,
            assets: &assets,
            nested_chapters: page.nested,
            files: &mut resources,
            footnotes: BTreeSet::new(),
            inline_notes: None,
            heading_anchors: false,
            epub: Some(EpubNotes {
                ids: HashSet::new(),
                chapters: &chapters,
                refused: &refused,
                origin: Origin::new(&page.chapter.source, 0),
                diagnostics: Vec::new(),
                in_link: false,
                waiting_notes: Vec::new(),
            }),
        };
        writer.blocks(&page.chapter.blocks, false);
        writer.footnotes(&document.footnotes);
        if let Some(mut notes) = writer.epub.take() {
            output.diagnostics.append(&mut notes.diagnostics);
        }
        let body = writer.out;
        let page_title = if xml_text(&page.chapter.title).trim().is_empty() {
            title.as_str()
        } else {
            page.chapter.title.as_str()
        };
        let xhtml = xhtml(page_title, language, &format!("../{STYLE_FILE}"), &body);
        files.insert(page.file.clone(), xhtml.into_bytes());
    }

    let cover = cover_image(settings, ctx, &mut resources, output);
    if let Some(image) = &cover {
        let body = format!(
            "<figure class=\"cover\"><img src=\"../{}\" alt=\"\" /></figure>\n",
            escape(image)
        );
        let xhtml = xhtml(&title, language, &format!("../{STYLE_FILE}"), &body);
        files.insert(COVER_FILE.to_string(), xhtml.into_bytes());
    }

    let nav = navigation(&title, &pages);
    files.insert(
        NAV_FILE.to_string(),
        xhtml(&title, language, STYLE_FILE, &nav).into_bytes(),
    );
    files.insert(STYLE_FILE.to_string(), STYLE.to_vec());
    for (path, bytes) in resources {
        files.insert(path.to_string_lossy().replace('\\', "/"), bytes);
    }

    let identifier = settings
        .identifier
        .clone()
        .unwrap_or_else(|| format!("nectar:{}:{}", site_title(ctx, document), document.path));
    let package = Package {
        title: &title,
        language,
        identifier: &identifier,
        modified,
        pages: &pages,
        nav_in_spine: document.contents,
        cover: cover.as_deref(),
    }
    .opf(&files);

    let mut entries: BTreeMap<String, Vec<u8>> = files
        .into_iter()
        .map(|(name, bytes)| (format!("{ROOT}/{name}"), bytes))
        .collect();
    entries.insert(format!("{ROOT}/content.opf"), package.into_bytes());
    entries.insert(
        "META-INF/container.xml".to_string(),
        CONTAINER_XML.as_bytes().to_vec(),
    );
    match zip::pack(&entries) {
        Ok(bytes) => {
            output
                .files
                .insert(PathBuf::from(epub_file(document)), bytes);
        }
        Err(e) => output.diagnostics.push(Diagnostic::error(
            format!("EPUB: {e}"),
            Some(Origin::new(&document.root.source, 0)),
        )),
    }
}

/// The media type of a file whose extension names a core media type of EPUB 3.3 for images.
fn image_media_type(path: &Path) -> Option<&'static str> {
    let extension = path.extension()?.to_string_lossy().to_ascii_lowercase();
    match extension.as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "svg" => Some("image/svg+xml"),
        _ => None,
    }
}

/// What keeps a file with these bytes out of the container: its extension names no core media
/// type, its first bytes do not fit the extension, or it is an SVG the publication cannot hold.
fn image_problem(path: &Path, bytes: &[u8]) -> Option<String> {
    let extension = path.extension()?.to_string_lossy().to_ascii_lowercase();
    let fits = match image_media_type(path) {
        Some("image/png") => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        Some("image/jpeg") => bytes.starts_with(b"\xFF\xD8\xFF"),
        Some("image/gif") => bytes.starts_with(b"GIF8"),
        Some("image/webp") => {
            bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP"
        }
        Some(_) => {
            return match std::str::from_utf8(bytes) {
                Ok(svg) => svg_problem(svg),
                Err(_) => Some("no text".to_string()),
            };
        }
        None => return Some("no core media type".to_string()),
    };
    (!fits).then(|| format!("its bytes are no {extension}"))
}

/// What keeps an SVG out of the container: it is not well-formed, holds a script, or refers to
/// anything outside itself. A link in it may lead to what a link in a chapter may, or into itself.
/// What else it holds is the project's (G-13).
pub(crate) fn svg_problem(svg: &str) -> Option<String> {
    const SVG: &str = "http://www.w3.org/2000/svg";
    let options = roxmltree::ParsingOptions {
        allow_dtd: true,
        ..Default::default()
    };
    let document = match roxmltree::Document::parse_with_options(svg, options) {
        Ok(document) => document,
        Err(e) => return Some(format!("not well-formed: {e}")),
    };
    if !document.root_element().has_tag_name((SVG, "svg")) {
        return Some("no SVG".to_string());
    }
    const OUTSIDE: &str = "it refers to something outside itself";
    let outside = |value: &str| {
        let value = value.to_ascii_lowercase();
        value.contains("@import")
            || value.split("url(").skip(1).any(|url| {
                !url.trim_start_matches(['\'', '"', ' ']).starts_with('#')
                    && !url
                        .trim_start_matches(['\'', '"', ' '])
                        .starts_with("data:")
            })
    };
    for node in document.descendants() {
        if node.pi().is_some_and(|pi| pi.target == "xml-stylesheet") {
            return Some(OUTSIDE.to_string());
        }
        if !node.is_element() {
            continue;
        }
        let name = node.tag_name().name();
        if name == "script" {
            return Some("it holds a script".to_string());
        }
        if name == "foreignObject" {
            return Some(OUTSIDE.to_string());
        }
        let style_text = node
            .children()
            .filter(|child| name == "style" && child.is_text())
            .filter_map(|child| child.text())
            .any(&outside);
        if style_text {
            return Some(OUTSIDE.to_string());
        }
        for attribute in node.attributes() {
            let key = attribute.name();
            if key.len() >= 5 && key.starts_with("on") {
                return Some("it holds a script".to_string());
            }
            let value = attribute.value();
            let within = value.starts_with('#') || value.starts_with("data:");
            let refers = match key {
                "href" if name == "a" => !value.starts_with('#') && !is_url(value),
                "href" | "src" => !within,
                _ => outside(value),
            };
            if refers {
                return Some(OUTSIDE.to_string());
            }
        }
    }
    None
}

/// The bytes of an asset, from the downloads of the document or the project.
fn read(
    source: &Path,
    document: &Document,
    ctx: &RenderContext<'_>,
    output: &mut Output,
) -> Option<Vec<u8>> {
    if let Some(bytes) = document.downloads.get(source) {
        return Some(bytes.clone());
    }
    let bytes = ctx.files.read(source).ok()?;
    output.sources.insert(source.to_path_buf());
    Some(bytes)
}

/// Puts the document's images into `resources`, and returns source path to path in the publication,
/// and the images it refuses, with the reason. An asset that is not embedded stays an alt text,
/// which the writer reports.
fn embed_assets(
    document: &Document,
    ctx: &RenderContext<'_>,
    resources: &mut BTreeMap<PathBuf, Vec<u8>>,
    output: &mut Output,
) -> (BTreeMap<PathBuf, PathBuf>, BTreeMap<PathBuf, String>) {
    let mut assets = BTreeMap::new();
    let mut refused = BTreeMap::new();
    for source in document.assets.keys() {
        let Some(bytes) = read(source, document, ctx, output) else {
            continue;
        };
        if let Some(problem) = image_problem(source, &bytes) {
            refused.insert(source.clone(), problem);
            continue;
        }
        let Some(extension) = source.extension() else {
            continue;
        };
        let extension = extension.to_string_lossy();
        let target = PathBuf::from(format!("images/i{:04}.{extension}", assets.len()));
        resources.insert(target.clone(), bytes);
        assets.insert(source.clone(), target);
    }
    (assets, refused)
}

/// Puts the cover image into `resources`, and returns its path in the publication.
fn cover_image(
    settings: &EpubSettings,
    ctx: &RenderContext<'_>,
    resources: &mut BTreeMap<PathBuf, Vec<u8>>,
    output: &mut Output,
) -> Option<String> {
    let value = settings.cover_image.as_deref()?;
    let source = PathBuf::from(value);
    let origin = || Some(Origin::new("nectar.toml", 0));
    let bytes = match ctx.files.read(&source) {
        Ok(bytes) => bytes,
        Err(e) => {
            output.diagnostics.push(Diagnostic::warning(
                format!("cannot read {value}: {e}"),
                origin(),
            ));
            return None;
        }
    };
    output.sources.insert(source.clone());
    let extension = source.extension().map(|e| e.to_string_lossy().into_owned());
    let problem = image_problem(&source, &bytes);
    match (extension, problem) {
        (Some(extension), None) => {
            let target = format!("images/cover.{extension}");
            resources.insert(PathBuf::from(&target), bytes);
            Some(target)
        }
        (_, problem) => {
            let problem = problem.unwrap_or_else(|| "no core media type".to_string());
            output.diagnostics.push(Diagnostic::warning(
                format!("the cover image {value} is not embedded in the EPUB: {problem}"),
                origin(),
            ));
            None
        }
    }
}

/// The title of the document, or the stem of its start document where it has none XML can hold.
fn publication_title(document: &Document) -> String {
    let title = xml_text(&document.title);
    if title.trim().is_empty() {
        let stem = document.root.source.file_stem().unwrap_or_default();
        stem.to_string_lossy().into_owned()
    } else {
        title
    }
}

/// Text for a document in the container, escaped and without what XML does not allow.
fn xml(text: &str) -> String {
    escape(&xml_text(text))
}

fn xhtml(title: &str, language: &str, style: &str, body: &str) -> String {
    let (title, language) = (xml(title), xml(language));
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE html>\n\
         <html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\" \
         lang=\"{language}\" xml:lang=\"{language}\">\n\
         <head><title>{title}</title>\
         <link rel=\"stylesheet\" type=\"text/css\" href=\"{style}\" /></head>\n\
         <body>\n{body}</body>\n</html>\n"
    )
}

struct NavItem {
    level: u8,
    title: String,
    href: String,
}

/// What a walk over a chapter finds.
enum Found<'d> {
    Heading(&'d Heading),
    /// A chapter and its level in the tree of chapters.
    Chapter(&'d Chapter, u8),
}

/// Visits a chapter, its headings wherever they stand, and, with `nested`, the chapters below it.
fn visit_chapter<'d>(chapter: &'d Chapter, nested: bool, level: u8, f: &mut dyn FnMut(Found<'d>)) {
    f(Found::Chapter(chapter, level));
    visit_blocks(&chapter.blocks, nested, level, f);
}

fn visit_blocks<'d>(blocks: &'d [Block], nested: bool, level: u8, f: &mut dyn FnMut(Found<'d>)) {
    for block in blocks {
        match &block.kind {
            BlockKind::Heading(heading) => f(Found::Heading(heading)),
            BlockKind::Chapter(chapter) if nested => {
                visit_chapter(chapter, nested, level.saturating_add(1), f);
            }
            BlockKind::Quote(children) | BlockKind::Alert(_, children) => {
                visit_blocks(children, nested, level, f);
            }
            BlockKind::List(list) => {
                for item in &list.items {
                    visit_blocks(&item.blocks, nested, level, f);
                }
            }
            _ => {}
        }
    }
}

/// The body of the navigation document: every heading of the document; a chapter without a heading
/// by its title; and the document itself when nothing else is there.
fn navigation(title: &str, pages: &[ContentDocument<'_>]) -> String {
    let mut items = Vec::new();
    for page in pages {
        visit_chapter(
            page.chapter,
            page.nested,
            page.level(),
            &mut |found| match found {
                Found::Heading(heading) if id_fits_link(&heading.id) => {
                    items.push(NavItem {
                        level: heading.level,
                        title: heading_title(heading),
                        href: format!("{}#{}", page.file, heading.id),
                    });
                }
                Found::Heading(_) => {}
                Found::Chapter(chapter, level) => {
                    if level > 1 && !has_own_heading(chapter) {
                        let href = if level > page.level() {
                            format!("{}#{}", page.file, chapter.id)
                        } else {
                            page.file.clone()
                        };
                        items.push(NavItem {
                            level,
                            title: chapter.title.clone(),
                            href,
                        });
                    }
                }
            },
        );
    }
    items.retain(|item| !xml_text(&item.title).trim().is_empty());
    if items.is_empty() {
        items.push(NavItem {
            level: 1,
            title: title.to_string(),
            href: pages[0].file.clone(),
        });
    }
    let mut out = String::from("<nav epub:type=\"toc\" id=\"toc\">\n");
    let mut next = 0;
    nav_list(&items, &mut next, 1, &mut out);
    out.push_str("</nav>\n");
    out
}

fn nav_list(items: &[NavItem], next: &mut usize, level: u8, out: &mut String) {
    out.push_str("<ol>\n");
    while *next < items.len() && items[*next].level >= level {
        let item = &items[*next];
        *next += 1;
        let _ = write!(
            out,
            "<li><a href=\"{}\">{}</a>",
            xml(&item.href),
            xml(&item.title)
        );
        if *next < items.len() && items[*next].level > item.level {
            nav_list(items, next, item.level + 1, out);
        }
        out.push_str("</li>\n");
    }
    out.push_str("</ol>\n");
}

/// The package document's content.
struct Package<'a> {
    title: &'a str,
    language: &'a str,
    identifier: &'a str,
    modified: &'a str,
    pages: &'a [ContentDocument<'a>],
    nav_in_spine: bool,
    cover: Option<&'a str>,
}

impl Package<'_> {
    /// The package document, whose manifest lists `files`, the publication's files by their paths.
    fn opf(&self, files: &BTreeMap<String, Vec<u8>>) -> String {
        let mut manifest = String::new();
        let mut ids: HashMap<&str, String> = HashMap::new();
        for (index, name) in files.keys().enumerate() {
            let id = format!("item{index}");
            let media_type = if name.ends_with(".xhtml") {
                "application/xhtml+xml"
            } else if name.ends_with(".css") {
                "text/css"
            } else {
                image_media_type(Path::new(name)).expect("only core media types are embedded")
            };
            let properties = if name == NAV_FILE {
                " properties=\"nav\""
            } else if self.cover == Some(name.as_str()) {
                " properties=\"cover-image\""
            } else {
                ""
            };
            let _ = writeln!(
                manifest,
                "<item id=\"{id}\" href=\"{}\" media-type=\"{media_type}\"{properties}/>",
                escape(name)
            );
            ids.insert(name, id);
        }

        let mut spine = String::new();
        let mut itemref = |name: &str| {
            let _ = writeln!(spine, "<itemref idref=\"{}\"/>", ids[name]);
        };
        if self.cover.is_some() {
            itemref(COVER_FILE);
        }
        if self.nav_in_spine {
            itemref(NAV_FILE);
        }
        for page in self.pages {
            itemref(&page.file);
        }

        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" \
             unique-identifier=\"identifier\">\n\
             <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n\
             <dc:identifier id=\"identifier\">{}</dc:identifier>\n\
             <dc:title>{}</dc:title>\n\
             <dc:language>{}</dc:language>\n\
             <meta property=\"dcterms:modified\">{}</meta>\n\
             </metadata>\n<manifest>\n{manifest}</manifest>\n<spine>\n{spine}</spine>\n</package>\n",
            xml(self.identifier),
            xml(self.title),
            xml(self.language),
            xml(self.modified),
        )
    }
}
