use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serde_json::{json, Value};

use crate::log;

const DENY_EXIT: i32 = 2;
const OUTPUT_LIMIT: usize = 64 * 1024;
const LONG_STRING: usize = 64 * 1024;

pub fn timeout() -> Duration {
    Duration::from_secs(std::env::var("FORMMCP_HOOK_TIMEOUT").ok().and_then(|v| v.parse().ok()).unwrap_or(10))
}

pub fn split(line: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut word = String::new();
    let mut in_word = false;
    let mut quote: Option<char> = None;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some('\''), c) => word.push(c),
            (Some(_), '\\') | (None, '\\') => {
                match chars.next_if(|next| matches!(next, '"' | '\'' | '\\')) {
                    Some(next) => word.push(next),
                    None => word.push('\\'),
                }
                in_word = true;
            }
            (Some(_), c) => word.push(c),
            (None, '"') | (None, '\'') => {
                quote = Some(c);
                in_word = true;
            }
            (None, c) if c.is_whitespace() => {
                if in_word {
                    out.push(std::mem::take(&mut word));
                    in_word = false;
                }
            }
            (None, c) => {
                word.push(c);
                in_word = true;
            }
        }
        if quote.is_some() {
            in_word = true;
        }
    }
    if quote.is_some() {
        return Err("the hook command has an unclosed quote".into());
    }
    if in_word {
        out.push(word);
    }
    Ok(out)
}

pub fn shorten(v: &mut Value) {
    match v {
        Value::String(s) if s.len() > LONG_STRING => *s = format!("[{} bytes left out by formMCP]", s.len()),
        Value::Array(items) => items.iter_mut().for_each(shorten),
        Value::Object(map) => map.values_mut().for_each(shorten),
        _ => {}
    }
}

struct Ran {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run(line: &str, input: &Value) -> Result<Ran, String> {
    let words = split(line)?;
    let (program, args) = words.split_first().ok_or("the hook command is empty")?;
    let mut child = Command::new(program).args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e| format!("could not start {program} ({e})"))?;
    let mut stdin = child.stdin.take().ok_or("no stdin for the hook")?;
    let text = input.to_string();
    thread::spawn(move || {
        let _ = stdin.write_all(text.as_bytes());
    });
    let mut stdout = child.stdout.take().ok_or("no stdout from the hook")?;
    let mut stderr = child.stderr.take().ok_or("no stderr from the hook")?;
    let errors = thread::spawn(move || {
        let mut err = Vec::new();
        let _ = stderr.by_ref().take(OUTPUT_LIMIT as u64).read_to_end(&mut err);
        let _ = std::io::copy(&mut stderr, &mut std::io::sink());
        err
    });
    let (send, receive) = mpsc::channel();
    thread::spawn(move || {
        let mut out = Vec::new();
        let _ = stdout.by_ref().take(OUTPUT_LIMIT as u64).read_to_end(&mut out);
        let _ = std::io::copy(&mut stdout, &mut std::io::sink());
        let err = errors.join().unwrap_or_default();
        let _ = send.send((out, err));
    });
    let limit = timeout();
    let (out, err) = match receive.recv_timeout(limit) {
        Ok(found) => found,
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("it did not finish within {} s", limit.as_secs()));
        }
    };
    let status = child.wait().map_err(|e| e.to_string())?;
    Ok(Ran { code: status.code(), stdout: String::from_utf8_lossy(&out).trim().to_string(), stderr: String::from_utf8_lossy(&err).trim().to_string() })
}

pub fn before(line: &str, event: &Value) -> Result<(), String> {
    if line.trim().is_empty() {
        return Ok(());
    }
    let ran = run(line, event).map_err(|e| {
        log::record("hook", json!({ "hook": "before", "ok": false, "error": e }));
        format!("formMCP's before-call hook failed, so the call was not made: {e}")
    })?;
    let said: Option<Value> = if ran.stdout.is_empty() { None } else { serde_json::from_str(&ran.stdout).ok() };
    let decision = said.as_ref().and_then(|v| v.get("decision")).and_then(Value::as_str).unwrap_or("allow");
    let reason = said.as_ref().and_then(|v| v.get("reason")).and_then(Value::as_str).map(str::to_string).unwrap_or_else(|| ran.stderr.lines().next().unwrap_or_default().to_string());
    let denied = match ran.code {
        Some(0) if ran.stdout.is_empty() => None,
        Some(0) if said.is_none() => Some("it printed something that is not JSON".to_string()),
        Some(0) => match decision {
            "allow" => None,
            "deny" => Some(reason),
            other => Some(format!("it answered decision {other:?}; use allow or deny")),
        },
        Some(DENY_EXIT) => Some(reason),
        code => Some(format!("it exited with {}", code.map(|c| c.to_string()).unwrap_or_else(|| "a signal".into()))),
    };
    log::record("hook", json!({ "hook": "before", "ok": denied.is_none(), "app": event.get("app"), "tool": event.get("tool"), "reason": denied }));
    match denied {
        None => Ok(()),
        Some(why) => Err(format!("formMCP's before-call hook refused this call{}", if why.is_empty() { String::new() } else { format!(": {why}") })),
    }
}

pub fn after(line: &str, event: Value) {
    if line.trim().is_empty() {
        return;
    }
    let line = line.to_string();
    thread::spawn(move || {
        if let Err(e) = run(&line, &event).and_then(|r| if r.code == Some(0) { Ok(()) } else { Err(format!("it exited with {:?}", r.code)) }) {
            log::record("hook", json!({ "hook": "after", "ok": false, "error": e }));
        }
    });
}
