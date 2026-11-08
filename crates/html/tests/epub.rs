//! Builds projects from files in memory into EPUBs and reads the container's bytes.
//!
//! Verifies: G-13, G-4, G-9, ADR-draft-epub-renderer, ADR-draft-epub-zip-writer

use std::collections::BTreeMap;
use std::path::Path;

use nectar_core::model::Graphic;
use nectar_core::{Build, CodeEngine, EngineError, EngineOutput, MemoryFiles, Project, Severity};
use nectar_html::{DEFAULT_MODIFIED, EpubRenderer, HtmlRenderer, PrintRenderer};

/// A valid PNG of one pixel; every project has it as `logo.png`.
const PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0xF8, 0xCF, 0xC0, 0xF0,
    0x1F, 0x00, 0x05, 0x00, 0x01, 0xFF, 0x89, 0x99, 0x3D, 0x1D, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
    0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];
const GRAPHIC: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1\" height=\"1\"/>";

const ENABLED: &str = "title = \"Site\"\n\n[[document]]\nsource = \"main.md\"\n\n[document.epub]\nenabled = true\nlanguage = \"de-AT\"\n";

struct Boxes;

impl CodeEngine for Boxes {
    fn name(&self) -> &str {
        "boxes"
    }

    fn supports(&self, language: &str) -> bool {
        language == "boxes"
    }

    fn render(&self, language: &str, code: &str) -> Result<EngineOutput, EngineError> {
        // A text that names a script draws one, as a program named in the configuration may.
        let svg = if code.contains("scripted") {
            "<svg xmlns=\"http://www.w3.org/2000/svg\"><script>x()</script></svg>"
        } else {
            GRAPHIC
        };
        Ok(EngineOutput::Graphic(Graphic {
            language: language.to_string(),
            svg: svg.to_string(),
            alt: "boxes".to_string(),
            code: code.to_string(),
        }))
    }
}

fn project(files: &[(&str, &str)]) -> Project {
    let root = MemoryFiles::new();
    root.insert("logo.png", PNG);
    for (file, text) in files {
        root.insert(file, *text);
    }
    let mut project = Project::load(Box::new(root)).unwrap();
    project.set_offline(true);
    project.add_engine(Box::new(Boxes));
    project
}

fn build(files: &[(&str, &str)]) -> Build {
    project(files).build(&[&EpubRenderer::default()])
}

fn warnings(build: &Build) -> Vec<String> {
    build
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .map(ToString::to_string)
        .collect()
}

struct Entry {
    name: String,
    method: u16,
    flags: u16,
    time: u16,
    date: u16,
    needed: u16,
    crc: u32,
    extra: u16,
    data: Vec<u8>,
}

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

/// The entries in the order of the archive, each read from its local header, and the same from
/// the central directory, which must agree with it.
fn entries(bytes: &[u8]) -> Vec<Entry> {
    let end = bytes.len() - 22;
    assert_eq!(u32_at(bytes, end), 0x0605_4b50, "the end of the archive");
    let count = usize::from(u16_at(bytes, end + 10));
    let mut central = u32_at(bytes, end + 16) as usize;
    let mut found = Vec::new();
    for _ in 0..count {
        assert_eq!(u32_at(bytes, central), 0x0201_4b50);
        let name_length = usize::from(u16_at(bytes, central + 28));
        let local = u32_at(bytes, central + 42) as usize;
        assert_eq!(u32_at(bytes, local), 0x0403_4b50);
        let packed_size = u32_at(bytes, local + 18) as usize;
        let local_name = usize::from(u16_at(bytes, local + 26));
        let extra = u16_at(bytes, local + 28);
        assert_eq!(u16_at(bytes, central + 30), 0, "extra field, central");
        // The central header repeats the local one from `version needed`.
        assert_eq!(
            bytes[central + 6..central + 28],
            bytes[local + 4..local + 26]
        );
        let start = local + 30 + local_name + usize::from(extra);
        let packed = &bytes[start..start + packed_size];
        let method = u16_at(bytes, local + 8);
        let data = match method {
            0 => packed.to_vec(),
            8 => miniz_oxide::inflate::decompress_to_vec(packed).unwrap(),
            other => panic!("method {other}"),
        };
        assert_eq!(data.len(), u32_at(bytes, local + 22) as usize);
        found.push(Entry {
            name: String::from_utf8(bytes[local + 30..local + 30 + local_name].to_vec()).unwrap(),
            method,
            flags: u16_at(bytes, local + 6),
            time: u16_at(bytes, local + 10),
            date: u16_at(bytes, local + 12),
            needed: u16_at(bytes, local + 4),
            crc: u32_at(bytes, local + 14),
            extra,
            data,
        });
        central += 46 + name_length;
    }
    found
}

fn contents(build: &Build, file: &str) -> BTreeMap<String, Vec<u8>> {
    let bytes = build
        .files
        .get(Path::new(file))
        .unwrap_or_else(|| panic!("no {file}: {:?}", build.diagnostics));
    entries(bytes)
        .into_iter()
        .map(|e| (e.name, e.data))
        .collect()
}

fn text(entries: &BTreeMap<String, Vec<u8>>, name: &str) -> String {
    String::from_utf8(entries[name].clone()).unwrap()
}

/// The text of every content document, the navigation document and the cover aside.
fn chapters(entries: &BTreeMap<String, Vec<u8>>) -> String {
    entries
        .iter()
        .filter(|(name, _)| name.starts_with("OEBPS/text/c0"))
        .map(|(_, bytes)| String::from_utf8(bytes.clone()).unwrap())
        .collect()
}

