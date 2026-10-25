//! Writes blocks and inlines of the document model as HTML.

use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use nectar_core::model::{
    AlertKind, Align, Block, BlockKind, Footnote, Graphic, HighlightedCode, Image, Inline, Table,
    Target, Token,
};

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
}

impl PageWriter<'_> {
    pub fn blocks(&mut self, blocks: &[Block], tight: bool) {
        for block in blocks {
            self.block(block, tight);
        }
    }

    fn block(&mut self, block: &Block, tight: bool) {
        match &block.kind {
            BlockKind::Chapter(chapter) => {
                if self.nested_chapters {
                    self.out.push_str("<section class=\"chapter\">\n");
                    self.blocks(&chapter.blocks, false);
                    self.out.push_str("</section>\n");
                }
            }
            BlockKind::Heading(heading) => {
                let level = heading.level.clamp(1, 6);
                let id = escape(&heading.id);
                let _ = write!(self.out, "<h{level} id=\"{id}\">");
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
                            let checked = if checked { " checked" } else { "" };
                            let _ = write!(
                                self.out,
                                "<li class=\"task\"><input type=\"checkbox\" disabled{checked}> "
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
            BlockKind::ThematicBreak => self.out.push_str("<hr>\n"),
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
            BlockKind::Html(html) => self.out.push_str(html),
        }
    }

    fn table(&mut self, table: &Table) {
        let total: u32 = table.columns.iter().map(|c| c.width).sum::<u32>().max(1);
        self.out
            .push_str("<div class=\"table\"><table>\n<colgroup>");
        for column in &table.columns {
            let percent = f64::from(column.width) * 100.0 / f64::from(total);
            let _ = write!(self.out, "<col style=\"width:{percent:.1}%\">");
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
        let _ = write!(
            self.out,
            "<pre class=\"code highlighted\" style=\"{style}\"><code class=\"language-{}\">",
            escape(&code.language)
        );
        for (index, line) in code.lines.iter().enumerate() {
            if index > 0 {
                self.out.push('\n');
            }
            for token in line {
                let style = token_style(token, code);
                if style.is_empty() {
                    self.out.push_str(&escape(&token.text));
                } else {
                    let _ = write!(
                        self.out,
                        "<span style=\"{style}\">{}</span>",
                        escape(&token.text)
                    );
                }
            }
        }
        self.out.push_str("</code></pre>\n");
    }

    /// Writes the picture to a file of its own, named by its content.
    fn graphic(&mut self, graphic: &Graphic) {
        let mut hasher = DefaultHasher::new();
        graphic.svg.hash(&mut hasher);
        let file = format!("_graphics/{:016x}.svg", hasher.finish());
        self.files
            .insert(PathBuf::from(&file), graphic.svg.as_bytes().to_vec());
        let _ = writeln!(
            self.out,
            "<figure class=\"graphic graphic-{}\"><img src=\"{}\" alt=\"{}\"></figure>",
            escape(&graphic.language),
            escape(&relative_url(self.page, &file)),
            escape(&graphic.alt)
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
                // `[][id]` shows nothing but its footnote.
                if !link.content.is_empty() || link.footnote == 0 {
                    let _ = write!(self.out, "<a href=\"{}\"", escape(&self.href(&link.target)));
                    if !link.title.is_empty() {
                        let _ = write!(self.out, " title=\"{}\"", escape(&link.title));
                    }
                    self.out.push('>');
                    self.inlines(&link.content);
                    self.out.push_str("</a>");
                }
                if link.footnote > 0 {
                    self.footnote_ref(link.footnote);
                }
            }
            Inline::Image(image) => self.image(image),
            Inline::FootnoteRef { number, .. } => self.footnote_ref(*number),
            Inline::Symbol(name) => {
                let _ = write!(
                    self.out,
                    "<span class=\"symbol\" aria-hidden=\"true\">{}</span>",
                    escape(name)
                );
            }
            Inline::SoftBreak => self.out.push('\n'),
            Inline::LineBreak => self.out.push_str("<br>\n"),
            Inline::Html(html) => self.out.push_str(html),
        }
    }

    fn footnote_ref(&mut self, number: usize) {
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
        let id = if self.footnotes.insert(number) {
            format!(" id=\"fnref-{number}\"")
        } else {
            String::new()
        };
        let _ = write!(
            self.out,
            "<sup class=\"footnote-ref\"><a href=\"#fn-{number}\"{id}>{number}</a></sup>"
        );
    }

    fn wrap(&mut self, tag: &str, children: &[Inline]) {
        let _ = write!(self.out, "<{tag}>");
        self.inlines(children);
        let _ = write!(self.out, "</{tag}>");
    }

    fn image(&mut self, image: &Image) {
        let mut style = String::new();
        let mut class = String::new();
        for (name, value) in &image.attributes {
            match name.as_str() {
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
        let _ = write!(self.out, "{class}>");
    }

    fn href(&self, target: &Target) -> String {
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
        for footnote in footnotes.iter().filter(|f| numbers.contains(&f.number)) {
            let number = footnote.number;
            let _ = write!(self.out, "<li id=\"fn-{number}\" value=\"{number}\">");
            self.blocks(&footnote.blocks, true);
            let _ = writeln!(
                self.out,
                " <a class=\"footnote-back\" href=\"#fnref-{number}\" aria-label=\"back\">↩</a></li>"
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
    css
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
