//! Loads a start document and merges every include into one document tree.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::diagnostic::{Diagnostic, Origin};
use crate::files::{Files, read_text};
use crate::model::{Block, BlockKind, Chapter, Code, Heading, Inline, Target, visit_blocks_mut};
use crate::parse::{Item, parse_file};

pub(crate) struct Loader<'a> {
    pub files: &'a dyn Files,
    pub vars: &'a BTreeMap<String, String>,
    pub include_drafts: bool,
    /// Every file read, relative to the project root.
    pub sources: BTreeSet<PathBuf>,
    pub diagnostics: Vec<Diagnostic>,
    /// Files currently being loaded, to detect include cycles.
    stack: Vec<PathBuf>,
}

impl<'a> Loader<'a> {
    pub fn new(
        files: &'a dyn Files,
        vars: &'a BTreeMap<String, String>,
        include_drafts: bool,
    ) -> Self {
        Self {
            files,
            vars,
            include_drafts,
            sources: BTreeSet::new(),
            diagnostics: Vec::new(),
            stack: Vec::new(),
        }
    }

    /// Loads `file` (relative to the project root) with its includes. `None` when the file
    /// cannot be read or is a draft that is left out.
    pub fn chapter(&mut self, file: &Path, title: &str, origin: Option<Origin>) -> Option<Chapter> {
        if self.stack.iter().any(|open| open == file) {
            self.diagnostics.push(Diagnostic::error(
                format!("include cycle: {} includes itself", file.display()),
                origin,
            ));
            return None;
        }
        let text = match read_text(self.files, file) {
            Ok(text) => text,
            Err(e) => {
                self.diagnostics.push(Diagnostic::error(
                    format!("cannot read {}: {e}", file.display()),
                    origin,
                ));
                return None;
            }
        };
        self.sources.insert(file.to_path_buf());

        let parsed = parse_file(file, &text, self.vars);
        self.diagnostics.extend(parsed.diagnostics);
        if parsed.front.draft && !self.include_drafts {
            return None;
        }

        self.stack.push(file.to_path_buf());
        let mut blocks = Vec::new();
        // Level of the last heading of this file; includes without a heading marker go below it.
        let mut level = 0;
        for item in parsed.items {
            match item {
                Item::Block(block) => {
                    if let BlockKind::Heading(heading) = &block.kind {
                        level = heading.level;
                    }
                    blocks.push(block);
                }
                Item::Include(include) => {
                    // A path that leads outside the root is loaded, so that reading reports it.
                    if matches!(self.files.is_file(&include.path), Ok(false)) {
                        self.diagnostics.push(Diagnostic::warning(
                            format!("included file {} does not exist", include.path.display()),
                            Some(include.fallback.origin.clone()),
                        ));
                        blocks.push(reported(include.fallback));
                        continue;
                    }
                    let origin = include.fallback.origin.clone();
                    let base = include
                        .level
                        .map_or(level, |marker| marker.saturating_sub(1));
                    if let Some(mut child) =
                        self.chapter(&include.path, &include.title, Some(origin.clone()))
                    {
                        shift_headings(&mut child.blocks, base);
                        blocks.push(Block {
                            origin,
                            kind: BlockKind::Chapter(child),
                        });
                    }
                }
            }
        }
        self.stack.pop();
        self.embed_files(&mut blocks);

        if !title.is_empty() {
            replace_first_title(&mut blocks, title);
        }
        let title = parsed
            .front
            .title
            .filter(|t| !t.is_empty())
            .or_else(|| first_heading(&blocks).map(|heading| Inline::plain_text(&heading.content)))
            .unwrap_or_else(|| {
                file.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            });

        Some(Chapter {
            id: String::new(),
            slug: parsed.front.slug.unwrap_or_default(),
            title,
            source: file.to_path_buf(),
            blocks,
            footnotes: parsed.footnotes,
        })
    }
}

