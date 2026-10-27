use std::collections::BTreeMap;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, RwLock};

/// The port for the project's files. Every path is relative to the project root; the adapter
/// keeps every access inside it, so the core neither knows the host's file system nor relies on
/// one existing.
pub trait Files: Send + Sync {
    /// The content of a file. Fails with [`io::ErrorKind::NotFound`] when there is none, and with
    /// [`io::ErrorKind::PermissionDenied`] when the path leads outside the root.
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;

    /// Whether a file exists at `path`. Fails when the path leads outside the root, so the reader
    /// reports that instead of a missing file.
    fn is_file(&self, path: &Path) -> io::Result<bool>;

    /// Every file below the directory `dir`, at any depth and sorted. Fails with
    /// [`io::ErrorKind::NotFound`] when there is no such directory.
    fn list(&self, dir: &Path) -> io::Result<Vec<PathBuf>>;
}

/// Reads a file as UTF-8 text.
pub(crate) fn read_text(files: &dyn Files, path: &Path) -> io::Result<String> {
    String::from_utf8(files.read(path)?).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// The project's files held in memory, as a host without a file system hands them over. Clones
/// share the files, so the host can change them between builds. A path that leads outside the
/// root, through `..` or from the root of a file system, is refused as on disk.
#[derive(Clone, Default)]
pub struct MemoryFiles(Arc<RwLock<BTreeMap<PathBuf, Vec<u8>>>>);

impl MemoryFiles {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds the file at `path`, relative to the project root, or replaces its content. A path
    /// that leaves the root is kept as given, where no read finds it.
    pub fn insert(&self, path: impl Into<PathBuf>, bytes: impl Into<Vec<u8>>) {
        let path = path.into();
        let path = key(&path).unwrap_or(path);
        self.0.write().unwrap().insert(path, bytes.into());
    }
}

impl<P: Into<PathBuf>, B: Into<Vec<u8>>> FromIterator<(P, B)> for MemoryFiles {
    fn from_iter<I: IntoIterator<Item = (P, B)>>(files: I) -> Self {
        let memory = Self::new();
        for (path, bytes) in files {
            memory.insert(path, bytes);
        }
        memory
    }
}

/// `path` as a key of the files, `.` and `..` taken out as on disk; a path that leaves the root
/// on the way is refused.
fn key(path: &Path) -> io::Result<PathBuf> {
    let outside = || {
        io::Error::new(
            io::ErrorKind::PermissionDenied,
            "the file lies outside the project",
        )
    };
    let mut key = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(name) => key.push(name),
            Component::CurDir => {}
            Component::ParentDir => {
                if !key.pop() {
                    return Err(outside());
                }
            }
            Component::RootDir | Component::Prefix(_) => return Err(outside()),
        }
    }
    Ok(key)
}

impl Files for MemoryFiles {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        let path = key(path)?;
        let files = self.0.read().unwrap();
        files
            .get(&path)
            .cloned()
            .ok_or_else(|| io::ErrorKind::NotFound.into())
    }

    fn is_file(&self, path: &Path) -> io::Result<bool> {
        Ok(self.0.read().unwrap().contains_key(&key(path)?))
    }

    /// A directory exists while a file lies below it.
    fn list(&self, dir: &Path) -> io::Result<Vec<PathBuf>> {
        let dir = key(dir)?;
        let files: Vec<_> = self
            .0
            .read()
            .unwrap()
            .keys()
            .filter(|path| path.starts_with(&dir) && **path != dir)
            .cloned()
            .collect();
        if files.is_empty() {
            return Err(io::ErrorKind::NotFound.into());
        }
        Ok(files)
    }
}
