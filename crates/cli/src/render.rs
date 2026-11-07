//! `nectar render`: one code engine on one text, its output written as SVG, PNG, HTML or ANSI
//! (ADR-0018). A PNG is the SVG rasterized with the bundled fonts only
//! (ADR-0017).

use std::path::Path;
use std::sync::{Arc, LazyLock};

use anyhow::{Context, Result, anyhow, bail};
use clap::ValueEnum;
use nectar_core::model::{HighlightedCode, Token};
use nectar_core::{CodeEngine, EngineOutput};
use nectar_html::standalone_highlighted;
use resvg::usvg::fontdb;
use resvg::{tiny_skia, usvg};

/// An output format of `nectar render`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    Svg,
    Png,
    Html,
    Ansi,
}

impl Format {
    fn name(self) -> String {
        match self {
            Format::Svg => "svg",
            Format::Png => "png",
            Format::Html => "html",
            Format::Ansi => "ansi",
        }
        .to_string()
    }
}

/// Runs `engine` on `code` and gives the bytes of its output in the format `-f` names, the
/// extension of `output` names, or the default. A format the output does not allow is an error,
/// named either way; an extension that names no format gives the default.
pub fn render(
    engine: &dyn CodeEngine,
    language: &str,
    code: &str,
    format: Option<Format>,
    output: Option<&Path>,
) -> Result<Vec<u8>> {
    let result = engine
        .render(language, code)
        .map_err(|e| anyhow!("{}: {e}", engine.name()))?;

    let (kind, allowed): (&str, &[Format]) = match &result {
        EngineOutput::Graphic(_) => ("a graphic", &[Format::Svg, Format::Png]),
        EngineOutput::Highlighted(_) => ("highlighted code", &[Format::Html, Format::Ansi]),
    };
    let from_extension = || {
        output
            .and_then(|path| path.extension())
            .and_then(|extension| extension.to_str())
            .and_then(|extension| Format::from_str(extension, true).ok())
    };
    let format = match format.or_else(from_extension) {
        Some(format) if allowed.contains(&format) => format,
        Some(format) => {
            let names: Vec<String> = allowed.iter().map(|f| f.name()).collect();
            bail!(
                "'{language}' gives {kind}, which has no format {}; choose {}",
                format.name(),
                names.join(" or ")
            );
        }
        None => allowed[0],
    };

    Ok(match (result, format) {
        (EngineOutput::Graphic(graphic), Format::Png) => png(&graphic.svg)?,
        (EngineOutput::Graphic(graphic), _) => {
            let mut svg = graphic.svg;
            svg.push('\n');
            svg.into_bytes()
        }
        (EngineOutput::Highlighted(code), Format::Ansi) => ansi(&code).into_bytes(),
        (EngineOutput::Highlighted(code), _) => standalone_highlighted(&code).into_bytes(),
    })
}

/// The fonts `nectar-pdf` bundles, and no system fonts, as one database for every graphic the
/// command line draws text in: the PNG and the rough engine (ADR-0017, ADR-0020).
pub fn bundled_fonts() -> Arc<fontdb::Database> {
    static FONTS: LazyLock<Arc<fontdb::Database>> =
        LazyLock::new(|| Arc::new(font_database(nectar_pdf::FONT_FILES)));
    FONTS.clone()
}

/// A database of `files`. usvg tries `serif` after every family the SVG names, so a family the
/// fonts lack falls back to Ubuntu Sans, whatever the order of the files.
fn font_database(files: &[&'static [u8]]) -> fontdb::Database {
    let mut fonts = fontdb::Database::new();
    for data in files {
        fonts.load_font_source(fontdb::Source::Binary(Arc::new(*data)));
    }
    fonts.set_serif_family("Ubuntu Sans");
    fonts.set_monospace_family("Victor Mono");
    fonts
}

/// Rasterizes `svg` at its own size, with the bundled fonts only, and resolves no image but one
/// embedded as a data URL, so nothing outside is read.
fn png(svg: &str) -> Result<Vec<u8>> {
    let options = usvg::Options {
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: usvg::ImageHrefResolver::default_data_resolver(),
            resolve_string: Box::new(|_, _| None),
        },
        fontdb: bundled_fonts(),
        ..usvg::Options::default()
    };

    let tree = usvg::Tree::from_str(svg, &options).context("the engine's SVG cannot be read")?;
    let size = tree.size().to_int_size();
    let mut pixmap =
        tiny_skia::Pixmap::new(size.width(), size.height()).context("the graphic has no area")?;
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
    pixmap.encode_png().context("cannot encode the PNG")
}

