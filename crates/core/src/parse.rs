//! Turns one Markdown file into blocks of the document model.
//!
//! Includes are recognised here but resolved by the loader. Every transformation of the source
//! text keeps line numbers intact, so origins point at the file as written.

use std::collections::{BTreeMap, HashMap};
use std::path::{Component, Path, PathBuf};
use std::sync::LazyLock;

use comrak::nodes::{AlertType, AstNode, ListType, NodeValue, TableAlignment};
use comrak::{Arena, Options, parse_document};
use regex::Regex;
use serde::Deserialize;

use crate::config::to_strings;
use crate::diagnostic::{Diagnostic, Origin};
use crate::model::{
    AlertKind, Align, Block, BlockKind, Code, Column, FootnoteDefinition, Heading, Image, Inline,
    Link, List, ListItem, Table, Target,
};

/// Optional TOML block between `+++` lines at the start of a file.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FrontMatter {
    pub title: Option<String>,
    pub slug: Option<String>,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub vars: BTreeMap<String, toml::Value>,
}

pub(crate) enum Item {
    Block(Block),
    Include(Include),
}

pub(crate) struct Include {
    /// Included file, relative to the project root.
    pub path: PathBuf,
    /// The link text; replaces the included file's first top-level heading when not empty.
    pub title: String,
    /// The level of the heading marker in front of the link, if any.
    pub level: Option<u8>,
    /// The link as an ordinary block, used when the file cannot be included.
    pub fallback: Block,
}

pub(crate) struct ParsedFile {
    pub front: FrontMatter,
    pub items: Vec<Item>,
    pub footnotes: Vec<FootnoteDefinition>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Parses `text`, the content of `file` (relative to the project root).
pub(crate) fn parse_file(file: &Path, text: &str, vars: &BTreeMap<String, String>) -> ParsedFile {
    let mut diagnostics = Vec::new();
    let (front, body) = split_front_matter(file, text, &mut diagnostics);

    let mut vars = vars.clone();
    vars.extend(to_strings(&front.vars));
    let body = substitute_variables(file, &body, &vars, &mut diagnostics);
    let (body, alerts) = preprocess(file, &body, &mut diagnostics);

    let arena = Arena::new();
    let root = parse_document(&arena, &body, &options());
    let mut converter = Converter {
        file,
        dir: file.parent().map(Path::to_path_buf).unwrap_or_default(),
        lines: body.lines().collect(),
        alerts,
        footnotes: Vec::new(),
        diagnostics,
    };
    let items = converter.items(root);
    ParsedFile {
        front,
        items,
        footnotes: converter.footnotes,
        diagnostics: converter.diagnostics,
    }
}

fn options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.alerts = true;
    options.extension.header_attributes = true;
    options.extension.link_attributes = true;
    options
}

/// Splits off the front matter and blanks its lines in the body.
fn split_front_matter(
    file: &Path,
    text: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> (FrontMatter, String) {
    let mut lines = text.lines();
    if lines.next().map(str::trim_end) != Some("+++") {
        return (FrontMatter::default(), text.to_string());
    }
    let Some(end) = lines.position(|line| line.trim_end() == "+++") else {
        diagnostics.push(Diagnostic::error(
            "front matter is not closed with +++",
            Some(Origin::new(file, 1)),
        ));
        return (FrontMatter::default(), text.to_string());
    };

    let toml_text: Vec<&str> = text.lines().skip(1).take(end).collect();
    let front = match toml::from_str(&toml_text.join("\n")) {
        Ok(front) => front,
        Err(e) => {
            diagnostics.push(Diagnostic::error(
                format!("invalid front matter: {e}"),
                Some(Origin::new(file, 1)),
            ));
            FrontMatter::default()
        }
    };
    let mut body = "\n".repeat(end + 2);
    for line in text.lines().skip(end + 2) {
        body.push_str(line);
        body.push('\n');
    }
    (front, body)
}

static VARIABLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{\s*([A-Za-z_][A-Za-z0-9_.-]*)\s*(?:;[^}\n]*)?\}\}").unwrap());

