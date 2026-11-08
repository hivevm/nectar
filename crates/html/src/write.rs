//! Writes blocks and inlines of the document model as HTML.

use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use nectar_core::model::{
    AlertKind, Align, Block, BlockKind, Chapter, Footnote, Graphic, HighlightedCode, Image, Inline,
    Table, Target, Token, TokenStyle,
};
use nectar_core::{Diagnostic, Origin};

use crate::relative_url;

/// Writes the content of one page.
pub(crate) struct PageWriter<'a> {
    pub out: String,
    /// Output file of the page, relative to the output root.
    pub page: &'a str,
    /// Heading and chapter ids to the output file that holds them.
    pub anchors: &'a HashMap<String, String>,
    pub assets: &'a BTreeMap<PathBuf, PathBuf>,
    /// Whether nested chapters are written into this page.
    pub nested_chapters: bool,
    /// Generated files, such as graphics, relative to the output root.
    pub files: &'a mut BTreeMap<PathBuf, Vec<u8>>,
    /// Footnotes referenced on this page.
    pub footnotes: BTreeSet<usize>,
    /// When set, a footnote is written where it is first referenced, as `<span class="footnote">`,
    /// for paged output that places it at the foot of the page.
    pub inline_notes: Option<&'a [Footnote]>,
    /// Whether headings end with a `#` link to themselves.
    pub heading_anchors: bool,
    /// Set for a content document of an EPUB: the writer writes well-formed XML, and leaves out and
    /// reports what such a document cannot carry.
    pub epub: Option<EpubNotes<'a>>,
}

/// What the ids of footnotes start with in an EPUB.
pub(crate) const FOOTNOTE_ID_PREFIX: &str = "_fn";

/// What the writer needs for an EPUB besides its usual fields, and what it reports.
pub(crate) struct EpubNotes<'a> {
    /// Chapter ids to the content document that holds them. An element carries such an id only
    /// when it is a heading's, which `anchors` holds.
    pub chapters: &'a HashMap<String, String>,
    /// Images that stay out of the container, with the reason.
    pub refused: &'a BTreeMap<PathBuf, String>,
    /// The origin of the block being written.
    pub origin: Origin,
    pub diagnostics: Vec<Diagnostic>,
    /// The ids written so far: an id must not repeat within a content document.
    pub ids: HashSet<String>,
    /// Inside a link, whose footnotes wait for its end: an `<a>` holds no `<a>`.
    pub in_link: bool,
    pub waiting_notes: Vec<usize>,
}