fn well_formed(name: &str, source: &str) {
    let options = roxmltree::ParsingOptions {
        allow_dtd: true,
        ..Default::default()
    };
    roxmltree::Document::parse_with_options(source, options)
        .unwrap_or_else(|e| panic!("{name}: {e}\n{source}"));
}

/// Every id a content document's links name is one of its elements or of the document they name,
/// as a reading system resolves them; and no id repeats within a document.
fn every_link_resolves(book: &BTreeMap<String, Vec<u8>>) {
    let options = || roxmltree::ParsingOptions {
        allow_dtd: true,
        ..Default::default()
    };
    let sources: BTreeMap<&String, &str> = book
        .iter()
        .filter(|(name, _)| name.ends_with(".xhtml"))
        .map(|(name, bytes)| (name, std::str::from_utf8(bytes).unwrap()))
        .collect();
    let mut ids: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (name, source) in &sources {
        let parsed = roxmltree::Document::parse_with_options(source, options()).unwrap();
        let found: Vec<String> = parsed
            .descendants()
            .filter_map(|n| n.attribute("id").map(str::to_string))
            .collect();
        let distinct: std::collections::BTreeSet<&String> = found.iter().collect();
        assert_eq!(
            distinct.len(),
            found.len(),
            "a repeated id in {name}: {found:?}"
        );
        ids.insert((*name).clone(), found);
    }
    for (name, source) in &sources {
        let parsed = roxmltree::Document::parse_with_options(source, options()).unwrap();
        for href in parsed.descendants().filter_map(|n| n.attribute("href")) {
            if href.contains(':') || href.ends_with(".css") {
                continue;
            }
            let (file, fragment) = href.split_once('#').unwrap_or((href, ""));
            let dir = Path::new(name.as_str()).parent().unwrap();
            let target = if file.is_empty() {
                (*name).clone()
            } else {
                let joined = dir.join(file).to_string_lossy().into_owned();
                joined.replace("text/../", "")
            };
            let known = ids
                .get(&target)
                .unwrap_or_else(|| panic!("{name} links to {target}, which is no document"));
            assert!(
                fragment.is_empty() || known.iter().any(|id| id == fragment),
                "{name} links to {href}, which no element carries"
            );
        }
    }
}

#[test]
fn a_document_without_the_setting_gives_no_epub() {
    for table in ["", "\n[document.epub]\nenabled = false\n"] {
        let toml = format!("[[document]]\nsource = \"main.md\"\n{table}");
        let build = build(&[("nectar.toml", &toml), ("main.md", "# Main\n")]);
        assert!(!build.has_errors(), "{:?}", build.diagnostics);
        assert!(
            build
                .files
                .keys()
                .all(|f| f.extension().is_none_or(|e| e != "epub")),
            "{table}"
        );
    }
}

#[test]
fn the_website_stays_byte_identical_and_links_only_an_enabled_epub() {
    let sources = [
        ("docs/part.md", "# Part\n"),
        (
            "main.md",
            "# Main\n\nLine  \nbreak[^n]\n\n***\n\n- [x] done\n- [ ] open\n\n|a|\n|-|\n|b|\n\n\
             ![logo](logo.png){width=10}\n\n```boxes\nab\n```\n\n@settings@\n\n\
             [^n]: A note.\n\n[Part](docs/part.md)\n",
        ),
    ];
    let plain = "[[document]]\nsource = \"main.md\"\n";
    let site = |toml: &str, epub: bool| {
        let mut files = sources.to_vec();
        files.push(("nectar.toml", toml));
        let web = HtmlRenderer {
            print_view: false,
            pdf: false,
            epub,
        };
        project(&files).build(&[&web])
    };
    let (without, with) = (site(plain, false), site(plain, true));
    assert_eq!(without.files, with.files, "no document enables the EPUB");
    let page = String::from_utf8(without.files[Path::new("index.html")].clone()).unwrap();
    assert!(!page.contains("EPUB"), "{page}");
    // HTML as the website had it before the EPUB: the forms the mode for XML writes otherwise.
    let html_forms = [
        "<br>",
        "<hr>",
        "<input type=\"checkbox\" disabled checked> ",
        "<input type=\"checkbox\" disabled> ",
        "<col style=\"width:100.0%\">",
        "<img src=\"logo.png\" alt=\"logo\" style=\"width:10px;\">",
        "<img src=\"_graphics/",
        "<span class=\"symbol\" aria-hidden=\"true\">settings</span>",
        "<a href=\"#fn-1\" id=\"fnref-1\">1</a>",
        "<li id=\"fn-1\" value=\"1\">",
        "href=\"#fnref-1\"",
    ];
    for form in html_forms {
        assert!(page.contains(form), "{form}: {page}");
    }
    // The print view writes the same block writer's HTML, with the footnote in place.
    let mut files = sources.to_vec();
    files.push(("nectar.toml", plain));
    let print = project(&files).build(&[&PrintRenderer]);
    let page = String::from_utf8(print.files[Path::new("print/index.html")].clone()).unwrap();
    for form in [
        "<br>",
        "<hr>",
        "<input type=\"checkbox\" disabled checked> ",
        "<col style=\"width:100.0%\">",
        "<span class=\"footnote\">",
        "<span class=\"symbol\" aria-hidden=\"true\">settings</span>",
    ] {
        assert!(page.contains(form), "{form}: {page}");
    }

    let enabled = site(
        &format!("{plain}\n[document.epub]\nenabled = true\nlanguage = \"en\"\n"),
        true,
    );
    let page = String::from_utf8(enabled.files[Path::new("index.html")].clone()).unwrap();
    assert!(
        page.contains("href=\"epub&#x2f;main.epub\" title=\"EPUB\""),
        "{page}"
    );
}

