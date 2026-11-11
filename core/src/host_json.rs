use serde::Serialize;
use serde_json::{json, Value};

struct Member {
    key: String,
    start: usize,
    colon: (usize, usize),
    value: (usize, usize),
}

struct Object {
    open: usize,
    close: usize,
    members: Vec<Member>,
}

const SERVERS: &str = "mcpServers";

fn ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && matches!(b[i], b' ' | b'\t' | b'\r' | b'\n') {
        i += 1;
    }
    i
}

fn string_end(b: &[u8], mut i: usize) -> usize {
    i += 1;
    while i < b.len() && b[i] != b'"' {
        if b[i] == b'\\' {
            i += 1;
        }
        i += 1;
    }
    i + 1
}

fn value_end(b: &[u8], i: usize) -> usize {
    match b.get(i) {
        Some(b'"') => string_end(b, i),
        Some(b'{' | b'[') => {
            let mut depth = 0usize;
            let mut j = i;
            while j < b.len() {
                match b[j] {
                    b'"' => {
                        j = string_end(b, j);
                        continue;
                    }
                    b'{' | b'[' => depth += 1,
                    b'}' | b']' => {
                        depth -= 1;
                        if depth == 0 {
                            return j + 1;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            j
        }
        _ => {
            let mut j = i;
            while j < b.len() && !matches!(b[j], b',' | b'}' | b']' | b' ' | b'\t' | b'\r' | b'\n') {
                j += 1;
            }
            j
        }
    }
}

fn object(text: &str, open: usize) -> Option<Object> {
    let b = text.as_bytes();
    if b.get(open) != Some(&b'{') {
        return None;
    }
    let mut members = Vec::new();
    let mut i = ws(b, open + 1);
    loop {
        match b.get(i)? {
            b'}' => return Some(Object { open, close: i, members }),
            b'"' => {
                let key_end = string_end(b, i);
                let key: String = serde_json::from_str(text.get(i..key_end)?).ok()?;
                let colon = ws(b, key_end);
                if b.get(colon) != Some(&b':') {
                    return None;
                }
                let start = ws(b, colon + 1);
                let end = value_end(b, start);
                text.get(start..end)?;
                members.push(Member { key, start: i, colon: (key_end, start), value: (start, end) });
                i = ws(b, end);
                if b.get(i) == Some(&b',') {
                    i = ws(b, i + 1);
                }
            }
            _ => return None,
        }
    }
}

fn root(text: &str) -> Option<Object> {
    let start = text.len() - text.trim_start_matches('\u{feff}').len();
    object(text, ws(text.as_bytes(), start))
}

fn find<'a>(object: &'a Object, key: &str) -> Option<(usize, &'a Member)> {
    object.members.iter().enumerate().rev().find(|(_, m)| m.key == key)
}

fn line_lead(text: &str, at: usize) -> &str {
    let start = text[..at].rfind('\n').map_or(0, |n| n + 1);
    let line = &text[start..];
    &line[..line.len() - line.trim_start_matches([' ', '\t']).len()]
}

fn own_line(text: &str, at: usize) -> Option<&str> {
    let lead = line_lead(text, at);
    let start = text[..at].rfind('\n').map_or(0, |n| n + 1);
    (start + lead.len() == at).then_some(lead)
}

fn render(value: &Value, unit: Option<&str>, prefix: &str, nl: &str) -> Option<String> {
    let Some(unit) = unit else {
        return serde_json::to_string(value).ok();
    };
    let mut out = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(&mut out, serde_json::ser::PrettyFormatter::with_indent(unit.as_bytes()));
    value.serialize(&mut ser).ok()?;
    Some(String::from_utf8(out).ok()?.replace('\n', &format!("{nl}{prefix}")))
}

struct Layout {
    unit: Option<String>,
    nl: &'static str,
}

fn layout(text: &str, top: &Object) -> Layout {
    let nl = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let unit = match top.members.first() {
        None => Some("  ".to_string()),
        Some(m) => own_line(text, m.start).map(|lead| if lead.is_empty() { "  ".to_string() } else { lead.to_string() }),
    };
    Layout { unit, nl }
}

fn insert(text: &str, into: &Object, key: &str, value: &Value, layout: &Layout) -> Option<String> {
    let name = serde_json::to_string(key).ok()?;
    let Some(last) = into.members.last() else {
        let piece = match layout.unit.as_deref() {
            Some(unit) => {
                let outer = line_lead(text, into.open);
                let inner = format!("{outer}{unit}");
                format!("{{{nl}{inner}{name}: {}{nl}{outer}}}", render(value, Some(unit), &inner, layout.nl)?, nl = layout.nl)
            }
            None => format!("{{{name}: {}}}", render(value, None, "", layout.nl)?),
        };
        return Some(format!("{}{piece}{}", &text[..into.open], &text[into.close + 1..]));
    };
    let gap = text[..last.start].trim_end_matches([' ', '\t', '\r', '\n']).len();
    let sep = &text[gap..last.start];
    let colon = &text[last.colon.0..last.colon.1];
    let rendered = match sep.rfind('\n') {
        Some(n) => render(value, layout.unit.as_deref(), &sep[n + 1..], layout.nl)?,
        None => render(value, None, "", layout.nl)?,
    };
    let at = last.value.1;
    Some(format!("{},{sep}{name}{colon}{rendered}{}", &text[..at], &text[at..]))
}

pub(crate) fn set(text: &str, server: &str, entry: &Value) -> Option<String> {
    let top = root(text)?;
    let layout = layout(text, &top);
    let Some((_, servers)) = find(&top, SERVERS) else {
        return insert(text, &top, SERVERS, &json!({ server: entry }), &layout);
    };
    let servers = object(text, servers.value.0)?;
    match find(&servers, server) {
        Some((_, own)) => {
            let line = own_line(text, own.start);
            let unit = line.and(layout.unit.as_deref());
            Some(format!("{}{}{}", &text[..own.value.0], render(entry, unit, line.unwrap_or(""), layout.nl)?, &text[own.value.1..]))
        }
        None => insert(text, &servers, server, entry, &layout),
    }
}

pub(crate) fn remove(text: &str, server: &str) -> Option<String> {
    let top = root(text)?;
    let (_, servers) = find(&top, SERVERS)?;
    let servers = object(text, servers.value.0)?;
    let (k, own) = find(&servers, server)?;
    let (cut_from, cut_to) = match (k, servers.members.len()) {
        (_, 1) => (servers.open + 1, servers.close),
        (0, _) => (own.start, servers.members[1].start),
        _ => (servers.members[k - 1].value.1, own.value.1),
    };
    Some(format!("{}{}", &text[..cut_from], &text[cut_to..]))
}