impl PageWriter<'_> {
    pub fn blocks(&mut self, blocks: &[Block], tight: bool) {
        for block in blocks {
            self.block(block, tight);
        }
    }

    fn block(&mut self, block: &Block, tight: bool) {
        let start = self.out.len();
        if let Some(notes) = &mut self.epub {
            notes.origin = block.origin.clone();
        }
        self.write_block(block, tight);
        if self.epub.is_some() && self.out[start..].chars().any(|c| !is_xml_char(c)) {
            let clean = xml_text(&self.out[start..]);
            self.out.truncate(start);
            self.out.push_str(&clean);
            if let Some(notes) = &mut self.epub {
                notes.origin = block.origin.clone();
            }
            self.left_out("a character XML does not allow is left out of the EPUB".to_string());
        }
    }

    fn write_block(&mut self, block: &Block, tight: bool) {
        match &block.kind {
            BlockKind::Chapter(chapter) => {
                if self.nested_chapters {
                    // In an EPUB, a link has to find a chapter without a heading, whose id no
                    // heading carries.
                    if let Some(notes) = self.epub.as_mut().filter(|_| !has_own_heading(chapter)) {
                        notes.ids.insert(chapter.id.clone());
                        let _ = writeln!(
                            self.out,
                            "<section class=\"chapter\" id=\"{}\">",
                            escape(&chapter.id)
                        );
                    } else {
                        self.out.push_str("<section class=\"chapter\">\n");
                    }
                    self.blocks(&chapter.blocks, false);
                    self.out.push_str("</section>\n");
                }
            }
            BlockKind::Heading(heading) => {
                let level = heading.level.clamp(1, 6);
                let id = escape(&heading.id);
                if self.keeps_id(&heading.id) {
                    let _ = write!(self.out, "<h{level} id=\"{id}\">");
                } else {
                    let _ = write!(self.out, "<h{level}>");
                }
                if !heading.number.is_empty() {
                    let _ = write!(
                        self.out,
                        "<span class=\"number\">{}</span> ",
                        escape(&heading.number)
                    );
                }
                self.inlines(&heading.content);
                if self.heading_anchors {
                    let _ = write!(
                        self.out,
                        "<a class=\"anchor\" href=\"#{id}\" aria-hidden=\"true\">#</a>"
                    );
                }
                let _ = writeln!(self.out, "</h{level}>");
            }
            BlockKind::Paragraph(inlines) if tight => {
                self.inlines(inlines);
                self.out.push('\n');
            }
            BlockKind::Paragraph(inlines) => {
                self.out.push_str("<p>");
                self.inlines(inlines);
                self.out.push_str("</p>\n");
            }
            BlockKind::Quote(blocks) => {
                self.out.push_str("<blockquote>\n");
                self.blocks(blocks, false);
                self.out.push_str("</blockquote>\n");
            }
            BlockKind::Alert(kind, blocks) => {
                let class = match kind {
                    AlertKind::Note => "note",
                    AlertKind::Success => "success",
                    AlertKind::Warning => "warning",
                    AlertKind::Error => "error",
                };
                let _ = writeln!(self.out, "<div class=\"alert alert-{class}\">");
                self.blocks(blocks, false);
                self.out.push_str("</div>\n");
            }
            BlockKind::List(list) => {
                if list.ordered {
                    if list.start == 1 {
                        self.out.push_str("<ol>\n");
                    } else {
                        let _ = writeln!(self.out, "<ol start=\"{}\">", list.start);
                    }
                } else {
                    self.out.push_str("<ul>\n");
                }
                for item in &list.items {
                    match item.task {
                        Some(checked) => {
                            let (disabled, on) = if self.epub.is_some() {
                                (" disabled=\"disabled\"", " checked=\"checked\"")
                            } else {
                                (" disabled", " checked")
                            };
                            let checked = if checked { on } else { "" };
                            let end = self.void_end();
                            let _ = write!(
                                self.out,
                                "<li class=\"task\"><input type=\"checkbox\"{disabled}{checked}{end} "
                            );
                        }
                        None => self.out.push_str("<li>"),
                    }
                    self.blocks(&item.blocks, list.tight);
                    self.out.push_str("</li>\n");
                }
                self.out
                    .push_str(if list.ordered { "</ol>\n" } else { "</ul>\n" });
            }
            BlockKind::Table(table) => self.table(table),
            BlockKind::ThematicBreak => {
                let _ = writeln!(self.out, "<hr{}", self.void_end());
            }
            BlockKind::Code(code) => {
                let class = if code.language.is_empty() {
                    String::new()
                } else {
                    format!(" class=\"language-{}\"", escape(&code.language))
                };
                let _ = writeln!(
                    self.out,
                    "<pre class=\"code\"><code{class}>{}</code></pre>",
                    escape(&code.code)
                );
            }
            BlockKind::HighlightedCode(code) => self.highlighted(code),
            BlockKind::Graphic(graphic) => self.graphic(graphic),
            BlockKind::Figure(caption, inner) => {
                self.out.push_str("<figure class=\"listing\">\n");
                self.block(inner, false);
                let _ = writeln!(
                    self.out,
                    "<figcaption>{}</figcaption></figure>",
                    escape(caption)
                );
            }
            BlockKind::Html(html) => self.raw_html(html),
        }
    }

    fn table(&mut self, table: &Table) {
        let total: u32 = table.columns.iter().map(|c| c.width).sum::<u32>().max(1);
        self.out
            .push_str("<div class=\"table\"><table>\n<colgroup>");
        for column in &table.columns {
            let percent = f64::from(column.width) * 100.0 / f64::from(total);
            let _ = write!(
                self.out,
                "<col style=\"width:{percent:.1}%\"{}",
                self.void_end()
            );
        }
        self.out.push_str("</colgroup>\n");
        let align = |index: usize| match table.columns.get(index).map(|c| c.align) {
            Some(Align::Left) => " style=\"text-align:left\"",
            Some(Align::Center) => " style=\"text-align:center\"",
            Some(Align::Right) => " style=\"text-align:right\"",
            _ => "",
        };
        if !table.header.is_empty() {
            self.out.push_str("<thead><tr>");
            for (index, cell) in table.header.iter().enumerate() {
                let _ = write!(self.out, "<th{}>", align(index));
                self.inlines(cell);
                self.out.push_str("</th>");
            }
            self.out.push_str("</tr></thead>\n");
        }
        self.out.push_str("<tbody>\n");
        for row in &table.rows {
            self.out.push_str("<tr>");
            for (index, cell) in row.iter().enumerate() {
                let _ = write!(self.out, "<td{}>", align(index));
                self.inlines(cell);
                self.out.push_str("</td>");
            }
            self.out.push_str("</tr>\n");
        }
        self.out.push_str("</tbody></table></div>\n");
    }

    /// Colours go in as custom properties: `--fg`, `--bg` for the block, `--c`, `--cb` for a
    /// token, and the same with a `d` for the dark theme. The stylesheet picks the pair for the
    /// colour scheme; a token without a colour inherits the block's.
    fn highlighted(&mut self, code: &HighlightedCode) {
        // A reading system may not know custom properties, and has one colour scheme of its own.
        if self.epub.is_some() {
            code_block(
                &mut self.out,
                &format!(
                    "class=\"code highlighted\" style=\"{}\"",
                    light_block_style(code)
                ),
                code,
                light_token_style,
            );
            return;
        }
        let mut style = String::new();
        let properties = [
            ("--fg", &code.foreground),
            ("--bg", &code.background),
            ("--dfg", &code.dark_foreground),
            ("--dbg", &code.dark_background),
        ];
        for (name, value) in properties {
            if let Some(value) = value {
                let _ = write!(style, "{name}:{value};");
            }
        }
        code_block(
            &mut self.out,
            &format!("class=\"code highlighted\" style=\"{style}\""),
            code,
            |token| token_style(token, code),
        );
    }

    /// Writes the picture to a file of its own, named by its content.
    fn graphic(&mut self, graphic: &Graphic) {
        if self.epub.is_some()
            && let Some(problem) = crate::epub::svg_problem(&graphic.svg)
        {
            self.left_out(format!(
                "the graphic is not embedded in the EPUB ({problem}); its code stays"
            ));
            let _ = writeln!(
                self.out,
                "<pre class=\"code\"><code>{}</code></pre>",
                escape(&graphic.code)
            );
            return;
        }
        let mut hasher = DefaultHasher::new();
        graphic.svg.hash(&mut hasher);
        let file = format!("_graphics/{:016x}.svg", hasher.finish());
        self.files
            .insert(PathBuf::from(&file), graphic.svg.as_bytes().to_vec());
        let _ = writeln!(
            self.out,
            "<figure class=\"graphic graphic-{}\"><img src=\"{}\" alt=\"{}\"{}</figure>",
            escape(&graphic.language),
            escape(&relative_url(self.page, &file)),
            escape(&graphic.alt),
            self.void_end()
        );
    }

    pub fn inlines(&mut self, inlines: &[Inline]) {
        for inline in inlines {
            self.inline(inline);
        }
    }

    fn inline(&mut self, inline: &Inline) {
        match inline {
            Inline::Text(text) => self.out.push_str(&escape(text)),
            Inline::Emphasis(children) => self.wrap("em", children),
            Inline::Strong(children) => self.wrap("strong", children),
            Inline::Highlight(children) => self.wrap("mark", children),
            Inline::Strikethrough(children) => self.wrap("del", children),
            Inline::Underline(children) => self.wrap("u", children),
            Inline::Code(code) => {
                let _ = write!(self.out, "<code>{}</code>", escape(code));
            }
            Inline::Link(link) => {
                if self.epub.is_some() && !self.has_target(&link.target) {
                    self.left_out(format!(
                        "the link to {} is left out of the EPUB; its text stays",
                        self.href(&link.target)
                    ));
                    self.inlines(&link.content);
                    if link.footnote > 0 {
                        self.footnote_ref(link.footnote);
                    }
                    return;
                }
                // `[][id]` shows nothing but its footnote.
                if !link.content.is_empty() || link.footnote == 0 {
                    let _ = write!(self.out, "<a href=\"{}\"", escape(&self.href(&link.target)));
                    if !link.title.is_empty() {
                        let _ = write!(self.out, " title=\"{}\"", escape(&link.title));
                    }
                    self.out.push('>');
                    if let Some(notes) = &mut self.epub {
                        notes.in_link = true;
                    }
                    self.inlines(&link.content);
                    self.out.push_str("</a>");
                    let waiting = match &mut self.epub {
                        Some(notes) => {
                            notes.in_link = false;
                            std::mem::take(&mut notes.waiting_notes)
                        }
                        None => Vec::new(),
                    };
                    for number in waiting {
                        self.footnote_ref(number);
                    }
                }
                if link.footnote > 0 {
                    self.footnote_ref(link.footnote);
                }
            }
            Inline::Image(image) => self.image(image),
            Inline::FootnoteRef { number, .. } => self.footnote_ref(*number),
            Inline::Symbol(name) if self.epub.is_some() => {
                self.left_out(format!("the symbol {name} is left out of the EPUB"));
            }
            Inline::Symbol(name) => {
                let _ = write!(
                    self.out,
                    "<span class=\"symbol\" aria-hidden=\"true\">{}</span>",
                    escape(name)
                );
            }
            Inline::SoftBreak => self.out.push('\n'),
            Inline::LineBreak => {
                let _ = writeln!(self.out, "<br{}", self.void_end());
            }
            Inline::Html(html) => self.raw_html(html),
        }
    }

    fn footnote_ref(&mut self, number: usize) {
        if self.epub.is_some() && number == 0 {
            self.left_out("a footnote inside a footnote is left out of the EPUB".to_string());
            return;
        }
        if let Some(notes) = &mut self.epub
            && notes.in_link
        {
            notes.waiting_notes.push(number);
            return;
        }
        if let Some(notes) = self.inline_notes {
            let first = self.footnotes.insert(number);
            match notes.iter().find(|note| note.number == number) {
                Some(note) if first => {
                    self.out.push_str("<span class=\"footnote\">");
                    self.blocks(&note.blocks, true);
                    self.out.push_str("</span>");
                }
                _ => {
                    let _ = write!(self.out, "<sup class=\"footnote-ref\">{number}</sup>");
                }
            }
            return;
        }
        let (note, back) = self.footnote_ids();
        let id = if self.footnotes.insert(number) {
            format!(" id=\"{back}{number}\"")
        } else {
            String::new()
        };
        let _ = write!(
            self.out,
            "<sup class=\"footnote-ref\"><a href=\"#{note}{number}\"{id}>{number}</a></sup>"
        );
    }

    fn wrap(&mut self, tag: &str, children: &[Inline]) {
        let _ = write!(self.out, "<{tag}>");
        self.inlines(children);
        let _ = write!(self.out, "</{tag}>");
    }

    fn image(&mut self, image: &Image) {
        if let Some(notes) = &self.epub
            && !matches!(&image.target, Target::Asset(path) if self.assets.contains_key(path))
        {
            let reason = match &image.target {
                Target::Asset(path) => notes.refused.get(path),
                _ => None,
            };
            let reason = reason.map(|r| format!(" ({r})")).unwrap_or_default();
            self.left_out(format!(
                "the image {} is not embedded in the EPUB{reason}; its alt text stays",
                self.href(&image.target)
            ));
            self.out.push_str(&escape(&image.alt));
            return;
        }
        let mut style = String::new();
        let mut class = String::new();
        for (name, value) in &image.attributes {
            match name.as_str() {
                "width" | "height"
                    if self.epub.is_some()
                        && !value
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '%')) => {}
                "width" | "height" => {
                    let unit = if value.chars().all(|c| c.is_ascii_digit()) {
                        "px"
                    } else {
                        ""
                    };
                    let _ = write!(style, "{name}:{value}{unit};");
                }
                "align" => class = format!(" class=\"align-{}\"", escape(value)),
                _ => {}
            }
        }
        let _ = write!(
            self.out,
            "<img src=\"{}\" alt=\"{}\"",
            escape(&self.href(&image.target)),
            escape(&image.alt)
        );
        if !image.title.is_empty() {
            let _ = write!(self.out, " title=\"{}\"", escape(&image.title));
        }
        if !style.is_empty() {
            let _ = write!(self.out, " style=\"{}\"", escape(&style));
        }
        let _ = write!(self.out, "{class}{}", self.void_end());
    }

    /// The end of a void element: an EPUB is XML.
    fn void_end(&self) -> &'static str {
        if self.epub.is_some() { " />" } else { ">" }
    }

    fn raw_html(&mut self, html: &str) {
        if self.epub.is_some() {
            self.left_out("raw HTML is left out of the EPUB".to_string());
        } else {
            self.out.push_str(html);
        }
    }

    fn left_out(&mut self, message: String) {
        if let Some(notes) = &mut self.epub {
            let origin = Some(notes.origin.clone());
            notes.diagnostics.push(Diagnostic::warning(message, origin));
        }
    }

    /// Whether a heading writes its id. In an EPUB, one must not be empty, as for a heading in a
    /// footnote, repeat within a content document, or take the prefix of a footnote's.
    fn keeps_id(&mut self, id: &str) -> bool {
        let Some(notes) = &mut self.epub else {
            return true;
        };
        if id.is_empty() {
            return false;
        }
        let holds = id_fits_link(id) && notes.ids.insert(id.to_string());
        if !holds {
            self.left_out(format!(
                "the id {id} of a heading is left out of the EPUB: it repeats another, \
                 takes a footnote's prefix, or cannot stand in a link"
            ));
        }
        holds
    }

    /// Whether a link leads somewhere in an EPUB: to a URL, to an id an element carries, or to a
    /// chapter, by its content document.
    fn has_target(&self, target: &Target) -> bool {
        match target {
            Target::External(url) => is_url(url),
            Target::Anchor(id) => self.anchors.contains_key(id) || self.chapter_file(id).is_some(),
            Target::Asset(_) | Target::File { .. } => false,
        }
    }

    /// The content document of a chapter, in an EPUB.
    fn chapter_file(&self, id: &str) -> Option<&str> {
        let notes = self.epub.as_ref()?;
        notes.chapters.get(id).map(String::as_str)
    }

    /// The prefixes of a footnote's id and of its reference's. In an EPUB, where a repeated id is
    /// an error, they differ from what a heading's id comes to from its text, and `keeps_id` keeps
    /// them apart from an id a writer gives a heading.
    fn footnote_ids(&self) -> (&'static str, &'static str) {
        if self.epub.is_some() {
            ("_fn-", "_fnref-")
        } else {
            ("fn-", "fnref-")
        }
    }

    fn href(&self, target: &Target) -> String {
        if let Target::Anchor(id) = target
            && !self.anchors.contains_key(id)
            && let Some(file) = self.chapter_file(id)
        {
            return relative_url(self.page, file);
        }
        match target {
            Target::External(url) => url.clone(),
            Target::Anchor(id) => match self.anchors.get(id) {
                Some(file) if file != self.page => {
                    format!("{}#{id}", relative_url(self.page, file))
                }
                _ => format!("#{id}"),
            },
            Target::Asset(path) => match self.assets.get(path) {
                Some(output) => relative_url(self.page, &output.to_string_lossy()),
                None => path.to_string_lossy().into_owned(),
            },
            Target::File { path, fragment } => {
                let mut href = path.to_string_lossy().into_owned();
                if let Some(fragment) = fragment {
                    href.push('#');
                    href.push_str(fragment);
                }
                href
            }
        }
    }

    /// Writes the footnotes referenced on this page.
    pub fn footnotes(&mut self, footnotes: &[Footnote]) {
        let numbers: Vec<usize> = self.footnotes.iter().copied().collect();
        if numbers.is_empty() {
            return;
        }
        self.out.push_str("<section class=\"footnotes\">\n<ol>\n");
        let (note, back) = self.footnote_ids();
        for footnote in footnotes.iter().filter(|f| numbers.contains(&f.number)) {
            let number = footnote.number;
            let _ = write!(self.out, "<li id=\"{note}{number}\" value=\"{number}\">");
            self.blocks(&footnote.blocks, true);
            let _ = writeln!(
                self.out,
                " <a class=\"footnote-back\" href=\"#{back}{number}\" aria-label=\"back\">↩</a></li>"
            );
        }
        self.out.push_str("</ol>\n</section>\n");
    }
}