#[test]
fn settings_that_do_not_hold_are_errors_and_give_no_epub() {
    for (table, expected) in [
        ("enabled = true\n", "needs a `language`"),
        ("enabled = true\nlanguage = \"\"\n", "no language tag"),
        (
            "enabled = true\nlanguage = \"not a tag\"\n",
            "no language tag",
        ),
        ("enabled = true\nlanguage = \"en-a\"\n", "no language tag"),
        (
            "enabled = true\nlanguage = \"en\"\nidentifier = \"\"\n",
            "`identifier` is empty",
        ),
        (
            "enabled = true\nlanguage = \"en\"\nidentifier = \"  \"\n",
            "`identifier` is empty",
        ),
        (
            "enabled = true\nlanguage = \"en\"\nauthor = \"x\"\n",
            "author",
        ),
    ] {
        let toml = format!("[[document]]\nsource = \"main.md\"\n\n[document.epub]\n{table}");
        let build = build(&[("nectar.toml", &toml), ("main.md", "# Main\n")]);
        assert!(build.has_errors(), "{table}");
        let shown: Vec<String> = build.diagnostics.iter().map(ToString::to_string).collect();
        assert!(
            shown.iter().any(|d| d.contains(expected)),
            "{table}: {shown:?}"
        );
        assert!(
            !build.files.contains_key(Path::new("epub/main.epub")),
            "{table}"
        );
    }
}

#[test]
fn the_publication_carries_the_required_metadata_and_a_full_manifest() {
    let build = build(&[
        ("nectar.toml", ENABLED),
        ("logo.svg", "<svg xmlns=\"http://www.w3.org/2000/svg\"/>"),
        (
            "main.md",
            "# Main & more\n\n![logo](logo.svg)\n\n```boxes\nab\n```\n\n[Part](part.md)\n\n[Silent](silent.md)\n",
        ),
        (
            "part.md",
            "# Part\n\n## Details\n\n### Deep\n\n[Inner](inner.md)\n",
        ),
        ("inner.md", "# Inner\n"),
        ("silent.md", "No heading.\n"),
    ]);
    assert!(!build.has_errors(), "{:?}", build.diagnostics);
    let book = contents(&build, "epub/main.epub");

    let opf = text(&book, "OEBPS/content.opf");
    let parsed = roxmltree::Document::parse(&opf).unwrap();
    let elements = |name: &str| -> Vec<roxmltree::Node<'_, '_>> {
        parsed
            .descendants()
            .filter(|n| n.has_tag_name(name))
            .collect()
    };
    let value = |name: &str| elements(name)[0].text().unwrap().to_string();
    assert_eq!(value("title"), "Main & more");
    assert_eq!(value("language"), "de-AT");
    assert_eq!(value("identifier"), "nectar:Site:");
    assert_eq!(parsed.root_element().attribute("version"), Some("3.0"));
    let modified: Vec<_> = elements("meta")
        .into_iter()
        .filter(|m| m.attribute("property") == Some("dcterms:modified"))
        .collect();
    assert_eq!(modified.len(), 1);
    assert_eq!(modified[0].text(), Some(DEFAULT_MODIFIED));

    // Every file of the publication is in the manifest, and nothing else.
    let mut manifest: Vec<String> = elements("item")
        .iter()
        .map(|item| format!("OEBPS/{}", item.attribute("href").unwrap()))
        .collect();
    manifest.sort();
    let in_container: Vec<String> = book
        .keys()
        .filter(|name| name.starts_with("OEBPS/") && *name != "OEBPS/content.opf")
        .cloned()
        .collect();
    assert_eq!(manifest, in_container);
    assert_eq!(
        elements("item")
            .iter()
            .filter(|i| i.attribute("properties") == Some("nav"))
            .count(),
        1
    );
    assert!(
        in_container
            .iter()
            .any(|n| n.ends_with(".svg") && n.contains("images/")),
        "the image: {in_container:?}"
    );
    assert!(
        in_container
            .iter()
            .any(|n| n.ends_with(".svg") && n.contains("_graphics/")),
        "the graphic: {in_container:?}"
    );

    // Navigation first, as `contents` is on by default, then the root and one per top chapter.
    let spine: Vec<&str> = elements("itemref")
        .iter()
        .map(|r| r.attribute("idref").unwrap())
        .collect();
    let href = |id: &str| {
        elements("item")
            .iter()
            .find(|i| i.attribute("id") == Some(id))
            .unwrap()
            .attribute("href")
            .unwrap()
            .to_string()
    };
    let order: Vec<String> = spine.iter().map(|id| href(id)).collect();
    assert_eq!(
        order,
        [
            "nav.xhtml",
            "text/c000.xhtml",
            "text/c001.xhtml",
            "text/c002.xhtml"
        ]
    );

    // The navigation document lists every chapter, the deepest and the one without a heading too.
    let nav = text(&book, "OEBPS/nav.xhtml");
    for item in [
        "Main &amp; more",
        "Part",
        "Details",
        "Deep",
        "Inner",
        "silent",
    ] {
        assert!(nav.contains(&format!("{item}</a>")), "{item}: {nav}");
    }
    assert!(nav.contains("epub:type=\"toc\""), "{nav}");

    // Every XML document in the container is well-formed.
    for (name, bytes) in &book {
        if name.ends_with(".xhtml") || name.ends_with(".opf") || name.ends_with(".xml") {
            let source = std::str::from_utf8(bytes).unwrap();
            let options = roxmltree::ParsingOptions {
                allow_dtd: true,
                ..Default::default()
            };
            roxmltree::Document::parse_with_options(source, options)
                .unwrap_or_else(|e| panic!("{name}: {e}\n{source}"));
        }
    }
}

