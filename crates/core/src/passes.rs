//! Transform passes over a loaded document. They run in the order of [`run`], once per build and
//! independent of the output format.

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::diagnostic::{Diagnostic, Origin};
use crate::download::{Downloader, download_images};
use crate::engine::{CodeEngine, EngineOutput};
use crate::files::Files;
use crate::model::{
    Block, BlockKind, Chapter, Document, Footnote, Inline, Link, Target, block_inlines_mut,
    visit_blocks_mut, visit_chapters_mut, visit_inlines_mut,
};
use crate::workers::Workers;

pub(crate) type EngineCache = Mutex<HashMap<u64, EngineOutput>>;

/// The code engines of a project, the workers that run them and the results of earlier builds.
pub(crate) struct CodeEngines<'a> {
    pub engines: &'a [Box<dyn CodeEngine>],
    pub workers: &'a dyn Workers,
    pub cache: &'a EngineCache,
}

pub(crate) fn run(
    document: &mut Document,
    files: &dyn Files,
    code_engines: &CodeEngines<'_>,
    downloader: &Downloader,
    allow_html: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !allow_html {
        raw_html_as_text(document, diagnostics);
    }
    assign_ids(document, diagnostics);
    number_headings(document);
    number_footnotes(document);
    resolve_links(document, diagnostics);
    collect_assets(document, files, diagnostics);
    download_images(document, downloader, diagnostics);
    run_code_engines(document, code_engines, diagnostics);
}

/// Shows raw HTML as the text it was written as, so that `<file>` in a sentence stays visible
/// instead of reaching a page as markup (G-9). One warning per block that holds any.
fn raw_html_as_text(document: &mut Document, diagnostics: &mut Vec<Diagnostic>) {
    visit_blocks_mut(&mut document.root.blocks, &mut |block| {
        let mut found = false;
        if let BlockKind::Html(html) = &mut block.kind {
            let mut inlines = Vec::new();
            for line in html.trim_end().lines() {
                if !inlines.is_empty() {
                    inlines.push(Inline::LineBreak);
                }
                inlines.push(Inline::Text(line.to_string()));
            }
            block.kind = BlockKind::Paragraph(inlines);
            found = true;
        }
        block_inlines_mut(block, &mut |inlines| {
            visit_inlines_mut(inlines, &mut |inline| {
                if let Inline::Html(html) = inline {
                    *inline = Inline::Text(std::mem::take(html));
                    found = true;
                }
            });
        });
        if found {
            diagnostics.push(Diagnostic::warning(
                "raw HTML shown as text; set html = true in nectar.toml to pass it through",
                Some(block.origin.clone()),
            ));
        }
    });
}

/// Turns a heading text into an id: lower case, words joined by `-`.
pub fn slugify(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    for c in text.trim().chars().flat_map(char::to_lowercase) {
        if c.is_alphanumeric() {
            slug.push(c);
        } else if (c.is_whitespace() || c == '-' || c == '_') && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "section".to_string()
    } else {
        slug.to_string()
    }
}

fn unique(base: String, used: &mut HashSet<String>) -> String {
    let mut candidate = base.clone();
    let mut counter = 1;
    while !used.insert(candidate.clone()) {
        candidate = format!("{base}-{counter}");
        counter += 1;
    }
    candidate
}

/// Gives every heading and chapter a unique id and every chapter a unique slug. Ids written as
/// `{#id}` are kept.
fn assign_ids(document: &mut Document, diagnostics: &mut Vec<Diagnostic>) {
    let mut used = HashSet::new();
    visit_blocks_mut(&mut document.root.blocks, &mut |block| {
        if let BlockKind::Heading(heading) = &block.kind
            && !heading.id.is_empty()
            && !used.insert(heading.id.clone())
        {
            diagnostics.push(Diagnostic::warning(
                format!("heading id '{}' is used more than once", heading.id),
                Some(block.origin.clone()),
            ));
        }
    });
    visit_blocks_mut(&mut document.root.blocks, &mut |block| {
        if let BlockKind::Heading(heading) = &mut block.kind
            && heading.id.is_empty()
        {
            heading.id = unique(slugify(&Inline::plain_text(&heading.content)), &mut used);
        }
    });

    let mut slugs = HashSet::new();
    visit_chapters_mut(&mut document.root, &mut |chapter| {
        let first_heading = chapter.blocks.iter().find_map(|block| match &block.kind {
            BlockKind::Heading(heading) => Some(heading.id.clone()),
            _ => None,
        });
        chapter.id = match first_heading {
            Some(id) => id,
            None => unique(slugify(&chapter.title), &mut used),
        };
        let slug = if chapter.slug.is_empty() {
            slugify(&chapter.title)
        } else {
            slugify(&chapter.slug)
        };
        chapter.slug = unique(slug, &mut slugs);
    });
}