/// Replaces `{{name}}` and `{{name;format}}`; the format is ignored. Unknown names stay as
/// written. They are reported in prose only: in code, such as a template language being
/// documented, `{{…}}` is usually meant literally.
fn substitute_variables(
    file: &Path,
    body: &str,
    vars: &BTreeMap<String, String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> String {
    let mut result = String::with_capacity(body.len());
    let mut fence: Option<(char, usize)> = None;
    for (index, line) in body.split_inclusive('\n').enumerate() {
        let content = line.trim_end_matches(['\n', '\r']);
        let in_code = fence.is_some();
        fence = next_fence_state(fence, content);
        let indented = content.starts_with('\t') || content.starts_with("    ");
        let code_line = in_code || fence.is_some() || indented;

        let mut offset = 0;
        for captures in VARIABLE.captures_iter(line) {
            let whole = captures.get(0).unwrap();
            let name = &captures[1];
            result.push_str(&line[offset..whole.start()]);
            match vars.get(name) {
                Some(value) => result.push_str(&value.replace('\n', " ")),
                None => {
                    let in_code_span = line[..whole.start()].matches('`').count() % 2 == 1;
                    if !code_line && !in_code_span {
                        diagnostics.push(Diagnostic::warning(
                            format!("unknown variable '{name}'"),
                            Some(Origin::new(file, index + 1)),
                        ));
                    }
                    result.push_str(whole.as_str());
                }
            }
            offset = whole.end();
        }
        result.push_str(&line[offset..]);
    }
    result
}

static ALERT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^!([nswe])(?:\s+(.*))?$").unwrap());
static ALERT_OPEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^!!([nswe])(?:\s+(.*))?$").unwrap());
static ALERT_CLOSE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^!!\s*$").unwrap());
static ANCHOR_WITH_SPACES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\]\(#([^()<>\n]*?\s[^()<>\n]*?)\)").unwrap());
static REFERENCE_DEFINITION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^ {0,3}\[([^\]^][^\]]*)\]:").unwrap());

/// Rewrites the dialect's line syntax into CommonMark, one line for one line:
///
/// - `!x text` becomes a block quote; `!!x` … `!!` a block quote over several lines. The kind of
///   alert is remembered by line number.
/// - `[text](#Heading with spaces)` gets angle brackets, so it parses as a link.
fn preprocess(
    file: &Path,
    body: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> (String, HashMap<usize, AlertKind>) {
    let mut result = String::with_capacity(body.len() + 64);
    let mut alerts = HashMap::new();
    let mut fence: Option<(char, usize)> = None;
    let mut open_alert: Option<usize> = None;
    // Reference labels and the line of their first definition; CommonMark uses the first.
    let mut references: HashMap<String, usize> = HashMap::new();

    for (index, line) in body.lines().enumerate() {
        let number = index + 1;
        let in_code = fence.is_some();
        fence = next_fence_state(fence, line);
        let is_code = in_code || fence.is_some();

        if open_alert.is_some() {
            if !is_code && ALERT_CLOSE.is_match(line) {
                open_alert = None;
            } else {
                result.push_str("> ");
                result.push_str(line);
            }
        } else if is_code {
            result.push_str(line);
        } else if let Some(captures) = ALERT_OPEN.captures(line) {
            alerts.insert(number, alert_kind(&captures[1]));
            open_alert = Some(number);
            result.push_str("> ");
            result.push_str(captures.get(2).map_or("", |m| m.as_str()));
        } else if let Some(captures) = ALERT.captures(line) {
            alerts.insert(number, alert_kind(&captures[1]));
            result.push_str("> ");
            result.push_str(captures.get(2).map_or("", |m| m.as_str()));
        } else {
            if let Some(captures) = REFERENCE_DEFINITION.captures(line) {
                let label = captures[1]
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .to_lowercase();
                if let Some(first) = references.get(&label) {
                    diagnostics.push(Diagnostic::warning(
                        format!("reference [{}] is already defined in line {first}; this definition is ignored", &captures[1]),
                        Some(Origin::new(file, number)),
                    ));
                } else {
                    references.insert(label, number);
                }
            }
            result.push_str(&ANCHOR_WITH_SPACES.replace_all(line, "](<#$1>)"));
        }
        result.push('\n');
    }

    if let Some(line) = open_alert {
        diagnostics.push(Diagnostic::warning(
            "alert is not closed with !!",
            Some(Origin::new(file, line)),
        ));
    }
    (result, alerts)
}

fn alert_kind(letter: &str) -> AlertKind {
    match letter {
        "s" => AlertKind::Success,
        "w" => AlertKind::Warning,
        "e" => AlertKind::Error,
        _ => AlertKind::Note,
    }
}

/// Tracks fenced code blocks line by line: returns the fence that is open after `line`.
fn next_fence_state(fence: Option<(char, usize)>, line: &str) -> Option<(char, usize)> {
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > 3 {
        return fence;
    }
    let trimmed = &line[indent..];
    let Some(marker) = trimmed.chars().next().filter(|c| *c == '`' || *c == '~') else {
        return fence;
    };
    let length = trimmed.chars().take_while(|c| *c == marker).count();
    match fence {
        Some((open, open_length)) => {
            let closes =
                marker == open && length >= open_length && trimmed[length..].trim().is_empty();
            if closes { None } else { fence }
        }
        None if length >= 3 => Some((marker, length)),
        None => None,
    }
}

static SCHEME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z][A-Za-z0-9+.-]*:").unwrap());
static SYMBOL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"@([a-z][a-z0-9_]*)@").unwrap());

