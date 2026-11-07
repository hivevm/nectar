//! Code engine that redraws an SVG as a sketch, in the hand-drawn style of Rough.js.
//!
//! The text of a `rough` block is one SVG document. `usvg` resolves it into paths with absolute
//! transforms and paint; `roughr` redraws each path with the options of the `data-rough-*`
//! attributes on its element or an ancestor; text becomes plain outlines in the fonts the engine
//! is given. A gradient fills with its first stop; raster images, patterns, filters, masks and
//! clip paths are not drawn; shapes inside markers take the default style.

mod prepare;
mod style;

use std::fmt::Write;
use std::sync::Arc;

use nectar_core::model::Graphic;
use nectar_core::{CodeEngine, EngineError, EngineOutput};
use roughr::Srgba;
use roughr::core::{FillStyle, Op, OpSetType, OpType, Options};
use roughr::generator::Generator;
use usvg::tiny_skia_path::{self, PathSegment};
use usvg::{Node, Transform, fontdb};

use crate::prepare::{SVG_NS, Styles, prepare};
pub use crate::style::Style;

pub const LANGUAGE: &str = "rough";

/// The seed of every shape that sets none, so the same text gives the same sketch.
const SEED: u64 = 1;

// roughr reaches getrandom only for options without a seed, and the engine always sets one.
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
fn no_entropy(_: &mut [u8]) -> Result<(), getrandom::Error> {
    Err(getrandom::Error::UNSUPPORTED)
}
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
getrandom::register_custom_getrandom!(no_entropy);

/// The rough engine. It draws text with the fonts it is given and no others.
pub struct Rough {
    fonts: Arc<fontdb::Database>,
    defaults: Style,
}

impl Rough {
    /// An engine that draws text with the faces of `fonts`. The caller's database decides the
    /// family a family it lacks falls back to: usvg tries `serif` after every family the SVG
    /// names.
    pub fn new(fonts: Arc<fontdb::Database>) -> Self {
        Rough {
            fonts,
            defaults: Style::default(),
        }
    }

    /// This engine with `defaults` below every `data-rough-*` attribute, as `[engine.rough]` of
    /// `nectar.toml` gives them (ADR-0021).
    pub fn with_defaults(self, defaults: Style) -> Self {
        Rough { defaults, ..self }
    }

    /// The sketch of `svg`, as an SVG document.
    pub fn sketch(&self, svg: &str) -> Result<String, String> {
        let (svg, styles) = prepare(svg, &self.defaults)?;
        let options = usvg::Options {
            image_href_resolver: usvg::ImageHrefResolver {
                resolve_data: Box::new(|_, _, _| None),
                resolve_string: Box::new(|_, _| None),
            },
            fontdb: self.fonts.clone(),
            ..usvg::Options::default()
        };
        let tree =
            usvg::Tree::from_str(&svg, &options).map_err(|e| format!("the text is no SVG: {e}"))?;
        let size = tree.size();
        let mut out = String::new();
        let _ = write!(
            out,
            "<svg xmlns=\"{SVG_NS}\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">",
            w = num(size.width()),
            h = num(size.height()),
        );
        let mut context = Context {
            styles: &styles,
            defaults: &self.defaults,
            transform: Transform::identity(),
            opacity: 1.0,
            plain: false,
        };
        draw_group(tree.root(), &mut context, &mut out);
        out.push_str("</svg>\n");
        Ok(out)
    }
}

impl CodeEngine for Rough {
    fn name(&self) -> &str {
        "rough"
    }

    fn supports(&self, language: &str) -> bool {
        language == LANGUAGE
    }

    fn render(&self, language: &str, code: &str) -> Result<EngineOutput, EngineError> {
        let svg = self.sketch(code)?;
        Ok(EngineOutput::Graphic(Graphic {
            language: language.to_string(),
            svg,
            alt: "sketch".to_string(),
            code: code.to_string(),
        }))
    }
}

/// A colour with its opacity, as SVG writes it.
#[derive(Debug, Clone, Copy)]
struct Paint {
    rgb: [u8; 3],
    opacity: f32,
}

impl Paint {
    fn from_usvg(paint: &usvg::Paint, opacity: f32) -> Option<Paint> {
        let (color, stop_opacity) = match paint {
            usvg::Paint::Color(color) => (*color, 1.0),
            usvg::Paint::LinearGradient(gradient) => first_stop(gradient.stops())?,
            usvg::Paint::RadialGradient(gradient) => first_stop(gradient.stops())?,
            usvg::Paint::Pattern(_) => return None,
        };
        Some(Paint {
            rgb: [color.red, color.green, color.blue],
            opacity: opacity * stop_opacity,
        })
    }

    fn faded(self, opacity: f32) -> Paint {
        Paint {
            opacity: self.opacity * opacity,
            ..self
        }
    }

    fn srgba(self) -> Srgba {
        let [r, g, b] = self.rgb.map(|c| f32::from(c) / 255.0);
        Srgba::new(r, g, b, self.opacity)
    }