/// Numbers the headings of the chapters in reading order: `1.`, `1.2.` and so on below the
/// top level, which stays unnumbered. Below a skipped level, numbers continue in Roman numerals.
fn number_headings(document: &mut Document) {
    if document.numbered {
        number_chapter_headings(&mut document.root, &mut Vec::new());
    }
}

fn number_chapter_headings(chapter: &mut Chapter, counters: &mut Vec<u32>) {
    for block in &mut chapter.blocks {
        match &mut block.kind {
            BlockKind::Chapter(child) => number_chapter_headings(child, counters),
            BlockKind::Heading(heading) => {
                let level = usize::from(heading.level);
                counters.truncate(level);
                if counters.len() < level {
                    counters.resize(level - 1, 0);
                    counters.push(1);
                } else if let Some(last) = counters.last_mut() {
                    *last += 1;
                }
                heading.number = section_number(counters);
            }
            _ => {}
        }
    }
}

fn section_number(counters: &[u32]) -> String {
    let mut number = String::new();
    let mut roman = false;
    for &counter in counters.iter().skip(1) {
        if counter == 0 {
            number.clear();
            roman = true;
        } else if roman {
            number.push_str(&to_roman(counter));
            number.push('.');
        } else {
            number.push_str(&counter.to_string());
            number.push('.');
        }
    }
    number
}

fn to_roman(mut value: u32) -> String {
    const NUMERALS: [(u32, &str); 13] = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut roman = String::new();
    for (amount, numeral) in NUMERALS {
        while value >= amount {
            roman.push_str(numeral);
            value -= amount;
        }
    }
    roman
}

/// A footnote created for a reference link: number, link title, URL and where the link stands.
type LinkNote = (usize, String, String, Origin);

/// Numbers footnote references and reference links to external targets in reading order, and
/// moves the footnotes to the document. Labels are scoped to the file they are written in; links
/// to the same URL in one file share a footnote.
fn number_footnotes(document: &mut Document) {
    let mut numbers: HashMap<(PathBuf, String), usize> = HashMap::new();
    let mut link_notes = Vec::new();
    number_chapter(&mut document.root, &mut numbers, &mut link_notes);

    let mut footnotes = Vec::new();
    visit_chapters_mut(&mut document.root, &mut |chapter| {
        for definition in std::mem::take(&mut chapter.footnotes) {
            if let Some(number) = numbers.get(&(chapter.source.clone(), definition.label)) {
                footnotes.push(Footnote {
                    number: *number,
                    blocks: definition.blocks,
                });
            }
        }
    });
    for (number, title, url, origin) in link_notes {
        let mut content = Vec::new();
        if !title.is_empty() {
            content.push(Inline::Text(format!("{title}: ")));
        }
        content.push(Inline::Link(Link {
            target: Target::External(url.clone()),
            title: String::new(),
            content: vec![Inline::Text(url)],
            reference: false,
            footnote: 0,
        }));
        footnotes.push(Footnote {
            number,
            blocks: vec![Block {
                origin,
                kind: BlockKind::Paragraph(content),
            }],
        });
    }
    footnotes.sort_by_key(|footnote| footnote.number);
    document.footnotes = footnotes;
}

