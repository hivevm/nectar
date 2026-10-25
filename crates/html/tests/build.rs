//! Builds a small project from sources to HTML, through the core and the HTML renderer.

use std::path::Path;

use nectar_core::model::Graphic;
use nectar_core::{CodeEngine, EngineError, EngineOutput, Project};
use nectar_html::{HtmlRenderer, PrintRenderer};

struct Diagrams;

impl CodeEngine for Diagrams {
    fn name(&self) -> &str {
        "diagrams"
    }

    fn supports(&self, language: &str) -> bool {
        language == "boxes"
    }

    fn render(&self, language: &str, code: &str) -> Result<EngineOutput, EngineError> {
        if code.contains("broken") {
            return Err("cannot draw".into());
        }
        Ok(EngineOutput::Graphic(Graphic {
            language: language.to_string(),
            svg: format!("<svg><text>{}</text></svg>", code.trim()),
            alt: "boxes".to_string(),
            code: code.to_string(),
        }))
    }
}

fn write(root: &Path, file: &str, text: &str) {
    let path = root.join(file);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn page(build: &nectar_core::Build, file: &str) -> String {
    String::from_utf8(build.files[Path::new(file)].clone()).unwrap()
}

#[test]
fn builds_split_and_single_page_documents() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(
        root,
        "nectar.toml",
        "title = \"Test\"\n\n[[document]]\nsource = \"docs/main.md\"\npath = \"main\"\n\n\
         [[document]]\nsource = \"docs/main.md\"\npath = \"all\"\nsplit = \"none\"\n",
    );
    write(
        root,
        "docs/main.md",
        "# Main\n\nGo to [details](#details).\n\n[Part](part.md)\n",
    );
    write(
        root,
        "docs/part.md",
        "# Part\n\n## Details\n\n```boxes\na b\n```\n\n```boxes\nbroken\n```\n\n![logo](img/logo.png)\n",
    );
    write(root, "docs/img/logo.png", "png");

    let mut project = Project::load(root).unwrap();
    project.add_engine(Box::new(Diagrams));
    let build = project.build(&[&HtmlRenderer::default()]);

    assert!(!build.has_errors(), "{:?}", build.diagnostics);
    assert_eq!(build.diagnostics.len(), 2, "{:?}", build.diagnostics);
    assert!(
        build
            .diagnostics
            .iter()
            .all(|d| d.message.contains("cannot draw"))
    );

    let main = page(&build, "main/index.html");
    assert!(main.contains("href=\"part/index.html#details\""), "{main}");
    let part = page(&build, "main/part/index.html");
    assert!(
        part.contains("<figure class=\"graphic graphic-boxes\"><img src=\"../../_graphics/"),
        "{part}"
    );
    assert!(
        part.contains("<code class=\"language-boxes\">broken"),
        "{part}"
    );
    assert!(part.contains("src=\"../img/logo.png\""), "{part}");

    let all = page(&build, "all/index.html");
    assert!(all.contains("href=\"#details\""), "{all}");
    assert!(all.contains("<section class=\"chapter\">"), "{all}");

    assert!(build.files.contains_key(Path::new("index.html")));
    assert!(build.files.contains_key(Path::new("main/img/logo.png")));
    assert!(build.files.contains_key(Path::new("_theme/style.css")));
    assert!(build.sources.contains(Path::new("docs/part.md")));
}

#[test]
fn numbers_contents_footnotes_and_embedded_files() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "nectar.toml", "[[document]]\nsource = \"main.md\"\n");
    write(
        root,
        "main.md",
        "# Manual\n\nSee the [spec][s] and ____this____.\n\n[s]: https://spec.org \"Spec\"\n\n[Part](part.md)\n",
    );
    write(
        root,
        "part.md",
        "# Part\n\n## Details\n\n![Main class](Main.java)\n",
    );
    write(root, "Main.java", "class Main {}\n");

    let build = Project::load(root)
        .unwrap()
        .build(&[&HtmlRenderer::default()]);
    assert!(build.diagnostics.is_empty(), "{:?}", build.diagnostics);

    let main = page(&build, "index.html");
    assert!(main.contains("<u>this</u>"), "{main}");
    assert!(
        main.contains("Spec: <a href=\"https://spec.org\">https://spec.org</a>"),
        "{main}"
    );
    assert!(
        main.contains("spec</a><sup class=\"footnote-ref\"><a href=\"#fn-1\""),
        "{main}"
    );
    assert!(main.contains("<nav class=\"contents\""), "{main}");
    assert!(
        main.contains("index.html#details\">1.1. Details</a>"),
        "{main}"
    );

    let part = page(&build, "part/index.html");
    assert!(
        part.contains("<h2 id=\"part\"><span class=\"number\">1.</span> Part"),
        "{part}"
    );
    assert!(part.contains("<figure class=\"listing\">"), "{part}");
    assert!(
        part.contains("<figcaption>Main class</figcaption>"),
        "{part}"
    );
    assert!(
        !part.contains("<nav class=\"contents\""),
        "only the first page has the contents"
    );
    assert!(build.sources.contains(Path::new("Main.java")));
}

