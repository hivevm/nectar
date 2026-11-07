//! Traces each path usvg makes back to the style of the shape it came from.
//!
//! usvg keeps no `data-rough-*` attribute, but it keeps the `id` of a shape on its path. So every
//! shape gets an id that is its own: one it lacks is inserted, a duplicate is renamed, and so is
//! one a shape expanded from an entity of the DTD shares, whose text the engine cannot touch. usvg gives
//! a path drawn through a `<use>` no id, so first every `<use>` is replaced with a group of its
//! attributes around a copy of the content it references.

use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::ops::Range;

use usvg::roxmltree::{self, Document, Node};

use crate::style::Style;

pub const SVG_NS: &str = "http://www.w3.org/2000/svg";
const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

/// The elements that are redrawn; usvg turns each into a path.
const SHAPES: &[&str] = &[
    "path", "rect", "circle", "ellipse", "line", "polyline", "polygon",
];

/// How many rounds of `<use>` within copied content are resolved.
const MAX_USE_DEPTH: usize = 8;

/// The text may grow by resolving `<use>` elements to this many times its length, or to
/// `MIN_BUDGET` bytes for a small one; what is left stays for usvg.
const GROWTH: usize = 16;
const MIN_BUDGET: usize = 1 << 20;

/// Elements whose content is not drawn where it stands; a `<use>` inside one is not resolved,
/// and inside a clip path usvg takes no group in its place.
const NOT_DRAWN: &[&str] = &["defs", "symbol", "marker", "clipPath", "mask", "pattern"];

/// The style of each shape, by the id usvg keeps on its path.
pub type Styles = HashMap<String, Style>;

/// The SVG with every `<use>` resolved and an id of its own on every shape, and the styles by
/// those ids.
pub fn prepare(svg: &str) -> Result<(String, Styles), String> {
    let svg = resolve_uses(svg)?;
    let document = parse(&svg)?;

    let mut prefix = String::from("nectar-rough-");
    while svg.contains(&prefix) {
        prefix.push('x');
    }
    let body = body_start(&document);
    let mut counts: HashMap<&str, usize> = HashMap::new();
    // The ids of shapes expanded from an entity, whose own text the engine cannot touch: a shape
    // of the body that shares one is renamed, so the entity's shape keeps the defaults.
    let mut from_entity: HashSet<&str> = HashSet::new();
    for node in document.descendants() {
        if let Some(id) = node.attribute("id") {
            *counts.entry(id).or_default() += 1;
            if node.range().start < body && is_svg(node, SHAPES) {
                from_entity.insert(id);
            }
        }
    }

    let mut styles = Styles::new();
    let mut edits: Vec<(Range<usize>, String)> = Vec::new();
    let mut seen: HashSet<&str> = HashSet::new();
    let mut effective: HashMap<roxmltree::NodeId, Style> = HashMap::new();
    for node in document.descendants().filter(|n| n.is_element()) {
        let inherited = node
            .parent_element()
            .and_then(|parent| effective.get(&parent.id()))
            .cloned()
            .unwrap_or_default();
        let style = Style::of(node)?.over(&inherited);
        effective.insert(node.id(), style.clone());

        if !is_svg(node, SHAPES) || node.range().start < body {
            continue;
        }
        let fresh = || format!("{prefix}{}", edits.len());
        let key = match node.attribute_node("id") {
            Some(id)
                if !from_entity.contains(id.value())
                    && (counts[id.value()] == 1 || seen.insert(id.value())) =>
            {
                id.value().to_string()
            }
            Some(id) => {
                let key = fresh();
                edits.push((id.range_value(), key.clone()));
                key
            }
            None => {
                let key = fresh();
                let at = node.range().start + 1 + name_length(&svg, node);
                edits.push((at..at, format!(" id=\"{key}\"")));
                key
            }
        };
        styles.insert(key, style);
    }
    Ok((apply(&svg, edits), styles))
}

fn parse(svg: &str) -> Result<Document<'_>, String> {
    let options = roxmltree::ParsingOptions {
        allow_dtd: true,
        ..roxmltree::ParsingOptions::default()
    };
    Document::parse_with_options(svg, options).map_err(|e| format!("the text is no SVG: {e}"))
}

/// Where the root element's text starts; an element expanded from an entity of the DTD has its
/// text before it.
fn body_start(document: &Document) -> usize {
    document.root_element().range().start
}

fn apply(svg: &str, mut edits: Vec<(Range<usize>, String)>) -> String {
    let mut out = svg.to_string();
    edits.sort_by_key(|(range, _)| range.start);
    for (range, text) in edits.iter().rev() {
        out.replace_range(range.clone(), text);
    }
    out
}

fn is_svg(node: Node, names: &[&str]) -> bool {
    let name = node.tag_name();
    name.namespace() == Some(SVG_NS) && names.contains(&name.name())
}

/// The length of the element name at the start of `node`'s text, its prefix included.
fn name_length(svg: &str, node: Node) -> usize {
    let start = node.range().start + 1;
    svg[start..]
        .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
        .unwrap_or(svg.len() - start)
}