#[test]
fn every_kind_of_block_is_well_formed_xml() {
    let build = build(&[
        ("nectar.toml", ENABLED),
        (
            "main.md",
            "# Main\n\nLine  \nbreak[^note]\n\n***\n\n- [x] done\n- [ ] open\n\n|a|b|\n|-|-|\n|`c`|d|\n\n\
             ![logo](logo.png \"A title\"){width=10}\n\n```\n<&>\n```\n\n[^note]: A note.\n",
        ),
    ]);
    assert!(!build.has_errors(), "{:?}", build.diagnostics);
    let book = contents(&build, "epub/main.epub");
    let page = text(&book, "OEBPS/text/c000.xhtml");
    let options = roxmltree::ParsingOptions {
        allow_dtd: true,
        ..Default::default()
    };
    roxmltree::Document::parse_with_options(&page, options)
        .unwrap_or_else(|e| panic!("{e}\n{page}"));
    for expected in [
        "<br />",
        "<hr />",
        "disabled=\"disabled\" checked=\"checked\" />",
        "<col style",
        "src=\"../images/i0000.png\"",
        "id=\"_fn-1\"",
    ] {
        assert!(page.contains(expected), "{expected}: {page}");
    }
    assert!(page.contains("lang=\"de-AT\" xml:lang=\"de-AT\""), "{page}");
}

#[test]
fn what_the_output_cannot_carry_is_left_out_and_reported_with_file_and_line() {
    let toml = format!("html = true\n{ENABLED}");
    let build = build(&[
        ("nectar.toml", toml.as_str()),
        ("old.bmp", "BM"),
        ("data.csv", "a,b"),
        (
            "main.md",
            "# Main\n\n<div>unclosed<br>RAWBLOCK</div>\n\nInline <b>RAWINLINE</b>.\n\n![old alt](old.bmp)\n\n\
             ![remote alt](https://example.org/r.png)\n\n[csv text](data.csv)\n\n@settings@\n",
        ),
    ]);
    assert!(!build.has_errors(), "{:?}", build.diagnostics);
    let page = chapters(&contents(&build, "epub/main.epub"));
    assert!(!page.contains("RAWBLOCK"), "{page}");
    assert!(!page.contains("<b>"), "{page}");
    assert!(!page.contains("<img"), "{page}");
    assert!(!page.contains("data.csv"), "{page}");
    assert!(!page.contains("settings"), "{page}");
    for kept in ["RAWINLINE", "old alt", "remote alt", "csv text"] {
        assert!(page.contains(kept), "{kept}: {page}");
    }

    let warnings = warnings(&build);
    for (line, message) in [
        (3, "raw HTML is left out of the EPUB"),
        (5, "raw HTML is left out of the EPUB"),
        (7, "the image old.bmp is not embedded in the EPUB"),
        (
            9,
            "the image https://example.org/r.png is not embedded in the EPUB",
        ),
        (11, "the link to data.csv is left out of the EPUB"),
        (13, "the symbol settings is left out of the EPUB"),
    ] {
        let expected = format!("main.md:{line}");
        assert!(
            warnings
                .iter()
                .any(|w| w.contains(&expected) && w.contains(message)),
            "{expected} {message}: {warnings:?}"
        );
    }
}

#[test]
fn the_settings_set_the_identifier_the_cover_and_whether_the_contents_are_read() {
    let toml = "[[document]]\nsource = \"main.md\"\ncontents = false\n\n[document.epub]\nenabled = true\n\
                language = \"en\"\nidentifier = \"urn:isbn:1 & 2\"\ncover_image = \"logo.png\"\n";
    let build = build(&[("nectar.toml", toml), ("main.md", "No heading.\n")]);
    assert!(!build.has_errors(), "{:?}", build.diagnostics);
    let book = contents(&build, "epub/main.epub");
    let opf = text(&book, "OEBPS/content.opf");
    assert!(opf.contains(">urn:isbn:1 &amp; 2</dc:identifier>"), "{opf}");
    assert!(
        opf.contains(
            "href=\"images/cover.png\" media-type=\"image/png\" properties=\"cover-image\""
        ),
        "{opf}"
    );
    assert_eq!(book["OEBPS/images/cover.png"], PNG);
    assert!(text(&book, "OEBPS/text/cover.xhtml").contains("../images/cover.png"));
    // The navigation document stays, out of the reading order, and lists the document itself.
    let parsed = roxmltree::Document::parse(&opf).unwrap();
    let refs = parsed
        .descendants()
        .filter(|n| n.has_tag_name("itemref"))
        .count();
    assert_eq!(refs, 2, "the cover and the text: {opf}");
    assert!(text(&book, "OEBPS/nav.xhtml").contains(">main</a>"));
}