fn token_style(token: &Token, code: &HighlightedCode) -> String {
    let style = &token.style;
    let mut css = String::new();
    match &token.dark {
        // Both colours, as soon as either theme gives the token one of its own.
        Some(dark) => {
            if style.color.is_some() || dark.color.is_some() {
                let light = style.color.as_ref().or(code.foreground.as_ref());
                let dark = dark.color.as_ref().or(code.dark_foreground.as_ref());
                if let (Some(light), Some(dark)) = (light, dark) {
                    let _ = write!(css, "--c:{light};--dc:{dark};");
                }
            }
            if style.background.is_some() || dark.background.is_some() {
                let light = style.background.as_ref().or(code.background.as_ref());
                let dark = dark.background.as_ref().or(code.dark_background.as_ref());
                if let (Some(light), Some(dark)) = (light, dark) {
                    let _ = write!(css, "--cb:{light};--dcb:{dark};");
                }
            }
        }
        None => {
            if let Some(color) = &style.color {
                let _ = write!(css, "--c:{color};");
            }
            if let Some(background) = &style.background {
                let _ = write!(css, "--cb:{background};");
            }
        }
    }
    font_style(&mut css, style);
    css
}

/// Highlighted code as a standalone `<pre>` with inline styles in the colours of the light
/// theme, for a page without the site's stylesheet (ADR-0017).
pub fn standalone_highlighted(code: &HighlightedCode) -> String {
    let mut out = String::new();
    code_block(
        &mut out,
        &format!("style=\"{}\"", light_block_style(code)),
        code,
        light_token_style,
    );
    out
}

