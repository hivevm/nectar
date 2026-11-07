//! The `data-rough-*` attributes: the style of a shape, read from it and its ancestors.

use roughr::core::FillStyle;
use usvg::roxmltree::Node;

const PREFIX: &str = "data-rough-";

/// The options of Rough.js an element sets; `None` leaves the option to an ancestor or the
/// default.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Style {
    pub fill_style: Option<FillStyle>,
    pub roughness: Option<f32>,
    pub bowing: Option<f32>,
    pub hachure_angle: Option<f32>,
    pub hachure_gap: Option<f32>,
    pub fill_weight: Option<f32>,
    pub dash_offset: Option<f32>,
    pub dash_gap: Option<f32>,
    pub zigzag_offset: Option<f32>,
    pub disable_multi_stroke: Option<bool>,
    pub preserve_vertices: Option<bool>,
    pub seed: Option<u64>,
}

impl Style {
    /// The `data-rough-*` attributes of `node` alone; an unknown name or an unreadable value is
    /// an error that names the attribute.
    pub fn of(node: Node) -> Result<Style, String> {
        let mut style = Style::default();
        for attribute in node.attributes() {
            let Some(name) = attribute.name().strip_prefix(PREFIX) else {
                continue;
            };
            let value = attribute.value().trim();
            let invalid = |expected: &str| {
                format!("{PREFIX}{name}=\"{value}\" is not valid; expected {expected}")
            };
            let number = || {
                value
                    .parse::<f32>()
                    .ok()
                    .filter(|n| n.is_finite())
                    .ok_or_else(|| invalid("a number"))
            };
            let flag = || match value {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(invalid("true or false")),
            };
            match name {
                "fill-style" => {
                    style.fill_style = Some(fill_style(value).ok_or_else(|| {
                        invalid("hachure, solid, zigzag, cross-hatch, dots, dashed or zigzag-line")
                    })?)
                }
                "roughness" => style.roughness = Some(number()?),
                "bowing" => style.bowing = Some(number()?),
                "hachure-angle" => style.hachure_angle = Some(number()?),
                "hachure-gap" => style.hachure_gap = Some(number()?),
                "fill-weight" => style.fill_weight = Some(number()?),
                "dash-offset" => style.dash_offset = Some(number()?),
                "dash-gap" => style.dash_gap = Some(number()?),
                "zigzag-offset" => style.zigzag_offset = Some(number()?),
                "disable-multi-stroke" => style.disable_multi_stroke = Some(flag()?),
                "preserve-vertices" => style.preserve_vertices = Some(flag()?),
                "seed" => {
                    style.seed = Some(
                        value
                            .parse::<u64>()
                            .map_err(|_| invalid("a whole number of zero or more"))?,
                    );
                }
                _ => {
                    return Err(format!(
                        "{PREFIX}{name} is not an attribute of the rough engine"
                    ));
                }
            }
        }
        Ok(style)
    }

    /// This style, with every option it leaves open taken from `ancestor`.
    pub fn over(self, ancestor: &Style) -> Style {
        Style {
            fill_style: self.fill_style.or(ancestor.fill_style),
            roughness: self.roughness.or(ancestor.roughness),
            bowing: self.bowing.or(ancestor.bowing),
            hachure_angle: self.hachure_angle.or(ancestor.hachure_angle),
            hachure_gap: self.hachure_gap.or(ancestor.hachure_gap),
            fill_weight: self.fill_weight.or(ancestor.fill_weight),
            dash_offset: self.dash_offset.or(ancestor.dash_offset),
            dash_gap: self.dash_gap.or(ancestor.dash_gap),
            zigzag_offset: self.zigzag_offset.or(ancestor.zigzag_offset),
            disable_multi_stroke: self.disable_multi_stroke.or(ancestor.disable_multi_stroke),
            preserve_vertices: self.preserve_vertices.or(ancestor.preserve_vertices),
            seed: self.seed.or(ancestor.seed),
        }
    }
}

fn fill_style(value: &str) -> Option<FillStyle> {
    Some(match value {
        "hachure" => FillStyle::Hachure,
        "solid" => FillStyle::Solid,
        "zigzag" => FillStyle::ZigZag,
        "cross-hatch" => FillStyle::CrossHatch,
        "dots" => FillStyle::Dots,
        "dashed" => FillStyle::Dashed,
        "zigzag-line" => FillStyle::ZigZagLine,
        _ => return None,
    })
}
