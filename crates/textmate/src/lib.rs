//! Code engine that highlights code with TextMate grammars and themes, through giallo and its
//! built-in set of grammars and themes.

use giallo::{FontStyle, HighlightOptions, Registry, ThemeVariant};
use nectar_core::model::{HighlightedCode, Token, TokenStyle};
use nectar_core::{CodeEngine, EngineError, EngineOutput};

pub const DEFAULT_THEME: &str = "github-light";

pub struct TextMate {
    registry: Registry,
    theme: String,
}

impl TextMate {
    /// Loads the built-in grammars and selects `theme` by name.
    pub fn new(theme: &str) -> Result<Self, EngineError> {
        let mut registry = Registry::builtin()?;
        registry.link_grammars();
        if !registry.contains_theme(theme) {
            return Err(format!("unknown TextMate theme '{theme}'").into());
        }
        Ok(Self {
            registry,
            theme: theme.to_string(),
        })
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
        let options = HighlightOptions::new(language, ThemeVariant::Single(self.theme.as_str()))
            .merge_whitespace(true)
            .merge_same_style_tokens(true);
        let highlighted = self
            .registry
            .highlight(code.strip_suffix('\n').unwrap_or(code), &options)?;
        let ThemeVariant::Single(theme) = highlighted.theme else {
            return Err("a single theme was requested".into());
        };
        let default = theme.default_style;

        let lines = highlighted
            .tokens
            .iter()
            .map(|line| {
                line.iter()
                    .map(|token| {
                        let ThemeVariant::Single(style) = token.style else {
                            unreachable!()
                        };
                        let font = style.font_style;
                        Token {
                            text: token.text.clone(),
                            style: TokenStyle {
                                color: (style.foreground != default.foreground)
                                    .then(|| style.foreground.as_hex()),
                                background: (style.background != default.background)
                                    .then(|| style.background.as_hex()),
                                bold: font.contains(FontStyle::BOLD),
                                italic: font.contains(FontStyle::ITALIC),
                                underline: font.contains(FontStyle::UNDERLINE),
                                strikethrough: font.contains(FontStyle::STRIKETHROUGH),
                            },
                        }
                    })
                    .collect()
            })
            .collect();

        Ok(EngineOutput::Highlighted(HighlightedCode {
            language: language.to_string(),
            foreground: Some(default.foreground.as_hex()),
            background: Some(default.background.as_hex()),
            lines,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_rust() {
        let engine = TextMate::new(DEFAULT_THEME).unwrap();
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
    fn unknown_theme_is_an_error() {
        assert!(TextMate::new("no-such-theme").is_err());
    }
}
