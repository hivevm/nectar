//! Layout and SVG output of railroad diagrams.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;

use crate::diagram::{Node, TextKind};

const FONT_FAMILY: &str = "Verdana,DejaVu Sans,Sans-serif";
const TEXT_SIZE: f64 = 12.0;
const LOOP_SIZE: f64 = 10.0;
const CONNECTOR: &str = "#222222";
/// Padding of text boxes: top, left, bottom, right.
const INSETS: (i32, i32, i32, i32) = (5, 10, 5, 10);

/// Advance widths of Verdana in units of 1/2048 em, used to size boxes without the font at hand.
/// The text is stretched to the computed width, so any font fits its box.
fn advance(c: char) -> u32 {
    match c {
        ' ' => 720,
        'i' | 'l' => 559,
        'j' => 700,
        'f' | 't' => 760,
        'r' => 874,
        'c' | 's' | 'z' => 1067,
        'v' | 'x' | 'y' | 'k' => 1208,
        'a' | 'e' => 1225,
        'b' | 'd' | 'g' | 'h' | 'n' | 'o' | 'p' | 'q' | 'u' => 1276,
        'm' => 1992,
        'w' => 1663,
        'I' => 862,
        'J' => 932,
        'L' => 1145,
        'F' | 'T' | 'Y' => 1220,
        'E' | 'P' => 1265,
        'A' | 'B' | 'K' | 'R' | 'S' | 'V' | 'X' | 'Z' => 1405,
        'C' => 1430,
        'D' | 'G' | 'H' | 'N' | 'U' => 1540,
        'O' | 'Q' => 1612,
        'M' => 1729,
        'W' => 2025,
        '0'..='9' => 1302,
        ',' | '.' | ':' | ';' => 745,
        '\'' => 549,
        '"' => 942,
        '!' => 806,
        '-' | '(' | ')' | '[' | ']' | '/' | '\\' | '|' => 932,
        '?' => 1112,
        '+' | '=' | '<' | '>' | '#' | '~' | '^' => 1716,
        '@' => 2048,
        '&' => 1495,
        '%' => 2207,
        _ => 1302,
    }
}

#[derive(Debug, Clone, Copy)]
struct Font {
    size: f64,
}

impl Font {
    fn width(self, text: &str) -> i32 {
        let units: u32 = text.chars().map(advance).sum();
        (f64::from(units) * self.size / 2048.0).round() as i32
    }

    fn height(self) -> i32 {
        (self.size * 1.2153).round() as i32
    }

    fn descent(self) -> i32 {
        (self.size * 0.2104).round() as i32
    }
}

const TEXT_FONT: Font = Font { size: TEXT_SIZE };
const LOOP_FONT: Font = Font { size: LOOP_SIZE };

#[derive(Debug, Clone)]
struct Layout {
    width: i32,
    height: i32,
    /// Distance from the top to the line that connects the element.
    connector: i32,
    font_descent: i32,
    cardinality: Option<(String, i32)>,
}

impl Layout {
    fn new(width: i32, height: i32, connector: i32) -> Self {
        Self {
            width,
            height,
            connector,
            font_descent: 0,
            cardinality: None,
        }
    }
}

/// Layouts, computed once per node and kept by address while a diagram is drawn.
#[derive(Default)]
struct Layouts {
    cache: HashMap<*const Node, Layout>,
}

impl Layouts {
    fn get(&mut self, node: &Node) -> Layout {
        if let Some(layout) = self.cache.get(&std::ptr::from_ref(node)) {
            return layout.clone();
        }
        let layout = self.compute(node);
        self.cache.insert(std::ptr::from_ref(node), layout.clone());
        layout
    }

