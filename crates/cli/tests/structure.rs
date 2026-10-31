//! The shape of the code (ADR-0013): the core, the renderers and
//! the engines that need nothing of the host use no technology a host decides, and no crate but
//! this one depends on an adapter.
//!
//! Verifies: ADR-0013, Q-1

use std::fs;
use std::path::{Path, PathBuf};

/// The crates that must run wherever Rust compiles, WebAssembly included.
const PORTABLE: &[&str] = &["core", "html", "pdf", "textmate", "railroad"];

/// The adapter crates; only the shell composes them.
const ADAPTERS: &[&str] = &[
    "nectar-fetch",
    "nectar-native",
    "nectar-process",
    "nectar-server",
];

/// Crates that bring a technology a host decides.
const TECHNOLOGY_CRATES: &[&str] = &[
    "axum",
    "hyper",
    "notify",
    "notify-debouncer-full",
    "reqwest",
    "tempfile",
    "tokio",
    "ureq",
];

/// Modules of the standard library that reach the host. `env` is one: the variables of a host
/// reach the core only as the shell hands them over.
const TECHNOLOGY_MODULES: &[&str] = &["env", "fs", "net", "process", "thread"];

fn crates_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// The names under `[dependencies]` and `[dev-dependencies]` of a manifest: the tests of a
/// portable crate run without disk too, against fakes behind the ports.
fn dependencies(manifest: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut inside = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            inside = line == "[dependencies]" || line == "[dev-dependencies]";
        } else if inside && let Some((name, _)) = line.split_once('=') {
            names.push(name.trim().to_string());
        }
    }
    names
}

/// The Rust sources below `dir`, recursively.
fn sources(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, found);
        } else if path.extension().is_some_and(|e| e == "rs") {
            found.push(path);
        }
    }
}

/// The modules of `TECHNOLOGY_MODULES` that `code` uses, as `std::fs` or within `std::{…}`.
/// Everything from the first `#[cfg(test)]` on is a test and not read.
fn technology_modules(code: &str) -> Vec<&'static str> {
    let code = code.split("#[cfg(test)]").next().unwrap_or_default();
    let mut used = Vec::new();
    for module in TECHNOLOGY_MODULES {
        let direct = code.match_indices("std::").any(|(i, _)| {
            let rest = &code[i + 5..];
            rest.starts_with(module)
                && !rest[module.len()..].starts_with(|c: char| c.is_alphanumeric() || c == '_')
        });
        let grouped = code.match_indices("std::{").any(|(i, _)| {
            let group = &code[i + 6..];
            let group = &group[..group.find('}').unwrap_or(group.len())];
            group.split([',', ' ', '\n']).any(|item| {
                item.trim() == *module || item.trim().starts_with(&format!("{module}::"))
            })
        });
        if direct || grouped {
            used.push(*module);
        }
    }
    used
}

#[test]
fn portable_crates_depend_on_no_adapter_and_no_technology_crate() {
    let mut breaches = Vec::new();
    for name in PORTABLE {
        let manifest = fs::read_to_string(crates_dir().join(name).join("Cargo.toml")).unwrap();
        for dependency in dependencies(&manifest) {
            if ADAPTERS.contains(&dependency.as_str())
                || TECHNOLOGY_CRATES.contains(&dependency.as_str())
            {
                breaches.push(format!("{name} depends on {dependency}"));
            }
        }
    }
    assert!(breaches.is_empty(), "{breaches:#?}");
}

#[test]
fn portable_crates_use_no_technology_module() {
    let mut found = Vec::new();
    for name in PORTABLE {
        let mut files = Vec::new();
        sources(&crates_dir().join(name).join("src"), &mut files);
        for file in files {
            let code = fs::read_to_string(&file).unwrap();
            let relative = file.strip_prefix(crates_dir()).unwrap().to_string_lossy();
            let relative = relative.replace('\\', "/");
            for module in technology_modules(&code) {
                found.push(format!("{relative}: {module}"));
            }
        }
    }
    assert!(
        found.is_empty(),
        "technology used outside the adapters: {found:#?}"
    );
}

#[test]
fn finds_direct_and_grouped_uses() {
    assert_eq!(technology_modules("use std::fs;"), ["fs"]);
    assert_eq!(
        technology_modules("let w = std::thread::scope(|s| {});"),
        ["thread"]
    );
    assert_eq!(
        technology_modules("use std::{io, net::TcpStream};"),
        ["net"]
    );
    assert_eq!(technology_modules("let vars = std::env::vars();"), ["env"]);
    assert!(technology_modules("use std::fmt; use std::process_id;").is_empty());
    assert!(technology_modules("#[cfg(test)]\nmod tests { use std::fs; }").is_empty());
}
