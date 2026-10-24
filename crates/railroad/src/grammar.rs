//! Grammar model and the (E)BNF parser.
//!
//! Accepted notation: `rule = expression ;` (also `::=`), `a b` or `a, b` for sequences, `a | b`
//! for choices, `'x'` and `"x"` for literals, `( )` for groups, `[ ]` for options, `{ }` for
//! repetitions, the postfix operators `*`, `+` and `?`, `n * a` for exactly n times,
//! `n * [ a ]` for at most n times, `(? text ?)` for special sequences and `(* comment *)`.

use std::str::Chars;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Choice(Vec<Expression>),
    Literal(String),
    Repetition {
        expression: Box<Expression>,
        min: u32,
        max: Option<u32>,
    },
    Rule(String),
    Sequence(Vec<Expression>),
    Special(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub name: String,
    pub expression: Expression,
}

/// Parses every rule of a grammar.
pub fn parse(text: &str) -> Result<Vec<Rule>, String> {
    let mut chars = text.chars();
    let mut rules = Vec::new();
    let mut name = String::new();
    while let Some(c) = chars.next() {
        match c {
            '=' => {
                let mut chunk = Chunk::new(Kind::Group);
                load_expression(&mut chunk, &mut chars, ';');
                let mut rule_name = name.as_str();
                for _ in 0..2 {
                    rule_name = rule_name.strip_suffix(':').unwrap_or(rule_name);
                }
                let rule_name = rule_name.trim().to_string();
                name.clear();
                chunk.prune()?;
                rules.push(Rule {
                    name: rule_name,
                    expression: chunk.build(),
                });
            }
            // A '(' outside a rule starts a comment.
            '(' => {
                if chars.next() != Some('*') {
                    return Err("expected a comment '(*' after '('".to_string());
                }
                let mut last = '\0';
                for c in chars.by_ref() {
                    if c == ')' && last == '*' {
                        break;
                    }
                    last = c;
                }
            }
            c => {
                if !c.is_whitespace() || !name.is_empty() {
                    name.push(c);
                }
            }
        }
    }
    Ok(rules)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Rule,
    RepetitionToken,
    Alternation,
    Group,
    Comment,
    Special,
    Literal,
    Option,
    Repetition,
    Choice,
}

/// An intermediate node of the parser, pruned into expressions afterwards.
#[derive(Debug, Clone)]
struct Chunk {
    kind: Kind,
    text: String,
    min: u32,
    max: Option<u32>,
    children: Vec<Chunk>,
}

impl Chunk {
    fn new(kind: Kind) -> Self {
        Self {
            kind,
            text: String::new(),
            min: 0,
            max: None,
            children: Vec::new(),
        }
    }

    fn with_text(kind: Kind, text: String) -> Self {
        Self {
            text,
            ..Self::new(kind)
        }
    }

    fn repetition(min: u32, max: Option<u32>, children: Vec<Chunk>) -> Self {
        Self {
            min,
            max,
            children,
            ..Self::new(Kind::Repetition)
        }
    }

    /// Applies the postfix operators, drops comments and turns `|` into a choice.
    fn prune(&mut self) -> Result<(), String> {
        let mut has_alternation = false;
        let mut i = self.children.len();
        while i > 0 {
            i -= 1;
            match self.children[i].kind {
                Kind::RepetitionToken => {
                    let token = self.children.remove(i).text;
                    if i == 0 {
                        return Err(format!("'{token}' has no expression before it"));
                    }
                    let previous =
                        std::mem::replace(&mut self.children[i - 1], Chunk::new(Kind::Group));
                    self.children[i - 1] = match token.as_str() {
                        "*" => match previous
                            .text
                            .parse::<u32>()
                            .ok()
                            .filter(|_| previous.kind == Kind::Rule)
                        {
                            // `n * a` and `n * [ a ]`
                            Some(count) => {
                                if i >= self.children.len() {
                                    return Err(format!("'{count} *' has no expression after it"));
                                }
                                let next = self.children.remove(i);
                                if next.kind == Kind::Option {
                                    Chunk::repetition(0, Some(count), next.children)
                                } else {
                                    Chunk::repetition(count, Some(count), vec![next])
                                }
                            }
                            None => Chunk::repetition(0, None, vec![previous]),
                        },
                        "+" => Chunk::repetition(1, None, vec![previous]),
                        _ => Chunk {
                            children: vec![previous],
                            ..Chunk::new(Kind::Option)
                        },
                    };
                    // The next round visits index i - 1 and so prunes the new chunk.
                }
                Kind::Comment => {
                    self.children.remove(i);
                }
                Kind::Alternation => has_alternation = true,
                Kind::Group => {
                    let child = &mut self.children[i];
                    if !child.children.is_empty() {
                        child.prune()?;
                        if child.children.len() == 1 {
                            self.children[i] = child.children.remove(0);
                        }
                    }
                }
                Kind::Option | Kind::Repetition => self.children[i].prune()?,
                _ => {}
            }
        }

        if has_alternation {
            let mut alternatives: Vec<Vec<Chunk>> = vec![Vec::new()];
            for chunk in self.children.drain(..) {
                if chunk.kind == Kind::Alternation {
                    alternatives.push(Vec::new());
                } else {
                    alternatives.last_mut().unwrap().push(chunk);
                }
            }
            let children = alternatives
                .into_iter()
                .map(|mut chunks| {
                    if chunks.len() == 1 {
                        chunks.remove(0)
                    } else {
                        Chunk {
                            children: chunks,
                            ..Chunk::new(Kind::Group)
                        }
                    }
                })
                .collect();
            self.children = vec![Chunk {
                children,
                ..Chunk::new(Kind::Choice)
            }];
        }
        Ok(())
    }

