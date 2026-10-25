use std::sync::{Mutex, OnceLock};

use regex::Regex;
use serde_json::Value;

pub const MARK: &str = "[redacted by formMCP]";
const SHORTEST_KNOWN: usize = 8;

static KNOWN: Mutex<Vec<String>> = Mutex::new(Vec::new());

const SHAPES: [&str; 11] = [
    r"-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----",
    r"\bfmd_[A-Za-z0-9_\-]{16,}",
    r"\bsk-ant-[A-Za-z0-9_\-]{20,}",
    r"\bsk-(?:proj-)?[A-Za-z0-9_\-]{32,}",
    r"\bgh[pousr]_[A-Za-z0-9]{30,}",
    r"\bgithub_pat_[A-Za-z0-9_]{40,}",
    r"\bxox[abposr]-[A-Za-z0-9\-]{10,}",
    r"\bAKIA[0-9A-Z]{16}\b",
    r"\bAIza[0-9A-Za-z_\-]{35}",
    r"\beyJ[A-Za-z0-9_\-]{10,}\.eyJ[A-Za-z0-9_\-]{10,}\.[A-Za-z0-9_\-]{10,}",
    r"(?i)\b(bearer|token|api[_-]?key|secret|password)([\x22']?\s*[:=]\s*[\x22']?|\s+)[A-Za-z0-9._~+/\-]{20,}=*",
];

fn shapes() -> &'static Vec<Regex> {
    static BUILT: OnceLock<Vec<Regex>> = OnceLock::new();
    BUILT.get_or_init(|| SHAPES.iter().filter_map(|s| Regex::new(s).ok()).collect())
}

pub fn remember(value: &str) {
    let value = value.trim();
    if value.chars().count() < SHORTEST_KNOWN {
        return;
    }
    if let Ok(mut known) = KNOWN.lock() {
        if !known.iter().any(|k| k == value) {
            known.push(value.to_string());
            known.sort_by_key(|k| std::cmp::Reverse(k.len()));
        }
    }
}

pub fn text(input: &str) -> (String, usize) {
    let mut out = input.to_string();
    let mut found = 0;
    if let Ok(known) = KNOWN.lock() {
        for k in known.iter() {
            let n = out.matches(k.as_str()).count();
            if n > 0 {
                found += n;
                out = out.replace(k.as_str(), MARK);
            }
        }
    }
    for (i, shape) in shapes().iter().enumerate() {
        let n = shape.find_iter(&out).count();
        if n == 0 {
            continue;
        }
        found += n;
        out = if i == SHAPES.len() - 1 { shape.replace_all(&out, format!("$1$2{MARK}")).into_owned() } else { shape.replace_all(&out, MARK).into_owned() };
    }
    (out, found)
}

pub fn value(v: &mut Value) -> usize {
    match v {
        Value::String(s) => {
            if s.len() > 8 * 1024 * 1024 {
                return 0;
            }
            let (clean, n) = text(s);
            if n > 0 {
                *s = clean;
            }
            n
        }
        Value::Array(items) => items.iter_mut().map(value).sum(),
        Value::Object(map) => map.iter_mut().filter(|(k, _)| !matches!(k.as_str(), "blob" | "data")).map(|(_, v)| value(v)).sum(),
        _ => 0,
    }
}