#[test]
fn a_cover_image_of_no_core_media_type_or_missing_is_reported() {
    for (image, message) in [
        ("cover.bmp", "no core media type"),
        ("gone.png", "cannot read gone.png"),
    ] {
        let toml = format!(
            "[[document]]\nsource = \"main.md\"\n\n[document.epub]\nenabled = true\nlanguage = \"en\"\ncover_image = \"{image}\"\n"
        );
        let build = build(&[
            ("nectar.toml", &toml),
            ("cover.bmp", "BM"),
            ("main.md", "# Main\n"),
        ]);
        assert!(!build.has_errors(), "{:?}", build.diagnostics);
        let warnings = warnings(&build);
        assert!(warnings.iter().any(|w| w.contains(message)), "{warnings:?}");
        assert!(!contents(&build, "epub/main.epub").contains_key("OEBPS/text/cover.xhtml"));
    }
}

#[test]
fn the_renderer_writes_the_time_it_is_handed() {
    let renderer = EpubRenderer {
        modified: Ok("2026-11-08T12:00:00Z".to_string()),
    };
    let build = project(&[("nectar.toml", ENABLED), ("main.md", "# Main\n")]).build(&[&renderer]);
    let opf = text(&contents(&build, "epub/main.epub"), "OEBPS/content.opf");
    assert!(opf.contains(">2026-11-08T12:00:00Z</meta>"), "{opf}");
}

#[test]
fn the_container_keeps_the_rules_of_an_ocf_zip_archive() {
    let sources = [
        ("nectar.toml", ENABLED),
        (
            "main.md",
            "# Main äöü\n\n![logo](logo.png)\n\n```boxes\nab\n```\n\n[Part](part.md)\n",
        ),
        ("part.md", "# Part\n"),
    ];
    let first = build(&sources);
    let bytes = &first.files[Path::new("epub/main.epub")];
    let entries = entries(bytes);

    let mimetype = &entries[0];
    assert_eq!(mimetype.name, "mimetype");
    assert_eq!(mimetype.method, 0, "not compressed");
    assert_eq!(mimetype.data, b"application/epub+zip");
    assert_eq!(mimetype.needed, 10);
    // At a fixed offset, by which a tool recognises the format.
    assert_eq!(&bytes[30..38], b"mimetype");

    let names: Vec<&str> = entries[1..].iter().map(|e| e.name.as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(names, sorted, "the other entries in sorted order");
    assert!(names.contains(&"META-INF/container.xml"));
    assert!(names.contains(&"OEBPS/content.opf"));

    for entry in &entries {
        assert_eq!(entry.extra, 0, "{}: no extra field", entry.name);
        assert_eq!(entry.flags & 1, 0, "{}: no encryption", entry.name);
        assert_ne!(entry.flags & (1 << 11), 0, "{}: UTF-8 names", entry.name);
        assert_eq!(entry.time, 0, "{}: fixed time", entry.name);
        assert_eq!(entry.date, (1 << 5) | 1, "{}: 1980-01-01", entry.name);
        assert_eq!(
            entry.crc,
            crc32fast::hash(&entry.data),
            "{}: checksum",
            entry.name
        );
        assert!([10, 20, 45].contains(&entry.needed), "{}", entry.name);
    }
    for entry in &entries[1..] {
        assert_eq!(entry.method, 8, "{}: Deflate", entry.name);
    }

    assert_eq!(
        first.files[Path::new("epub/main.epub")],
        build(&sources).files[Path::new("epub/main.epub")]
    );
}

#[test]
fn every_byte_of_an_image_and_a_graphic_comes_from_its_source() {
    let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"2\" height=\"2\"/>";
    let toml = format!("{ENABLED}cover_image = \"logo.png\"\n");
    let build = build(&[
        ("nectar.toml", toml.as_str()),
        ("draw.svg", svg),
        (
            "main.md",
            "# Main\n\n![a](logo.png)\n\n![b](draw.svg)\n\n```boxes\nab\n```\n",
        ),
    ]);
    assert!(!build.has_errors(), "{:?}", build.diagnostics);
    let book = contents(&build, "epub/main.epub");
    let bytes = |suffix: &str, dir: &str| -> Vec<&Vec<u8>> {
        book.iter()
            .filter(|(name, _)| name.starts_with(dir) && name.ends_with(suffix))
            .map(|(_, bytes)| bytes)
            .collect()
    };
    assert_eq!(bytes(".png", "OEBPS/images/i"), [&PNG.to_vec()]);
    assert_eq!(bytes(".png", "OEBPS/images/cover"), [&PNG.to_vec()]);
    assert_eq!(bytes(".svg", "OEBPS/images/i"), [&svg.as_bytes().to_vec()]);
    assert_eq!(
        bytes(".svg", "OEBPS/_graphics/"),
        [&GRAPHIC.as_bytes().to_vec()]
    );
    assert_eq!(text(&book, "mimetype"), "application/epub+zip");
}

#[test]
fn an_image_that_was_downloaded_is_embedded_from_the_downloads() {
    // A `data:` URI is a download that needs no network: its bytes are the text `GIF8`.
    let build = build(&[
        ("nectar.toml", ENABLED),
        (
            "main.md",
            "# Main\n\n![gif](data:image/gif;base64,R0lGOA==)\n",
        ),
    ]);
    assert!(!build.has_errors(), "{:?}", build.diagnostics);
    let book = contents(&build, "epub/main.epub");
    let images: Vec<_> = book
        .iter()
        .filter(|(name, _)| name.starts_with("OEBPS/images/"))
        .collect();
    assert_eq!(images.len(), 1, "{:?}", book.keys());
    assert!(images[0].0.ends_with(".gif"), "{}", images[0].0);
    assert_eq!(images[0].1, b"GIF8");
    assert!(chapters(&book).contains("<img src=\"../images/"));
}

#[test]
fn a_character_xml_does_not_allow_is_left_out_and_reported() {
    let build = build(&[
        ("nectar.toml", ENABLED),
        (
            "main.md",
            "# Main\x0c title\n\nText\n\n```\nterminal \x1b[0m output\x0c\n```\n",
        ),
    ]);
    assert!(!build.has_errors(), "{:?}", build.diagnostics);
    let book = contents(&build, "epub/main.epub");
    for (name, bytes) in &book {
        if name.ends_with(".xhtml") || name.ends_with(".opf") {
            well_formed(name, std::str::from_utf8(bytes).unwrap());
        }
    }
    assert!(chapters(&book).contains("terminal [0m output"));
    let warnings = warnings(&build);
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("main.md:") && w.contains("a character XML does not allow")),
        "{warnings:?}"
    );
}