    fn build(&self) -> Expression {
        match self.kind {
            Kind::Group => match self.children.len() {
                1 => self.children[0].build(),
                _ => Expression::Sequence(self.children.iter().map(Chunk::build).collect()),
            },
            Kind::Choice => {
                if self.children.len() == 1 {
                    return self.children[0].build();
                }
                let mut expressions = Vec::new();
                let mut has_line = false;
                for chunk in &self.children {
                    let mut expression = chunk.build();
                    // An optional branch becomes a mandatory one plus an empty branch.
                    if let Expression::Repetition {
                        expression: inner,
                        min: 0,
                        max,
                    } = expression
                    {
                        has_line = true;
                        expression = if max == Some(1) {
                            *inner
                        } else {
                            Expression::Repetition {
                                expression: inner,
                                min: 1,
                                max,
                            }
                        };
                    }
                    match expression {
                        Expression::Choice(list) => expressions.extend(list),
                        other => expressions.push(other),
                    }
                }
                let ends_empty = matches!(expressions.last(), Some(Expression::Sequence(list)) if list.is_empty());
                if has_line && !ends_empty {
                    expressions.push(Expression::Sequence(Vec::new()));
                }
                Expression::Choice(expressions)
            }
            Kind::Rule => Expression::Rule(self.text.clone()),
            Kind::Literal => Expression::Literal(self.text.clone()),
            Kind::Special => Expression::Special(self.text.clone()),
            Kind::Option => {
                if let [only] = self.children.as_slice() {
                    if only.kind == Kind::Choice {
                        let mut children = only.children.clone();
                        children.push(Chunk::new(Kind::Group));
                        return Chunk {
                            children,
                            ..Chunk::new(Kind::Choice)
                        }
                        .build();
                    }
                    return optional(only.build());
                }
                optional(Expression::Sequence(
                    self.children.iter().map(Chunk::build).collect(),
                ))
            }
            Kind::Repetition => {
                let expression = match self.children.as_slice() {
                    [only] => only.build(),
                    children => Expression::Sequence(children.iter().map(Chunk::build).collect()),
                };
                Expression::Repetition {
                    expression: Box::new(expression),
                    min: self.min,
                    max: self.max,
                }
            }
            Kind::Comment | Kind::Alternation | Kind::RepetitionToken => {
                Expression::Sequence(Vec::new())
            }
        }
    }
}

fn optional(expression: Expression) -> Expression {
    Expression::Repetition {
        expression: Box::new(expression),
        min: 0,
        max: Some(1),
    }
}

