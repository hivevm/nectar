//! The print view loads the theme's stylesheets into Paged.js, which splits a selector at every
//! comma, those inside `:is()` included: `.markdown :is(h2, h3) + *` became the invalid
//! ` h3) + *` and stopped the layout before the first page (d9d56d6).

/// The stylesheets the print view loads.
const STYLESHEETS: [(&str, &str); 2] = [
    ("style.css", include_str!("../theme/static/style.css")),
    ("print.css", include_str!("../theme/static/print.css")),
];

/// The lines of rules whose selector holds a comma inside the parentheses of a pseudo-class.
/// Rules inside `@media screen` reach no printed page and keep their lists.
fn commas_inside_pseudo_classes(css: &str) -> Vec<usize> {
    let mut found = Vec::new();
    // Per open block: whether only screens read it.
    let mut blocks: Vec<bool> = Vec::new();
    let mut prelude = String::new();
    let (mut line, mut prelude_line) = (1, 1);
    let mut rest = css;
    while let Some(c) = rest.chars().next() {
        if let Some(comment) = rest.strip_prefix("/*") {
            let end = comment.find("*/").map_or(comment.len(), |end| end + 2);
            line += comment[..end].matches('\n').count();
            rest = &comment[end..];
            continue;
        }
        rest = &rest[c.len_utf8()..];
        if c == '"' || c == '\'' {
            // A string, such as a `content`, holds no structure of the stylesheet.
            let end = rest.find(c).map_or(rest.len(), |end| end + 1);
            line += rest[..end].matches('\n').count();
            prelude.push(c);
            prelude.push_str(&rest[..end]);
            rest = &rest[end..];
            continue;
        }
        match c {
            '{' => {
                let selector = prelude.trim();
                // `@media screen, print` reaches paper too.
                let screen = blocks.last().copied().unwrap_or(false)
                    || (selector.starts_with("@media screen") && !selector.contains(','));
                if !selector.starts_with('@') && !screen && comma_in_parentheses(selector) {
                    found.push(prelude_line);
                }
                blocks.push(screen);
                prelude.clear();
            }
            '}' => {
                blocks.pop();
                prelude.clear();
            }
            ';' => prelude.clear(),
            _ => {
                if prelude.trim().is_empty() && !c.is_whitespace() {
                    prelude_line = line;
                }
                prelude.push(c);
            }
        }
        if c == '\n' {
            line += 1;
        }
    }
    found
}

fn comma_in_parentheses(selector: &str) -> bool {
    let mut depth = 0;
    selector.chars().any(|c| {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        c == ',' && depth > 0
    })
}

#[test]
fn finds_lists_in_pseudo_classes_outside_screen_rules() {
    let css = "/* :is(a, b) */\na { color: rgb(1, 2, 3); }\n.x :is(h2, h3) + * { margin: 0; }\n\
               @media screen { .y :is(a, b) { color: red; } }\n@page { @top-left { content: \"a, b\"; } }\n\
               .z:not(.a), .w { color: red; }\n.q::after { content: \"{ /*\"; }\n\
               @media screen, print { .r :is(a, b) { color: red; } }\n";
    assert_eq!(commas_inside_pseudo_classes(css), [3, 8]);
}

#[test]
fn the_stylesheets_of_the_print_view_keep_lists_out_of_pseudo_classes() {
    for (name, css) in STYLESHEETS {
        let lines = commas_inside_pseudo_classes(css);
        assert!(
            lines.is_empty(),
            "{name}: a comma inside a pseudo-class on lines {lines:?}; write the selector out"
        );
    }
}
