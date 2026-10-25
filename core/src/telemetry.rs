use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::log;
use crate::CORE_VERSION;

const BOUNDS: [f64; 10] = [10.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0, 30000.0];
static WARNED: AtomicBool = AtomicBool::new(false);
static IN_FLIGHT: AtomicUsize = AtomicUsize::new(0);

pub struct Span<'a> {
    pub name: String,
    pub start: SystemTime,
    pub ms: u64,
    pub ok: bool,
    pub error: Option<&'a str>,
    pub attributes: Vec<(&'static str, String)>,
}

pub fn endpoint_problem(endpoint: &str) -> Option<String> {
    let e = endpoint.trim();
    let scheme_ok = e.starts_with("http://") || e.starts_with("https://");
    if !scheme_ok || e.len() > 2048 || e.chars().any(char::is_whitespace) || e.split("://").nth(1).is_none_or(|h| h.is_empty()) {
        return Some(format!("{e:?} is not an OpenTelemetry HTTP address; give the collector's base address, for example http://127.0.0.1:4318"));
    }
    None
}

fn nanos(t: SystemTime) -> String {
    t.duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0).to_string()
}

fn hex(len: usize) -> String {
    let mut bytes = vec![0u8; len];
    if getrandom::getrandom(&mut bytes).is_err() {
        bytes.iter_mut().enumerate().for_each(|(i, b)| *b = (i as u8).wrapping_add(1));
    }
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn attributes(list: &[(&'static str, String)]) -> Value {
    Value::Array(list.iter().map(|(k, v)| json!({ "key": k, "value": { "stringValue": v } })).collect())
}

fn resource() -> Value {
    json!({ "attributes": [
        { "key": "service.name", "value": { "stringValue": "formmcp" } },
        { "key": "service.version", "value": { "stringValue": CORE_VERSION } }
    ] })
}

fn headers() -> Vec<(String, String)> {
    std::env::var("OTEL_EXPORTER_OTLP_HEADERS")
        .unwrap_or_default()
        .split(',')
        .filter_map(|pair| pair.split_once('=').map(|(k, v)| (k.trim().to_string(), v.trim().to_string())))
        .filter(|(k, _)| !k.is_empty())
        .collect()
}

fn post(endpoint: &str, path: &str, body: &Value) -> Result<(), String> {
    let url = format!("{}{path}", endpoint.trim().trim_end_matches('/'));
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(5)).build();
    let mut request = agent.post(&url).set("Content-Type", "application/json");
    for (k, v) in headers() {
        request = request.set(&k, &v);
    }
    request.send_string(&body.to_string()).map(|_| ()).map_err(|e| format!("{url}: {e}"))
}

pub fn export(endpoint: Option<&str>, span: Span) {
    let Some(endpoint) = endpoint.filter(|e| endpoint_problem(e).is_none()).map(str::to_string) else { return };
    let end = span.start + Duration::from_millis(span.ms);
    let mut attrs = span.attributes.clone();
    attrs.push(("formmcp.outcome", if span.ok { "ok".into() } else { "error".into() }));
    let status = if span.ok { json!({ "code": 1 }) } else { json!({ "code": 2, "message": span.error.unwrap_or("failed") }) };
    let traces = json!({ "resourceSpans": [{ "resource": resource(), "scopeSpans": [{ "scope": { "name": "formmcp", "version": CORE_VERSION }, "spans": [{
        "traceId": hex(16), "spanId": hex(8), "name": span.name, "kind": 2,
        "startTimeUnixNano": nanos(span.start), "endTimeUnixNano": nanos(end),
        "attributes": attributes(&attrs), "status": status
    }] }] }] });
    let point = attributes(&attrs);
    let mut buckets = vec![0u64; BOUNDS.len() + 1];
    let slot = BOUNDS.iter().position(|b| (span.ms as f64) <= *b).unwrap_or(BOUNDS.len());
    buckets[slot] = 1;
    let metrics = json!({ "resourceMetrics": [{ "resource": resource(), "scopeMetrics": [{ "scope": { "name": "formmcp", "version": CORE_VERSION }, "metrics": [
        { "name": "formmcp.operations", "unit": "1", "description": "Tool calls and app starts through formMCP", "sum": { "aggregationTemporality": 1, "isMonotonic": true, "dataPoints": [
            { "attributes": point, "startTimeUnixNano": nanos(span.start), "timeUnixNano": nanos(end), "asInt": "1" } ] } },
        { "name": "formmcp.duration", "unit": "ms", "description": "How long each one took", "histogram": { "aggregationTemporality": 1, "dataPoints": [
            { "attributes": attributes(&attrs), "startTimeUnixNano": nanos(span.start), "timeUnixNano": nanos(end), "count": "1", "sum": span.ms as f64,
              "bucketCounts": buckets.iter().map(u64::to_string).collect::<Vec<_>>(), "explicitBounds": BOUNDS } ] } }
    ] }] }] });
    IN_FLIGHT.fetch_add(1, Ordering::SeqCst);
    thread::spawn(move || {
        let sent = post(&endpoint, "/v1/traces", &traces).and_then(|()| post(&endpoint, "/v1/metrics", &metrics));
        if let Err(e) = sent {
            if !WARNED.swap(true, Ordering::Relaxed) {
                log::record("telemetry", json!({ "ok": false, "error": e }));
            }
        }
        IN_FLIGHT.fetch_sub(1, Ordering::SeqCst);
    });
}

pub fn flush(limit: Duration) {
    let until = std::time::Instant::now() + limit;
    while IN_FLIGHT.load(Ordering::SeqCst) > 0 && std::time::Instant::now() < until {
        thread::sleep(Duration::from_millis(20));
    }
}
