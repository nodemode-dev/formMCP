mod support;

use std::time::{Duration, Instant};

use serde_json::{json, Value};
use support::*;

const TOOLS: [&str; 10] = ["family_apps", "suggest_app", "app_tools", "call_app", "app_resources", "read_app_resource", "app_prompts", "get_app_prompt", "extensions_list", "extension_command"];

fn names(list: &Value) -> Vec<String> {
    let mut n: Vec<String> = list["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap().to_string()).collect();
    n.sort();
    n
}

fn expected() -> Vec<String> {
    let mut n: Vec<String> = TOOLS.iter().map(|s| s.to_string()).collect();
    n.sort();
    n
}

fn text(reply: &Value) -> String {
    reply["result"]["content"].as_array().map(|c| c.iter().filter_map(|x| x["text"].as_str()).collect::<Vec<_>>().join(" ")).unwrap_or_default()
}

#[test]
fn stdio_initializes_and_lists_its_eight_tools() {
    let s = Sandbox::new("stdio");
    let mut m = Stdio2::start(&s, &[]);
    let init = m.initialize();
    assert_eq!(init["result"]["serverInfo"]["name"], "formmcp");
    assert_eq!(init["result"]["serverInfo"]["version"], env!("CARGO_PKG_VERSION"));
    let list = m.call(2, "tools/list", json!({}));
    assert_eq!(names(&list), expected());
    for tool in list["result"]["tools"].as_array().unwrap() {
        assert!(!tool["description"].as_str().unwrap_or("").trim().is_empty(), "{}", tool["name"]);
        assert_eq!(tool["inputSchema"]["type"], "object", "{}", tool["name"]);
    }
}

#[test]
fn a_read_only_tool_returns_the_family() {
    let s = Sandbox::new("family-tool");
    let mut m = Stdio2::start(&s, &[]);
    m.initialize();
    let reply = m.call(2, "tools/call", json!({ "name": "family_apps", "arguments": {} }));
    assert_ne!(reply["result"]["isError"], json!(true), "{reply}");
    assert!(text(&reply).contains("formvis") || reply["result"]["structuredContent"]["count"].as_u64().unwrap_or(0) > 0, "{reply}");
}

#[test]
fn an_unknown_tool_or_method_is_an_error_not_a_crash() {
    let s = Sandbox::new("unknown");
    let mut m = Stdio2::start(&s, &[]);
    m.initialize();
    let unknown_tool = m.call(2, "tools/call", json!({ "name": "no_such_tool", "arguments": {} }));
    assert!(unknown_tool.get("error").is_some() || unknown_tool["result"]["isError"] == json!(true), "{unknown_tool}");
    let unknown_method = m.call(3, "no/such/method", json!({}));
    assert!(unknown_method.get("error").is_some(), "{unknown_method}");
    let still = m.call(4, "tools/list", json!({}));
    assert_eq!(names(&still), expected());
}

#[test]
fn a_line_that_is_not_utf8_is_a_parse_error_and_the_server_stays_up() {
    let s = Sandbox::new("client-not-utf8");
    let mut m = Stdio2::start(&s, &[]);
    m.initialize();
    m.send_raw(b"\xff\xfe not json\n");
    let error = m.next(Duration::from_secs(30));
    assert_eq!(error["error"]["code"], json!(-32700), "{error}");
    let still = m.call(2, "tools/list", json!({}));
    assert_eq!(names(&still), expected());
}

#[test]
fn calling_an_app_that_is_not_installed_fails_at_once_with_the_reason() {
    let s = Sandbox::new("unreachable");
    let mut m = Stdio2::start(&s, &[]);
    m.initialize();
    let started = Instant::now();
    let reply = m.call(2, "tools/call", json!({ "name": "call_app", "arguments": { "app": "formvis", "tool": "get_document", "arguments": {} } }));
    assert!(started.elapsed() < Duration::from_secs(10));
    let all = format!("{reply}");
    assert!(all.contains("cannot be reached") && all.contains("not installed"), "{all}");
}

#[test]
fn calling_an_app_whose_server_dies_at_start_fails_promptly() {
    let s = Sandbox::new("dies");
    std::fs::create_dir_all(s.path("dev/formpdf")).unwrap();
    let launcher = s.path("dev/formpdf/formpdf");
    std::fs::write(&launcher, "#!/bin/sh\necho 'formpdf: no engine here' >&2\nexit 3\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&launcher, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let mut m = Stdio2::start(&s, &[("FORMMCP_DEV_ROOTS", s.path("dev").to_str().unwrap()), ("FORMMCP_START_TIMEOUT", "600")]);
    m.initialize();
    let started = Instant::now();
    let reply = m.call(2, "tools/call", json!({ "name": "app_tools", "arguments": { "app": "formpdf" } }));
    assert!(started.elapsed() < Duration::from_secs(15), "took {:?}", started.elapsed());
    let all = format!("{reply}");
    assert!(all.contains("did not start") || all.contains("stopped"), "{all}");
}

#[cfg(unix)]
#[test]
fn timeouts_too_large_for_a_deadline_wait_without_one() {
    use std::os::unix::fs::PermissionsExt;
    let s = Sandbox::new("huge-timeout");
    std::fs::create_dir_all(s.path("dev/formpdf")).unwrap();
    let launcher = s.path("dev/formpdf/formpdf");
    std::fs::write(&launcher, "#!/bin/sh\necho 'formpdf: no engine here' >&2\nexit 3\n").unwrap();
    std::fs::set_permissions(&launcher, std::fs::Permissions::from_mode(0o755)).unwrap();
    let mut m = Stdio2::start(&s, &[("FORMMCP_DEV_ROOTS", s.path("dev").to_str().unwrap()), ("FORMMCP_START_TIMEOUT", "18446744073709551615"), ("FORMMCP_IDLE_MINUTES", "18446744073709551615")]);
    m.initialize();
    let reply = m.call(2, "tools/call", json!({ "name": "app_tools", "arguments": { "app": "formpdf" } }));
    assert!(format!("{reply}").contains("did not start"), "{reply}");
    assert_eq!(names(&m.call(3, "tools/list", json!({}))), expected());
}

#[cfg(unix)]
#[test]
fn an_app_that_writes_bytes_that_are_not_utf8_still_answers() {
    use std::os::unix::fs::PermissionsExt;
    let s = Sandbox::new("not-utf8");
    std::fs::create_dir_all(s.path("dev/formpdf")).unwrap();
    let launcher = s.path("dev/formpdf/formpdf");
    let script = concat!(
        "#!/bin/sh\n",
        "i=0\n",
        "while [ $i -lt 10000 ]; do printf 'engine log \\377\\376 line\\n' >&2; i=$((i+1)); done\n",
        "printf 'not json \\377\\n'\n",
        "read -r line\n",
        "printf '%s\\n' '{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"protocolVersion\":\"2025-06-18\",\"capabilities\":{\"tools\":{}},\"serverInfo\":{\"name\":\"fakepdf\",\"version\":\"1\"}}}'\n",
        "read -r line\n",
        "read -r line\n",
        "printf '%s\\n' '{\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"tools\":[{\"name\":\"ping\",\"inputSchema\":{\"type\":\"object\"}}]}}'\n",
        "while read -r line; do :; done\n",
    );
    std::fs::write(&launcher, script).unwrap();
    std::fs::set_permissions(&launcher, std::fs::Permissions::from_mode(0o755)).unwrap();
    let mut m = Stdio2::start(&s, &[("FORMMCP_DEV_ROOTS", s.path("dev").to_str().unwrap()), ("FORMMCP_START_TIMEOUT", "20")]);
    m.initialize();
    let reply = m.call(2, "tools/call", json!({ "name": "app_tools", "arguments": { "app": "formpdf" } }));
    assert_ne!(reply["result"]["isError"], json!(true), "{reply}");
    assert_eq!(reply["result"]["structuredContent"]["tools"][0]["name"], "ping", "{reply}");
}

#[test]
fn an_app_with_no_mcp_server_is_refused_with_the_reason() {
    let s = Sandbox::new("no-mcp");
    let family = bundled();
    let without = family["apps"].as_array().unwrap().iter().find(|a| a["mcp"] == json!(false)).expect("the family has an app without an MCP server")["id"].as_str().unwrap().to_string();
    let mut m = Stdio2::start(&s, &[]);
    m.initialize();
    let reply = m.call(2, "tools/call", json!({ "name": "app_tools", "arguments": { "app": without } }));
    assert!(format!("{reply}").contains("has no MCP server"), "{reply}");
}

fn token(s: &Sandbox) -> String {
    s.text(&["http", "token"], &[]).trim().to_string()
}

fn post(http: &Http, headers: &[(&str, String)], body: Value) -> (u16, Vec<(String, String)>, String) {
    let mut all = vec![("Host", format!("127.0.0.1:{}", http.port)), ("Content-Type", "application/json".to_string())];
    all.extend(headers.iter().cloned());
    http.raw("POST", "/mcp", &all, &body.to_string())
}

fn init_body() -> Value {
    json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "family-test", "version": "1" } } })
}