#[test]
fn menu_is_a_tree_open_along_the_current_page() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "nectar.toml", "[[document]]\nsource = \"main.md\"\n");
    write(root, "main.md", "# Main\n\n[A](a.md)\n[B](b.md)\n");
    write(root, "a.md", "# A\n\n[A1](a1.md)\n");
    write(root, "a1.md", "# A1\n");
    write(root, "b.md", "# B\n\n[B1](b1.md)\n");
    write(root, "b1.md", "# B1\n");

    let build = Project::load(root)
        .unwrap()
        .build(&[&HtmlRenderer::default()]);
    assert!(build.diagnostics.is_empty(), "{:?}", build.diagnostics);
    let page = page(&build, "a1/index.html");
    let menu = &page[page.find("book-menu-tree").unwrap()..page.find("</nav>").unwrap()];

    assert!(
        menu.contains("id=\"menu-a\" checked hidden"),
        "the parent of the page is open: {menu}"
    );
    assert!(
        menu.contains("id=\"menu-b\" hidden"),
        "other branches are closed: {menu}"
    );
    assert!(
        menu.contains("class=\"active\" aria-current=\"page\">1.1. A1</a>"),
        "{menu}"
    );
    let a = menu.find(">1. A</a>").unwrap();
    let a1 = menu.find(">1.1. A1</a>").unwrap();
    let b = menu.find(">2. B</a>").unwrap();
    assert!(a < a1 && a1 < b, "A1 is nested below A: {menu}");
    assert_eq!(menu[a..b].matches("<ul>").count(), 1, "{menu}");
}

#[test]
fn single_page_menu_lists_the_headings() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(
        root,
        "nectar.toml",
        "[[document]]\nsource = \"main.md\"\nsplit = \"none\"\n",
    );
    write(root, "main.md", "# Main\n\n## One\n\n### Inner\n\n## Two\n");

    let build = Project::load(root)
        .unwrap()
        .build(&[&HtmlRenderer::default()]);
    let page = page(&build, "index.html");
    let menu = &page[page.find("book-menu-tree").unwrap()..page.find("</nav>").unwrap()];
    assert!(menu.contains("href=\"#one\">1. One</a>"), "{menu}");
    assert!(menu.contains("id=\"menu-one\" hidden"), "{menu}");
    assert!(menu.contains("href=\"#inner\">1.1. Inner</a>"), "{menu}");
}

#[test]
fn print_view_is_one_paged_file_per_document() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(
        root,
        "nectar.toml",
        "title = \"Site\"\n\n[[document]]\nsource = \"main.md\"\n\n[document.print]\nsubtitle = \"Version 1\"\nwatermark = \"draft\"\nchapter_pages = true\n",
    );
    write(
        root,
        "main.md",
        "# Manual\n\nIntro[^n].\n\n[^n]: The note.\n\n[Part](part.md)\n",
    );
    write(root, "part.md", "# Part\n\n## Details\n");

    let web = HtmlRenderer {
        print_view: true,
        pdf: false,
    };
    let build = Project::load(root).unwrap().build(&[&web, &PrintRenderer]);
    assert!(build.diagnostics.is_empty(), "{:?}", build.diagnostics);

    let print = page(&build, "print/index.html");
    assert!(print.contains("paged.polyfill.min.js"), "{print}");
    assert!(
        print.contains("<h1 class=\"cover-title\">Manual</h1>"),
        "{print}"
    );
    assert!(
        print.contains("<p class=\"cover-subtitle\">Version 1</p>"),
        "{print}"
    );
    assert!(
        print.contains("class=\"print chapter-pages watermark\""),
        "{print}"
    );
    assert!(
        print.contains("print&#x2f;draft.png") || print.contains("print/draft.png"),
        "{print}"
    );
    assert!(
        print.contains("<span class=\"footnote\">The note.\n</span>"),
        "{print}"
    );
    assert!(
        print.contains("<section class=\"chapter\">"),
        "chapters are in one file: {print}"
    );
    assert!(
        print.contains("href=\"#details\">1.1. Details</a>"),
        "{print}"
    );
    assert!(
        !print.contains("class=\"anchor\""),
        "no heading anchors in print: {print}"
    );
    assert!(
        build
            .files
            .contains_key(Path::new("_theme/print/paged.polyfill.min.js"))
    );

    let web_page = page(&build, "part/index.html");
    assert!(web_page.contains("class=\"book-print\""), "{web_page}");
    let web_root = page(&build, "index.html");
    assert!(
        web_root.contains("fn-1"),
        "the web keeps footnotes at the end: {web_root}"
    );
}