fn number_chapter(
    chapter: &mut Chapter,
    numbers: &mut HashMap<(PathBuf, String), usize>,
    link_notes: &mut Vec<LinkNote>,
) {
    let source = chapter.source.clone();
    for block in &mut chapter.blocks {
        if let BlockKind::Chapter(child) = &mut block.kind {
            number_chapter(child, numbers, link_notes);
            continue;
        }
        visit_blocks_mut(std::slice::from_mut(block), &mut |block| {
            let origin = block.origin.clone();
            block_inlines_mut(block, &mut |inlines| {
                visit_inlines_mut(inlines, &mut |inline| match inline {
                    Inline::FootnoteRef { label, number } => {
                        let next = numbers.len() + 1;
                        *number = *numbers
                            .entry((source.clone(), label.clone()))
                            .or_insert(next);
                    }
                    Inline::Link(link) if link.reference => {
                        let Target::External(url) = &link.target else {
                            return;
                        };
                        let next = numbers.len() + 1;
                        // A NUL cannot occur in a footnote label, so link keys never collide.
                        let key = (source.clone(), format!("\0{url}"));
                        link.footnote = *numbers.entry(key).or_insert_with(|| {
                            link_notes.push((
                                next,
                                link.title.clone(),
                                url.clone(),
                                origin.clone(),
                            ));
                            next
                        });
                    }
                    _ => {}
                });
            });
        });
    }
}

/// Resolves anchors by id, by slug or by heading text, and turns links to chapters into anchors.
fn resolve_links(document: &mut Document, diagnostics: &mut Vec<Diagnostic>) {
    let mut ids = HashSet::new();
    let mut by_slug = HashMap::new();
    let mut by_text = HashMap::new();
    visit_blocks_mut(&mut document.root.blocks, &mut |block| {
        if let BlockKind::Heading(heading) = &block.kind {
            let text = Inline::plain_text(&heading.content);
            ids.insert(heading.id.clone());
            by_slug
                .entry(slugify(&text))
                .or_insert_with(|| heading.id.clone());
            by_text
                .entry(text.trim().to_lowercase())
                .or_insert_with(|| heading.id.clone());
        }
    });
    let mut chapters = HashMap::new();
    visit_chapters_mut(&mut document.root, &mut |chapter| {
        ids.insert(chapter.id.clone());
        chapters.insert(chapter.source.clone(), chapter.id.clone());
    });

    let find = |fragment: &str| -> Option<String> {
        if ids.contains(fragment) {
            return Some(fragment.to_string());
        }
        by_slug
            .get(&slugify(fragment))
            .or_else(|| by_text.get(&fragment.trim().to_lowercase()))
            .cloned()
    };

    let mut resolve = |block: &mut Block| {
        let origin = block.origin.clone();
        block_inlines_mut(block, &mut |inlines| {
            visit_inlines_mut(inlines, &mut |inline| {
                let Inline::Link(link) = inline else { return };
                let resolved = match &link.target {
                    Target::Anchor(fragment) if fragment.is_empty() => None,
                    Target::Anchor(fragment) => match find(fragment) {
                        Some(id) => Some(Target::Anchor(id)),
                        None => {
                            diagnostics.push(Diagnostic::warning(
                                format!("no heading for '#{fragment}'"),
                                Some(origin.clone()),
                            ));
                            None
                        }
                    },
                    Target::File { path, fragment } => match chapters.get(path) {
                        Some(chapter_id) => {
                            let id = match fragment.as_deref().map(|f| (f, find(f))) {
                                None => chapter_id.clone(),
                                Some((_, Some(id))) => id,
                                Some((fragment, None)) => {
                                    diagnostics.push(Diagnostic::warning(
                                        format!(
                                            "no heading for '#{fragment}' in {}",
                                            path.display()
                                        ),
                                        Some(origin.clone()),
                                    ));
                                    chapter_id.clone()
                                }
                            };
                            Some(Target::Anchor(id))
                        }
                        None => {
                            diagnostics.push(Diagnostic::warning(
                                format!("{} is not part of the document", path.display()),
                                Some(origin.clone()),
                            ));
                            Some(Target::Asset(path.clone()))
                        }
                    },
                    _ => None,
                };
                if let Some(target) = resolved {
                    link.target = target;
                }
            });
        });
    };
    visit_blocks_mut(&mut document.root.blocks, &mut resolve);
    for footnote in &mut document.footnotes {
        visit_blocks_mut(&mut footnote.blocks, &mut resolve);
    }
}