/// The colours of a code block in the light theme, as plain properties.
fn light_block_style(code: &HighlightedCode) -> String {
    let mut style = String::new();
    if let Some(color) = &code.foreground {
        let _ = write!(style, "color:{color};");
    }
    if let Some(background) = &code.background {
        let _ = write!(style, "background:{background};");
    }
    style
}

/// The colours and the font of a token in the light theme, as plain properties.
fn light_token_style(token: &Token) -> String {
    let mut css = String::new();
    if let Some(color) = &token.style.color {
        let _ = write!(css, "color:{color};");
    }
    if let Some(background) = &token.style.background {
        let _ = write!(css, "background:{background};");
    }
    font_style(&mut css, &token.style);
    css
}

/// Writes `code` as `<pre ATTRIBUTES><code>`, a span around each token `style_of` gives a style.
fn code_block(
    out: &mut String,
    attributes: &str,
    code: &HighlightedCode,
    style_of: impl Fn(&Token) -> String,
) {
    let _ = write!(
        out,
        "<pre {attributes}><code class=\"language-{}\">",
        escape(&code.language)
    );
    for (index, line) in code.lines.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        for token in line {
            let style = style_of(token);
            if style.is_empty() {
                out.push_str(&escape(&token.text));
            } else {
                let _ = write!(
                    out,
                    "<span style=\"{style}\">{}</span>",
                    escape(&token.text)
                );
            }
        }
    }
    out.push_str("</code></pre>\n");
}