struct Converter<'a> {
    file: &'a Path,
    dir: PathBuf,
    lines: Vec<&'a str>,
    alerts: HashMap<usize, AlertKind>,
    footnotes: Vec<FootnoteDefinition>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Converter<'a> {
    fn origin(&self, node: &AstNode<'_>) -> Origin {
        Origin::new(self.file, node.data().sourcepos.start.line)
    }

    /// The source text of the node's line, from the node's first column.
    fn source_from(&self, line: usize, column: usize) -> &str {
        self.lines
            .get(line.wrapping_sub(1))
            .and_then(|text| text.get(column.saturating_sub(1)..))
            .unwrap_or_default()
    }

    /// Whether the last character of the node in the source is `byte`.
    fn ends_with(&self, node: &AstNode<'_>, byte: u8) -> bool {
        let end = node.data().sourcepos.end;
        self.source_from(end.line, end.column).as_bytes().first() == Some(&byte)
    }

    /// `____text____` parses as strong text inside strong text; returns the inner node.
    fn underline<'n>(&self, node: &'n AstNode<'n>) -> Option<&'n AstNode<'n>> {
        let start = node.data().sourcepos.start;
        if !self
            .source_from(start.line, start.column)
            .starts_with("____")
        {
            return None;
        }
        let mut children = node.children();
        let inner = children.next()?;
        let is_strong = matches!(inner.data().value, NodeValue::Strong);
        (is_strong && children.next().is_none()).then_some(inner)
    }

    fn items<'n>(&mut self, root: &'n AstNode<'n>) -> Vec<Item> {
        let mut items = Vec::new();
        for child in root.children() {
            if let Some(includes) = self.includes(child) {
                items.extend(includes.into_iter().map(Item::Include));
            } else if let Some(block) = self.block(child) {
                items.push(Item::Block(block));
            }
        }
        items
    }

    /// A heading that holds nothing but a link to a local Markdown file, or a paragraph whose
    /// lines each hold nothing but such a link. Each link includes its file.
    fn includes<'n>(&mut self, node: &'n AstNode<'n>) -> Option<Vec<Include>> {
        let level = match &node.data().value {
            NodeValue::Paragraph => None,
            NodeValue::Heading(heading) => Some(heading.level),
            _ => return None,
        };
        let mut links = Vec::new();
        let mut line_has_link = false;
        for child in node.children() {
            match &child.data().value {
                NodeValue::Link(_) if !line_has_link => {
                    links.push(child);
                    line_has_link = true;
                }
                NodeValue::SoftBreak | NodeValue::LineBreak if line_has_link => {
                    line_has_link = false
                }
                NodeValue::Text(text) if text.trim().is_empty() => {}
                _ => return None,
            }
        }
        if links.is_empty() || (level.is_some() && links.len() > 1) {
            return None;
        }

        let mut includes = Vec::new();
        for link in links {
            let url = match &link.data().value {
                NodeValue::Link(link) => link.url.clone(),
                _ => return None,
            };
            if url.contains('#') || SCHEME.is_match(&url) || url.starts_with('/') {
                return None;
            }
            let path = normalize(&self.dir.join(&url))?;
            if !path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
            {
                return None;
            }
            includes.push((link, path));
        }

        let includes = includes
            .into_iter()
            .map(|(link, path)| {
                let content = self.inlines(link);
                let title = Inline::plain_text(&content);
                let mut fallback = Vec::new();
                self.inline(link, &mut fallback);
                Include {
                    path,
                    title,
                    level,
                    fallback: Block {
                        origin: self.origin(link),
                        kind: BlockKind::Paragraph(fallback),
                    },
                }
            })
            .collect();
        Some(includes)
    }

    fn blocks<'n>(&mut self, node: &'n AstNode<'n>) -> Vec<Block> {
        node.children()
            .filter_map(|child| self.block(child))
            .collect()
    }

    fn block<'n>(&mut self, node: &'n AstNode<'n>) -> Option<Block> {
        let origin = self.origin(node);
        let value = node.data().value.clone();
        let kind = match value {
            NodeValue::Paragraph => BlockKind::Paragraph(self.inlines(node)),
            NodeValue::Heading(heading) => {
                let id = node
                    .data()
                    .attrs
                    .as_ref()
                    .and_then(|attrs| attrs.id.clone())
                    .unwrap_or_default();
                BlockKind::Heading(Heading {
                    level: heading.level,
                    id,
                    number: String::new(),
                    content: self.inlines(node),
                })
            }
            NodeValue::BlockQuote => {
                let children = self.blocks(node);
                match self.alerts.get(&origin.line) {
                    Some(kind) => BlockKind::Alert(*kind, children),
                    None => BlockKind::Quote(children),
                }
            }
            NodeValue::Alert(alert) => {
                let kind = match alert.alert_type {
                    AlertType::Tip => AlertKind::Success,
                    AlertType::Warning => AlertKind::Warning,
                    AlertType::Caution => AlertKind::Error,
                    _ => AlertKind::Note,
                };
                BlockKind::Alert(kind, self.blocks(node))
            }
            NodeValue::List(list) => BlockKind::List(List {
                ordered: list.list_type == ListType::Ordered,
                start: list.start,
                tight: list.tight,
                items: node.children().map(|item| self.list_item(item)).collect(),
            }),
            NodeValue::CodeBlock(code) => BlockKind::Code(Code {
                language: code
                    .info
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_string(),
                code: code.literal.clone(),
            }),
            NodeValue::HtmlBlock(html) => BlockKind::Html(html.literal),
            NodeValue::ThematicBreak => BlockKind::ThematicBreak,
            NodeValue::Table(table) => {
                BlockKind::Table(self.table(node, &table.alignments, origin.line))
            }
            NodeValue::FootnoteDefinition(definition) => {
                let blocks = self.blocks(node);
                self.footnotes.push(FootnoteDefinition {
                    label: definition.name,
                    blocks,
                });
                return None;
            }
            NodeValue::FrontMatter(_) => return None,
            other => {
                self.diagnostics.push(Diagnostic::warning(
                    format!("unsupported block {other:?} is left out"),
                    Some(origin),
                ));
                return None;
            }
        };
        Some(Block { origin, kind })
    }

    fn list_item<'n>(&mut self, node: &'n AstNode<'n>) -> ListItem {
        let task = match &node.data().value {
            NodeValue::TaskItem(task) => Some(task.symbol.is_some()),
            _ => None,
        };
        ListItem {
            task,
            blocks: self.blocks(node),
        }
    }

    fn table<'n>(
        &mut self,
        node: &'n AstNode<'n>,
        alignments: &[TableAlignment],
        line: usize,
    ) -> Table {
        // `line` is 1-based, so it indexes the delimiter row below the header.
        let widths = self
            .lines
            .get(line)
            .map(|row| delimiter_widths(row))
            .unwrap_or_default();
        let columns = alignments
            .iter()
            .enumerate()
            .map(|(index, alignment)| Column {
                align: match alignment {
                    TableAlignment::Left => Align::Left,
                    TableAlignment::Center => Align::Center,
                    TableAlignment::Right => Align::Right,
                    TableAlignment::None => Align::Auto,
                },
                width: if widths.len() == alignments.len() {
                    widths[index].max(1)
                } else {
                    1
                },
            })
            .collect();

        let mut table = Table {
            columns,
            header: Vec::new(),
            rows: Vec::new(),
        };
        for row in node.children() {
            let is_header = matches!(row.data().value, NodeValue::TableRow(true));
            let cells: Vec<Vec<Inline>> = row.children().map(|cell| self.inlines(cell)).collect();
            if is_header {
                table.header = cells;
            } else {
                table.rows.push(cells);
            }
        }
        table
    }

    fn inlines<'n>(&mut self, node: &'n AstNode<'n>) -> Vec<Inline> {
        let mut inlines = Vec::new();
        for child in node.children() {
            self.inline(child, &mut inlines);
        }
        symbols(highlight_markers(inlines))
    }

    fn inline<'n>(&mut self, node: &'n AstNode<'n>, out: &mut Vec<Inline>) {
        let value = node.data().value.clone();
        let inline = match value {
            NodeValue::Text(text) => Inline::Text(text.into_owned()),
            NodeValue::Emph => Inline::Emphasis(self.inlines(node)),
            NodeValue::Strong => match self.underline(node) {
                Some(inner) => Inline::Underline(self.inlines(inner)),
                None => Inline::Strong(self.inlines(node)),
            },
            NodeValue::Strikethrough => Inline::Strikethrough(self.inlines(node)),
            NodeValue::Highlight => Inline::Highlight(self.inlines(node)),
            NodeValue::Code(code) => Inline::Code(code.literal),
            NodeValue::Link(link) => {
                let target = self.target(&link.url, &self.origin(node));
                Inline::Link(Link {
                    target,
                    title: link.title.clone(),
                    content: self.inlines(node),
                    reference: self.ends_with(node, b']'),
                    footnote: 0,
                })
            }
            NodeValue::Image(link) => {
                let target = self.target(&link.url, &self.origin(node));
                let attributes = node
                    .data()
                    .attrs
                    .as_ref()
                    .map(|attrs| attrs.pairs.clone())
                    .unwrap_or_default();
                let alt = Inline::plain_text(&self.inlines(node));
                Inline::Image(Image {
                    target,
                    title: link.title.clone(),
                    alt,
                    attributes,
                })
            }
            NodeValue::FootnoteReference(reference) => Inline::FootnoteRef {
                label: reference.name.clone(),
                number: 0,
            },
            NodeValue::SoftBreak => Inline::SoftBreak,
            NodeValue::LineBreak => Inline::LineBreak,
            NodeValue::HtmlInline(html) => Inline::Html(html),
            _ => {
                for child in node.children() {
                    self.inline(child, out);
                }
                return;
            }
        };
        out.push(inline);
    }

    /// Classifies a link or image URL. Relative paths resolve against the file's directory.
    fn target(&mut self, url: &str, origin: &Origin) -> Target {
        if let Some(fragment) = url.strip_prefix('#') {
            return Target::Anchor(fragment.to_string());
        }
        if url.is_empty() || SCHEME.is_match(url) || url.starts_with('/') {
            return Target::External(url.to_string());
        }
        let (path, fragment) = match url.split_once('#') {
            Some((path, fragment)) => (path, Some(fragment.to_string())),
            None => (url, None),
        };
        let Some(path) = normalize(&self.dir.join(path)) else {
            self.diagnostics.push(Diagnostic::warning(
                format!("'{url}' points outside the project"),
                Some(origin.clone()),
            ));
            return Target::External(url.to_string());
        };
        let is_markdown = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"));
        if is_markdown {
            Target::File { path, fragment }
        } else {
            Target::Asset(path)
        }
    }
}

