//! Railroad elements: the drawable form of a grammar expression.

use crate::grammar::Expression;

/// A rule reference with this name breaks the diagram into a new line.
pub const LINE_BREAK: &str = "\\";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// Continues the diagram on a new line; only directly inside the top-level sequence.
    Break,
    Choice(Vec<Node>),
    /// A branch without elements.
    Line,
    /// `element`, then back along `separator` (or an empty track) for repetitions. The
    /// cardinality counts the passes back.
    Loop {
        element: Box<Node>,
        separator: Option<Box<Node>>,
        min: u32,
        max: Option<u32>,
    },
    Sequence(Vec<Node>),
    Text(TextKind, String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextKind {
    Rule,
    Literal,
    Special,
}

pub fn to_node(expression: &Expression) -> Node {
    match expression {
        Expression::Choice(expressions) => Node::Choice(expressions.iter().map(to_node).collect()),
        Expression::Literal(text) => Node::Text(TextKind::Literal, text.clone()),
        Expression::Special(text) => Node::Text(TextKind::Special, text.clone()),
        Expression::Rule(name) if name == LINE_BREAK => Node::Break,
        Expression::Rule(name) => Node::Text(TextKind::Rule, name.clone()),
        Expression::Repetition {
            expression,
            min,
            max,
        } => {
            let element = to_node(expression);
            let passes_back = max.map(|max| max.saturating_sub(1));
            if *min == 0 {
                let repeated = if max.is_none_or(|max| max > 1) {
                    loop_node(element, None, 0, passes_back)
                } else {
                    element
                };
                Node::Choice(vec![repeated, Node::Line])
            } else {
                loop_node(element, None, min - 1, passes_back)
            }
        }
        Expression::Sequence(expressions) => sequence(expressions),
    }
}

fn loop_node(element: Node, separator: Option<Node>, min: u32, max: Option<u32>) -> Node {
    Node::Loop {
        element: Box::new(element),
        separator: separator.map(Box::new),
        min,
        max: max.map(|max| max.max(min)),
    }
}

/// Draws `a (',' a)*` as a loop over `a` back through `','`, and `a a*` as a loop over `a`.
fn sequence(expressions: &[Expression]) -> Node {
    let mut nodes = Vec::new();
    let mut i = 0;
    while i < expressions.len() {
        let expression = &expressions[i];
        if let Some(Expression::Repetition {
            expression: repeated,
            min,
            max,
        }) = expressions.get(i + 1)
            && max.is_none_or(|max| max > 1)
        {
            match repeated.as_ref() {
                Expression::Sequence(parts)
                    if parts.len() == 2
                        && matches!(parts[0], Expression::Literal(_))
                        && &parts[1] == expression =>
                {
                    nodes.push(loop_node(
                        to_node(expression),
                        Some(to_node(&parts[0])),
                        *min,
                        *max,
                    ));
                    i += 2;
                    continue;
                }
                Expression::Rule(name) if matches!(expression, Expression::Rule(own) if own == name) =>
                {
                    nodes.push(loop_node(to_node(expression), None, *min, *max));
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }
        nodes.push(to_node(expression));
        i += 1;
    }
    Node::Sequence(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::parse;

    fn node(text: &str) -> Node {
        to_node(&parse(text).unwrap()[0].expression)
    }

    fn text(kind: TextKind, value: &str) -> Node {
        Node::Text(kind, value.to_string())
    }

    #[test]
    fn optional_and_repeated_elements_become_branches_and_loops() {
        let a = text(TextKind::Rule, "a");
        assert_eq!(node("r = [a];"), Node::Choice(vec![a.clone(), Node::Line]));
        assert_eq!(
            node("r = {a};"),
            Node::Choice(vec![loop_node(a.clone(), None, 0, None), Node::Line])
        );
        assert_eq!(node("r = a+;"), loop_node(a, None, 0, None));
    }

    #[test]
    fn separated_lists_loop_through_the_separator() {
        let a = text(TextKind::Rule, "a");
        let comma = text(TextKind::Literal, ",");
        assert_eq!(
            node("r = a {',' a} b;"),
            Node::Sequence(vec![
                loop_node(a.clone(), Some(comma), 0, None),
                text(TextKind::Rule, "b")
            ])
        );
        assert_eq!(
            node("r = x a a*;"),
            Node::Sequence(vec![text(TextKind::Rule, "x"), loop_node(a, None, 0, None)])
        );
    }

    #[test]
    fn backslash_breaks_the_line() {
        assert_eq!(
            node("r = a \\ b;"),
            Node::Sequence(vec![
                text(TextKind::Rule, "a"),
                Node::Break,
                text(TextKind::Rule, "b")
            ])
        );
    }
}
