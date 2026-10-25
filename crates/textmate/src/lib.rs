//! Code engine that highlights code with TextMate grammars and themes, through giallo and its
//! built-in set of grammars and themes.

use giallo::{FontStyle, HighlightOptions, Registry, Style, ThemeVariant};
use nectar_core::model::{HighlightedCode, Token, TokenStyle};
use nectar_core::{CodeEngine, EngineError, EngineOutput};

pub const DEFAULT_THEME: &str = "github-light";
pub const DEFAULT_DARK_THEME: &str = "github-dark";

pub struct TextMate {
    registry: Registry,
    theme: String,
    dark_theme: Option<String>,
}

impl TextMate {
    /// Loads the built-in grammars and selects `theme` by name, and `dark_theme` for dark
    /// colour schemes if given.
    pub fn new(theme: &str, dark_theme: Option<&str>) -> Result<Self, EngineError> {
        let mut registry = Registry::builtin()?;
        registry.link_grammars();
        for name in std::iter::once(theme).chain(dark_theme) {
            if !registry.contains_theme(name) {
                return Err(format!("unknown TextMate theme '{name}'").into());
            }
        }
        Ok(Self {
            registry,
            theme: theme.to_string(),
            dark_theme: dark_theme.map(str::to_string),
        })
    }
}

/// A token's style, leaving out what equals the theme's default.
fn token_style(style: &Style, default: &Style) -> TokenStyle {
    let font = style.font_style;
    TokenStyle {
        color: (style.foreground != default.foreground).then(|| style.foreground.as_hex()),
        background: (style.background != default.background).then(|| style.background.as_hex()),
        bold: font.contains(FontStyle::BOLD),
        italic: font.contains(FontStyle::ITALIC),
        underline: font.contains(FontStyle::UNDERLINE),
        strikethrough: font.contains(FontStyle::STRIKETHROUGH),
    }
}

impl CodeEngine for TextMate {
    fn name(&self) -> &str {
        "textmate"
    }

    fn supports(&self, language: &str) -> bool {
        self.registry.contains_grammar(language)
    }

    fn render(&self, language: &str, code: &str) -> Result<EngineOutput, EngineError> {
        let themes = match &self.dark_theme {
            Some(dark) => ThemeVariant::Dual {
                light: self.theme.as_str(),
                dark: dark.as_str(),
            },
            None => ThemeVariant::Single(self.theme.as_str()),
        };
        let options = HighlightOptions::new(language, themes)
            .merge_whitespace(true)
            .merge_same_style_tokens(true);
        let highlighted = self
            .registry
            .highlight(code.strip_suffix('\n').unwrap_or(code), &options)?;
        let (light, dark) = match highlighted.theme {
            ThemeVariant::Single(theme) => (theme.default_style, None),
            ThemeVariant::Dual { light, dark } => (light.default_style, Some(dark.default_style)),
        };

        let lines = highlighted
            .tokens
            .iter()
            .map(|line| {
                line.iter()
                    .map(|token| {
                        let (style, dark_style) = match token.style {
                            ThemeVariant::Single(style) => (style, None),
                            ThemeVariant::Dual { light, dark } => (light, Some(dark)),
                        };
                        Token {
                            text: token.text.clone(),
                            style: token_style(&style, &light),
                            dark: dark_style
                                .zip(dark)
                                .map(|(style, default)| token_style(&style, &default)),
                        }
                    })
                    .collect()
            })
            .collect();

        Ok(EngineOutput::Highlighted(HighlightedCode {
            language: language.to_string(),
            foreground: Some(light.foreground.as_hex()),
            background: Some(light.background.as_hex()),
            dark_foreground: dark.map(|d| d.foreground.as_hex()),
            dark_background: dark.map(|d| d.background.as_hex()),
            lines,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_rust() {
        let engine = TextMate::new(DEFAULT_THEME, None).unwrap();
        assert!(engine.supports("rust"));
        assert!(!engine.supports("no-such-language"));

        let EngineOutput::Highlighted(code) =
            engine.render("rust", "fn main() {}\nlet x = 1;\n").unwrap()
        else {
            panic!("expected highlighted code");
        };
        assert_eq!(code.lines.len(), 2);
        let first: String = code.lines[0].iter().map(|t| t.text.as_str()).collect();
        assert_eq!(first, "fn main() {}");
        assert!(code.lines[0].iter().any(|t| t.style.color.is_some()));
    }

    #[test]
    fn highlights_for_light_and_dark() {
        let engine = TextMate::new(DEFAULT_THEME, Some(DEFAULT_DARK_THEME)).unwrap();
        let EngineOutput::Highlighted(code) = engine.render("rust", "fn main() {}\n").unwrap()
        else {
            panic!("expected highlighted code");
        };
        assert!(code.dark_background.is_some());
        assert_ne!(code.dark_background, code.background);
        let keyword = &code.lines[0][0];
        assert_eq!(keyword.text, "fn");
        assert_ne!(keyword.dark.as_ref().unwrap().color, keyword.style.color);
        assert!(TextMate::new(DEFAULT_THEME, Some("no-such-theme")).is_err());
    }

    #[test]
    fn unknown_theme_is_an_error() {
        assert!(TextMate::new("no-such-theme", None).is_err());
    }
}
