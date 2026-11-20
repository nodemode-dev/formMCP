#![cfg(unix)]

mod support;

use std::cell::Cell;
use std::path::PathBuf;
use std::time::Duration;

use formmcp_core::programs::{child_path, find_with, known_dirs, override_var, shell_dirs, Os};
use serde_json::json;
use support::*;

fn none(_: &str) -> Option<PathBuf> {
    None
}

#[test]
fn a_program_is_looked_for_in_the_family_order_and_the_shell_is_asked_last() {
    let s = Sandbox::new("programs-order");
    let (chosen, process, known, shell) = (s.path("chosen/claude-x"), s.path("process"), s.path("known"), s.path("shell"));
    let asked = Cell::new(0);
    let ask = || {
        asked.set(asked.get() + 1);
        vec![shell.clone()]
    };
    let find = |chosen: Option<PathBuf>| find_with("claude", chosen, &[process.clone()], &[known.clone()], &ask);

    assert_eq!(find(None), None);
    assert_eq!(asked.get(), 1);
    stand_in(&shell.join("claude"), "#!/bin/sh\n");
    assert_eq!(find(None), Some(shell.join("claude")));
    stand_in(&known.join("claude"), "#!/bin/sh\n");
    assert_eq!(find(None), Some(known.join("claude")));
    stand_in(&process.join("claude"), "#!/bin/sh\n");
    assert_eq!(find(None), Some(process.join("claude")));
    assert_eq!(find(Some(chosen.clone())), Some(process.join("claude")), "a chosen file that cannot run was taken");
    stand_in(&chosen, "#!/bin/sh\n");
    assert_eq!(find(Some(chosen.clone())), Some(chosen));
    assert_eq!(asked.get(), 2, "the shell was asked although the program was found before it");
    assert_eq!(override_var("claude"), "FORMMCP_CLAUDE_BIN");
}

#[test]
fn the_known_folders_hold_the_home_and_system_install_folders() {
    let s = Sandbox::new("programs-known");
    let home = s.path("home");
    let mac = known_dirs(Os::Macos, &home, &none);
    let linux = known_dirs(Os::Linux, &home, &|name| (name == "NPM_CONFIG_PREFIX").then(|| PathBuf::from("/prefix")));
    for dir in [PathBuf::from("/opt/homebrew/bin"), PathBuf::from("/usr/local/bin"), home.join(".local/bin"), home.join(".claude/local")] {
        assert!(mac.contains(&dir), "{dir:?} missing from {mac:?}");
    }
    assert_eq!(linux[0], PathBuf::from("/prefix/bin"));
    assert!(linux.contains(&home.join(".local/bin")) && linux.contains(&PathBuf::from("/usr/local/bin")), "{linux:?}");
}

#[test]
fn the_shell_is_asked_for_its_path_and_a_shell_it_does_not_know_is_not_started() {
    let s = Sandbox::new("programs-shell");
    let shell = s.path("shells/bash");
    stand_in(&shell, "#!/bin/sh\necho noise from a profile\nPATH=\"/set/by/the/profile:/usr/bin\"\neval \"$4\"\n");
    assert_eq!(shell_dirs(&shell, Duration::from_secs(5)), Some(vec![PathBuf::from("/set/by/the/profile"), PathBuf::from("/usr/bin")]));

    let marker = s.path("ran");
    let other = s.path("shells/sh-standin");
    stand_in(&other, &format!("#!/bin/sh\ntouch '{}'\n", marker.display()));
    assert_eq!(shell_dirs(&other, Duration::from_secs(2)), None);
    assert!(!marker.exists());
}

#[test]
fn a_started_program_gets_its_own_folder_first_on_its_path() {
    let s = Sandbox::new("programs-child");
    let exe = s.path("node/bin/claude");
    stand_in(&exe, "#!/bin/sh\n");
    std::fs::create_dir_all(s.path("known")).unwrap();
    let path = child_path(&exe, &[PathBuf::from("/from/shell")], vec![s.path("known"), s.path("not-there")], vec![PathBuf::from("/usr/bin"), PathBuf::from("/from/shell")]);
    let dirs: Vec<PathBuf> = std::env::split_paths(&path).collect();
    assert_eq!(dirs[0], s.path("node/bin"));
    assert!(dirs.ends_with(&[PathBuf::from("/from/shell"), s.path("known"), PathBuf::from("/usr/bin")]), "{dirs:?}");
}

#[test]
fn the_tests_reach_only_a_stand_in_for_claude_code() {
    let s = Sandbox::new("programs-stand-in");
    let status = s.json(&["agents", "status", "claude"], &[]);
    assert_eq!(status["clients"][0]["state"], json!("not-connected"), "{status}");
    assert_eq!(s.agent_starts(), ["mcp get formmcp"]);

    let other = s.path("elsewhere/claude-dev");
    stand_in(&other, &format!("#!/bin/sh\necho \"chosen $*\" >> '{}'\necho 'Scope: User config'\necho 'Command: /x/formmcp'\n", s.path(AGENT_STARTS).display()));
    let status = s.json(&["agents", "status", "claude"], &[("FORMMCP_CLAUDE_BIN", other.to_str().unwrap())]);
    assert_eq!(status["clients"][0]["state"], json!("connected"), "{status}");
    assert_eq!(s.agent_starts(), ["mcp get formmcp", "chosen mcp get formmcp"]);
}
