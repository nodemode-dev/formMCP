use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use formmcp_core::clients::{self, Client, Launcher, Link};

static ENV: Mutex<()> = Mutex::new(());

struct Scratch {
    dir: PathBuf,
    _guard: MutexGuard<'static, ()>,
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn scratch(name: &str) -> Scratch {
    let guard = ENV.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join(format!("formmcp-host-files-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join(".codex")).unwrap();
    fs::create_dir_all(dir.join(".cursor")).unwrap();
    std::env::set_var("HOME", &dir);
    std::env::set_var("USERPROFILE", &dir);
    std::env::set_var("CODEX_HOME", dir.join(".codex"));
    for key in ["FORMMCP_DEV_ROOTS", "FORMPORTAL_ROOT", "FORMPORTAL_BIN", "FORMMCP_CATALOG", "FORMAI_CATALOG_PUBKEY", "FORMMCP_LAUNCHER"] {
        std::env::remove_var(key);
    }
    Scratch { dir, _guard: guard }
}

fn launcher() -> Launcher {
    Launcher { program: PathBuf::from("/x/formmcp"), args: vec!["mcp".to_string()] }
}

fn config(scratch: &Scratch, client: Client) -> PathBuf {
    let path = clients::state(client).config.expect("a config path");
    assert!(path.starts_with(&scratch.dir), "{} is outside the scratch dir", path.display());
    path
}

fn connect(client: Client, on: bool) -> Result<Link, String> {
    let mut out = clients::connect(&[client], &launcher(), on);
    out.remove(0).map(|s| s.state).map_err(|(_, e)| e)
}

fn files(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    names.sort();
    names
}

fn parses(text: &str) -> toml_edit::DocumentMut {
    text.parse::<toml_edit::DocumentMut>().unwrap_or_else(|e| panic!("not valid TOML ({}):\n{text}", e.message()))
}

#[test]
fn codex_header_text_inside_a_multi_line_string_is_not_a_section() {
    for quote in ["\"\"\"", "'''"] {
        let scratch = scratch(if quote.starts_with('"') { "codex-basic" } else { "codex-literal" });
        let path = config(&scratch, Client::Codex);
        let original = format!("[profiles.a]\nnote = {quote}\n[mcp_servers.formmcp]\ncommand = \"/x/formmcp\"\n{quote}\n");
        fs::write(&path, &original).unwrap();
        parses(&original);

        assert_eq!(clients::state(Client::Codex).state, Link::NotConnected, "a header inside {quote} read as a section");
        assert_eq!(connect(Client::Codex, true), Ok(Link::Connected));

        let text = fs::read_to_string(&path).unwrap();
        assert!(text.starts_with(&original), "the string was changed:\n{text}");
        let doc = parses(&text);
        assert_eq!(doc["profiles"]["a"]["note"].as_str(), Some("[mcp_servers.formmcp]\ncommand = \"/x/formmcp\"\n"));
        assert_eq!(doc["mcp_servers"]["formmcp"]["command"].as_str(), Some("/x/formmcp"));

        assert_eq!(connect(Client::Codex, false), Ok(Link::NotConnected));
        assert_eq!(fs::read_to_string(&path).unwrap().trim_end(), original.trim_end());
    }
}

#[test]
fn codex_section_holding_a_multi_line_string_is_replaced_whole() {
    let scratch = scratch("codex-own-string");
    let path = config(&scratch, Client::Codex);
    let original = "[mcp_servers.formmcp]\ncommand = \"/old/formmcp\"\nnote = \"\"\"\n[profiles.b]\nx = 1\n\"\"\"\n\n# context7\n[mcp_servers.context7]\ncommand = \"npx\"\n";
    fs::write(&path, original).unwrap();

    assert_eq!(clients::state(Client::Codex).state, Link::Connected);
    assert_eq!(connect(Client::Codex, true), Ok(Link::Connected));

    let text = fs::read_to_string(&path).unwrap();
    let doc = parses(&text);
    assert_eq!(doc["mcp_servers"]["formmcp"]["command"].as_str(), Some("/x/formmcp"));
    assert!(doc.get("profiles").is_none(), "text from the old string was left behind:\n{text}");
    assert!(text.ends_with("\n# context7\n[mcp_servers.context7]\ncommand = \"npx\"\n"), "the next section changed:\n{text}");
}

#[test]
fn codex_config_that_does_not_parse_is_refused_and_left_alone() {
    let scratch = scratch("codex-invalid");
    let path = config(&scratch, Client::Codex);
    let original = "[mcp_servers.other\ncommand = \"npx\"\n";
    fs::write(&path, original).unwrap();
    let before = files(path.parent().unwrap());

    let err = connect(Client::Codex, true).expect_err("an unparseable config.toml was accepted");
    assert!(err.contains("not valid TOML") && err.contains("not changed"), "message: {err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
    assert_eq!(files(path.parent().unwrap()), before, "a file was added beside config.toml");

    let state = clients::state(Client::Codex);
    assert_eq!(state.state, Link::NotConnected);
    assert!(state.note.as_deref().is_some_and(|n| n.contains("not valid TOML")), "note: {:?}", state.note);
}

const CURSOR_OTHER: &str = "    \"zeta\": {\n      \"command\": \"npx\",\n      \"args\": [\"-y\", \"caf\\u00e9\"],\n      \"timeout\": 1.50\n    },\n    \"alpha\": {\"command\": \"uvx\"}";

#[test]
fn cursor_connect_and_remove_keep_every_other_byte() {
    let scratch = scratch("cursor-bytes");
    let path = config(&scratch, Client::Cursor);
    let original = format!("{{\n    \"theme\": \"dark\",\n    \"mcpServers\": {{\n{CURSOR_OTHER}\n    }},\n    \"aaa\": 2.50\n}}\n");
    fs::write(&path, &original).unwrap();

    assert_eq!(connect(Client::Cursor, true), Ok(Link::Connected));
    let text = fs::read_to_string(&path).unwrap();
    let head = format!("{{\n    \"theme\": \"dark\",\n    \"mcpServers\": {{\n{CURSOR_OTHER},\n    \"formmcp\": ");
    assert!(text.starts_with(&head), "the text before the new entry changed:\n{text}");
    assert!(text.ends_with("\n    },\n    \"aaa\": 2.50\n}\n"), "the text after the new entry changed:\n{text}");
    let data: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(data["mcpServers"]["formmcp"]["command"], "/x/formmcp");

    assert_eq!(connect(Client::Cursor, true), Ok(Link::Connected));
    assert_eq!(fs::read_to_string(&path).unwrap(), text, "a second Connect changed the file");

    assert_eq!(connect(Client::Cursor, false), Ok(Link::NotConnected));
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
}

#[test]
fn cursor_reconnect_replaces_only_the_formmcp_value() {
    let scratch = scratch("cursor-replace");
    let path = config(&scratch, Client::Cursor);
    let head = format!("{{\n\t\"mcpServers\": {{\n{},\n\t\t\"formmcp\": ", CURSOR_OTHER.replace("    ", "\t"));
    let tail = ",\n\t\t\"last\": {\"command\": \"z\"}\n\t}\n}";
    fs::write(&path, format!("{head}{{\"command\": \"/old/formmcp\", \"args\": []}}{tail}")).unwrap();

    assert_eq!(clients::state(Client::Cursor).state, Link::Connected);
    assert_eq!(connect(Client::Cursor, true), Ok(Link::Connected));

    let text = fs::read_to_string(&path).unwrap();
    assert!(text.starts_with(&head) && text.ends_with(tail), "text outside the formmcp value changed:\n{text}");
    let data: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(data["mcpServers"]["formmcp"]["command"], "/x/formmcp");
}

#[test]
fn cursor_file_with_comments_is_refused_with_the_reason() {
    let scratch = scratch("cursor-comments");
    let path = config(&scratch, Client::Cursor);
    let original = "{\n  // mine\n  \"mcpServers\": {}\n}\n";
    fs::write(&path, original).unwrap();

    let state = clients::state(Client::Cursor);
    assert!(state.note.as_deref().is_some_and(|n| n.contains("comments")), "note: {:?}", state.note);

    let err = connect(Client::Cursor, true).expect_err("a file with comments was accepted");
    assert!(err.contains("comments") && err.contains("not changed"), "message: {err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
}

#[test]
fn cursor_compact_crlf_and_byte_order_mark_files_round_trip() {
    let shapes = [
        "{\"mcpServers\":{\"a\":{\"command\":\"x\",\"n\":1.50}},\"z\":[1,2]}",
        "{\r\n  \"mcpServers\": {\r\n    \"a\": {\r\n      \"command\": \"x\"\r\n    }\r\n  }\r\n}\r\n",
        "\u{feff}{\n  \"mcpServers\": {\n    \"a\": {\"command\": \"x\"}\n  }\n}\n",
        "{\"mcpServers\": {}, \"b\": \"\\u00e9\"}",
    ];
    for (n, original) in shapes.iter().enumerate() {
        let scratch = scratch(&format!("cursor-shape-{n}"));
        let path = config(&scratch, Client::Cursor);
        fs::write(&path, original).unwrap();

        assert_eq!(connect(Client::Cursor, true), Ok(Link::Connected), "shape {n}");
        let text = fs::read_to_string(&path).unwrap();
        let mut data: serde_json::Value = serde_json::from_str(text.trim_start_matches('\u{feff}')).unwrap_or_else(|e| panic!("shape {n}: {e}\n{text}"));
        assert_eq!(data["mcpServers"]["formmcp"]["command"], "/x/formmcp", "shape {n}");
        assert!(!original.contains("\r\n") || !text.replace("\r\n", "").contains('\n'), "shape {n} mixed line endings:\n{text:?}");
        data["mcpServers"].as_object_mut().unwrap().remove("formmcp");
        let before: serde_json::Value = serde_json::from_str(original.trim_start_matches('\u{feff}')).unwrap();
        assert_eq!(data, before, "shape {n}: other settings changed");

        assert_eq!(connect(Client::Cursor, false), Ok(Link::NotConnected), "shape {n}");
        assert_eq!(&fs::read_to_string(&path).unwrap(), original, "shape {n} did not round trip");
    }
}