    /// The attributes `{name}="#rrggbb"` and, when not opaque, `{name}-opacity`.
    fn attributes(self, name: &str) -> String {
        let [r, g, b] = self.rgb;
        let mut text = format!(" {name}=\"#{r:02x}{g:02x}{b:02x}\"");
        if self.opacity < 1.0 {
            let _ = write!(text, " {name}-opacity=\"{}\"", num(self.opacity));
        }
        text
    }
}

fn first_stop(stops: &[usvg::Stop]) -> Option<(usvg::Color, f32)> {
    stops
        .first()
        .map(|stop| (stop.color(), stop.opacity().get()))
}

/// What a path is drawn in: the transform into the document's coordinates and the opacity of the
/// groups around it, and whether it is text.
#[derive(Clone)]
struct Context<'a> {
    styles: &'a Styles,
    defaults: &'a Style,
    transform: Transform,
    opacity: f32,
    plain: bool,
}

impl Context<'_> {
    fn style_of(&self, path: &usvg::Path) -> Style {
        self.styles.get(path.id()).unwrap_or(self.defaults).clone()
    }
}

/// Draws every visible path below `group`, with the transform and the opacity accumulated as
/// resvg does: the paths of a text's flattened group carry no absolute transform of their own.
fn draw_group<'a>(group: &'a usvg::Group, context: &mut Context<'a>, out: &mut String) {
    let outer = context.clone();
    context.transform = context.transform.pre_concat(group.transform());
    context.opacity *= group.opacity().get();
    for node in group.children() {
        match node {
            Node::Group(group) => draw_group(group, context, out),
            Node::Path(path) if path.is_visible() => {
                if context.plain {
                    draw_plain(path, context, out);
                } else {
                    draw_sketch(path, context, out);
                }
            }
            Node::Text(text) => {
                let plain = std::mem::replace(&mut context.plain, true);
                draw_group(text.flattened(), context, out);
                context.plain = plain;
            }
            Node::Path(_) | Node::Image(_) => {}
        }
    }
    *context = outer;
}

/// The fill and the stroke of `path`, faded by the groups around it, the stroke with its width
/// in the units of the document.
fn paints(path: &usvg::Path, context: &Context) -> (Option<Paint>, Option<(Paint, f32)>) {
    let fill = path.fill().and_then(|fill| {
        Paint::from_usvg(fill.paint(), fill.opacity().get()).map(|p| p.faded(context.opacity))
    });
    let stroke = path.stroke().and_then(|stroke| {
        let paint = Paint::from_usvg(stroke.paint(), stroke.opacity().get())?;
        let width = stroke.width().get() * scale(context.transform);
        Some((paint.faded(context.opacity), width))
    });
    (fill, stroke)
}

fn draw_plain(path: &usvg::Path, context: &Context, out: &mut String) {
    let Some(d) = path_data(path.data(), context.transform) else {
        return;
    };
    let (fill, stroke) = paints(path, context);
    let _ = write!(out, "<path d=\"{d}\"");
    match fill {
        Some(fill) => out.push_str(&fill.attributes("fill")),
        None => out.push_str(" fill=\"none\""),
    }
    if let Some((stroke, width)) = stroke {
        out.push_str(&stroke.attributes("stroke"));
        let _ = write!(out, " stroke-width=\"{}\"", num(width));
    }
    out.push_str("/>");
}

fn draw_sketch(path: &usvg::Path, context: &Context, out: &mut String) {
    let style = context.style_of(path);
    let Some(transformed) = path.data().clone().transform(context.transform) else {
        return;
    };
    let d = path_data(&transformed, Transform::identity()).unwrap_or_default();
    let (fill, stroke) = paints(path, context);
    // SVG fills a line too, which has no area; its hachure would be a row of dots.
    let fill = fill.filter(|_| has_area(&transformed));
    if fill.is_none() && stroke.is_none() {
        return;
    }
    let stroke_width = stroke.map_or(1.0, |(_, width)| width);
    let options = options(&style, fill, stroke, stroke_width);
    let fill_weight = style
        .fill_weight
        .filter(|weight| *weight >= 0.0)
        .unwrap_or(stroke_width / 2.0);

    let drawable = Generator::default().path::<f64>(d, &Some(options));
    for set in &drawable.sets {
        let d = ops_to_path(&set.ops);
        if d.is_empty() {
            continue;
        }
        let _ = write!(out, "<path d=\"{d}\"");
        match (&set.op_set_type, fill, stroke) {
            (OpSetType::Path, _, Some((stroke, width))) => {
                out.push_str(" fill=\"none\"");
                out.push_str(&stroke.attributes("stroke"));
                let _ = write!(out, " stroke-width=\"{}\"", num(width));
            }
            (OpSetType::FillPath, Some(fill), _) => {
                out.push_str(&fill.attributes("fill"));
                out.push_str(" fill-rule=\"evenodd\" stroke=\"none\"");
            }
            (OpSetType::FillSketch, Some(fill), _) => {
                out.push_str(" fill=\"none\"");
                out.push_str(&fill.attributes("stroke"));
                let _ = write!(out, " stroke-width=\"{}\"", num(fill_weight));
            }
            _ => out.push_str(" fill=\"none\" stroke=\"none\""),
        }
        out.push_str("/>");
    }
}