#[test]
fn a_link_without_a_target_in_the_publication_keeps_only_its_text() {
    let build = build(&[
        ("nectar.toml", ENABLED),
        (
            "main.md",
            "# Main\n\n[to nowhere](#nope), [absolute](/abs/path), [outside](../../o.txt), \
             [spaced](<https://example.org/b c>), [empty]() and [escape](https://e.org/%zz).\n\n\
             [to the web](https://example.org/?a=1&b=2), [mail](mailto:a@example.org).\n\n\
             [silent](silent.md), [in list](part.md#inner).\n\n\
             [Missing](missing.md)\n\n[Part](part.md)\n\n[Silent](silent.md)\n",
        ),
        ("part.md", "# Part\n\n- ## Inner\n\n[Below](below.md)\n"),
        ("below.md", "No heading below.\n"),
        ("silent.md", "No heading.\n\nBack to [below](below.md).\n"),
    ]);
    assert!(!build.has_errors(), "{:?}", build.diagnostics);
    let book = contents(&build, "epub/main.epub");
    every_link_resolves(&book);
    let page = text(&book, "OEBPS/text/c000.xhtml");
    for gone in ["#nope", "/abs/path", "o.txt", "b c", "%zz", "missing.md"] {
        assert!(!page.contains(gone), "{gone}: {page}");
    }
    for text in [
        "to nowhere",
        "absolute",
        "outside",
        "spaced",
        "empty",
        "escape",
    ] {
        assert!(page.contains(text), "{text}: {page}");
    }
    assert!(
        page.contains("href=\"https://example.org/?a=1&amp;b=2\""),
        "{page}"
    );
    assert!(page.contains("href=\"mailto:a@example.org\""), "{page}");
    assert!(page.contains(">silent</a>"), "{page}");
    assert!(page.contains(">in list</a>"), "{page}");
    let warnings = warnings(&build);
    for target in ["#nope", "/abs/path", "../../o.txt"] {
        let message = format!("the link to {target} is left out of the EPUB");
        assert!(
            warnings.iter().any(|w| w.contains(&message)),
            "{target}: {warnings:?}"
        );
    }
    // Every chapter is in the navigation document, the deepest without a heading too.
    let nav = text(&book, "OEBPS/nav.xhtml");
    for item in ["Part", "Inner", "silent", "below"] {
        assert!(nav.contains(&format!("{item}</a>")), "{item}: {nav}");
    }
}

#[test]
fn footnotes_stay_outside_links_and_keep_their_ids_apart() {
    let build = build(&[
        ("nectar.toml", ENABLED),
        (
            "main.md",
            "# Main\n\n## fn 1\n\n[linked[^a]](https://example.org/) and plain[^b].\n\n\
             [^a]: First.\n[^b]: Second, with a third[^c].\n[^c]: Third.\n",
        ),
    ]);
    assert!(!build.has_errors(), "{:?}", build.diagnostics);
    let book = contents(&build, "epub/main.epub");
    every_link_resolves(&book);
    let page = text(&book, "OEBPS/text/c000.xhtml");
    well_formed("page", &page);
    let parsed = roxmltree::Document::parse_with_options(
        &page,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        parsed
            .descendants()
            .filter(|n| n.has_tag_name("a"))
            .all(|a| a.ancestors().skip(1).all(|n| !n.has_tag_name("a"))),
        "an <a> inside an <a>: {page}"
    );
    for note in ["First.", "Second"] {
        assert!(page.contains(note), "{note}: {page}");
    }
    let warnings = warnings(&build);
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("a footnote inside a footnote is left out")),
        "{warnings:?}"
    );
}

