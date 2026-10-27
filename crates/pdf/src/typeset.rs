//! Translates the document model into Typst source.
//!
//! Text always goes in as Typst strings, `#"…";`, so no character of the document can be read
//! as Typst markup. Every embedded expression ends with `;`.

use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use nectar_core::model::{
    AlertKind, Align, Block, BlockKind, Chapter, Document, Footnote, Graphic, HighlightedCode,
    Image, Inline, Table, Target, TokenStyle,
};
use nectar_core::{Diagnostic, Origin, PrintSettings, pdf_file};

const BRAND: &str = "#f2b408";
const LINK: &str = "#946300";

/// The page geometry, translated from the CSS values of the settings.
pub(crate) struct Page {
    /// `paper: "a4"` or `width: …, height: …`, with `flipped` when needed.
    pub size: String,
    pub margin: String,
}

impl Page {
    pub fn from_settings(settings: &PrintSettings) -> Result<Self, String> {
        Ok(Self {
            size: paper(&settings.size)?,
            margin: margin(&settings.margin)?,
        })
    }
}

/// `A4`, `letter`, `A5 landscape` or two lengths.
fn paper(size: &str) -> Result<String, String> {
    let words: Vec<String> = size.split_whitespace().map(str::to_lowercase).collect();
    let landscape = words.iter().any(|w| w == "landscape");
    let rest: Vec<&str> = words
        .iter()
        .map(String::as_str)
        .filter(|w| *w != "landscape" && *w != "portrait")
        .collect();
    let base = match rest.as_slice() {
        [name] => {
            let paper = match *name {
                "letter" => "us-letter",
                "legal" => "us-legal",
                "ledger" => "us-tabloid",
                name if is_iso_paper(name) => name,
                _ => return Err(format!("unknown page size '{size}'")),
            };
            format!("paper: \"{paper}\"")
        }
        [width, height] if is_length(width) && is_length(height) => {
            format!("width: {width}, height: {height}")
        }
        _ => return Err(format!("unknown page size '{size}'")),
    };
    Ok(if landscape {
        format!("{base}, flipped: true")
    } else {
        base
    })
}

fn is_iso_paper(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some('a' | 'b' | 'c'))
        && chars.as_str().parse::<u8>().is_ok_and(|n| n <= 10)
}

/// CSS margin shorthand with one to four lengths.
fn margin(value: &str) -> Result<String, String> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    if parts.is_empty() || !parts.iter().all(|p| is_length(p)) {
        return Err(format!("unknown margin '{value}'"));
    }
    let (top, right, bottom, left) = match parts.as_slice() {
        [all] => (*all, *all, *all, *all),
        [y, x] => (*y, *x, *y, *x),
        [top, x, bottom] => (*top, *x, *bottom, *x),
        [top, right, bottom, left] => (*top, *right, *bottom, *left),
        _ => return Err(format!("unknown margin '{value}'")),
    };
    Ok(format!(
        "(top: {top}, right: {right}, bottom: {bottom}, left: {left})"
    ))
}

/// A length Typst and CSS agree on.
fn is_length(value: &str) -> bool {
    let number = value.trim_end_matches(char::is_alphabetic);
    let unit = &value[number.len()..];
    matches!(unit, "mm" | "cm" | "in" | "pt") && number.parse::<f64>().is_ok_and(|n| n >= 0.0)
}

/// A Typst string literal.
pub(crate) fn string(text: &str) -> String {
    let mut literal = String::with_capacity(text.len() + 2);
    literal.push('"');
    for c in text.chars() {
        match c {
            '"' => literal.push_str("\\\""),
            '\\' => literal.push_str("\\\\"),
            '\n' | '\r' | '\t' => literal.push(' '),
            c => literal.push(c),
        }
    }
    literal.push('"');
    literal
}