#[test]
fn http_refuses_without_the_token_and_answers_with_it() {
    let s = Sandbox::new("http-token");
    let http = Http::start(&s);
    let t = token(&s);
    assert!(t.len() >= 32, "the token is long enough to guess at: {t:?}");

    let (status, headers, _) = post(&http, &[], init_body());
    assert_eq!(status, 401);
    assert!(header(&headers, "www-authenticate").unwrap_or("").contains("Bearer"));
    assert_eq!(post(&http, &[("Authorization", "Bearer wrong".into())], init_body()).0, 401);
    assert_eq!(post(&http, &[("Authorization", format!("Bearer {t}x"))], init_body()).0, 401);

    let (status, headers, body) = post(&http, &[("Authorization", format!("Bearer {t}"))], init_body());
    assert_eq!(status, 200, "{body}");
    let session = header(&headers, "mcp-session-id").expect("a session id").to_string();
    let list_body = json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {} });
    assert_eq!(post(&http, &[("Authorization", format!("Bearer {t}"))], list_body.clone()).0, 400, "a call before initialize has no session");
    let (status, _, body) = post(&http, &[("Authorization", format!("Bearer {t}")), ("Mcp-Session-Id", session)], list_body);
    assert_eq!(status, 200);
    assert_eq!(names(&serde_json::from_str(&body).unwrap()), expected());
}