#[test]
fn an_image_or_a_graphic_the_publication_cannot_hold_is_refused_and_reported() {
    let svg = |body: &str| format!("<svg xmlns=\"http://www.w3.org/2000/svg\">{body}</svg>");
    let scripted = svg("<script>x()</script>");
    let event = "<svg xmlns=\"http://www.w3.org/2000/svg\" onload=\"x()\"/>".to_string();
    let remote = svg("<image href=\"https://example.org/r.png\"/>");
    let local = svg("<image href=\"other.png\"/>");
    let styled = svg("<rect style=\"fill:url(https://example.org/#a)\"/>");
    let linked = svg("<a href=\"https://example.org/\"><rect fill=\"url(#g)\"/></a>");
    let build = build(&[
        ("nectar.toml", ENABLED),
        ("scripted.svg", &scripted),
        ("event.svg", &event),
        ("remote.svg", &remote),
        ("local.svg", &local),
        ("styled.svg", &styled),
        ("broken.svg", "<svg"),
        ("html.svg", "<html/>"),
        ("linked.svg", &linked),
        ("mislabelled.jpg", "png bytes"),
        ("truncated.webp", "RIFF"),
        (
            "main.md",
            "# Main\n\n![s](scripted.svg)\n\n![e](event.svg)\n\n![r](remote.svg)\n\n\
             ![l](local.svg)\n\n![y](styled.svg)\n\n![b](broken.svg)\n\n![h](html.svg)\n\n\
             ![m](mislabelled.jpg)\n\n![t](truncated.webp)\n\n![fine](linked.svg)\n\n\
             ```boxes\nscripted\n```\n",
        ),
    ]);
    assert!(!build.has_errors(), "{:?}", build.diagnostics);
    let book = contents(&build, "epub/main.epub");
    let images: Vec<&String> = book.keys().filter(|n| n.contains("images/")).collect();
    assert_eq!(images.len(), 1, "only the SVG with a link: {images:?}");
    assert!(!book.keys().any(|n| n.contains("_graphics/")));
    assert!(
        chapters(&book).contains("<code>scripted"),
        "the graphic keeps its code"
    );

    let warnings = warnings(&build);
    for (image, reason) in [
        ("scripted.svg", "it holds a script"),
        ("event.svg", "it holds a script"),
        ("remote.svg", "it refers to something outside itself"),
        ("local.svg", "it refers to something outside itself"),
        ("styled.svg", "it refers to something outside itself"),
        ("broken.svg", "not well-formed"),
        ("html.svg", "no SVG"),
        ("mislabelled.jpg", "its bytes are no jpg"),
        ("truncated.webp", "its bytes are no webp"),
    ] {
        assert!(
            warnings.iter().any(
                |w| w.contains(&format!("the image {image} is not embedded")) && w.contains(reason)
            ),
            "{image}: {warnings:?}"
        );
    }
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("the graphic is not embedded") && w.contains("holds a script")),
        "{warnings:?}"
    );
}

#[test]
fn an_empty_title_falls_back_to_the_stem_of_the_start_document() {
    let toml = format!("html = true\n{ENABLED}");
    for source in ["# \n\nText.\n", "# <b></b>\n\nText.\n"] {
        let build = build(&[("nectar.toml", toml.as_str()), ("main.md", source)]);
        assert!(!build.has_errors(), "{:?}", build.diagnostics);
        let book = contents(&build, "epub/main.epub");
        let opf = text(&book, "OEBPS/content.opf");
        assert!(opf.contains("<dc:title>main</dc:title>"), "{source}: {opf}");
        let nav = text(&book, "OEBPS/nav.xhtml");
        assert!(nav.contains("<title>main</title>"), "{source}: {nav}");
        assert!(nav.contains(">main</a>"), "{source}: {nav}");
        let page = text(&book, "OEBPS/text/c000.xhtml");
        assert!(page.contains("<title>main</title>"), "{source}: {page}");
    }
}

#[test]
fn a_time_handed_over_as_malformed_stops_only_a_document_that_enables_the_output() {
    let renderer = EpubRenderer {
        modified: Err("SOURCE_DATE_EPOCH is malformed: soon".to_string()),
    };
    let enabled = project(&[("nectar.toml", ENABLED), ("main.md", "# Main\n")]).build(&[&renderer]);
    assert!(enabled.has_errors());
    assert!(
        enabled.diagnostics[0]
            .to_string()
            .contains("SOURCE_DATE_EPOCH")
    );
    assert!(!enabled.files.contains_key(Path::new("epub/main.epub")));

    let plain = "[[document]]\nsource = \"main.md\"\n";
    let without = project(&[("nectar.toml", plain), ("main.md", "# Main\n")]).build(&[&renderer]);
    assert!(!without.has_errors(), "{:?}", without.diagnostics);
}

#[test]
fn a_language_tag_keeps_the_order_of_bcp_47() {
    for (tag, holds) in [
        ("en", true),
        ("de-AT", true),
        ("sr-Latn-RS", true),
        ("es-419", true),
        ("zh-cmn-Hans-CN", true),
        ("de-1996", true),
        ("en-a-bbb", true),
        ("en-x-a", true),
        ("x-a", false),
        ("en-a", false),
        ("en-AT-Latn", false),
        ("zh-aaa-bbb-ccc-ddd", false),
        ("en-", false),
        ("e", false),
        ("not a tag", false),
    ] {
        let toml = format!(
            "[[document]]\nsource = \"main.md\"\n\n[document.epub]\nenabled = true\nlanguage = \"{tag}\"\n"
        );
        let build = build(&[("nectar.toml", &toml), ("main.md", "# Main\n")]);
        assert_eq!(!build.has_errors(), holds, "{tag}: {:?}", build.diagnostics);
    }
}

