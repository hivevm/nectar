//! Code engine that draws railroad diagrams from (E)BNF grammars, in-process.
//!
//! Every rule of a `railroad` block is drawn under its name; the drawings are stacked into one
//! SVG. A rule reference written as `\` continues the rule on a new line.

mod diagram;
mod grammar;
mod svg;

use nectar_core::model::Graphic;
use nectar_core::{CodeEngine, EngineError, EngineOutput};

pub const LANGUAGE: &str = "railroad";

#[derive(Debug, Default)]
pub struct Railroad;

/// Draws every rule of `grammar` into one SVG document.
pub fn to_svg(grammar: &str) -> Result<String, String> {
    let rules = grammar::parse(grammar)?;
    if rules.is_empty() {
        return Err("no rule found; rules are written as `name = expression ;`".to_string());
    }
    let drawings: Vec<svg::Drawing> = rules
        .iter()
        .map(|rule| svg::draw(&rule.name, &diagram::to_node(&rule.expression)))
        .collect();
    Ok(svg::document(&drawings))
}

impl CodeEngine for Railroad {
    fn name(&self) -> &str {
        "railroad"
    }

    fn supports(&self, language: &str) -> bool {
        language == LANGUAGE
    }

    fn render(&self, language: &str, code: &str) -> Result<EngineOutput, EngineError> {
        let svg = to_svg(code)?;
        Ok(EngineOutput::Graphic(Graphic {
            language: language.to_string(),
            svg,
            alt: "railroad diagram".to_string(),
            code: code.to_string(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SELECT: &str = "H2_SELECT =
    'SELECT' [ 'TOP' term ] [ 'DISTINCT' | 'ALL' ] selectExpression {',' selectExpression} \\
    'FROM' tableExpression {',' tableExpression} [ 'WHERE' expression ] \\
    [ 'GROUP BY' expression {',' expression} ] [ 'HAVING' expression ] \\
    [ ( 'UNION' [ 'ALL' ] | 'MINUS' | 'EXCEPT' | 'INTERSECT' ) select ] [ 'ORDER BY' order {',' order} ] \\
    [ 'LIMIT' expression [ 'OFFSET' expression ] [ 'SAMPLE_SIZE' rowCountInt ] ] \\
    [ 'FOR UPDATE' ];";

    #[test]
    fn draws_a_grammar_on_several_lines() {
        let svg = to_svg(SELECT).unwrap();
        assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains(">H2_SELECT</text>"));
        for word in [
            "SELECT",
            "FROM",
            "selectExpression",
            "INTERSECT",
            "FOR UPDATE",
        ] {
            assert!(svg.contains(&format!(">{word}</text>")), "{word} missing");
        }
        assert!(
            !svg.contains(">\\</text>"),
            "the line break is drawn as a box"
        );
        assert_eq!(svg.matches("<path ").count(), 1, "connectors are one path");
    }

    #[test]
    fn stacks_several_rules() {
        let svg = to_svg("a = 'x' | b;\nb = { 'y' } (? any ?);").unwrap();
        assert!(svg.contains(">a</text>") && svg.contains(">b</text>"));
        assert_eq!(svg.matches("<g transform=").count(), 2);
        assert!(svg.contains("<polygon"), "special sequences are hexagons");
    }

    #[test]
    fn loop_labels_count_the_passes() {
        let svg = to_svg("r = 3 * a b+ 2 * [c] 2 * d {e};").unwrap();
        assert!(svg.contains(">3×</text>"), "{svg}");
        assert!(svg.contains(">1..2×</text>"), "{svg}");
        assert!(svg.contains(">2×</text>"), "{svg}");
        assert_eq!(
            svg.matches("×</text>").count(),
            3,
            "plain repetitions have no label: {svg}"
        );
    }

    #[test]
    fn escapes_text() {
        let svg = to_svg("r = 'a<b' \"it's\";").unwrap();
        assert!(svg.contains(">a&lt;b</text>"));
        assert!(svg.contains(">it&apos;s</text>"));
    }

    #[test]
    fn reports_grammars_without_rules() {
        assert!(Railroad.render(LANGUAGE, "just text").is_err());
        assert!(Railroad.supports("railroad") && !Railroad.supports("ebnf"));
    }
}