pub(crate) struct Typesetter<'a> {
    pub out: String,
    document: &'a Document,
    settings: &'a PrintSettings,
    /// Heading and chapter ids to Typst labels.
    labels: HashMap<String, String>,
    footnotes_written: HashSet<usize>,
    /// Files the source refers to, by absolute virtual path.
    pub files: HashMap<String, Vec<u8>>,
    /// Project files to load, by absolute virtual path.
    pub wanted: BTreeMap<String, PathBuf>,
    /// What the PDF leaves out.
    pub diagnostics: Vec<Diagnostic>,
    /// Origin of the block being written.
    origin: Origin,
}

impl<'a> Typesetter<'a> {
    pub fn new(document: &'a Document, settings: &'a PrintSettings) -> Self {
        let mut labels = HashMap::new();
        collect_labels(&document.root, &mut labels);
        Self {
            out: String::new(),
            document,
            settings,
            labels,
            footnotes_written: HashSet::new(),
            files: HashMap::new(),
            wanted: BTreeMap::new(),
            diagnostics: Vec::new(),
            origin: Origin::default(),
        }
    }

    /// Writes the whole source: page setup, cover, contents and the document.
    pub fn document(
        &mut self,
        site_title: &str,
        page: &Page,
        cover_image: Option<&str>,
        watermark: Option<&str>,
    ) {
        let title = string(&self.document.title);
        let _ = write!(
            self.out,
            r##"#set document(title: {title})
#set text(font: "Ubuntu Sans", size: 10.5pt)
#set par(leading: 0.7em, spacing: 1.1em)
#set heading(numbering: none)
#show heading: set block(above: 1.6em, below: 0.8em)
#show heading.where(level: 1): set text(size: 22pt)
#show heading.where(level: 2): set text(size: 17pt)
#show heading.where(level: 3): set text(size: 13pt)
#show heading.where(level: 4): set text(size: 11.5pt)
#show link: set text(fill: rgb("{LINK}"))
#show raw: set text(font: "Victor Mono")
#show table.cell.where(y: 0): strong
#show outline.entry.where(level: 1): set text(weight: "bold")
#set footnote.entry(separator: line(length: 30%, stroke: 0.5pt + luma(200)))
#set page({size}, margin: {margin},
  header: context {{
    // The chapter that starts on this page, or else the last one before it.
    let chapters = selector(heading.where(level: 1)).or(heading.where(level: 2))
    let page = here().page()
    let on-page = query(chapters).filter(h => h.location().page() == page)
    let before = query(chapters.before(here()))
    let current = if on-page.len() > 0 {{ on-page.first() }} else if before.len() > 0 {{ before.last() }}
    set text(size: 8pt, fill: luma(110))
    grid(columns: (1fr, 1fr), align(left, {title}), align(right, if current != none {{ current.body }}))
  }},
  footer: context align(center, text(size: 9pt, fill: luma(110), counter(page).display())),
)
"##,
            size = page.size,
            margin = page.margin,
        );