/// Records every local file that links and images refer to, with its place in the output.
fn collect_assets(document: &mut Document, files: &dyn Files, diagnostics: &mut Vec<Diagnostic>) {
    let base = document
        .root
        .source
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let prefix = PathBuf::from(&document.path);
    let mut assets = std::mem::take(&mut document.assets);

    let mut collect = |block: &mut Block| {
        let origin = block.origin.clone();
        block_inlines_mut(block, &mut |inlines| {
            visit_inlines_mut(inlines, &mut |inline| {
                let target = match inline {
                    Inline::Link(link) => &link.target,
                    Inline::Image(image) => &image.target,
                    _ => return,
                };
                let Target::Asset(path) = target else { return };
                if assets.contains_key(path) {
                    return;
                }
                // A path that leads outside the root is kept, so that reading reports it.
                if matches!(files.is_file(path), Ok(false)) {
                    diagnostics.push(Diagnostic::warning(
                        format!("{} does not exist", path.display()),
                        Some(origin.clone()),
                    ));
                    return;
                }
                let output = match path.strip_prefix(&base) {
                    Ok(relative) => prefix.join(relative),
                    Err(_) => prefix.join("_assets").join(path),
                };
                assets.insert(path.clone(), output);
            });
        });
    };
    visit_blocks_mut(&mut document.root.blocks, &mut collect);
    for footnote in &mut document.footnotes {
        visit_blocks_mut(&mut footnote.blocks, &mut collect);
    }
    document.assets = assets;
}

struct Job<'e> {
    key: u64,
    engine: &'e dyn CodeEngine,
    language: String,
    code: String,
}

/// Hands every code block to the first engine that supports its language. Blocks not in the
/// cache are rendered by the workers, each distinct block once; failed blocks stay code.
fn run_code_engines(
    document: &mut Document,
    code_engines: &CodeEngines<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let CodeEngines {
        engines,
        workers,
        cache,
    } = *code_engines;
    if engines.is_empty() {
        return;
    }
    let engine_for = |language: &str| -> Option<&dyn CodeEngine> {
        if language.is_empty() {
            return None;
        }
        engines
            .iter()
            .find(|engine| engine.supports(language))
            .map(|engine| engine.as_ref())
    };

    let mut jobs: HashMap<u64, Job<'_>> = HashMap::new();
    visit_blocks_mut(&mut document.root.blocks, &mut |block| {
        let BlockKind::Code(code) = &block.kind else {
            return;
        };
        let Some(engine) = engine_for(&code.language) else {
            return;
        };
        let key = cache_key(engine, &code.language, &code.code);
        if !cache.lock().unwrap().contains_key(&key) {
            jobs.entry(key).or_insert_with(|| Job {
                key,
                engine,
                language: code.language.clone(),
                code: code.code.clone(),
            });
        }
    });
    let failures = render(jobs.into_values().collect(), workers, cache);

    visit_blocks_mut(&mut document.root.blocks, &mut |block| {
        let BlockKind::Code(code) = &block.kind else {
            return;
        };
        let Some(engine) = engine_for(&code.language) else {
            return;
        };
        let key = cache_key(engine, &code.language, &code.code);
        if let Some(output) = cache.lock().unwrap().get(&key) {
            block.kind = match output.clone() {
                EngineOutput::Highlighted(highlighted) => BlockKind::HighlightedCode(highlighted),
                EngineOutput::Graphic(graphic) => BlockKind::Graphic(graphic),
            };
        } else if let Some(message) = failures.get(&key) {
            diagnostics.push(Diagnostic::warning(
                format!(
                    "{} could not render the {} block: {message}",
                    engine.name(),
                    code.language
                ),
                Some(block.origin.clone()),
            ));
        }
    });
}

fn cache_key(engine: &dyn CodeEngine, language: &str, code: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    (engine.name(), language, code).hash(&mut hasher);
    hasher.finish()
}

