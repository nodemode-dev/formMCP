mod support;

use serde_json::{json, Value};
use support::*;

fn ids(listing: &Value) -> Vec<String> {
    listing["apps"].as_array().unwrap().iter().map(|a| a["id"].as_str().unwrap().to_string()).collect()
}

fn app<'a>(listing: &'a Value, id: &str) -> &'a Value {
    listing["apps"].as_array().unwrap().iter().find(|a| a["id"] == id).unwrap_or_else(|| panic!("{id} is not listed"))
}

#[test]
fn with_no_catalogue_the_bundled_family_is_listed_with_its_flags() {
    let s = Sandbox::new("bundled");
    let listing = s.json(&["apps"], &[]);
    let bundled = bundled();
    let expected: Vec<String> = bundled["apps"].as_array().unwrap().iter().map(|a| a["id"].as_str().unwrap().to_string()).collect();
    assert_eq!(listing["count"], json!(expected.len()));
    assert_eq!(ids(&listing), expected);
    assert_eq!(expected.len(), 24);
    for want in bundled["apps"].as_array().unwrap() {
        let got = app(&listing, want["id"].as_str().unwrap());
        assert_eq!(got["mcp"], want["mcp"], "{}", want["id"]);
    }
    assert!(!ids(&listing).contains(&"formmcp".to_string()), "formMCP never lists itself");
    let mcp_only = s.json(&["apps", "--mcp-only"], &[]);
    assert_eq!(mcp_only["count"], json!(bundled["apps"].as_array().unwrap().iter().filter(|a| a["mcp"] == json!(true)).count()));
    assert!(listing["source"].as_str().unwrap().contains("catalogue was not used"));
}

#[test]
fn a_signed_catalogue_updates_the_family_and_never_lists_formmcp() {
    let s = Sandbox::new("signed");
    let k = key(21);
    let body = catalogue(&[product("formvis", false, Some("9.8.7")), product("formnew", true, None), product("formmcp", true, Some("1.0.0"))]);
    let path = write_catalogue(&s.root, &body, Some(&k));
    let env = [("FORMMCP_CATALOG", path.to_str().unwrap()), ("FORMAI_CATALOG_PUBKEY", &public(&k))];
    let listing = s.json(&["apps"], &env);
    assert!(listing["source"].as_str().unwrap().contains("signed"), "{}", listing["source"]);
    let formvis = app(&listing, "formvis");
    assert_eq!(formvis["version"], "9.8.7");
    assert_eq!(formvis["mcp"], json!(false), "the catalogue's agents.mcp wins over the bundled flag");
    assert_eq!(app(&listing, "formnew")["mcp"], json!(true));
    assert!(!ids(&listing).contains(&"formmcp".to_string()));
    let mut sorted = ids(&listing);
    sorted.sort();
    assert_eq!(ids(&listing), sorted);
}

#[test]
fn a_catalogue_with_one_changed_byte_is_not_used() {
    let s = Sandbox::new("tampered");
    let k = key(22);
    let body = catalogue(&[product("formvis", false, Some("9.8.7"))]);
    let path = write_catalogue(&s.root, &body, Some(&k));
    let mut tampered = body.into_bytes();
    let at = tampered.len() / 2;
    tampered[at] ^= 0x01;
    std::fs::write(&path, tampered).unwrap();
    let listing = s.json(&["apps"], &[("FORMMCP_CATALOG", path.to_str().unwrap()), ("FORMAI_CATALOG_PUBKEY", &public(&k))]);
    assert!(listing["source"].as_str().unwrap().contains("signature"), "{}", listing["source"]);
    assert_ne!(app(&listing, "formvis")["version"], "9.8.7");
    assert_eq!(app(&listing, "formvis")["mcp"], json!(true), "the bundled flag stays when the catalogue is refused");
}

#[test]
fn a_catalogue_signed_by_another_key_is_not_used() {
    let s = Sandbox::new("other-key");
    let body = catalogue(&[product("formvis", false, Some("9.8.7"))]);
    let path = write_catalogue(&s.root, &body, Some(&key(23)));
    let listing = s.json(&["apps"], &[("FORMMCP_CATALOG", path.to_str().unwrap()), ("FORMAI_CATALOG_PUBKEY", &public(&key(24)))]);
    assert_ne!(app(&listing, "formvis")["version"], "9.8.7");
}

#[test]
fn an_unsigned_catalogue_is_used_only_when_allowed() {
    let s = Sandbox::new("unsigned");
    let body = catalogue(&[product("formvis", true, Some("9.8.7"))]);
    let path = write_catalogue(&s.root, &body, None);
    let refused = s.json(&["apps"], &[("FORMMCP_CATALOG", path.to_str().unwrap())]);
    assert!(refused["source"].as_str().unwrap().contains("not signed"));
    let allowed = s.json(&["apps"], &[("FORMMCP_CATALOG", path.to_str().unwrap()), ("FORMMCP_ALLOW_UNSIGNED", "1")]);
    assert_eq!(app(&allowed, "formvis")["version"], "9.8.7");
}

#[test]
fn a_catalogue_over_plain_http_elsewhere_is_refused_without_a_request() {
    let s = Sandbox::new("plain-http");
    let started = std::time::Instant::now();
    let listing = s.json(&["apps"], &[("FORMMCP_CATALOG", "http://198.51.100.7/apps.json"), ("FORMMCP_ALLOW_UNSIGNED", "1")]);
    assert!(listing["source"].as_str().unwrap().contains("is not https"), "{}", listing["source"]);
    assert!(started.elapsed() < std::time::Duration::from_secs(5));
}

#[test]
fn reach_sees_only_installs_and_checkouts_on_this_computer() {
    let s = Sandbox::new("reach");
    let nothing = s.json(&["reach"], &[]);
    assert!(nothing["apps"].as_array().unwrap().iter().all(|r| r["reach"]["route"].is_null()), "nothing is reachable in an empty sandbox");

    std::fs::write(s.path("portal/installed.json"), json!({ "installs": [{ "product": "formvis", "channel": "stable" }, { "product": "formcut", "channel": "beta" }] }).to_string()).unwrap();
    let portal = s.path("bin/formportal-cli");
    std::fs::write(&portal, "#!/bin/sh\nexit 0\n").unwrap();
    make_executable(&portal);
    std::fs::create_dir_all(s.path("dev/formpdf")).unwrap();
    let launcher = s.path("dev/formpdf/formpdf");
    std::fs::write(&launcher, "#!/bin/sh\nexit 0\n").unwrap();
    make_executable(&launcher);

    let reach = s.json(&["reach"], &[("FORMMCP_DEV_ROOTS", s.path("dev").to_str().unwrap())]);
    let row = |id: &str| reach["apps"].as_array().unwrap().iter().find(|r| r["id"] == id).unwrap().clone();
    assert_eq!(row("formvis")["reach"]["installed"], json!(true));
    assert_eq!(row("formvis")["reach"]["route"]["via"], "formportal");
    assert_eq!(row("formcut")["reach"]["installed"], json!(false), "only stable installs count");
    assert_eq!(row("formpdf")["reach"]["route"]["via"], "dev");
    for r in reach["apps"].as_array().unwrap() {
        if r["mcp"] == json!(false) {
            assert!(r["reach"]["route"].is_null(), "{} has no MCP server and no route", r["id"]);
        }
    }
}

#[cfg(unix)]
fn make_executable(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(not(unix))]
fn make_executable(_: &std::path::Path) {}