/// The weight, slant and decoration of a token, the same in every theme.
fn font_style(css: &mut String, style: &TokenStyle) {
    if style.bold {
        css.push_str("font-weight:bold;");
    }
    if style.italic {
        css.push_str("font-style:italic;");
    }
    match (style.underline, style.strikethrough) {
        (true, true) => css.push_str("text-decoration:underline line-through;"),
        (true, false) => css.push_str("text-decoration:underline;"),
        (false, true) => css.push_str("text-decoration:line-through;"),
        (false, false) => {}
    }
}

/// Whether XML 1.0 allows the character.
fn is_xml_char(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r' | ' '..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..)
}

/// The text without the characters XML 1.0 does not allow.
pub(crate) fn xml_text(text: &str) -> String {
    text.chars().filter(|c| is_xml_char(*c)).collect()
}

/// Whether an EPUB can name an id in a link: a `%` would be read as an escape, and the prefix of a
/// footnote's id is taken. Other characters a writer gives an id are kept out with them.
pub(crate) fn id_fits_link(id: &str) -> bool {
    !id.starts_with(FOOTNOTE_ID_PREFIX)
        && id
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Whether a chapter has a heading among its own blocks, whose id is then the chapter's.
pub(crate) fn has_own_heading(chapter: &Chapter) -> bool {
    chapter
        .blocks
        .iter()
        .any(|block| matches!(block.kind, BlockKind::Heading(_)))
}

/// Whether a link's target is a URL an EPUB may name: of `http`, `https` or `mailto`, and of only
/// what RFC 3986 allows unquoted. An allow-list, since the core also keeps paths outside the
/// project as external targets, and a reading system need not follow any other scheme.
pub(crate) fn is_url(url: &str) -> bool {
    let holds = |part: &str, extra: &str| {
        let escapes = part
            .split('%')
            .skip(1)
            .all(|rest| rest.chars().take(2).filter(char::is_ascii_hexdigit).count() == 2);
        escapes
            && part.chars().all(|c| {
                c.is_alphanumeric() || "-._~!$&'()*+,;=:@%".contains(c) || extra.contains(c)
            })
    };
    if let Some(address) = url.strip_prefix("mailto:") {
        return address.contains('@') && holds(address, "?/");
    }
    let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
    else {
        return false;
    };
    let (rest, fragment) = rest.split_once('#').unwrap_or((rest, ""));
    let (rest, query) = rest.split_once('?').unwrap_or((rest, ""));
    let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
    let (host, port) = authority.split_once(':').unwrap_or((authority, ""));
    // A last label of digits or hex digits would make the host an address, to be held to its form.
    let labels: Vec<&str> = host.strip_suffix('.').unwrap_or(host).split('.').collect();
    let address = labels.last().is_some_and(|last| {
        last.chars().all(|c| c.is_ascii_digit()) || last.to_ascii_lowercase().starts_with("0x")
    });
    labels.iter().all(|label| {
        (1..=63).contains(&label.len())
            && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    }) && !address
        && (port.is_empty() || port.parse::<u16>().is_ok())
        && holds(path, "/")
        && holds(query, "/?")
        && holds(fragment, "/?")
}

pub(crate) fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(c),
        }
    }
    escaped
}
