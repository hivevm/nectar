//! The `data-rough-*` attributes: the style of a shape, read from it and its ancestors.

use roughr::core::FillStyle;
use usvg::roxmltree::Node;

const PREFIX: &str = "data-rough-";

/// The options of Rough.js an element sets; `None` leaves the option to an ancestor or the
/// default. Outside the crate it is built only by `set_default`, which reads and checks each
/// option as an attribute's value is read and checked.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Style {
    pub(crate) fill_style: Option<FillStyle>,
    pub(crate) roughness: Option<f32>,
    pub(crate) bowing: Option<f32>,
    pub(crate) hachure_angle: Option<f32>,
    pub(crate) hachure_gap: Option<f32>,
    pub(crate) fill_weight: Option<f32>,
    pub(crate) dash_offset: Option<f32>,
    pub(crate) dash_gap: Option<f32>,
    pub(crate) zigzag_offset: Option<f32>,
    pub(crate) disable_multi_stroke: Option<bool>,
    pub(crate) preserve_vertices: Option<bool>,
    pub(crate) seed: Option<u64>,
}

impl Style {
    /// The `data-rough-*` attributes of `node` alone; an unknown name or an unreadable value is
    /// an error that names the attribute.
    pub(crate) fn of(node: Node) -> Result<Style, String> {
        let mut style = Style::default();
        for attribute in node.attributes() {
            if let Some(name) = attribute.name().strip_prefix(PREFIX)
                && !style.set(name, attribute.value().trim(), &|| {
                    format!("{PREFIX}{name}=\"{}\"", attribute.value().trim())
                })?
            {
                return Err(format!(
                    "{PREFIX}{name} is not an attribute of the rough engine"
                ));
            }
        }
        Ok(style)
    }

    /// Sets the option a key of `[engine.rough]` names, in snake case, from its value as text, a
    /// TOML string's content or a number's or boolean's digits; `shown` is the value as the file
    /// writes it, for the message. An unknown key or an unreadable value is an error that names
    /// the key.
    pub fn set_default(&mut self, key: &str, value: &str, shown: &str) -> Result<(), String> {
        let label = || format!("{key} = {shown}");
        if key.contains('-') || !self.set(&key.replace('_', "-"), value.trim(), &label)? {
            return Err(format!("{key} is not a setting of the rough engine"));
        }
        Ok(())
    }

    /// Sets the option `name` names, in kebab case, from `value`: `false` for an unknown name.
    fn set(&mut self, name: &str, value: &str, label: &dyn Fn() -> String) -> Result<bool, String> {
        let invalid = |expected: &str| format!("{} is not valid; expected {expected}", label());
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
                self.fill_style = Some(fill_style(value).ok_or_else(|| {
                    invalid("hachure, solid, zigzag, cross-hatch, dots, dashed or zigzag-line")
                })?)
            }
            "roughness" => self.roughness = Some(number()?),
            "bowing" => self.bowing = Some(number()?),
            "hachure-angle" => self.hachure_angle = Some(number()?),
            "hachure-gap" => self.hachure_gap = Some(number()?),
            "fill-weight" => self.fill_weight = Some(number()?),
            "dash-offset" => self.dash_offset = Some(number()?),
            "dash-gap" => self.dash_gap = Some(number()?),
            "zigzag-offset" => self.zigzag_offset = Some(number()?),
            "disable-multi-stroke" => self.disable_multi_stroke = Some(flag()?),
            "preserve-vertices" => self.preserve_vertices = Some(flag()?),
            "seed" => {
                self.seed = Some(
                    value
                        .parse::<u64>()
                        .map_err(|_| invalid("a whole number of zero or more"))?,
                );
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// This style, with every option it leaves open taken from `ancestor`.
    pub(crate) fn over(self, ancestor: &Style) -> Style {
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