/// The options of roughr for one shape: Rough.js's defaults, the shape's style over them, and
/// always a seed.
fn options(
    style: &Style,
    fill: Option<Paint>,
    stroke: Option<(Paint, f32)>,
    stroke_width: f32,
) -> Options {
    let mut options = Options {
        // Rough.js's default; roughr's is 2.
        bowing: Some(style.bowing.unwrap_or(1.0)),
        seed: Some(style.seed.unwrap_or(SEED)),
        stroke: stroke.map(|(paint, _)| paint.srgba()),
        stroke_width: Some(stroke_width),
        fill: fill.map(Paint::srgba),
        fill_style: Some(style.fill_style.unwrap_or(FillStyle::Hachure)),
        ..Options::default()
    };
    let set = |target: &mut Option<f32>, value: Option<f32>| {
        if value.is_some() {
            *target = value;
        }
    };
    set(&mut options.roughness, style.roughness);
    set(&mut options.hachure_angle, style.hachure_angle);
    set(&mut options.hachure_gap, style.hachure_gap);
    set(&mut options.fill_weight, style.fill_weight);
    set(&mut options.dash_offset, style.dash_offset);
    set(&mut options.dash_gap, style.dash_gap);
    set(&mut options.zigzag_offset, style.zigzag_offset);
    if let Some(value) = style.disable_multi_stroke {
        options.disable_multi_stroke = Some(value);
    }
    if let Some(value) = style.preserve_vertices {
        options.preserve_vertices = Some(value);
    }
    options
}

/// Whether a subpath of `path` strays from a straight line by more than a hundredth of a unit,
/// control points included.
fn has_area(path: &tiny_skia_path::Path) -> bool {
    let mut subpath: Vec<tiny_skia_path::Point> = Vec::new();
    let mut found = false;
    let mut check = |points: &[tiny_skia_path::Point]| {
        let Some(&first) = points.first() else {
            return;
        };
        let far = points
            .iter()
            .copied()
            .max_by(|a, b| first.distance(*a).total_cmp(&first.distance(*b)))
            .unwrap_or(first);
        let length = first.distance(far);
        if length > 0.0 {
            let (dx, dy) = (far.x - first.x, far.y - first.y);
            found |= points
                .iter()
                .any(|p| ((p.x - first.x) * dy - (p.y - first.y) * dx).abs() / length > 0.01);
        }
    };
    for segment in path.segments() {
        match segment {
            PathSegment::MoveTo(p) => {
                check(&subpath);
                subpath = vec![p];
            }
            PathSegment::LineTo(p) => subpath.push(p),
            PathSegment::QuadTo(c, p) => subpath.extend([c, p]),
            PathSegment::CubicTo(c1, c2, p) => subpath.extend([c1, c2, p]),
            PathSegment::Close => {}
        }
    }
    check(&subpath);
    found
}

/// The mean scale of `transform`, which a stroke width is multiplied by.
fn scale(transform: Transform) -> f32 {
    let (sx, sy) = transform.get_scale();
    (sx * sy).sqrt()
}

/// SVG path data for `path` under `transform`, or `None` for a path the transform collapses.
fn path_data(path: &tiny_skia_path::Path, transform: Transform) -> Option<String> {
    let path = path.clone().transform(transform)?;
    let mut d = String::new();
    for segment in path.segments() {
        let _ = match segment {
            PathSegment::MoveTo(p) => write!(d, "M{} {} ", num(p.x), num(p.y)),
            PathSegment::LineTo(p) => write!(d, "L{} {} ", num(p.x), num(p.y)),
            PathSegment::QuadTo(c, p) => {
                write!(d, "Q{} {} {} {} ", num(c.x), num(c.y), num(p.x), num(p.y))
            }
            PathSegment::CubicTo(c1, c2, p) => write!(
                d,
                "C{} {} {} {} {} {} ",
                num(c1.x),
                num(c1.y),
                num(c2.x),
                num(c2.y),
                num(p.x),
                num(p.y)
            ),
            PathSegment::Close => write!(d, "Z "),
        };
    }
    Some(d.trim_end().to_string())
}

/// SVG path data for roughr's operations. roughr's own `ops_to_path` writes a move as a line.
fn ops_to_path(ops: &[Op<f64>]) -> String {
    let mut d = String::new();
    for op in ops {
        let data: Vec<String> = op.data.iter().map(|v| num(*v)).collect();
        let _ = match op.op {
            OpType::Move => write!(d, "M{} {} ", data[0], data[1]),
            OpType::LineTo => write!(d, "L{} {} ", data[0], data[1]),
            OpType::BCurveTo => write!(
                d,
                "C{} {} {} {} {} {} ",
                data[0], data[1], data[2], data[3], data[4], data[5]
            ),
        };
    }
    d.trim_end().to_string()
}

/// `value` rounded to two decimals, without a needless fraction or a negative zero.
fn num(value: impl Into<f64>) -> String {
    let rounded = (value.into() * 100.0).round() / 100.0;
    if rounded == 0.0 {
        "0".to_string()
    } else {
        rounded.to_string()
    }
}