    fn compute(&mut self, node: &Node) -> Layout {
        match node {
            Node::Break | Node::Line => Layout::new(0, 10, 5),
            Node::Choice(nodes) => {
                let (mut width, mut height, mut connector) = (0, 0, 0);
                for (i, child) in nodes.iter().enumerate() {
                    let layout = self.get(child);
                    if i == 0 {
                        connector = layout.connector;
                    } else {
                        height += 5;
                    }
                    height += layout.height;
                    width = width.max(layout.width);
                }
                Layout::new(width + 40, height, connector)
            }
            Node::Loop {
                element,
                separator,
                min,
                max,
            } => {
                // The passes through the element, unless it is just "once or more".
                let text = match max {
                    None if *min <= 1 => None,
                    None => Some(format!("≥{min}×")),
                    Some(max) if max == min => Some(format!("{min}×")),
                    Some(max) => Some(format!("{min}..{max}×")),
                };
                let cardinality = text.map(|text| {
                    let width = LOOP_FONT.width(&text) + 6;
                    (text, width)
                });
                let inner = self.get(element);
                let mut width = inner.width;
                let mut height = inner.height;
                let mut connector = inner.connector;
                match separator {
                    Some(separator) => {
                        let back = self.get(separator);
                        width = width.max(back.width);
                        height += 5 + back.height;
                        connector += 5 + back.height;
                    }
                    None => {
                        height += 15;
                        connector += 15;
                    }
                }
                width += 40 + cardinality.as_ref().map_or(0, |c| c.1);
                Layout {
                    width,
                    height,
                    connector,
                    font_descent: if cardinality.is_some() {
                        LOOP_FONT.descent()
                    } else {
                        0
                    },
                    cardinality,
                }
            }
            Node::Sequence(nodes) => {
                let (mut width, mut above, mut below) = (0, 0, 0);
                for (i, child) in nodes.iter().enumerate() {
                    let layout = self.get(child);
                    if i > 0 {
                        width += 10;
                    }
                    width += layout.width;
                    above = above.max(layout.connector);
                    below = below.max(layout.height - layout.connector);
                }
                Layout::new(width, above + below, above)
            }
            Node::Text(_, text) => {
                let (top, left, bottom, right) = INSETS;
                let height = TEXT_FONT.height();
                let descent = TEXT_FONT.descent();
                Layout {
                    width: TEXT_FONT.width(text) + left + right,
                    height: height + top + bottom,
                    // Aligned on the text's baseline.
                    connector: top + height - descent,
                    font_descent: descent,
                    cardinality: None,
                }
            }
        }
    }
}

/// A connector piece: a straight line or a path of curves.
enum Connector {
    Line { x1: i32, y1: i32, x2: i32, y2: i32 },
    Path(Path),
}

struct Path {
    start: (i32, i32),
    end: (i32, i32),
    d: String,
}

impl Path {
    fn append(&mut self, start: (i32, i32), d: &str, end: (i32, i32)) {
        self.move_to(start);
        self.d.push_str(d);
        self.end = end;
    }

    fn move_to(&mut self, (x, y): (i32, i32)) {
        let (dx, dy) = (x - self.end.0, y - self.end.1);
        if dx != 0 || dy != 0 {
            let _ = write!(self.d, "m{dx}{}{dy}", if dy >= 0 { " " } else { "" });
        }
        self.end = (x, y);
    }

    fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let (ex, ey) = self.end;
        if x1 == x2 && ex == x1 {
            if ey == y1 || ey == y1 - 1 {
                let _ = write!(self.d, "v{}", y2 - ey);
                self.end = (x2, y2);
                return;
            }
            if ey == y2 || ey == y2 + 1 {
                let _ = write!(self.d, "v{}", y1 - ey);
                self.end = (x1, y1);
                return;
            }
        } else if y1 == y2 && ey == y1 {
            if ex == x1 || ex == x1 - 1 {
                let _ = write!(self.d, "h{}", x2 - ex);
                self.end = (x2, y2);
                return;
            }
            if ex == x2 || ex == x2 + 1 {
                let _ = write!(self.d, "h{}", x1 - ex);
                self.end = (x1, y1);
                return;
            }
        }
        self.move_to((x1, y1));
        if x1 == x2 {
            let _ = write!(self.d, "v{}", y2 - y1);
        } else if y1 == y2 {
            let _ = write!(self.d, "h{}", x2 - x1);
        } else {
            let _ = write!(
                self.d,
                "l{}{}{}",
                x2 - x1,
                if y2 - y1 >= 0 { " " } else { "" },
                y2 - y1
            );
        }
        self.end = (x2, y2);
    }
}

