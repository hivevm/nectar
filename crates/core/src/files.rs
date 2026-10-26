use std::io;
use std::path::{Path, PathBuf};

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

#[cfg(test)]
pub(crate) mod memory {
    use std::collections::BTreeMap;

    use super::*;

    /// Project files held in memory, for the core's tests.
    #[derive(Default)]
    pub(crate) struct Memory(pub BTreeMap<PathBuf, Vec<u8>>);

    impl Memory {
        pub fn with(files: &[(&str, &str)]) -> Self {
            Self(
                files
                    .iter()
                    .map(|(path, text)| (PathBuf::from(path), text.as_bytes().to_vec()))
                    .collect(),
            )
        }
    }

    impl Files for Memory {
        fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
            self.0
                .get(path)
                .cloned()
                .ok_or_else(|| io::ErrorKind::NotFound.into())
        }

        fn is_file(&self, path: &Path) -> io::Result<bool> {
            Ok(self.0.contains_key(path))
        }

        fn list(&self, dir: &Path) -> io::Result<Vec<PathBuf>> {
            let files: Vec<_> = self
                .0
                .keys()
                .filter(|path| path.starts_with(dir) && *path != dir)
                .cloned()
                .collect();
            if files.is_empty() {
                return Err(io::ErrorKind::NotFound.into());
            }
            Ok(files)
        }
    }
}