        if self.settings.cover {
            self.out.push_str(
                "#page(header: none, footer: none, margin: (x: 25mm, top: 70mm, bottom: 25mm))[\n",
            );
            if let Some(image) = cover_image {
                let _ = writeln!(
                    self.out,
                    "#image({}, height: 60mm);\n#v(12mm);",
                    string(image)
                );
            }
            let _ = writeln!(
                self.out,
                "#text(size: 11pt, fill: luma(110), tracking: 0.08em, upper({}));\n#v(4mm);\n#line(length: 100%, stroke: 2.5mm + rgb(\"{BRAND}\"));\n#v(6mm);\n#text(size: 34pt, weight: \"bold\", {title});",
                string(site_title)
            );
            if let Some(subtitle) = &self.settings.subtitle {
                let _ = writeln!(
                    self.out,
                    "#v(6mm);\n#text(size: 15pt, {});",
                    string(subtitle)
                );
            }
            self.out.push_str("]\n");
        }
        if let Some(image) = watermark {
            let _ = writeln!(
                self.out,
                "#set page(background: align(center + horizon, image({}, width: 55%)))",
                string(image)
            );
        }
        if self.document.contents {
            self.out.push_str(
                "#outline(title: [Contents], target: selector(heading.where(level: 2)).or(heading.where(level: 3)))\n#pagebreak(weak: true)\n",
            );
        }
        self.out.push('\n');
        let blocks = &self.document.root.blocks;
        self.blocks(blocks, true);
    }

    fn blocks(&mut self, blocks: &[Block], top: bool) {
        for block in blocks {
            self.block(block, top);
            self.out.push_str("\n\n");
        }
    }

    fn block(&mut self, block: &Block, top: bool) {
        self.origin = block.origin.clone();
        match &block.kind {
            BlockKind::Chapter(chapter) => self.chapter(chapter, top),
            BlockKind::Heading(heading) => {
                let level = heading.level.clamp(1, 6);
                let _ = write!(self.out, "#heading(level: {level})[");
                if !heading.number.is_empty() {
                    let _ = write!(
                        self.out,
                        "#text(fill: luma(120), {});#\" \";",
                        string(&heading.number)
                    );
                }
                self.inlines(&heading.content);
                self.out.push(']');
                if let Some(label) = self.labels.get(&heading.id) {
                    let _ = write!(self.out, " <{label}>");
                }
            }
            BlockKind::Paragraph(inlines) => {
                if let [Inline::Image(image)] = inlines.as_slice() {
                    self.image(image, true);
                } else {
                    self.inlines(inlines);
                }
            }
            BlockKind::Quote(blocks) => {
                self.out.push_str(
                    "#block(inset: (left: 10pt, y: 2pt), stroke: (left: 2pt + luma(200)))[\n",
                );
                self.blocks(blocks, false);
                self.out.push_str("];");
            }
            BlockKind::Alert(kind, blocks) => {
                let color = match kind {
                    AlertKind::Note => "#1e66f5",
                    AlertKind::Success => "#2f7d1f",
                    AlertKind::Warning => "#f2a408",
                    AlertKind::Error => "#d20f39",
                };
                let _ = writeln!(
                    self.out,
                    "#block(width: 100%, inset: 8pt, radius: 2pt, fill: rgb(\"{color}\").lighten(88%), stroke: (left: 3pt + rgb(\"{color}\")))["
                );
                self.blocks(blocks, false);
                self.out.push_str("];");
            }
            BlockKind::List(list) => {
                let function = if list.ordered { "enum" } else { "list" };
                let _ = write!(self.out, "#{function}(tight: {}", list.tight);
                if list.ordered && list.start != 1 {
                    let _ = write!(self.out, ", start: {}", list.start);
                }
                for item in &list.items {
                    self.out.push_str(", [");
                    if let Some(checked) = item.task {
                        let symbol = if checked {
                            "check_box"
                        } else {
                            "check_box_outline_blank"
                        };
                        let _ = write!(
                            self.out,
                            "#box(baseline: 15%, text(font: \"Material Icons\", {}));#\" \";",
                            string(symbol)
                        );
                    }
                    self.blocks(&item.blocks, false);
                    self.out.push(']');
                }
                self.out.push_str(");");
            }
            BlockKind::Table(table) => self.table(table),
            // A thematic break starts a new page in print, as the documents written for the
            // former Gradle plugin expect; the website shows a rule.
            BlockKind::ThematicBreak => self.out.push_str("#pagebreak(weak: true)\n"),
            BlockKind::Code(code) => {
                let _ = write!(
                    self.out,
                    "#block(width: 100%, inset: 8pt, radius: 3pt, fill: luma(248), stroke: 0.5pt + luma(225), raw(block: true, {}));",
                    string_lines(&code.code)
                );
            }
            BlockKind::HighlightedCode(code) => self.highlighted(code),
            BlockKind::Graphic(graphic) => self.graphic(graphic),
            BlockKind::Figure(caption, inner) => {
                self.out.push_str("#figure(numbering: none, caption: [");
                let _ = write!(self.out, "#{};", string(caption));
                self.out.push_str("])[#set align(left)\n");
                self.block(inner, false);
                self.out.push_str("\n];");
            }
            BlockKind::Html(_) => self.left_out("raw HTML is left out of the PDF"),
        }
    }

    /// A chapter of the document starts on a new page, after its title page if wanted.
    fn chapter(&mut self, chapter: &Chapter, top: bool) {
        if top {
            self.out.push_str("#pagebreak(weak: true)\n");
            if self.settings.chapter_pages {
                let number = chapter.blocks.iter().find_map(|block| match &block.kind {
                    BlockKind::Heading(heading) => Some(heading.number.clone()),
                    _ => None,
                });
                let _ = writeln!(
                    self.out,
                    "#page(header: none, footer: none)[#v(25%)\n#text(size: 96pt, weight: \"bold\", fill: rgb(\"{BRAND}\"), {});\n\n#text(size: 26pt, weight: \"bold\", {});\n]",
                    string(&number.unwrap_or_default()),
                    string(&chapter.title)
                );
            }
        }
        let first_heading = chapter.blocks.iter().find_map(|block| match &block.kind {
            BlockKind::Heading(heading) => Some(heading.id.as_str()),
            _ => None,
        });
        if first_heading != Some(chapter.id.as_str())
            && let Some(label) = self.labels.get(&chapter.id)
        {
            let _ = writeln!(self.out, "#metadata(none) <{label}>");
        }
        self.blocks(&chapter.blocks, false);
    }

    fn table(&mut self, table: &Table) {
        let columns: Vec<String> = table
            .columns
            .iter()
            .map(|c| format!("{}fr", c.width.max(1)))
            .collect();
        let aligns: Vec<&str> = table
            .columns
            .iter()
            .map(|c| match c.align {
                Align::Center => "center",
                Align::Right => "right",
                Align::Left | Align::Auto => "left",
            })
            .collect();
        let _ = write!(
            self.out,
            "#table(columns: ({},), align: ({},), stroke: 0.5pt + luma(200), inset: 5pt",
            columns.join(", "),
            aligns.join(", ")
        );
        if !table.header.is_empty() {
            self.out.push_str(", table.header(");
            for (index, cell) in table.header.iter().enumerate() {
                if index > 0 {
                    self.out.push_str(", ");
                }
                self.out.push('[');
                self.inlines(cell);
                self.out.push(']');
            }
            self.out.push(')');
        }
        for row in &table.rows {
            for index in 0..table.columns.len() {
                self.out.push_str(", [");
                if let Some(cell) = row.get(index) {
                    self.inlines(cell);
                }
                self.out.push(']');
            }
        }
        self.out.push_str(");");
    }

    /// Code with the colours of the highlighting engine. Spaces become non-breaking spaces, so
    /// indentation survives.
    fn highlighted(&mut self, code: &HighlightedCode) {
        let fill = code
            .background
            .as_deref()
            .map_or("luma(248)".to_string(), color);
        let text_color = code
            .foreground
            .as_deref()
            .map_or("luma(30)".to_string(), color);
        // The tokens stand in for a raw element of the plain code, so the tagged PDF marks the
        // block as code while it keeps the engine's colours. The sizes are set inside the rule,
        // since raw scales whatever stands in for it to 0.8em.
        let plain: Vec<String> = code
            .lines
            .iter()
            .map(|line| line.iter().map(|token| token.text.as_str()).collect())
            .collect();
        let _ = writeln!(
            self.out,
            "#block(width: 100%, inset: 8pt, radius: 3pt, fill: {fill}, stroke: 0.5pt + luma(225), breakable: true)[#show raw: _ => [#set text(font: \"Victor Mono\", size: 8.5pt, fill: {text_color}, hyphenate: false); #set par(justify: false, leading: 0.55em); "
        );
        for (index, line) in code.lines.iter().enumerate() {
            if index > 0 {
                self.out.push_str("#linebreak();");
            }
            for token in line {
                let text = string(&token.text.replace(' ', "\u{a0}"));
                let style = token_style(&token.style);
                if style.is_empty() {
                    let _ = write!(self.out, "#{text};");
                } else {
                    let _ = write!(self.out, "#text({style}{text});");
                }
            }
        }
        let _ = write!(
            self.out,
            "\n]; #raw(block: true, {})\n];",
            string_lines(&plain.join("\n"))
        );
    }

    /// A diagram: as wide as it is, but never wider than the column.
    fn graphic(&mut self, graphic: &Graphic) {
        let mut hasher = DefaultHasher::new();
        graphic.svg.hash(&mut hasher);
        let path = format!("/_graphics/{:016x}.svg", hasher.finish());
        self.files
            .insert(path.clone(), graphic.svg.as_bytes().to_vec());
        let path = string(&path);
        let _ = write!(
            self.out,
            "#align(center, layout(size => {{ let picture = image({path}); if measure(picture).width > size.width {{ image({path}, width: 100%) }} else {{ picture }} }}));"
        );
    }

    fn image(&mut self, image: &Image, block: bool) {
        let path = match &image.target {
            Target::Asset(path) if self.document.assets.contains_key(path) => virtual_path(path),
            Target::External(url) => {
                let shown: String = url.chars().take(80).collect();
                self.left_out(&format!("the image {shown} is not embedded in the PDF"));
                return self.text(&image.alt);
            }
            _ => return self.text(&image.alt),
        };
        self.wanted
            .insert(path.clone(), image_source(&image.target));
        let mut arguments = string(&path);
        let mut align = "left";
        for (name, value) in &image.attributes {
            match name.as_str() {
                "width" | "height" => {
                    if let Some(length) = image_length(value) {
                        let _ = write!(arguments, ", {name}: {length}");
                    }
                }
                "align" if value == "center" || value == "right" => align = value,
                _ => {}
            }
        }
        if block {
            let _ = write!(self.out, "#align({align}, image({arguments}));");
        } else {
            let _ = write!(self.out, "#box(image({arguments}));");
        }
    }

    fn inlines(&mut self, inlines: &[Inline]) {
        for inline in inlines {
            self.inline(inline);
        }
    }

    fn inline(&mut self, inline: &Inline) {
        match inline {
            Inline::Text(text) => self.text(text),
            Inline::Emphasis(children) => self.wrap("emph", children),
            Inline::Strong(children) => self.wrap("strong", children),
            Inline::Highlight(children) => self.wrap("highlight", children),
            Inline::Strikethrough(children) => self.wrap("strike", children),
            Inline::Underline(children) => self.wrap("underline", children),
            Inline::Code(code) => {
                let _ = write!(self.out, "#raw({});", string(code));
            }
            Inline::Link(link) => {
                let destination = match &link.target {
                    Target::External(url) => Some(string(url)),
                    Target::Anchor(id) => self.labels.get(id).map(|label| format!("<{label}>")),
                    // A local file, where it lies in the output relative to the PDF.
                    Target::Asset(path) => self.document.assets.get(path).map(|output| {
                        string(&relative_url(
                            &pdf_file(self.document),
                            &output.to_string_lossy(),
                        ))
                    }),
                    Target::File { .. } => None,
                };
                match destination {
                    Some(destination) if !link.content.is_empty() => {
                        let _ = write!(self.out, "#link({destination})[");
                        self.inlines(&link.content);
                        self.out.push_str("];");
                    }
                    _ => self.inlines(&link.content),
                }
                if link.footnote > 0 {
                    self.footnote(link.footnote);
                }
            }
            Inline::Image(image) => self.image(image, false),
            Inline::FootnoteRef { number, .. } => self.footnote(*number),
            Inline::Symbol(name) => {
                let _ = write!(
                    self.out,
                    "#box(baseline: 15%, text(font: \"Material Icons\", {}));",
                    string(name)
                );
            }
            Inline::SoftBreak => self.out.push_str("#\" \";"),
            Inline::LineBreak => self.out.push_str("#linebreak();"),
            Inline::Html(_) => self.left_out("raw HTML is left out of the PDF"),
        }
    }

    /// A footnote at the foot of the page where it is first referenced; later references repeat
    /// its number.
    fn footnote(&mut self, number: usize) {
        let document = self.document;
        let note = document.footnotes.iter().find(|note| note.number == number);
        match note {
            Some(note) if self.footnotes_written.insert(number) => {
                self.out.push_str("#footnote[");
                self.note(note);
                self.out.push_str("];");
            }
            _ => {
                let _ = write!(self.out, "#super({});", string(&number.to_string()));
            }
        }
    }

    fn note(&mut self, note: &Footnote) {
        for (index, block) in note.blocks.iter().enumerate() {
            if index > 0 {
                self.out.push_str("#\" \";");
            }
            match &block.kind {
                BlockKind::Paragraph(inlines) => self.inlines(inlines),
                _ => self.block(block, false),
            }
        }
    }

    fn left_out(&mut self, message: &str) {
        self.diagnostics
            .push(Diagnostic::warning(message, Some(self.origin.clone())));
    }

    fn wrap(&mut self, function: &str, children: &[Inline]) {
        let _ = write!(self.out, "#{function}[");
        self.inlines(children);
        self.out.push_str("];");
    }

    fn text(&mut self, text: &str) {
        if !text.is_empty() {
            let _ = write!(self.out, "#{};", string(text));
        }
    }
}