/// The drawing of one diagram: connectors merged into one path, boxes and texts.
#[derive(Default)]
struct Canvas {
    connectors: Vec<Connector>,
    elements: String,
    classes: BTreeMap<&'static str, String>,
}

impl Canvas {
    fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let (x1, x2) = (x1.min(x2), x1.max(x2));
        let (y1, y2) = (y1.min(y2), y1.max(y2));
        if let Some(Connector::Line {
            x1: a1,
            y1: b1,
            x2: a2,
            y2: b2,
        }) = self.connectors.last_mut()
        {
            // Extend the previous line when the new one continues it.
            if x1 == x2 && *a1 == *a2 && x1 == *a1 && y2 >= *b1 - 1 && y1 <= *b2 + 1 {
                *b1 = (*b1).min(y1);
                *b2 = (*b2).max(y2);
                return;
            }
            if y1 == y2 && *b1 == *b2 && y1 == *b1 && x2 >= *a1 - 1 && x1 <= *a2 + 1 {
                *a1 = (*a1).min(x1);
                *a2 = (*a2).max(x2);
                return;
            }
        }
        self.connectors.push(Connector::Line { x1, y1, x2, y2 });
    }

    fn path(&mut self, x1: i32, y1: i32, d: &str, x2: i32, y2: i32) {
        if let Some(Connector::Path(path)) = self.connectors.last_mut() {
            path.append((x1, y1), d, (x2, y2));
            return;
        }
        self.connectors.push(Connector::Path(Path {
            start: (x1, y1),
            end: (x2, y2),
            d: d.to_string(),
        }));
    }

    fn class(&mut self, name: &'static str, definition: String) -> &'static str {
        self.classes.entry(name).or_insert(definition);
        name
    }

    fn connector_element(&mut self) -> String {
        let mut path: Option<Path> = None;
        for connector in &self.connectors {
            match (connector, path.as_mut()) {
                (Connector::Line { x1, y1, x2, y2 }, None) => {
                    let mut first = Path {
                        start: (*x1, *y1),
                        end: (*x1, *y1),
                        d: format!("M{x1} {y1}"),
                    };
                    first.line(*x1, *y1, *x2, *y2);
                    path = Some(first);
                }
                (Connector::Path(p), None) => {
                    let (x, y) = p.start;
                    path = Some(Path {
                        start: p.start,
                        end: p.end,
                        d: format!("M{x} {y}{}", p.d),
                    });
                }
                (Connector::Line { x1, y1, x2, y2 }, Some(path)) => path.line(*x1, *y1, *x2, *y2),
                (Connector::Path(p), Some(path)) => path.append(p.start, &p.d, p.end),
            }
        }
        match path {
            Some(path) => {
                let class = self.class("c", format!("fill:none;stroke:{CONNECTOR};"));
                format!("<path class=\"{class}\" d=\"{}\"/>", path.d)
            }
            None => String::new(),
        }
    }
}

fn font_style(font: Font, bold: bool) -> String {
    let weight = if bold { "font-weight:bold;" } else { "" };
    format!(
        "fill:#000000;font-family:{FONT_FAMILY};font-size:{}px;{weight}",
        font.size
    )
}

struct Renderer<'c> {
    layouts: Layouts,
    canvas: &'c mut Canvas,
}