/// Renders the jobs on the workers, stores results in the cache and returns the failures.
fn render(jobs: Vec<Job<'_>>, workers: &dyn Workers, cache: &EngineCache) -> HashMap<u64, String> {
    let failures = Mutex::new(HashMap::new());
    workers.run(jobs.len(), &|index| {
        let job = &jobs[index];
        match job.engine.render(&job.language, &job.code) {
            Ok(output) => {
                cache.lock().unwrap().insert(job.key, output);
            }
            Err(e) => {
                failures.lock().unwrap().insert(job.key, e.to_string());
            }
        }
    });
    failures.into_inner().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::MemoryFiles;
    use crate::workers::InTurn;

    #[test]
    fn slugs() {
        assert_eq!(slugify("Header of Document"), "header-of-document");
        assert_eq!(slugify("  C++ & Rust: 2024!  "), "c-rust-2024");
        assert_eq!(slugify("Überblick"), "überblick");
        assert_eq!(slugify("!!!"), "section");
    }

    fn document(files: &[(&str, &str)]) -> (Document, Vec<Diagnostic>) {
        let project: MemoryFiles = files.iter().copied().collect();
        let vars = std::collections::BTreeMap::new();
        let mut loader = crate::load::Loader::new(&project, &vars, false);
        let root = loader.chapter(Path::new(files[0].0), "", None).unwrap();
        let mut document = Document {
            title: root.title.clone(),
            path: String::new(),
            split: crate::config::Split::Chapter,
            numbered: true,
            contents: true,
            print: toml::Table::new(),
            root,
            footnotes: Vec::new(),
            assets: std::collections::BTreeMap::new(),
            downloads: std::collections::BTreeMap::new(),
        };
        let mut diagnostics = loader.diagnostics;
        run(
            &mut document,
            &project,
            &CodeEngines {
                engines: &[],
                workers: &InTurn,
                cache: &Mutex::new(HashMap::new()),
            },
            &Downloader::new(),
            false,
            &mut diagnostics,
        );
        (document, diagnostics)
    }

    fn numbers(document: &mut Document) -> Vec<(String, String)> {
        let mut found = Vec::new();
        visit_blocks_mut(&mut document.root.blocks, &mut |block| {
            if let BlockKind::Heading(heading) = &block.kind {
                found.push((heading.number.clone(), Inline::plain_text(&heading.content)));
            }
        });
        found
    }

    #[test]
    fn headings_are_numbered_across_chapters() {
        let (mut document, _) = document(&[
            (
                "main.md",
                "# Manual\n\n## Intro\n\n[Setup](setup.md)\n\n## Next\n\n#### Skipped\n",
            ),
            ("setup.md", "# Setup\n\n## Linux\n"),
        ]);
        let found = numbers(&mut document);
        let expected = [
            ("", "Manual"),
            ("1.", "Intro"),
            ("1.1.", "Setup"),
            ("1.1.1.", "Linux"),
            ("2.", "Next"),
            ("I.", "Skipped"),
        ];
        let expected: Vec<(String, String)> = expected
            .iter()
            .map(|(n, t)| (n.to_string(), t.to_string()))
            .collect();
        assert_eq!(found, expected);
    }

    #[test]
    fn reference_links_to_external_targets_become_footnotes() {
        let (document, diagnostics) = document(&[(
            "main.md",
            "# M\n\nSee [spec][s], note[^n], [again][s] and [][t].\n\n\
             [s]: https://spec.org \"Spec\"\n[t]: https://t.org\n[^n]: A note.\n",
        )]);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let BlockKind::Paragraph(inlines) = &document.root.blocks[1].kind else {
            panic!("expected a paragraph");
        };
        let notes: Vec<usize> = inlines
            .iter()
            .filter_map(|inline| match inline {
                Inline::Link(link) => Some(link.footnote),
                Inline::FootnoteRef { number, .. } => Some(*number),
                _ => None,
            })
            .collect();
        assert_eq!(notes, [1, 2, 1, 3]);
        assert_eq!(document.footnotes.len(), 3);
        let BlockKind::Paragraph(first) = &document.footnotes[0].blocks[0].kind else {
            panic!("expected a paragraph");
        };
        assert_eq!(Inline::plain_text(first), "Spec: https://spec.org");
    }

    #[test]
    fn roman_numerals() {
        assert_eq!(to_roman(4), "IV");
        assert_eq!(to_roman(1994), "MCMXCIV");
    }
}
