use std::fs;
use std::path::{Path, PathBuf};

const REPO_FROM_CRATE: &str = "..";

const VERSION_SITES: &[(&str, &str, char)] = &[
    ("Cargo.toml", "version = \"", '"'),
    ("src-tauri/tauri.conf.json", "  \"version\": \"", '"'),
    ("frontend/package.json", "  \"version\": \"", '"'),
    ("README.md", "  <img src=\"https://img.shields.io/badge/version-v", '-'),
    (".docs/wiki/Home.md", "**Current version:** v", ' '),
];

const STRIP_EXEMPT: &[&str] = &[];

const SKIP_DIRS: &[&str] = &["target", "node_modules", ".git", "dist", "venv"];

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(REPO_FROM_CRATE).canonicalize().expect("repository root")
}

fn version_at(rel: &str, prefix: &str, end: char) -> Option<String> {
    let text = fs::read_to_string(repo().join(rel)).ok()?;
    let line = text.lines().find(|l| l.starts_with(prefix))?;
    let rest = &line[prefix.len()..];
    Some(rest[..rest.find(end)?].to_string())
}

fn one_digit_patch(v: &str) -> bool {
    let parts: Vec<&str> = v.split('.').collect();
    parts.len() == 3
        && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
        && parts[2].len() == 1
}

fn cargo_tomls(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if !SKIP_DIRS.contains(&name.as_str()) {
                cargo_tomls(&path, out);
            }
        } else if name == "Cargo.toml" {
            out.push(path);
        }
    }
}

fn stripping_profiles(text: &str) -> Vec<String> {
    let mut section = String::new();
    let mut found = Vec::new();
    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.starts_with('[') {
            section = line.trim_matches(|c| c == '[' || c == ']').trim().to_string();
            continue;
        }
        if !section.starts_with("profile") {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim().trim_matches('"');
            if key.trim() == "strip" && value != "false" && value != "none" {
                found.push(format!("[{section}] strip = {value}"));
            }
        }
    }
    found
}

#[test]
fn f4_every_version_site_agrees_and_has_a_one_digit_patch() {
    let found: Vec<(&str, Option<String>)> = VERSION_SITES.iter().map(|(rel, prefix, end)| (*rel, version_at(rel, prefix, *end))).collect();
    let missing: Vec<&str> = found.iter().filter(|(_, v)| v.is_none()).map(|(rel, _)| *rel).collect();
    assert!(missing.is_empty(), "no version found in {missing:?}");
    let first = found[0].1.clone().unwrap();
    for (rel, v) in &found {
        assert_eq!(v.as_deref(), Some(first.as_str()), "{rel} disagrees with {}", found[0].0);
    }
    assert!(one_digit_patch(&first), "{first} is not x.y.z with a one-digit patch");
    assert_eq!(first, env!("CARGO_PKG_VERSION"), "the crate under test disagrees with the version sites");
}

#[test]
fn f9_no_cargo_release_profile_strips() {
    let root = repo();
    let mut files = Vec::new();
    cargo_tomls(&root, &mut files);
    assert!(!files.is_empty());
    let mut found = Vec::new();
    for path in files {
        let rel = path.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/");
        if STRIP_EXEMPT.contains(&rel.as_str()) {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        found.extend(stripping_profiles(&text).into_iter().map(|p| format!("{rel} {p}")));
    }
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn f9_the_profile_reader_sees_a_stripped_profile() {
    assert_eq!(stripping_profiles("[profile.release]\nstrip = true\n").len(), 1);
    assert_eq!(stripping_profiles("[profile.release.build-override]\nstrip = \"symbols\"\n").len(), 1);
    assert!(stripping_profiles("[profile.release]\nstrip = false\n[package]\nstrip = true\n").is_empty());
}