#[test]
fn http_refuses_a_foreign_host_or_origin_even_with_the_token() {
    let s = Sandbox::new("http-guard");
    let http = Http::start(&s);
    let auth = ("Authorization", format!("Bearer {}", token(&s)));
    for host in ["evil.example", "evil.example:5625", "localhost.evil.com", "192.168.1.10"] {
        let (status, _, _) = http.raw("POST", "/mcp", &[("Host", host.to_string()), auth.clone()], &init_body().to_string());
        assert_eq!(status, 403, "host {host}");
    }
    for origin in ["http://evil.example", "https://localhost.evil.com", "null"] {
        assert_eq!(post(&http, &[auth.clone(), ("Origin", origin.to_string())], init_body()).0, 403, "origin {origin}");
    }
    assert_eq!(post(&http, &[auth.clone(), ("Origin", format!("http://127.0.0.1:{}", http.port))], init_body()).0, 200);
    assert_eq!(post(&http, &[auth.clone(), ("Origin", format!("http://localhost:{}", http.port))], init_body()).0, 200);
}

#[test]
fn http_answers_only_at_mcp_and_sends_no_stream() {
    let s = Sandbox::new("http-paths");
    let http = Http::start(&s);
    let auth = ("Authorization", format!("Bearer {}", token(&s)));
    let host = ("Host", format!("127.0.0.1:{}", http.port));
    assert_eq!(http.raw("GET", "/", &[host.clone(), auth.clone()], "").0, 404);
    let (status, headers, _) = http.raw("GET", "/mcp", &[host.clone(), auth.clone()], "");
    assert_eq!(status, 405);
    assert_eq!(header(&headers, "allow"), Some("POST, DELETE"));
    assert_eq!(post(&http, &[auth.clone(), ("MCP-Protocol-Version", "1999-01-01".into())], init_body()).0, 400);
    let (status, _, _) = http.raw("POST", "/mcp", &[host, auth], "{not json");
    assert_eq!(status, 400);
}

#[test]
fn http_refuses_a_chunk_size_that_would_overflow_and_keeps_serving() {
    let s = Sandbox::new("http-chunk");
    let http = Http::start(&s);
    let auth = ("Authorization", format!("Bearer {}", token(&s)));
    let host = ("Host", format!("127.0.0.1:{}", http.port));
    let chunked = ("Transfer-Encoding", "chunked".to_string());
    let (status, _, _) = http.raw("POST", "/mcp", &[host.clone(), auth.clone(), chunked], "1\r\n{\r\nffffffffffffffff\r\nx\r\n0\r\n\r\n");
    assert_eq!(status, 413);
    assert_eq!(post(&http, &[auth], init_body()).0, 200);
}

#[test]
fn a_rotated_token_stops_the_old_one() {
    let s = Sandbox::new("http-rotate");
    let http = Http::start(&s);
    let old = token(&s);
    s.text(&["http", "rotate"], &[]);
    let new = token(&s);
    assert_ne!(old, new);
    assert_eq!(post(&http, &[("Authorization", format!("Bearer {old}"))], init_body()).0, 401);
    assert_eq!(post(&http, &[("Authorization", format!("Bearer {new}"))], init_body()).0, 200);
}

#[test]
fn http_lists_the_same_tools_as_stdio() {
    let s = Sandbox::new("same-tools");
    let mut m = Stdio2::start(&s, &[]);
    m.initialize();
    let over_stdio = names(&m.call(2, "tools/list", json!({})));
    let http = Http::start(&s);
    let t = token(&s);
    let (_, headers, _) = post(&http, &[("Authorization", format!("Bearer {t}"))], init_body());
    let session = header(&headers, "mcp-session-id").unwrap().to_string();
    let (_, _, body) = post(&http, &[("Authorization", format!("Bearer {t}")), ("Mcp-Session-Id", session)], json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {} }));
    assert_eq!(names(&serde_json::from_str(&body).unwrap()), over_stdio);
}
