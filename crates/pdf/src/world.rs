//! The Typst world of one document: the generated source, the files it refers to and the
//! bundled fonts, all in memory.

use std::collections::HashMap;
use std::sync::LazyLock;

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};

static FONT_FILES: &[&[u8]] = &[
    include_bytes!("../fonts/UbuntuSans-Regular.ttf"),
    include_bytes!("../fonts/UbuntuSans-Italic.ttf"),
    include_bytes!("../fonts/UbuntuSans-Bold.ttf"),
    include_bytes!("../fonts/UbuntuSans-BoldItalic.ttf"),
    include_bytes!("../fonts/VictorMono-Medium.ttf"),
    include_bytes!("../fonts/VictorMono-MediumItalic.ttf"),
    include_bytes!("../fonts/VictorMono-Bold.ttf"),
    include_bytes!("../fonts/VictorMono-BoldItalic.ttf"),
    include_bytes!("../fonts/MaterialIcons-Regular.ttf"),
];

static FONTS: LazyLock<Vec<Font>> = LazyLock::new(|| {
    FONT_FILES
        .iter()
        .flat_map(|data| Font::iter(Bytes::new(*data)))
        .collect()
});
static BOOK: LazyLock<LazyHash<FontBook>> =
    LazyLock::new(|| LazyHash::new(FontBook::from_fonts(FONTS.iter())));
static LIBRARY: LazyLock<LazyHash<Library>> = LazyLock::new(|| LazyHash::new(Library::default()));

/// The file id of an absolute path in the project root, such as `/main.typ`.
pub(crate) fn file_id(path: &str) -> Option<FileId> {
    let path = VirtualPath::new(path).ok()?;
    Some(RootedPath::new(VirtualRoot::Project, path).intern())
}

pub(crate) struct DocumentWorld {
    main: Source,
    files: HashMap<FileId, Bytes>,
}

impl DocumentWorld {
    /// `files` maps absolute virtual paths, as the source refers to them, to their content.
    pub fn new(source: String, files: HashMap<String, Vec<u8>>) -> Self {
        let main = Source::new(file_id("/main.typ").expect("a valid path"), source);
        let files = files
            .into_iter()
            .filter_map(|(path, bytes)| Some((file_id(&path)?, Bytes::new(bytes))))
            .collect();
        Self { main, files }
    }

    pub fn main_source(&self) -> &Source {
        &self.main
    }
}

impl World for DocumentWorld {
    fn library(&self) -> &LazyHash<Library> {
        &LIBRARY
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &BOOK
    }

    fn main(&self) -> FileId {
        self.main.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            Ok(self.main.clone())
        } else {
            Err(FileError::NotSource)
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.main.id() {
            return Ok(Bytes::from_string(self.main.clone()));
        }
        self.files
            .get(&id)
            .cloned()
            .ok_or_else(|| FileError::NotFound(id.vpath().get_with_slash().into()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        FONTS.get(index).cloned()
    }

    fn today(&self, _offset: Option<Duration>) -> Option<Datetime> {
        None
    }
}