/// The link that stands in for a missing include. It points to nowhere, which is reported once
/// here, so the link and asset passes leave it alone.
fn reported(mut block: Block) -> Block {
    crate::model::block_inlines_mut(&mut block, &mut |inlines| {
        crate::model::visit_inlines_mut(inlines, &mut |inline| {
            if let Inline::Link(link) = inline
                && let Target::File { path, .. } = &link.target
            {
                link.target = Target::External(path.to_string_lossy().into_owned());
            }
        });
    });
    block
}

impl Loader<'_> {
    /// Replaces an image that stands alone in its paragraph and points to a file that is not a
    /// picture, such as `![Main](src/Main.java)` or `![](grammar.railroad)`, with the file's
    /// content as code. The code engines then highlight or draw it; the image text becomes the
    /// caption.
    fn embed_files(&mut self, blocks: &mut [Block]) {
        let files = self.files;
        let sources = &mut self.sources;
        let diagnostics = &mut self.diagnostics;
        visit_blocks_mut(blocks, &mut |block| {
            let BlockKind::Paragraph(inlines) = &block.kind else {
                return;
            };
            let [Inline::Image(image)] = inlines.as_slice() else {
                return;
            };
            let Target::Asset(path) = &image.target else {
                return;
            };
            let Some(extension) = path.extension().map(|e| e.to_string_lossy().to_lowercase())
            else {
                return;
            };
            if PICTURES.contains(&extension.as_str()) {
                return;
            }
            let code = match read_text(files, path) {
                Ok(code) => code,
                Err(e) => {
                    diagnostics.push(Diagnostic::warning(
                        format!("cannot embed {}: {e}", path.display()),
                        Some(block.origin.clone()),
                    ));
                    return;
                }
            };
            sources.insert(path.clone());
            let caption = image.alt.clone();
            let code = Block {
                origin: block.origin.clone(),
                kind: BlockKind::Code(Code {
                    language: language_for(&extension).to_string(),
                    code,
                }),
            };
            block.kind = if caption.is_empty() {
                code.kind
            } else {
                BlockKind::Figure(caption, Box::new(code))
            };
        });
    }
}

/// File extensions shown as pictures; images with other extensions embed the file as code.
const PICTURES: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "svg", "webp", "avif", "bmp", "ico", "tif", "tiff",
];

/// The code-block language for a file extension.
fn language_for(extension: &str) -> &str {
    match extension {
        "rs" => "rust",
        "py" => "python",
        "js" | "mjs" | "cjs" => "javascript",
        "ts" => "typescript",
        "kt" | "kts" => "kotlin",
        "h" | "hpp" | "cc" | "cxx" => "cpp",
        "sh" | "bash" => "shellscript",
        "yml" => "yaml",
        "md" => "markdown",
        "gv" => "dot",
        "puml" | "plantuml" => "uml",
        other => other,
    }
}

fn shift_headings(blocks: &mut [Block], by: u8) {
    if by == 0 {
        return;
    }
    visit_blocks_mut(blocks, &mut |block| {
        if let BlockKind::Heading(heading) = &mut block.kind {
            heading.level = heading.level.saturating_add(by).min(6);
        }
    });
}

/// Replaces the content of the first top-level heading of the file itself.
fn replace_first_title(blocks: &mut [Block], title: &str) {
    for block in blocks {
        if let BlockKind::Heading(heading) = &mut block.kind
            && heading.level == 1
        {
            heading.content = vec![Inline::Text(title.to_string())];
            return;
        }
    }
}