impl Renderer<'_> {
    fn render(&mut self, node: &Node, x: i32, y: i32) {
        match node {
            Node::Break | Node::Line => {}
            Node::Choice(nodes) => self.choice(node, nodes, x, y),
            Node::Loop {
                element, separator, ..
            } => self.repetition(node, element, separator.as_deref(), x, y),
            Node::Sequence(nodes) => {
                let connector = y + self.layouts.get(node).connector;
                let mut offset = 0;
                for (i, child) in nodes.iter().enumerate() {
                    let layout = self.layouts.get(child);
                    let x2 = x + offset;
                    if i > 0 {
                        self.canvas.line(x2 - 10, connector, x2, connector);
                    }
                    self.render(child, x2, connector - layout.connector);
                    offset += 10 + layout.width;
                }
            }
            Node::Text(kind, text) => self.text(node, *kind, text, x, y),
        }
    }

    fn choice(&mut self, node: &Node, nodes: &[Node], x: i32, y: i32) {
        let layout = self.layouts.get(node);
        let y1 = y + layout.connector;
        let x1 = x + 10;
        let x2 = x + layout.width - 10;
        let inner_x = x + 20;
        let mut inner_y = y;
        let last = nodes.len() - 1;
        for (i, child) in nodes.iter().enumerate() {
            let child_layout = self.layouts.get(child);
            let y2 = inner_y + child_layout.connector;
            if i == 0 {
                self.canvas.line(x1 - 10, y1, x1 + 10, y1);
            } else {
                if i == last {
                    self.canvas.path(x1 - 5, y1, "q5 0 5 5", x1, y1 + 5);
                    self.canvas.line(x1, y1 + 5, x1, y2 - 5);
                }
                self.canvas.path(x1, y2 - 5, "q0 5 5 5", x1 + 5, y2);
                self.canvas.line(x1 + 5, y2, inner_x, y2);
            }
            self.render(child, inner_x, inner_y);
            if i == 0 {
                self.canvas
                    .line(inner_x + child_layout.width, y2, x2 + 10, y2);
            } else {
                self.canvas
                    .line(x2 - 5, y2, inner_x + child_layout.width, y2);
                self.canvas.path(x2 - 5, y2, "q5 0 5-5", x2, y2 - 5);
                if i == last {
                    self.canvas.line(x2, y2 - 5, x2, y1 + 5);
                    self.canvas.path(x2, y1 + 5, "q0-5 5-5", x2 + 5, y1);
                }
            }
            inner_y += child_layout.height + 5;
        }
    }

    fn repetition(
        &mut self,
        node: &Node,
        element: &Node,
        separator: Option<&Node>,
        x: i32,
        y: i32,
    ) {
        let layout = self.layouts.get(node);
        let element_width = self.layouts.get(element).width;
        let mut max_width = element_width;
        let mut element_y = y;
        let mut back_y = y;
        let mut separator_x = 0;
        let mut separator_width = 0;
        if let Some(separator) = separator {
            let back = self.layouts.get(separator);
            separator_width = back.width;
            max_width = max_width.max(separator_width);
            separator_x = x + 20 + (max_width - separator_width) / 2;
            element_y += 5 + back.height;
            back_y += back.connector;
        } else {
            element_y += 15;
            back_y += 5;
        }
        let cardinality_width = layout.cardinality.as_ref().map_or(0, |c| c.1);
        let x1 = x + 10;
        let x2 = x + 20 + max_width + 10 + cardinality_width;
        let y2 = y + layout.connector;
        let shift = (max_width - element_width) / 2;

        self.canvas.line(x1 - 10, y2, x1 + 10 + shift, y2);
        self.canvas.path(x1 + 5, y2, "q-5 0-5-5", x1, y2 - 5);
        self.canvas.line(x1, y2 - 5, x1, back_y + 5);
        self.canvas.path(x1, back_y + 5, "q0-5 5-5", x1 + 5, back_y);
        let mut back_start = x1 + 5;
        if let Some(separator) = separator {
            self.canvas.line(x1 + 5, back_y, separator_x, back_y);
            self.render(separator, separator_x, y);
            back_start = separator_x + separator_width;
        }
        self.canvas.line(back_start, back_y, x2 - 5, back_y);
        self.canvas.path(x2 - 5, back_y, "q5 0 5 5", x2, back_y + 5);
        self.canvas.line(x2, back_y + 5, x2, y2 - 5);
        self.canvas.path(x2, y2 - 5, "q0 5-5 5", x2 - 5, y2);
        if let Some((text, width)) = &layout.cardinality {
            let class = self.canvas.class("u", font_style(LOOP_FONT, false));
            let _ = write!(
                self.canvas.elements,
                "<text class=\"{class}\" x=\"{}\" y=\"{}\">{}</text>",
                x2 - width,
                y2 - 5,
                escape(text)
            );
        }
        self.render(element, x + 20 + shift, element_y);
        self.canvas.line(
            x2 - cardinality_width - 10 - shift,
            y2,
            x + layout.width,
            y2,
        );
    }

    fn text(&mut self, node: &Node, kind: TextKind, text: &str, x: i32, y: i32) {
        let layout = self.layouts.get(node);
        let (top, left, _, right) = INSETS;
        let (width, height) = (layout.width, layout.height);
        let text_class = self.canvas.class("t", font_style(TEXT_FONT, false));
        match kind {
            TextKind::Rule => {
                let class = self
                    .canvas
                    .class("r", format!("fill:#d3f0ff;stroke:{CONNECTOR};"));
                let _ = write!(
                    self.canvas.elements,
                    "<rect class=\"{class}\" x=\"{x}\" y=\"{y}\" width=\"{width}\" height=\"{height}\"/>"
                );
            }
            TextKind::Literal => {
                let class = self
                    .canvas
                    .class("l", format!("fill:#90d9ff;stroke:{CONNECTOR};"));
                let radius = (INSETS.0 + INSETS.1 + INSETS.2 + INSETS.3) / 4;
                let _ = write!(
                    self.canvas.elements,
                    "<rect class=\"{class}\" x=\"{x}\" y=\"{y}\" width=\"{width}\" height=\"{height}\" rx=\"{radius}\"/>"
                );
            }
            TextKind::Special => {
                // The connector runs behind the hexagon's tips.
                let connector = y + layout.connector;
                self.canvas.line(x, connector, x + left, connector);
                let class = self
                    .canvas
                    .class("s", format!("fill:#e4f4ff;stroke:{CONNECTOR};"));
                let middle = y + height / 2;
                let _ = write!(
                    self.canvas.elements,
                    "<polygon class=\"{class}\" points=\"{x} {middle} {} {y} {} {y} {} {middle} {} {} {} {}\"/>",
                    x + left,
                    x + width - right,
                    x + width,
                    x + width - right,
                    y + height,
                    x + left,
                    y + height
                );
                self.canvas
                    .line(x + width, connector, x + width - right, connector);
            }
        }
        let text_x = x + left;
        let text_y = y + top + TEXT_FONT.height() - layout.font_descent;
        let _ = write!(
            self.canvas.elements,
            "<text class=\"{text_class}\" x=\"{text_x}\" y=\"{text_y}\" textLength=\"{}\" lengthAdjust=\"spacingAndGlyphs\">{}</text>",
            width - left - right,
            escape(text)
        );
    }
}

