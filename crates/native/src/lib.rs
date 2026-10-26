//! Adapters for a machine: the project's files in a directory on disk, and threads that run the
//! code engines side by side.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use nectar_core::{Files, Workers};

/// The project's files in the directory `root`. Paths that lead outside it, through `..` or a
/// symbolic link, are refused.
pub struct Directory {
    root: PathBuf,
}

impl Directory {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Where `path` lies on disk, after every link, when that is inside the root.
    fn resolve(&self, path: &Path) -> io::Result<PathBuf> {
        let resolved = self.root.join(path).canonicalize()?;
        if !resolved.starts_with(self.root.canonicalize()?) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "the file lies outside the project",
            ));
        }
        Ok(resolved)
    }
}

impl Files for Directory {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(self.resolve(path)?)
    }

    fn is_file(&self, path: &Path) -> io::Result<bool> {
        match self.resolve(path) {
            Ok(resolved) => Ok(resolved.is_file()),
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => Err(e),
            Err(_) => Ok(false),
        }
    }

    fn list(&self, dir: &Path) -> io::Result<Vec<PathBuf>> {
        let resolved = self.resolve(dir)?;
        if !resolved.is_dir() {
            return Err(io::ErrorKind::NotFound.into());
        }
        let mut files = Vec::new();
        list_files(fs::read_dir(resolved)?, dir, &mut files);
        Ok(files)
    }
}

/// Collects the files of `entries` and below as paths below `dir`, sorted by name. Symbolic links
/// and directories that cannot be read are left out, so the listing stays inside the root.
fn list_files(entries: fs::ReadDir, dir: &Path, files: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = dir.join(entry.file_name());
        match entry.file_type() {
            Ok(kind) if kind.is_dir() => {
                if let Ok(below) = fs::read_dir(entry.path()) {
                    list_files(below, &path, files);
                }
            }
            Ok(kind) if kind.is_file() => files.push(path),
            _ => {}
        }
    }
}

/// Runs tasks on as many threads as the machine has cores.
pub struct Threads;

impl Workers for Threads {
    fn run(&self, count: usize, task: &(dyn Fn(usize) + Sync)) {
        let threads = thread::available_parallelism()
            .map_or(4, |n| n.get())
            .min(count);
        let next = AtomicUsize::new(0);
        thread::scope(|scope| {
            for _ in 0..threads {
                scope.spawn(|| {
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        if index >= count {
                            break;
                        }
                        task(index);
                    }
                });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn write(root: &Path, file: &str, text: &str) {
        let path = root.join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    #[test]
    fn reads_and_lists_the_files_below_the_root() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "docs/b.md", "b");
        write(dir.path(), "docs/a.md", "a");
        write(dir.path(), "docs/img/c.png", "c");
        let files = Directory::new(dir.path());

        assert_eq!(files.read(Path::new("docs/a.md")).unwrap(), b"a");
        assert!(files.is_file(Path::new("docs/a.md")).unwrap());
        assert!(!files.is_file(Path::new("docs/gone.md")).unwrap());
        assert!(!files.is_file(Path::new("docs")).unwrap());
        assert!(!files.is_file(Path::new("docs/a.md/b.md")).unwrap());
        assert_eq!(
            files.list(Path::new("docs")).unwrap(),
            ["docs/a.md", "docs/b.md", "docs/img/c.png"].map(PathBuf::from)
        );
        let missing = files.list(Path::new("themes")).unwrap_err();
        assert_eq!(missing.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn refuses_paths_that_lead_outside_the_root() {
        let outer = tempfile::tempdir().unwrap();
        write(outer.path(), "secret.txt", "secret");
        write(outer.path(), "project/main.md", "# Main");
        let files = Directory::new(outer.path().join("project"));

        let path = Path::new("../secret.txt");
        let refused = files.read(path).unwrap_err();
        assert_eq!(refused.kind(), io::ErrorKind::PermissionDenied);
        assert!(files.is_file(path).is_err());
        assert!(files.list(Path::new("..")).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn refuses_links_that_lead_outside_the_root() {
        let outer = tempfile::tempdir().unwrap();
        write(outer.path(), "secret.txt", "secret");
        write(outer.path(), "project/main.md", "# Main");
        let root = outer.path().join("project");
        std::os::unix::fs::symlink(outer.path().join("secret.txt"), root.join("link.txt")).unwrap();
        std::os::unix::fs::symlink(outer.path(), root.join("up")).unwrap();
        let files = Directory::new(&root);

        let refused = files.read(Path::new("link.txt")).unwrap_err();
        assert_eq!(refused.kind(), io::ErrorKind::PermissionDenied);
        assert!(files.is_file(Path::new("link.txt")).is_err());
        assert!(files.list(Path::new("up")).is_err());
        assert_eq!(
            files.list(Path::new("")).unwrap(),
            [PathBuf::from("main.md")]
        );
    }

    #[test]
    fn threads_run_every_task_once() {
        let done = Mutex::new(Vec::new());
        Threads.run(100, &|index| done.lock().unwrap().push(index));
        let mut done = done.into_inner().unwrap();
        done.sort();
        assert_eq!(done, (0..100).collect::<Vec<_>>());
        Threads.run(0, &|_| panic!("no task to run"));
    }
}