#[test]
fn print_view_can_be_turned_off_and_reports_bad_settings() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "main.md", "# Manual\n");

    write(
        root,
        "nectar.toml",
        "[[document]]\nsource = \"main.md\"\n[document.print]\nenabled = false\n",
    );
    let web = HtmlRenderer {
        print_view: true,
        pdf: false,
    };
    let build = Project::load(root).unwrap().build(&[&web, &PrintRenderer]);
    assert!(!build.files.contains_key(Path::new("print/index.html")));
    assert!(!page(&build, "index.html").contains("book-print"));

    write(
        root,
        "nectar.toml",
        "[[document]]\nsource = \"main.md\"\n[document.print]\nsize = 3\n",
    );
    let build = Project::load(root).unwrap().build(&[&web, &PrintRenderer]);
    assert!(build.has_errors(), "{:?}", build.diagnostics);
}

#[test]
fn images_from_data_uris_become_files_of_the_site() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "nectar.toml", "[[document]]\nsource = \"main.md\"\n");
    write(
        root,
        "main.md",
        "# Manual\n\n![dot](data:image/png;base64,iVBORw0KGgo=)\n\n![web](https://example.org/a.png)\n",
    );

    let mut project = Project::load(root).unwrap();
    project.set_offline(true);
    let build = project.build(&[&HtmlRenderer::default()]);
    assert!(build.diagnostics.is_empty(), "{:?}", build.diagnostics);
    let file = build
        .files
        .keys()
        .find(|path| path.starts_with("_remote") && path.extension().is_some_and(|e| e == "png"))
        .expect("the data URI is a file of the site")
        .clone();
    assert_eq!(build.files[&file], b"\x89PNG\r\n\x1a\n");
    let index = page(&build, "index.html");
    assert!(
        // URLs use `/` on every platform.
        index.contains(&format!(
            "src=\"{}\"",
            file.to_string_lossy().replace('\\', "/")
        )),
        "{index}"
    );
    assert!(
        index.contains("src=\"https://example.org/a.png\""),
        "offline keeps the link: {index}"
    );
}

struct Colours;

impl CodeEngine for Colours {
    fn name(&self) -> &str {
        "colours"
    }

    fn supports(&self, language: &str) -> bool {
        language == "x"
    }

    fn render(&self, language: &str, _code: &str) -> Result<EngineOutput, EngineError> {
        use nectar_core::model::{HighlightedCode, Token, TokenStyle};
        let style = |color: Option<&str>| TokenStyle {
            color: color.map(str::to_string),
            ..TokenStyle::default()
        };
        Ok(EngineOutput::Highlighted(HighlightedCode {
            language: language.to_string(),
            foreground: Some("#111111".into()),
            background: Some("#ffffff".into()),
            dark_foreground: Some("#eeeeee".into()),
            dark_background: Some("#000000".into()),
            lines: vec![vec![
                Token {
                    text: "key".into(),
                    style: style(Some("#aa0000")),
                    dark: Some(style(None)),
                },
                Token {
                    text: " plain".into(),
                    style: style(None),
                    dark: Some(style(None)),
                },
            ]],
        }))
    }
}

#[test]
fn highlighted_code_carries_light_and_dark_colours() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "nectar.toml", "[[document]]\nsource = \"main.md\"\n");
    write(root, "main.md", "# Main\n\n```x\nkey plain\n```\n");

    let mut project = Project::load(root).unwrap();
    project.add_engine(Box::new(Colours));
    let build = project.build(&[&HtmlRenderer::default()]);
    let index = page(&build, "index.html");
    assert!(
        index.contains("style=\"--fg:#111111;--bg:#ffffff;--dfg:#eeeeee;--dbg:#000000;\""),
        "{index}"
    );
    // A colour of either theme brings both; the dark one falls back to the dark foreground.
    assert!(
        index.contains("<span style=\"--c:#aa0000;--dc:#eeeeee;\">key</span>"),
        "{index}"
    );
    assert!(
        index.contains("</span> plain</code>"),
        "tokens without colour inherit: {index}"
    );
}

// Verifies: G-9
#[test]
fn raw_html_is_text_unless_the_project_allows_it() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "nectar.toml", "[[document]]\nsource = \"main.md\"\n");
    write(
        root,
        "main.md",
        "# Manual\n\nRun `nectar` with -c <file>.\n\n<div>block</div>\n",
    );

    let build = Project::load(root)
        .unwrap()
        .build(&[&HtmlRenderer::default()]);
    let html = page(&build, "index.html");
    assert!(html.contains("-c &lt;file&gt;."), "{html}");
    assert!(html.contains("&lt;div&gt;block&lt;/div&gt;"), "{html}");
    let warnings: Vec<_> = build
        .diagnostics
        .iter()
        .map(|d| (d.message.as_str(), d.origin.as_ref().map(|o| o.line)))
        .collect();
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert!(
        warnings
            .iter()
            .all(|(m, line)| m.contains("raw HTML shown as text") && line.is_some()),
        "{warnings:?}"
    );

    write(
        root,
        "nectar.toml",
        "html = true\n[[document]]\nsource = \"main.md\"\n",
    );
    let build = Project::load(root)
        .unwrap()
        .build(&[&HtmlRenderer::default()]);
    let html = page(&build, "index.html");
    assert!(html.contains("-c <file>."), "{html}");
    assert!(html.contains("<div>block</div>"), "{html}");
    assert!(build.diagnostics.is_empty(), "{:?}", build.diagnostics);
}