#[test]
fn only_a_url_an_epub_may_name_stays_a_link() {
    let kept = [
        "https://example.org/",
        "http://example.org",
        "https://example.org:8080/a/b?x=1&y=2#top",
        "https://example.org/a_(b)~c,d;e",
        "https://example.org/%C3%A4",
        "https://example.org.:443/",
        "https://example.org/ä",
        "mailto:a@example.org",
    ];
    let dropped = [
        "https://example.org/?x[]=1",
        "https://example.org/[x]",
        "https://example.org/#a#b",
        "https://",
        "https://[bad",
        "https://example.org:port/",
        "https://example.org:65536",
        "https://example..org/",
        "https://.example.org/",
        "https://999.1.1.1/",
        "https://0x1/",
        "http:/a",
        "file:///etc/x",
        "data:text/html,x",
        "javascript:x()",
        "ftp://example.org/",
        "foo:bar",
        "C:/x/y",
        "mailto:nobody",
    ];
    let mut source = String::from("# Main\n\n");
    for (index, url) in kept.iter().chain(dropped.iter()).enumerate() {
        source.push_str(&format!("[link{index}](<{url}>)\n\n"));
    }
    let build = build(&[("nectar.toml", ENABLED), ("main.md", &source)]);
    assert!(!build.has_errors(), "{:?}", build.diagnostics);
    let page = chapters(&contents(&build, "epub/main.epub"));
    for (index, _) in kept.iter().enumerate() {
        assert!(
            page.contains(&format!(">link{index}</a>")),
            "{}: {page}",
            kept[index]
        );
    }
    for (index, url) in dropped.iter().enumerate() {
        let index = index + kept.len();
        assert!(
            page.contains(&format!("<p>link{index}</p>")),
            "{url}: {page}"
        );
    }
}

#[test]
fn an_svg_is_held_to_what_a_chapter_may_name() {
    let svg = |body: &str| format!("<svg xmlns=\"http://www.w3.org/2000/svg\">{body}</svg>");
    let xlink = "xmlns:xlink=\"http://www.w3.org/1999/xlink\"";
    let cases = [
        ("a.svg", svg("<a href=\"other.html\"><rect/></a>"), false),
        (
            "b.svg",
            format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" {xlink}><a xlink:href=\"other.html\"><rect/></a></svg>"
            ),
            false,
        ),
        (
            "c.svg",
            svg("<a href=\"javascript:x()\"><rect/></a>"),
            false,
        ),
        (
            "d.svg",
            format!("<?xml-stylesheet href=\"https://e.org/s.css\"?>{}", svg("")),
            false,
        ),
        (
            "e.svg",
            svg("<rect style=\"fill:URL(https://e.org/#a)\"/>"),
            false,
        ),
        (
            "f.svg",
            svg("<style>/* a */<![CDATA[]]>@IMPORT 'x.css';</style>"),
            false,
        ),
        (
            "g.svg",
            svg(
                "<foreignObject><img xmlns=\"http://www.w3.org/1999/xhtml\" src=\"https://e.org/r.png\"/></foreignObject>",
            ),
            false,
        ),
        ("i.svg", format!("<?plantuml 1?>{}", svg("")), true),
        (
            "h.svg",
            svg(
                "<a href=\"https://example.org/\"><rect one=\"1\" fill=\"url(#g)\"/></a><a href=\"#own\"/>",
            ),
            true,
        ),
    ];
    let mut files: Vec<(&str, &str)> = vec![("nectar.toml", ENABLED)];
    let mut source = String::from("# Main\n\n");
    for (name, content, _) in &cases {
        files.push((name, content));
        source.push_str(&format!("![{name}]({name})\n\n"));
    }
    files.push(("main.md", &source));
    let build = build(&files);
    assert!(!build.has_errors(), "{:?}", build.diagnostics);
    let warnings = warnings(&build);
    for (name, _, holds) in &cases {
        let refused = warnings
            .iter()
            .any(|w| w.contains(&format!("the image {name} is not embedded")));
        assert_eq!(!refused, *holds, "{name}: {warnings:?}");
    }
}

#[test]
fn a_heading_writes_no_id_an_epub_cannot_hold() {
    let build = build(&[
        ("nectar.toml", ENABLED),
        (
            "main.md",
            "# Main\n\n## A {#x}\n\n## B {#x}\n\n## C {#_fn-1}\n\n## D {#100%}\n\n## E {#a%20b}\n\n\
             See [a](#x), [c](#_fn-1), [d](#100%), [e](#a%20b), [deep](deep.md), a note[^n].\n\n\
             [^n]: ## A heading in a note\n\n[Part](part.md)\n",
        ),
        (
            "part.md",
            "# Part\n\n[Deep](deep.md)\n\n[Inner](inner.md)\n\nA note[^m].\n\n[^m]: ## H {#inner}\n",
        ),
        ("deep.md", "# Deep {#_fnord}\n"),
        ("inner.md", "No heading.\n"),
    ]);
    assert!(!build.has_errors(), "{:?}", build.diagnostics);
    let book = contents(&build, "epub/main.epub");
    every_link_resolves(&book);
    let page = text(&book, "OEBPS/text/c000.xhtml");
    assert!(!page.contains("id=\"\""), "{page}");
    assert_eq!(page.matches("id=\"x\"").count(), 1, "{page}");
    assert_eq!(page.matches("id=\"_fn-1\"").count(), 1, "{page}");
    let warnings = warnings(&build);
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("the id x of a heading is left out")),
        "{warnings:?}"
    );
}