/// 24-bit colour escape sequences in the colours of the light theme. Every line is filled with
/// the theme's background to the terminal's edge, so the code stands on one block, readable on a
/// dark terminal too, and ends with a reset.
fn ansi(code: &HighlightedCode) -> String {
    let fill = match code.background.as_deref().and_then(rgb) {
        Some((r, g, b)) => format!("\x1b[0;48;2;{r};{g};{b}m\x1b[K"),
        None => String::new(),
    };
    let mut out = String::new();
    for line in &code.lines {
        for token in line {
            out.push_str(&sgr(token, code));
            out.push_str(&visible(&token.text));
        }
        out.push_str(&fill);
        out.push_str("\x1b[0m\n");
    }
    out
}

/// `text` with its control characters shown instead of sent, so the code cannot steer the
/// terminal: C0 and DEL as their Unicode pictures, C1, such as the escape starter U+009B, as the
/// replacement character; a tab stays a tab.
fn visible(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '\t' => c,
            '\u{0}'..='\u{1f}' => char::from_u32(0x2400 + c as u32).unwrap_or(c),
            '\u{7f}' => '\u{2421}',
            '\u{80}'..='\u{9f}' => '\u{fffd}',
            _ => c,
        })
        .collect()
}

fn sgr(token: &Token, code: &HighlightedCode) -> String {
    let style = &token.style;
    let mut parameters = vec!["0".to_string()];
    let color = style.color.as_ref().or(code.foreground.as_ref());
    if let Some((r, g, b)) = color.and_then(|c| rgb(c)) {
        parameters.push(format!("38;2;{r};{g};{b}"));
    }
    let background = style.background.as_ref().or(code.background.as_ref());
    if let Some((r, g, b)) = background.and_then(|c| rgb(c)) {
        parameters.push(format!("48;2;{r};{g};{b}"));
    }
    for (on, parameter) in [
        (style.bold, "1"),
        (style.italic, "3"),
        (style.underline, "4"),
        (style.strikethrough, "9"),
    ] {
        if on {
            parameters.push(parameter.to_string());
        }
    }
    format!("\x1b[{}m", parameters.join(";"))
}

/// The red, green and blue of `#rrggbb` or `#rrggbbaa`; the alpha is dropped.
fn rgb(color: &str) -> Option<(u8, u8, u8)> {
    let hex = color.strip_prefix('#')?;
    if hex.len() != 6 && hex.len() != 8 {
        return None;
    }
    let channel = |at: usize| u8::from_str_radix(hex.get(at..at + 2)?, 16).ok();
    Some((channel(0)?, channel(2)?, channel(4)?))
}

#[cfg(test)]
mod tests {
    use nectar_rough::Rough;

    use super::*;

    /// One line of text in `family`.
    fn text(family: &str) -> String {
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"120\" height=\"40\">\
             <text x=\"4\" y=\"28\" font-size=\"20\" font-family=\"{family}\">Ag 1</text></svg>"
        )
    }

    // Verifies: ADR-0017, ADR-0020
    #[test]
    fn a_family_the_fonts_lack_falls_back_to_the_same_family_in_a_png_and_a_sketch() {
        let png = |family: &str| png(&text(family)).unwrap();
        assert_eq!(png("Nope"), png("Ubuntu Sans"));
        assert_ne!(png("Nope"), png("Victor Mono"), "the PNG shows its font");

        let rough = Rough::new(bundled_fonts());
        let sketch = |family: &str| rough.sketch(&text(family)).unwrap();
        assert_eq!(sketch("Nope"), sketch("Ubuntu Sans"));
        assert_ne!(
            sketch("Nope"),
            sketch("Victor Mono"),
            "the sketch shows its font"
        );
    }

    // Verifies: ADR-0020
    #[test]
    fn the_order_of_the_font_files_changes_no_sketch() {
        let mut reversed = nectar_pdf::FONT_FILES.to_vec();
        reversed.reverse();
        let sketch = |files: &[&'static [u8]]| {
            Rough::new(Arc::new(font_database(files)))
                .sketch(&text("Nope"))
                .unwrap()
        };
        assert_eq!(sketch(&reversed), sketch(nectar_pdf::FONT_FILES));
    }
}
