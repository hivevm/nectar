//! Highlighted code as a standalone `<pre>`: inline colours of the light theme, no stylesheet
//! (ADR-0017), written by the same token writer and escape as the site's.

use nectar_core::model::{HighlightedCode, Token, TokenStyle};
use nectar_html::standalone_highlighted;

fn token(text: &str, style: TokenStyle) -> Token {
    Token {
        text: text.to_string(),
        style,
        dark: Some(TokenStyle {
            color: Some("#ffffff".into()),
            ..TokenStyle::default()
        }),
    }
}

// Verifies: ADR-0017
#[test]
fn highlighted_code_is_a_pre_with_the_inline_colours_of_the_light_theme() {
    let code = HighlightedCode {
        language: "x\"y".into(),
        foreground: Some("#111111".into()),
        background: Some("#eeeeee".into()),
        dark_foreground: Some("#dddddd".into()),
        dark_background: Some("#222222".into()),
        lines: vec![
            vec![
                token("plain <&> 'q' \"d\"", TokenStyle::default()),
                token(
                    "k",
                    TokenStyle {
                        color: Some("#0000ff".into()),
                        background: Some("#ffff00".into()),
                        bold: true,
                        italic: true,
                        ..TokenStyle::default()
                    },
                ),
            ],
            vec![],
            vec![
                token(
                    "u",
                    TokenStyle {
                        underline: true,
                        ..TokenStyle::default()
                    },
                ),
                token(
                    "s",
                    TokenStyle {
                        strikethrough: true,
                        ..TokenStyle::default()
                    },
                ),
                token(
                    "us",
                    TokenStyle {
                        underline: true,
                        strikethrough: true,
                        ..TokenStyle::default()
                    },
                ),
            ],
        ],
    };
    assert_eq!(
        standalone_highlighted(&code),
        "<pre style=\"color:#111111;background:#eeeeee;\"><code class=\"language-x&quot;y\">\
         plain &lt;&amp;&gt; &#39;q&#39; &quot;d&quot;\
         <span style=\"color:#0000ff;background:#ffff00;font-weight:bold;font-style:italic;\">k</span>\
         \n\n\
         <span style=\"text-decoration:underline;\">u</span>\
         <span style=\"text-decoration:line-through;\">s</span>\
         <span style=\"text-decoration:underline line-through;\">us</span>\
         </code></pre>\n"
    );
}

// Verifies: ADR-0017
#[test]
fn a_block_without_colours_has_an_empty_style() {
    let code = HighlightedCode {
        language: "text".into(),
        foreground: None,
        background: None,
        dark_foreground: None,
        dark_background: None,
        lines: vec![vec![Token {
            text: "a".into(),
            style: TokenStyle::default(),
            dark: None,
        }]],
    };
    assert_eq!(
        standalone_highlighted(&code),
        "<pre style=\"\"><code class=\"language-text\">a</code></pre>\n"
    );
}