/// Labels for every heading and chapter id, valid Typst label names.
fn collect_labels(chapter: &Chapter, labels: &mut HashMap<String, String>) {
    let add = |id: &str, labels: &mut HashMap<String, String>| {
        if !id.is_empty() && !labels.contains_key(id) {
            let label = format!("nectar-{}", labels.len());
            labels.insert(id.to_string(), label);
        }
    };
    add(&chapter.id, labels);
    for block in &chapter.blocks {
        match &block.kind {
            BlockKind::Heading(heading) => add(&heading.id, labels),
            BlockKind::Chapter(child) => collect_labels(child, labels),
            _ => {}
        }
    }
}

/// The URL of output file `to`, relative to output file `from`. Both use `/`.
fn relative_url(from: &str, to: &str) -> String {
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

/// The absolute virtual path of a project file.
pub(crate) fn virtual_path(path: &Path) -> String {
    let parts: Vec<String> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    format!("/{}", parts.join("/"))
}

fn image_source(target: &Target) -> PathBuf {
    match target {
        Target::Asset(path) => path.clone(),
        _ => PathBuf::new(),
    }
}

/// Image sizes: numbers are CSS pixels, otherwise lengths and percentages.
fn image_length(value: &str) -> Option<String> {
    if let Ok(pixels) = value.parse::<f64>() {
        return Some(format!("{}pt", pixels * 0.75));
    }
    if let Some(percent) = value.strip_suffix('%') {
        return percent.parse::<f64>().ok().map(|p| format!("{p}%"));
    }
    is_length(value).then(|| value.to_string())
}

fn color(value: &str) -> String {
    format!("rgb({})", string(value))
}

fn token_style(style: &TokenStyle) -> String {
    let mut arguments = String::new();
    if let Some(value) = &style.color {
        let _ = write!(arguments, "fill: {}, ", color(value));
    }
    if style.bold {
        arguments.push_str("weight: \"bold\", ");
    }
    if style.italic {
        arguments.push_str("style: \"italic\", ");
    }
    arguments
}

/// A string literal that keeps line breaks, for raw code.
fn string_lines(text: &str) -> String {
    let mut literal = String::with_capacity(text.len() + 2);
    literal.push('"');
    for c in text.strip_suffix('\n').unwrap_or(text).chars() {
        match c {
            '"' => literal.push_str("\\\""),
            '\\' => literal.push_str("\\\\"),
            '\n' => literal.push_str("\\n"),
            '\r' => {}
            '\t' => literal.push_str("    "),
            c => literal.push(c),
        }
    }
    literal.push('"');
    literal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_sizes_and_margins() {
        assert_eq!(paper("A4").unwrap(), "paper: \"a4\"");
        assert_eq!(paper("letter").unwrap(), "paper: \"us-letter\"");
        assert_eq!(
            paper("A5 landscape").unwrap(),
            "paper: \"a5\", flipped: true"
        );
        assert_eq!(paper("210mm 297mm").unwrap(), "width: 210mm, height: 297mm");
        assert!(paper("huge").is_err());
        assert_eq!(
            margin("20mm 18mm").unwrap(),
            "(top: 20mm, right: 18mm, bottom: 20mm, left: 18mm)"
        );
        assert!(margin("2em").is_err());
    }

    // Verifies: G-3
    #[test]
    fn every_page_but_the_cover_has_a_running_header_and_a_page_number() {
        let document = Document {
            title: "Manual".to_string(),
            path: String::new(),
            split: nectar_core::Split::Chapter,
            numbered: false,
            contents: false,
            print: Default::default(),
            root: Chapter {
                id: "manual".to_string(),
                slug: String::new(),
                title: "Manual".to_string(),
                source: PathBuf::from("main.md"),
                blocks: Vec::new(),
                footnotes: Vec::new(),
            },
            footnotes: Vec::new(),
            assets: BTreeMap::new(),
            downloads: BTreeMap::new(),
        };
        let settings = PrintSettings::default();
        let mut typesetter = Typesetter::new(&document, &settings);
        typesetter.document("Site", &Page::from_settings(&settings).unwrap(), None, None);
        let source = typesetter.out;

        // Running headers and page numbers are pagination artifacts, untagged in the PDF.
        let page = &source[source.find("#set page(").unwrap()..];
        let header = page.find("header: context").expect("a running header");
        let footer = page.find("footer: context").expect("a footer");
        assert!(header < footer, "{source}");
        assert!(
            page[footer..].starts_with("footer: context align(center, text(")
                && page[footer..].contains("counter(page).display()"),
            "{source}"
        );
        assert!(
            source.contains("#page(header: none, footer: none"),
            "a cover without them: {source}"
        );
    }

    #[test]
    fn highlighted_code_keeps_its_colours_inside_a_raw_element() {
        let document = Document {
            title: String::new(),
            path: String::new(),
            split: nectar_core::Split::Chapter,
            numbered: false,
            contents: false,
            print: Default::default(),
            root: Chapter {
                id: String::new(),
                slug: String::new(),
                title: String::new(),
                source: PathBuf::from("main.md"),
                blocks: Vec::new(),
                footnotes: Vec::new(),
            },
            footnotes: Vec::new(),
            assets: BTreeMap::new(),
            downloads: BTreeMap::new(),
        };
        let settings = PrintSettings::default();
        let mut typesetter = Typesetter::new(&document, &settings);
        let token = |text: &str, color: Option<&str>| nectar_core::model::Token {
            text: text.to_string(),
            style: TokenStyle {
                color: color.map(str::to_string),
                ..TokenStyle::default()
            },
            dark: None,
        };
        typesetter.highlighted(&HighlightedCode {
            language: "rust".to_string(),
            foreground: None,
            background: None,
            dark_foreground: None,
            dark_background: None,
            lines: vec![
                vec![token("fn", Some("#d73a49")), token(" main() {}", None)],
                vec![token("}", None)],
            ],
        });
        let source = typesetter.out;
        let shown = source
            .find("#show raw: _ => [")
            .expect("tokens stand in for raw");
        let red = source
            .find("#text(fill: rgb(\"#d73a49\"), \"fn\")")
            .expect("the colour");
        let raw = source
            .find("#raw(block: true, \"fn main() {}\\n}\")")
            .expect("the code");
        assert!(shown < red && red < raw, "{source}");
    }

    #[test]
    fn strings_cannot_escape() {
        assert_eq!(string("a \"b\" \\ #c [d]"), "\"a \\\"b\\\" \\\\ #c [d]\"");
    }
}
