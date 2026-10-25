use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::settings;

const LIMIT: u64 = 1024 * 1024;
const TAIL_MAX: u64 = 64 * 1024;

pub fn path() -> PathBuf {
    settings::dir().join("log.jsonl")
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

pub fn record(event: &str, fields: Value) {
    let file = path();
    if let Some(dir) = file.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if fs::metadata(&file).map(|m| m.len() > LIMIT).unwrap_or(false) {
        let _ = fs::rename(&file, file.with_file_name("log.1.jsonl"));
    }
    let mut line = json!({ "t": now_ms(), "event": event, "pid": std::process::id() });
    if let (Some(base), Some(extra)) = (line.as_object_mut(), fields.as_object()) {
        for (k, v) in extra {
            base.insert(k.clone(), v.clone());
        }
    }
    crate::redact::value(&mut line);
    if let Ok(mut out) = OpenOptions::new().create(true).append(true).open(&file) {
        let _ = writeln!(out, "{line}");
    }
}

pub fn tail(from: u64) -> (Vec<Value>, u64) {
    let file = path();
    let Ok(mut handle) = fs::File::open(&file) else { return (Vec::new(), 0) };
    let size = handle.metadata().map(|m| m.len()).unwrap_or(0);
    let start = if from > size { 0 } else { from.max(size.saturating_sub(TAIL_MAX)) };
    if handle.seek(SeekFrom::Start(start)).is_err() {
        return (Vec::new(), size);
    }
    let mut text = String::new();
    if handle.read_to_string(&mut text).is_err() {
        return (Vec::new(), size);
    }
    let mut lines: Vec<&str> = text.split('\n').collect();
    let complete_end = if text.ends_with('\n') { size } else { size - lines.last().map(|l| l.len() as u64).unwrap_or(0) };
    if !text.ends_with('\n') {
        lines.pop();
    }
    let skip_first = start > 0 && start != from;
    let events = lines
        .into_iter()
        .enumerate()
        .filter(|(i, l)| !(skip_first && *i == 0) && !l.trim().is_empty())
        .filter_map(|(_, l)| serde_json::from_str(l).ok())
        .collect();
    (events, complete_end)
}
