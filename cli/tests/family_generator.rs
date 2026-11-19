use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::Value;

const PYTHON: &str = if cfg!(windows) { "python" } else { "python3" };

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn generator(root: &Path, spec: &Path, check: bool) -> Output {
    let mut command = Command::new(PYTHON);
    command.arg("-I").arg(root.join("setup").join("family.py")).arg("--family").arg(spec);
    if check {
        command.arg("--check");
    }
    command.output().expect("python3 runs")
}

fn said(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string() + &String::from_utf8_lossy(&output.stderr)
}

fn copy_of_the_generator(tag: &str) -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!("formmcp-test-{}-{tag}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&root);
    for dir in ["setup", "core/data"] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    for file in ["setup/family.py", "setup/family.spec.json", "core/data/family.json"] {
        std::fs::copy(repo().join(file), root.join(file)).unwrap();
    }
    root
}

fn spec_with(root: &Path, name: &str, change: impl Fn(&mut Value)) -> PathBuf {
    let mut spec: Value = serde_json::from_slice(&std::fs::read(root.join("setup/family.spec.json")).unwrap()).unwrap();
    change(&mut spec);
    let path = root.join(name);
    std::fs::write(&path, serde_json::to_string_pretty(&spec).unwrap() + "\n").unwrap();
    path
}

#[test]
fn the_committed_list_is_what_the_kept_spec_generates() {
    let root = repo();
    let before = std::fs::read(root.join("core/data/family.json")).unwrap();
    let out = generator(&root, &root.join("setup/family.spec.json"), true);
    assert!(out.status.success(), "{}", said(&out));
    assert_eq!(std::fs::read(root.join("core/data/family.json")).unwrap(), before, "--check writes nothing");
}

#[test]
fn the_spec_is_kept_only_when_it_changes_the_list() {
    let root = copy_of_the_generator("kept");
    let list = root.join("core/data/family.json");
    let kept = root.join("setup/family.spec.json");
    let original = std::fs::read(&kept).unwrap();

    let changes_the_list = spec_with(&root, "changes.json", |spec| {
        let app = spec["apps"].as_array_mut().unwrap().iter_mut().find(|a| a["public"] == true).unwrap();
        app["summary"] = Value::from("A summary that the spec changed.");
    });
    let out = generator(&root, &changes_the_list, true);
    assert_eq!(out.status.code(), Some(1), "{}", said(&out));
    assert!(said(&out).contains("core/data/family.json"), "{}", said(&out));
    assert_eq!(std::fs::read(&kept).unwrap(), original, "--check writes nothing");

    let out = generator(&root, &changes_the_list, false);
    assert!(out.status.success(), "{}", said(&out));
    assert_eq!(std::fs::read(&kept).unwrap(), std::fs::read(&changes_the_list).unwrap(), "a spec that changed the list is kept byte for byte");
    assert!(std::fs::read_to_string(&list).unwrap().contains("A summary that the spec changed."));
    assert!(generator(&root, &kept, true).status.success(), "the kept copy now passes --check");
    let list_now = std::fs::read(&list).unwrap();
    let kept_now = std::fs::read(&kept).unwrap();

    let unused_field = spec_with(&root, "unused.json", |spec| {
        spec["generated"] = Value::from("2030-01-01T00:00:00Z");
        spec["content_sha256"] = Value::from("0".repeat(64));
        spec["apps"][0]["licence"] = Value::from("Another-Licence");
        spec["source"]["registry_version"] = Value::from("99.0.0");
        spec["apps"].as_array_mut().unwrap().push(serde_json::json!({ "id": "formprivate", "public": false }));
    });
    assert_ne!(std::fs::read(&unused_field).unwrap(), kept_now);
    for check in [true, false, false] {
        let out = generator(&root, &unused_field, check);
        assert!(out.status.success(), "{}", said(&out));
        assert_eq!(std::fs::read(&list).unwrap(), list_now);
        assert_eq!(std::fs::read(&kept).unwrap(), kept_now, "a spec that differs only in a field the list does not use leaves the kept copy alone");
    }

    let again = generator(&root, &changes_the_list, false);
    assert!(again.status.success(), "{}", said(&again));
    assert!(said(&again).contains("left alone"), "{}", said(&again));
    assert_eq!(std::fs::read(&kept).unwrap(), kept_now);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_spec_that_cannot_be_read_is_an_error_not_drift() {
    let root = copy_of_the_generator("unreadable");
    let list = std::fs::read(root.join("core/data/family.json")).unwrap();
    let out = generator(&root, &root.join("missing.json"), true);
    assert_eq!(out.status.code(), Some(2), "{}", said(&out));
    let out = generator(&root, &root.join("missing.json"), false);
    assert_eq!(out.status.code(), Some(2), "{}", said(&out));
    assert_eq!(std::fs::read(root.join("core/data/family.json")).unwrap(), list);
    let _ = std::fs::remove_dir_all(&root);
}