/// `svg` with every drawn `<use>` replaced by its content, round by round, so a `<use>` within
/// copied content is resolved in the next round. A `<use>` that cannot be resolved stays for
/// usvg, and so does every `<use>` once the text has grown past its budget.
fn resolve_uses(svg: &str) -> Result<String, String> {
    let budget = MIN_BUDGET.max(svg.len() * GROWTH);
    let mut svg = svg.to_string();
    for _ in 0..MAX_USE_DEPTH {
        let document = parse(&svg)?;
        let body = body_start(&document);
        let edits: Vec<_> = document
            .descendants()
            .filter(|n| is_svg(*n, &["use"]) && n.range().start >= body)
            .filter(|n| !n.ancestors().any(|a| is_svg(a, NOT_DRAWN)))
            .filter_map(|node| Some((node.range(), replacement(&svg, &document, node)?)))
            .collect();
        if edits.is_empty() {
            break;
        }
        let mut resolved = apply(&svg, edits.clone());
        // A copy that does not parse where it lands, such as one that relies on a namespace
        // declared on an ancestor of the original, is left out; the others stay.
        if parse(&resolved).is_err() {
            let fits =
                |edit: &(Range<usize>, String)| parse(&apply(&svg, vec![edit.clone()])).is_ok();
            resolved = apply(&svg, edits.into_iter().filter(fits).collect());
        }
        if resolved.len() > budget || resolved == svg {
            break;
        }
        svg = resolved;
    }
    Ok(svg)
}

/// The group that stands for `node`, a `<use>`: its attributes, its position as a translation
/// after its transform, and the content it references, or `None` when it cannot be resolved.
fn replacement(svg: &str, document: &Document, node: Node) -> Option<String> {
    let target = target(document, node)?;
    let body = body_start(document);
    // SVG draws nothing for a circular reference; left to usvg, an indirect one would unroll
    // until its node limit.
    if is_cyclic(document, node, target) {
        return Some(String::new());
    }
    let in_dtd = target.descendants().any(|d| d.range().start < body);
    let sized_svg =
        is_svg(target, &["svg"]) && (node.has_attribute("width") || node.has_attribute("height"));
    if in_dtd || sized_svg {
        return None;
    }
    let length = |name: &str| -> Option<Option<f64>> {
        match node.attribute(name) {
            None => Some(None),
            Some(value) => value
                .trim()
                .trim_end_matches("px")
                .parse::<f64>()
                .ok()
                .map(Some),
        }
    };
    let (x, y) = (length("x")?.unwrap_or(0.0), length("y")?.unwrap_or(0.0));
    // The prefix of the `<use>`, so the elements written for it are in its namespace.
    let name = &svg[node.range().start + 1..][..name_length(svg, node)];
    let prefix = name.strip_suffix("use").unwrap_or("");

    let mut group = format!("<{prefix}g");
    copy_attributes(
        svg,
        node,
        &["x", "y", "width", "height", "href", "id", "transform"],
        &mut group,
    );
    let transform = node.attribute("transform").unwrap_or("");
    let _ = write!(
        group,
        " transform=\"{} translate({x} {y})\">",
        escape(transform)
    );

    if is_svg(target, &["symbol"]) {
        let _ = write!(group, "<{prefix}svg");
        for name in ["viewBox", "preserveAspectRatio"] {
            if let Some(value) = target.attribute(name) {
                let _ = write!(group, " {name}=\"{}\"", escape(value));
            }
        }
        for name in ["width", "height"] {
            let value = node.attribute(name).unwrap_or("100%");
            let _ = write!(group, " {name}=\"{}\"", escape(value));
        }
        // The symbol's own attributes, its paint and style among them, on a group inside.
        let _ = write!(group, "><{prefix}g");
        copy_attributes(
            svg,
            target,
            &[
                "id",
                "viewBox",
                "preserveAspectRatio",
                "x",
                "y",
                "width",
                "height",
                "refX",
                "refY",
            ],
            &mut group,
        );
        group.push('>');
        if let (Some(first), Some(last)) = (target.first_child(), target.last_child()) {
            group.push_str(&svg[first.range().start..last.range().end]);
        }
        let _ = write!(group, "</{prefix}g></{prefix}svg>");
    } else {
        group.push_str(&svg[target.range()]);
    }
    let _ = write!(group, "</{prefix}g>");
    Some(group)
}

/// Whether `referenced`, followed through the `<use>` elements within it, reaches an ancestor of
/// `node`, by id, since copies keep the ids of what they copy: SVG forbids such a cycle.
fn is_cyclic(document: &Document, node: Node, referenced: Node) -> bool {
    let ancestors: HashSet<&str> = node.ancestors().filter_map(|a| a.attribute("id")).collect();
    let mut pending = vec![referenced];
    let mut visited: HashSet<&str> = HashSet::new();
    while let Some(next) = pending.pop() {
        let Some(id) = next.attribute("id") else {
            continue;
        };
        if ancestors.contains(id) {
            return true;
        }
        if !visited.insert(id) {
            continue;
        }
        pending.extend(
            next.descendants()
                .filter(|d| is_svg(*d, &["use"]))
                .filter_map(|u| target(document, u)),
        );
    }
    false
}

/// Writes the attributes of `node` as its text has them, but those `skipped` names, unqualified,
/// and `xlink:href`.
fn copy_attributes(svg: &str, node: Node, skipped: &[&str], out: &mut String) {
    for attribute in node.attributes() {
        let skip = match attribute.namespace() {
            None => skipped.contains(&attribute.name()),
            Some(XLINK_NS) => attribute.name() == "href",
            Some(_) => false,
        };
        if !skip {
            let _ = write!(out, " {}", &svg[attribute.range()]);
        }
    }
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

/// The element a `<use>` references within the document.
fn target<'a>(document: &'a Document, node: Node) -> Option<Node<'a, 'a>> {
    let href = node
        .attribute("href")
        .or_else(|| node.attribute((XLINK_NS, "href")))?;
    let id = href.trim().strip_prefix('#')?;
    document
        .descendants()
        .find(|n| n.is_element() && n.attribute("id") == Some(id))
}