/// Resolves `.` and `..` without touching the file system. `None` when the path leaves the root.
pub(crate) fn normalize(path: &Path) -> Option<PathBuf> {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => result.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                if !result.pop() {
                    return None;
                }
            }
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(result)
}

/// Counts the `-` of every cell in a table's delimiter row, such as `| --- | :-----: |`.
fn delimiter_widths(row: &str) -> Vec<u32> {
    let row = row.trim_start_matches(['>', ' ', '\t']).trim();
    let row = row.strip_prefix('|').unwrap_or(row);
    let row = row.strip_suffix('|').unwrap_or(row);
    row.split('|')
        .map(|cell| cell.matches('-').count() as u32)
        .collect()
}

enum Piece {
    Inline(Inline),
    Marker { open: bool, close: bool },
}

/// Wraps text between `//` markers into highlights. A marker opens before a non-blank character
/// and closes after one; `://` as in URLs never opens.
fn highlight_markers(inlines: Vec<Inline>) -> Vec<Inline> {
    let has_marker = inlines
        .iter()
        .any(|inline| matches!(inline, Inline::Text(text) if text.contains("//")));
    if !has_marker {
        return inlines;
    }

    let mut pieces = Vec::new();
    for inline in inlines {
        let Inline::Text(text) = inline else {
            pieces.push(Piece::Inline(inline));
            continue;
        };
        let mut rest = text.as_str();
        let mut before: Option<char> = None;
        while let Some(index) = rest.find("//") {
            let prev = rest[..index].chars().next_back().or(before);
            let next = rest[index + 2..].chars().next();
            if !rest[..index].is_empty() {
                pieces.push(Piece::Inline(Inline::Text(rest[..index].to_string())));
            }
            let open = next.is_some_and(|c| !c.is_whitespace() && c != '/')
                && prev != Some(':')
                && prev != Some('/');
            let close = prev.is_some_and(|c| !c.is_whitespace() && c != '/')
                && !next.is_some_and(|c| c.is_alphanumeric() || c == '/');
            pieces.push(Piece::Marker { open, close });
            before = Some('/');
            rest = &rest[index + 2..];
        }
        if !rest.is_empty() {
            pieces.push(Piece::Inline(Inline::Text(rest.to_string())));
        }
    }

    let mut result: Vec<Inline> = Vec::new();
    let mut opened: Option<usize> = None;
    for piece in pieces {
        match piece {
            Piece::Inline(inline) => result.push(inline),
            Piece::Marker { close: true, .. } if opened.is_some() => {
                let start = opened.take().unwrap();
                let content = result.split_off(start);
                result.push(Inline::Highlight(content));
            }
            Piece::Marker { open: true, .. } if opened.is_none() => opened = Some(result.len()),
            Piece::Marker { .. } => result.push(Inline::Text("//".to_string())),
        }
    }
    if let Some(start) = opened {
        result.insert(start, Inline::Text("//".to_string()));
    }
    merge_texts(result)
}

