//! Runs `nectar serve` on a project in a temporary directory and talks to it over HTTP.

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// Long enough for a slow machine; the test waits for events, it does not time them.
const PATIENCE: Duration = Duration::from_secs(60);

/// The server process, stopped when the test ends, however it ends.
struct Server(Child);

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Starts `nectar serve` on a port the system picks, and returns it with that port and the lines
/// it writes to standard error.
fn start(root: &Path) -> (Server, u16, mpsc::Receiver<String>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nectar"))
        .arg("-C")
        .arg(root)
        .args(["--offline", "serve", "--port", "0"])
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stderr = child.stderr.take().unwrap();
    let server = Server(child);
    let (lines, receive) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            let _ = lines.send(line);
        }
    });
    loop {
        let line = receive
            .recv_timeout(PATIENCE)
            .expect("the server did not start");
        // serving on http://127.0.0.1:<port>/ (Ctrl+C to stop)
        if let Some(address) = line.strip_prefix("serving on http://") {
            let address = &address[..address.find('/').unwrap()];
            let port = address.rsplit(':').next().unwrap().parse().unwrap();
            return (server, port, receive);
        }
    }
}

/// The body of `GET path`.
fn get(port: u16, path: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

/// The stream of build versions a browser listens to.
struct Reload(BufReader<TcpStream>);

impl Reload {
    fn connect(port: u16) -> Self {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream.set_read_timeout(Some(PATIENCE)).unwrap();
        write!(
            stream,
            "GET /_nectar/reload HTTP/1.1\r\nHost: localhost\r\n\r\n"
        )
        .unwrap();
        Self(BufReader::new(stream))
    }

    /// The next version announced before `deadline`. The stream also carries keep-alive
    /// comments, so a read alone never times out.
    fn next_version(&mut self, deadline: Instant) -> String {
        let mut line = String::new();
        loop {
            assert!(Instant::now() < deadline, "no new build announced in time");
            line.clear();
            self.0.read_line(&mut line).expect("no new build announced");
            if let Some(version) = line.trim_end().strip_prefix("data: ") {
                return version.to_string();
            }
        }
    }
}

/// The page at `/` once a build announced on `reload` shows `expected`. A change may come as
/// several file events and so as several builds; only the last one has to show it.
fn page_showing(reload: &mut Reload, port: u16, expected: &str) -> String {
    let deadline = Instant::now() + PATIENCE;
    loop {
        reload.next_version(deadline);
        let page = get(port, "/");
        if page.contains(expected) {
            return page;
        }
    }
}

// Verifies: G-6
#[test]
fn serve_rebuilds_on_change_reloads_browsers_and_shows_errors_on_the_page() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::write(
        root.join("nectar.toml"),
        "[[document]]\nsource = \"main.md\"\n",
    )
    .unwrap();
    fs::write(root.join("main.md"), "# Main\n\nVersion one.\n").unwrap();
    let (_server, port, _) = start(root);

    let mut reload = Reload::connect(port);
    let mut other = Reload::connect(port);
    let page = page_showing(&mut reload, port, "Version one.");
    assert!(page.contains("/_nectar/reload"), "the page listens: {page}");
    let first = other.next_version(Instant::now() + PATIENCE);

    fs::write(root.join("main.md"), "# Main\n\nVersion two.\n").unwrap();
    page_showing(&mut reload, port, "Version two.");
    let second = other.next_version(Instant::now() + PATIENCE);
    assert_ne!(
        first, second,
        "every connected browser hears of the rebuild"
    );

    // The panel comes in through the reload script, as a string of it.
    fs::write(root.join("main.md"), "# Main\n\n[Itself](main.md)\n").unwrap();
    let page = page_showing(&mut reload, port, "main.md:3: include cycle");
    assert!(page.contains("Build failed: 1 errors"), "{page}");

    fs::write(root.join("main.md"), "# Main\n\nVersion three.\n").unwrap();
    let page = page_showing(&mut reload, port, "Version three.");
    assert!(!page.contains("include cycle"), "{page}");
}

/// Copies the files below `from` to `to`, leaving out the output of earlier builds.
fn copy(from: &Path, to: &Path) {
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            if entry.file_name() != "public" {
                fs::create_dir_all(&target).unwrap();
                copy(&entry.path(), &target);
            }
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// The milliseconds of the next rebuild that `nectar serve` reports after a change.
fn next_rebuild(lines: &mpsc::Receiver<String>) -> u128 {
    loop {
        let line = lines.recv_timeout(PATIENCE).expect("no rebuild reported");
        if let Some(rest) = line.strip_prefix("built ")
            && line.contains("after a change")
        {
            let ms = rest
                .split(" in ")
                .nth(1)
                .unwrap()
                .split(' ')
                .next()
                .unwrap();
            return ms.parse().unwrap();
        }
    }
}

// Verifies: Q-4
#[test]
fn a_one_line_edit_of_the_demo_rebuilds_in_under_200_ms() {
    let demo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/demo");
    let dir = tempfile::tempdir().unwrap();
    copy(&demo, dir.path());
    let manual = dir.path().join("content/manual.md");
    let text = fs::read_to_string(&manual).unwrap();
    let (_server, _, lines) = start(dir.path());

    // The first change warms what the first build left cold; the engine cache is warm already.
    let mut timings = Vec::new();
    for edit in ["first", "second"] {
        let edited = text.replacen("This demo shows", &format!("This {edit} demo shows"), 1);
        assert_ne!(edited, text, "the demo changed: find another line to edit");
        fs::write(&manual, edited).unwrap();
        timings.push(next_rebuild(&lines));
    }
    assert!(timings[1] < 200, "the rebuild took {} ms", timings[1]);
}