/// One rule drawn: its size and its SVG elements, relative to its top left corner.
pub(crate) struct Drawing {
    pub width: i32,
    pub height: i32,
    body: String,
    classes: BTreeMap<&'static str, String>,
}

const TITLE_HEIGHT: i32 = 20;

/// Draws a rule under its name. A top-level sequence with breaks is drawn on several lines that
/// are joined by a path.
pub(crate) fn draw(name: &str, root: &Node) -> Drawing {
    let mut lines: Vec<Node> = Vec::new();
    match root {
        Node::Sequence(nodes) => {
            let mut current = Vec::new();
            for node in nodes {
                if *node == Node::Break {
                    if !current.is_empty() {
                        lines.push(single_or_sequence(std::mem::take(&mut current)));
                    }
                } else {
                    current.push(node.clone());
                }
            }
            if !current.is_empty() {
                lines.push(single_or_sequence(current));
            }
        }
        other => lines.push(other.clone()),
    }
    if lines.is_empty() {
        lines.push(Node::Sequence(Vec::new()));
    }

    let mut canvas = Canvas::default();
    let mut renderer = Renderer {
        layouts: Layouts::default(),
        canvas: &mut canvas,
    };
    let continued = lines.len() > 1;
    let count = lines.len();

    let mut width = 5;
    let mut height = 5;
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            height += 5;
        }
        let layout = renderer.layouts.get(line);
        let mut line_width = 5 + layout.width + 5;
        if continued && i > 0 {
            line_width += 8;
        }
        if continued && i < count - 1 {
            line_width += 8;
        }
        width = width.max(line_width);
        height += layout.height + 5;
        if continued {
            height += 20;
        }
    }

    let mut y = 5 + TITLE_HEIGHT;
    let mut previous_end = (0, 0);
    let mut between_y = 0;
    for (i, line) in lines.iter().enumerate() {
        let layout = renderer.layouts.get(line);
        let y1 = y + layout.connector;
        let mut x = 0;
        if continued && i > 0 {
            // Down from the end of the previous line, back to the left and down into this one.
            let (end_x, end_y) = previous_end;
            let start_x = x + 8;
            let canvas = &mut *renderer.canvas;
            canvas.path(end_x, end_y, "q5 0 5 5", end_x + 5, end_y + 5);
            canvas.line(end_x + 5, end_y + 5, end_x + 5, between_y - 5);
            canvas.path(end_x + 5, between_y - 5, "q0 5-5 5", end_x, between_y);
            canvas.line(end_x, between_y, start_x, between_y);
            canvas.path(start_x, between_y, "q-5 0-5 5", start_x - 5, between_y + 5);
            canvas.line(start_x - 5, between_y + 5, start_x - 5, y1 - 5);
            canvas.path(start_x - 5, y1 - 5, "q0 5 5 5", start_x, y1);
            x += 8;
        }
        renderer.canvas.line(x, y1, x + 5, y1);
        x += 5;
        renderer.render(line, x, y);
        x += layout.width;
        renderer.canvas.line(x, y1, x + 5, y1);
        x += 5;
        if continued && i < count - 1 {
            previous_end = (x, y1);
        }
        y += layout.height + 10;
        if continued {
            y += 5;
            between_y = y;
            y += 15;
        }
    }

    let title_class = canvas.class("n", font_style(TEXT_FONT, true));
    let mut body = format!(
        "<text class=\"{title_class}\" x=\"0\" y=\"{}\">{}</text>",
        TEXT_FONT.height(),
        escape(name)
    );
    body.push_str(&canvas.connector_element());
    body.push_str(&canvas.elements);
    let title_width = Font {
        size: TEXT_SIZE * 1.1,
    }
    .width(name);
    Drawing {
        width: width.max(title_width),
        height: height + TITLE_HEIGHT,
        body,
        classes: canvas.classes,
    }
}

fn single_or_sequence(mut nodes: Vec<Node>) -> Node {
    if nodes.len() == 1 {
        nodes.remove(0)
    } else {
        Node::Sequence(nodes)
    }
}

/// Stacks the drawings into one SVG document.
pub(crate) fn document(drawings: &[Drawing]) -> String {
    let width = drawings.iter().map(|d| d.width).max().unwrap_or(0);
    let gap = 15;
    let height =
        drawings.iter().map(|d| d.height).sum::<i32>() + gap * (drawings.len() as i32 - 1).max(0);
    let mut classes = BTreeMap::new();
    for drawing in drawings {
        classes.extend(
            drawing
                .classes
                .iter()
                .map(|(name, definition)| (*name, definition.clone())),
        );
    }

    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\">"
    );
    svg.push_str("<defs><style type=\"text/css\">");
    for (name, definition) in &classes {
        let _ = write!(svg, ".{name}{{{definition}}}");
    }
    svg.push_str("</style></defs>");
    let mut y = 0;
    for drawing in drawings {
        let _ = write!(
            svg,
            "<g transform=\"translate(0,{y})\">{}</g>",
            drawing.body
        );
        y += drawing.height + gap;
    }
    svg.push_str("</svg>");
    svg
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