/// Reads the content of `parent` up to `stop`.
fn load_expression(parent: &mut Chunk, chars: &mut Chars<'_>, stop: char) {
    let mut text = String::new();
    if parent.kind == Kind::Literal {
        for c in chars.by_ref() {
            if c == stop {
                break;
            }
            text.push(c);
        }
        parent.text = text;
        return;
    }

    let flush = |parent: &mut Chunk, text: &mut String| {
        let content = text.trim();
        if !content.is_empty() {
            parent
                .children
                .push(Chunk::with_text(Kind::Rule, content.to_string()));
        }
        text.clear();
    };
    let mut first = true;
    let mut special: Option<char> = None;
    let mut last = '\0';
    while let Some(c) = chars.next() {
        if first && parent.kind == Kind::Group && (c == '*' || c == '?') {
            special = Some(c);
        }
        first = false;

        if let Some(marker) = special {
            // `(* comment *)` or `(? special sequence ?)`
            if c == ')' && last == marker {
                parent.kind = if marker == '*' {
                    Kind::Comment
                } else {
                    Kind::Special
                };
                let mut inner = text.chars();
                inner.next();
                inner.next_back();
                parent.text = inner.as_str().trim().to_string();
                return;
            }
            if !text.is_empty() || !c.is_whitespace() {
                text.push(c);
            }
            last = c;
            continue;
        }

        if c == stop {
            flush(parent, &mut text);
            return;
        }
        match c {
            ',' | ' ' | '\n' | '\r' | '\t' => flush(parent, &mut text),
            '|' => {
                flush(parent, &mut text);
                parent.children.push(Chunk::new(Kind::Alternation));
            }
            '*' | '+' | '?' => {
                flush(parent, &mut text);
                parent
                    .children
                    .push(Chunk::with_text(Kind::RepetitionToken, c.to_string()));
            }
            '"' | '\'' | '(' | '[' | '{' => {
                flush(parent, &mut text);
                let (kind, close) = match c {
                    '(' => (Kind::Group, ')'),
                    '[' => (Kind::Option, ']'),
                    '{' => (Kind::Repetition, '}'),
                    quote => (Kind::Literal, quote),
                };
                let mut child = Chunk::new(kind);
                load_expression(&mut child, chars, close);
                parent.children.push(child);
            }
            c => {
                if !text.is_empty() || !c.is_whitespace() {
                    text.push(c);
                }
            }
        }
        last = c;
    }
    flush(parent, &mut text);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(name: &str) -> Expression {
        Expression::Rule(name.to_string())
    }

    fn literal(text: &str) -> Expression {
        Expression::Literal(text.to_string())
    }

    fn repeat(expression: Expression, min: u32, max: Option<u32>) -> Expression {
        Expression::Repetition {
            expression: Box::new(expression),
            min,
            max,
        }
    }

    fn single(text: &str) -> Expression {
        let rules = parse(text).unwrap();
        assert_eq!(rules.len(), 1, "{rules:?}");
        rules.into_iter().next().unwrap().expression
    }

    #[test]
    fn sequences_and_names() {
        let rules = parse("r1 = a b;\nr2 ::= c, d;").unwrap();
        assert_eq!(rules[0].name, "r1");
        assert_eq!(
            rules[0].expression,
            Expression::Sequence(vec![rule("a"), rule("b")])
        );
        assert_eq!(rules[1].name, "r2");
        assert_eq!(
            rules[1].expression,
            Expression::Sequence(vec![rule("c"), rule("d")])
        );
    }

    #[test]
    fn postfix_operators() {
        assert_eq!(single("r = a*;"), repeat(rule("a"), 0, None));
        assert_eq!(single("r = a+;"), repeat(rule("a"), 1, None));
        assert_eq!(single("r = a?;"), repeat(rule("a"), 0, Some(1)));
        assert_eq!(single("r = 3 * a;"), repeat(rule("a"), 3, Some(3)));
        assert_eq!(single("r = 3 * [a];"), repeat(rule("a"), 0, Some(3)));
    }

    #[test]
    fn brackets() {
        assert_eq!(single("r = [a];"), repeat(rule("a"), 0, Some(1)));
        assert_eq!(single("r = {a};"), repeat(rule("a"), 0, None));
        assert_eq!(
            single("r = a (',' a)*;"),
            Expression::Sequence(vec![
                rule("a"),
                repeat(Expression::Sequence(vec![literal(","), rule("a")]), 0, None)
            ])
        );
    }

    #[test]
    fn choices_with_optional_branches() {
        assert_eq!(
            single("r = 'a' | \"b\";"),
            Expression::Choice(vec![literal("a"), literal("b")])
        );
        assert_eq!(
            single("r = ['x' | 'y'];"),
            Expression::Choice(vec![
                literal("x"),
                literal("y"),
                Expression::Sequence(vec![])
            ])
        );
        assert_eq!(
            single("r = a | c | ();"),
            Expression::Choice(vec![rule("a"), rule("c"), Expression::Sequence(vec![])])
        );
    }

    #[test]
    fn comments_and_special_sequences() {
        assert_eq!(
            single("(* leading *) r = a (* inside *) (? [a-zA-Z]+ ?);"),
            Expression::Sequence(vec![
                rule("a"),
                Expression::Special("[a-zA-Z]+".to_string())
            ])
        );
    }

    #[test]
    fn literals_keep_special_characters() {
        assert_eq!(
            single("r = 3 * 'a<b\"';"),
            repeat(literal("a<b\""), 3, Some(3))
        );
    }

    #[test]
    fn operators_without_operand_are_errors() {
        assert!(parse("r = * a;").is_err());
        assert!(parse("r = 3 *;").is_err());
    }
}