fn first_heading(blocks: &[Block]) -> Option<&Heading> {
    blocks.iter().find_map(|block| match &block.kind {
        BlockKind::Heading(heading) => Some(heading),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::MemoryFiles;
    use std::sync::{Arc, Mutex};

    fn write(files: &MemoryFiles, file: &str, text: &str) {
        files.insert(file, text);
    }

    fn headings(blocks: &[Block], out: &mut Vec<(u8, String)>) {
        for block in blocks {
            match &block.kind {
                BlockKind::Heading(h) => out.push((h.level, Inline::plain_text(&h.content))),
                BlockKind::Chapter(c) => headings(&c.blocks, out),
                _ => {}
            }
        }
    }

    #[test]
    fn includes_shift_headings_and_take_the_link_title() {
        let files = MemoryFiles::new();
        write(
            &files,
            "main.md",
            "# Manual\n\n## Usage\n\n[Install](parts/install.md)\n\n# [Appendix](parts/appendix.md)\n",
        );
        write(&files, "parts/install.md", "# Installing\n\n## Linux\n");
        write(&files, "parts/appendix.md", "# A\n");

        let vars = BTreeMap::new();
        let mut loader = Loader::new(&files, &vars, false);
        let chapter = loader.chapter(Path::new("main.md"), "", None).unwrap();
        assert!(loader.diagnostics.is_empty(), "{:?}", loader.diagnostics);

        let mut found = Vec::new();
        headings(&chapter.blocks, &mut found);
        let expected = [
            (1, "Manual"),
            (2, "Usage"),
            (3, "Install"),
            (4, "Linux"),
            (1, "Appendix"),
        ];
        let expected: Vec<(u8, String)> =
            expected.iter().map(|(l, t)| (*l, t.to_string())).collect();
        assert_eq!(found, expected);
        assert_eq!(loader.sources.len(), 3);
    }

    #[test]
    fn missing_includes_are_reported_once() {
        let files = MemoryFiles::new();
        write(
            &files,
            "nectar.toml",
            "[[document]]\nsource = \"main.md\"\n",
        );
        write(&files, "main.md", "# Main\n\n[Gone](gone.md)\n");
        let build = crate::Project::load(Box::new(files)).unwrap().documents();
        let messages: Vec<&str> = build
            .diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect();
        assert_eq!(messages, ["included file gone.md does not exist"]);
    }

    /// Refuses `secret.md` as an adapter refuses a path that leads outside the project.
    struct Refusing(MemoryFiles);

    impl Files for Refusing {
        fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
            self.is_file(path)?;
            self.0.read(path)
        }

        fn is_file(&self, path: &Path) -> std::io::Result<bool> {
            if path == Path::new("secret.md") {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "the file lies outside the project",
                ));
            }
            self.0.is_file(path)
        }

        fn list(&self, dir: &Path) -> std::io::Result<Vec<PathBuf>> {
            self.0.list(dir)
        }
    }

    #[test]
    fn a_refused_include_is_an_error_and_a_missing_asset_a_warning() {
        let files = Refusing(MemoryFiles::from_iter([
            ("nectar.toml", "[[document]]\nsource = \"main.md\"\n"),
            (
                "main.md",
                "# Main\n\n[Secret](secret.md)\n\n![Picture](secret.png)\n",
            ),
        ]));
        let documents = crate::Project::load(Box::new(files)).unwrap().documents();
        let messages: Vec<_> = documents
            .diagnostics
            .iter()
            .map(|d| (d.severity, d.message.as_str()))
            .collect();
        assert_eq!(
            messages,
            [
                (
                    crate::Severity::Error,
                    "cannot read secret.md: the file lies outside the project"
                ),
                (crate::Severity::Warning, "secret.png does not exist"),
            ]
        );
    }

    /// Records every path asked for.
    struct Recording(MemoryFiles, Arc<Mutex<Vec<PathBuf>>>);

    impl Files for Recording {
        fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
            self.1.lock().unwrap().push(path.to_path_buf());
            self.0.read(path)
        }

        fn is_file(&self, path: &Path) -> std::io::Result<bool> {
            self.1.lock().unwrap().push(path.to_path_buf());
            self.0.is_file(path)
        }

        fn list(&self, dir: &Path) -> std::io::Result<Vec<PathBuf>> {
            self.1.lock().unwrap().push(dir.to_path_buf());
            self.0.list(dir)
        }
    }

    // Verifies: G-7
    #[test]
    fn links_includes_and_images_outside_the_root_are_reported_and_not_followed() {
        let files = MemoryFiles::from_iter([
            ("nectar.toml", "[[document]]\nsource = \"docs/main.md\"\n"),
            (
                "docs/main.md",
                "# Main\n\n[Outside](../../outside.md)\n\n![Picture](../../picture.png)\n\n\
                 Get [the file](../../../etc/passwd).\n",
            ),
        ]);
        let asked = Arc::new(Mutex::new(Vec::new()));
        let build = crate::Project::load(Box::new(Recording(files, asked.clone())))
            .unwrap()
            .build(&[]);

        let messages: Vec<_> = build
            .diagnostics
            .iter()
            .map(|d| (d.origin.as_ref().unwrap().line, d.message.as_str()))
            .collect();
        assert_eq!(
            messages,
            [
                (3, "'../../outside.md' points outside the project"),
                (5, "'../../picture.png' points outside the project"),
                (7, "'../../../etc/passwd' points outside the project"),
            ]
        );
        let asked = asked.lock().unwrap();
        assert!(
            asked.iter().all(|path| {
                (path.starts_with("docs") || path == Path::new("nectar.toml"))
                    && path
                        .components()
                        .all(|c| matches!(c, std::path::Component::Normal(_)))
            }),
            "{asked:?}"
        );
        assert!(build.files.is_empty());
    }

    #[test]
    fn include_cycles_are_errors() {
        let files = MemoryFiles::new();
        write(&files, "a.md", "# A\n\n[B](b.md)\n");
        write(&files, "b.md", "# B\n\n[A](a.md)\n");

        let vars = BTreeMap::new();
        let mut loader = Loader::new(&files, &vars, false);
        loader.chapter(Path::new("a.md"), "", None).unwrap();
        assert_eq!(loader.diagnostics.len(), 1);
        assert!(loader.diagnostics[0].message.contains("cycle"));
    }

    #[test]
    fn drafts_are_left_out_unless_requested() {
        let files = MemoryFiles::new();
        write(&files, "main.md", "# Main\n\n[Draft](draft.md)\n");
        write(&files, "draft.md", "+++\ndraft = true\n+++\n# Draft\n");

        let vars = BTreeMap::new();
        let count = |include_drafts| {
            let mut loader = Loader::new(&files, &vars, include_drafts);
            let chapter = loader.chapter(Path::new("main.md"), "", None).unwrap();
            chapter
                .blocks
                .iter()
                .filter(|b| matches!(b.kind, BlockKind::Chapter(_)))
                .count()
        };
        assert_eq!(count(false), 0);
        assert_eq!(count(true), 1);
    }

    #[test]
    fn images_of_source_files_embed_the_file() {
        let files = MemoryFiles::new();
        write(
            &files,
            "main.md",
            "# Main\n\n![Entry point](src/Main.java)\n\n![](g.railroad)\n\n![logo](logo.png)\n",
        );
        write(&files, "src/Main.java", "class Main {}\n");
        write(&files, "g.railroad", "r = a;\n");

        let vars = BTreeMap::new();
        let mut loader = Loader::new(&files, &vars, false);
        let chapter = loader.chapter(Path::new("main.md"), "", None).unwrap();
        assert!(loader.diagnostics.is_empty(), "{:?}", loader.diagnostics);

        let BlockKind::Figure(caption, inner) = &chapter.blocks[1].kind else {
            panic!("expected a figure, got {:?}", chapter.blocks[1].kind);
        };
        assert_eq!(caption, "Entry point");
        assert!(
            matches!(&inner.kind, BlockKind::Code(code) if code.language == "java" && code.code == "class Main {}\n")
        );
        assert!(
            matches!(&chapter.blocks[2].kind, BlockKind::Code(code) if code.language == "railroad")
        );
        assert!(
            matches!(&chapter.blocks[3].kind, BlockKind::Paragraph(_)),
            "pictures stay images"
        );
        assert!(loader.sources.contains(Path::new("src/Main.java")));
    }
}