/// Turns `@name@` in text into symbols.
fn symbols(inlines: Vec<Inline>) -> Vec<Inline> {
    let mut result = Vec::with_capacity(inlines.len());
    for inline in inlines {
        let Inline::Text(text) = &inline else {
            result.push(inline);
            continue;
        };
        let mut offset = 0;
        for captures in SYMBOL.captures_iter(text) {
            let whole = captures.get(0).unwrap();
            let glued = text[..whole.start()]
                .chars()
                .next_back()
                .is_some_and(char::is_alphanumeric);
            if glued || whole.start() < offset {
                continue;
            }
            if whole.start() > offset {
                result.push(Inline::Text(text[offset..whole.start()].to_string()));
            }
            result.push(Inline::Symbol(captures[1].to_string()));
            offset = whole.end();
        }
        if offset == 0 {
            result.push(inline);
        } else if offset < text.len() {
            result.push(Inline::Text(text[offset..].to_string()));
        }
    }
    result
}

fn merge_texts(inlines: Vec<Inline>) -> Vec<Inline> {
    let mut result: Vec<Inline> = Vec::with_capacity(inlines.len());
    for inline in inlines {
        if let (Inline::Text(text), Some(Inline::Text(last))) = (&inline, result.last_mut()) {
            last.push_str(text);
            continue;
        }
        result.push(inline);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> ParsedFile {
        parse_file(Path::new("doc/test.md"), text, &BTreeMap::new())
    }

    fn blocks(text: &str) -> Vec<BlockKind> {
        parse(text)
            .items
            .into_iter()
            .map(|item| match item {
                Item::Block(block) => block.kind,
                Item::Include(_) => panic!("unexpected include"),
            })
            .collect()
    }

    fn paragraph(text: &str) -> Vec<Inline> {
        match blocks(text).remove(0) {
            BlockKind::Paragraph(inlines) => inlines,
            other => panic!("expected a paragraph, got {other:?}"),
        }
    }

    #[test]
    fn single_line_alert() {
        let kinds = blocks("!w Take care\n\nnext");
        assert!(
            matches!(&kinds[0], BlockKind::Alert(AlertKind::Warning, children) if children.len() == 1)
        );
        assert!(matches!(&kinds[1], BlockKind::Paragraph(_)));
    }

    #[test]
    fn multi_line_alert_spans_paragraphs() {
        let kinds = blocks("!!e First\n\nSecond\n!!\n\nafter");
        assert!(
            matches!(&kinds[0], BlockKind::Alert(AlertKind::Error, children) if children.len() == 2)
        );
        assert!(matches!(&kinds[1], BlockKind::Paragraph(_)));
    }

    #[test]
    fn alert_syntax_inside_code_stays_code() {
        let kinds = blocks("~~~\n!n not an alert\n~~~\n");
        assert!(matches!(&kinds[0], BlockKind::Code(code) if code.code == "!n not an alert\n"));
    }

    #[test]
    fn highlight_markers_wrap_text() {
        let inlines = paragraph("a //marked// text, see https://example.org");
        assert!(
            matches!(&inlines[1], Inline::Highlight(content) if Inline::plain_text(content) == "marked")
        );
        assert!(
            !inlines
                .iter()
                .skip(2)
                .any(|i| matches!(i, Inline::Highlight(_)))
        );
    }

    #[test]
    fn unmatched_marker_stays_text() {
        assert_eq!(
            Inline::plain_text(&paragraph("path //tmp only")),
            "path //tmp only"
        );
    }

    #[test]
    fn symbols_in_text() {
        let inlines = paragraph("@web@ and mail@host.org@x");
        assert!(matches!(&inlines[0], Inline::Symbol(name) if name == "web"));
        assert_eq!(
            inlines
                .iter()
                .filter(|i| matches!(i, Inline::Symbol(_)))
                .count(),
            1
        );
    }

    #[test]
    fn table_widths_from_delimiter_row() {
        let kinds = blocks("| a | b |\n| -- | :------: |\n| 1 | 2 |\n");
        let BlockKind::Table(table) = &kinds[0] else {
            panic!("expected a table")
        };
        assert_eq!(
            table.columns.iter().map(|c| c.width).collect::<Vec<_>>(),
            [2, 6]
        );
        assert_eq!(table.columns[1].align, Align::Center);
        assert_eq!(table.rows.len(), 1);
    }

    #[test]
    fn consecutive_include_lines_include_each_file() {
        let parsed = parse(
            "# T\r\n\r\n[A](a.md)\r\n[B](b.md)  \r\n[C](c.md)\r\n\r\n[D](d.md) and text\n[E](e.md)\n",
        );
        let includes: Vec<(String, usize)> = parsed
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Include(include) => {
                    Some((include.title.clone(), include.fallback.origin.line))
                }
                Item::Block(_) => None,
            })
            .collect();
        let expected = [("A", 3), ("B", 4), ("C", 5)];
        let expected: Vec<(String, usize)> =
            expected.iter().map(|(t, l)| (t.to_string(), *l)).collect();
        assert_eq!(includes, expected);
        assert!(
            matches!(parsed.items.last(), Some(Item::Block(_))),
            "mixed paragraphs stay text"
        );
    }

    #[test]
    fn include_link_alone_on_its_line() {
        let parsed = parse(
            "# Title\n\n[Setup](parts/setup.md)\n\n## [More](../more.md)\n\nsee [x](other.md) too",
        );
        let includes: Vec<&Include> = parsed
            .items
            .iter()
            .filter_map(|item| {
                if let Item::Include(include) = item {
                    Some(include)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(includes.len(), 2);
        assert_eq!(includes[0].path, Path::new("doc/parts/setup.md"));
        assert_eq!(includes[0].title, "Setup");
        assert_eq!(includes[1].path, Path::new("more.md"));
        assert_eq!(includes[1].level, Some(2));
    }

    #[test]
    fn front_matter_keeps_line_numbers() {
        let parsed =
            parse("+++\ntitle = \"T\"\n[vars]\nv = \"1.0\"\n+++\nVersion {{v}} {{missing}}\n");
        assert_eq!(parsed.front.title.as_deref(), Some("T"));
        let Item::Block(block) = &parsed.items[0] else {
            panic!("expected a block")
        };
        assert_eq!(block.origin.line, 6);
        let BlockKind::Paragraph(inlines) = &block.kind else {
            panic!("expected a paragraph")
        };
        assert_eq!(Inline::plain_text(inlines), "Version 1.0 {{missing}}");
        assert_eq!(parsed.diagnostics.len(), 1);
    }

    #[test]
    fn variables_with_format_and_unknown_names_in_code() {
        let vars = BTreeMap::from([("VERSION".to_string(), "2.1".to_string())]);
        let parsed = parse_file(
            Path::new("t.md"),
            "# Manual v. {{VERSION;0.00}}\n\n```xml\n<f format=\"{{Name}}\"/>\n```\n\n    {{Indented}}\n\nUse `{{Code}}`, not {{Typo}}.\n",
            &vars,
        );
        let Item::Block(block) = &parsed.items[0] else {
            panic!("expected a block")
        };
        let BlockKind::Heading(heading) = &block.kind else {
            panic!("expected a heading")
        };
        assert_eq!(Inline::plain_text(&heading.content), "Manual v. 2.1");
        let warnings: Vec<&str> = parsed
            .diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect();
        assert_eq!(warnings, ["unknown variable 'Typo'"]);
    }

    #[test]
    fn anchor_with_spaces_is_a_link() {
        let inlines = paragraph("[text](#Header of Document)");
        assert!(
            matches!(&inlines[0], Inline::Link(link) if link.target == Target::Anchor("Header of Document".into()))
        );
    }

    #[test]
    fn four_underscores_underline() {
        let inlines = paragraph("a ____line____ and __strong__");
        assert!(
            matches!(&inlines[1], Inline::Underline(content) if Inline::plain_text(content) == "line")
        );
        assert!(matches!(&inlines[3], Inline::Strong(_)));
    }

    #[test]
    fn reference_links_are_marked() {
        let inlines =
            paragraph("[inline](https://a.org) [ref][x] [][x]\n\n[x]: https://b.org \"B\"");
        let references: Vec<bool> = inlines
            .iter()
            .filter_map(|i| {
                if let Inline::Link(link) = i {
                    Some(link.reference)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(references, [false, true, true]);
    }

    #[test]
    fn duplicate_reference_definitions_are_reported() {
        let parsed = parse(
            "[a][x]\n\n[x]: https://one.org\n[X]: https://two.org\n[^x]: note\n```\n[x]: code\n```\n",
        );
        assert_eq!(parsed.diagnostics.len(), 1, "{:?}", parsed.diagnostics);
        assert_eq!(parsed.diagnostics[0].origin.as_ref().unwrap().line, 4);
    }

    #[test]
    fn targets_outside_the_project_are_external() {
        let parsed = parse("![a](../../secret.png)");
        assert_eq!(parsed.diagnostics.len(), 1);
    }
}
