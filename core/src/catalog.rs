use std::collections::HashMap;
use std::io::Read;
use std::time::Duration;

use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::Deserialize;

use crate::family::{App, Family};
use crate::reach::portal_root;

pub const DEFAULT_CATALOG: &str = "https://github.com/nodemode-ai/formcompiler/releases/download/catalog/apps.json";
pub const KEY_ENV: &str = "FORMAI_CATALOG_PUBKEY";
const TRUSTED_KEYS: &str = include_str!("../catalog_keys.txt");
const CATALOG_FORMAT: &str = "formai-catalog";
const LOOPBACK: [&str; 4] = ["http://127.0.0.1:", "http://127.0.0.1/", "http://localhost:", "http://localhost/"];

#[derive(Deserialize)]
struct Catalog {
    format: String,
    #[serde(default)]
    products: Vec<Product>,
}

#[derive(Deserialize)]
struct Product {
    id: String,
    name: String,
    kind: String,
    summary: String,
    repository: String,
    state: String,
    #[serde(default)]
    agents: Option<Agents>,
    #[serde(default)]
    channels: HashMap<String, Release>,
}

#[derive(Deserialize)]
struct Agents {
    #[serde(default)]
    mcp: bool,
}

#[derive(Deserialize)]
struct Release {
    version: String,
}

pub fn keys() -> Vec<String> {
    let mut keys: Vec<String> = TRUSTED_KEYS.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).map(str::to_string).collect();
    if let Ok(extra) = std::env::var(KEY_ENV) {
        keys.extend(extra.split(',').map(|k| k.trim().to_string()).filter(|k| !k.is_empty()));
    }
    keys
}

fn decode(text: &str) -> Option<Vec<u8>> {
    base64::engine::general_purpose::STANDARD.decode(text.trim()).ok()
}

pub fn verify(body: &[u8], signature: &str, keys: &[String]) -> Result<(), String> {
    if keys.is_empty() {
        return Err("this build trusts no catalogue key yet".into());
    }
    let raw = decode(signature).ok_or("its signature is not valid base64")?;
    let signature = Signature::from_slice(&raw).map_err(|_| "its signature is not 64 bytes")?;
    for key in keys {
        let Some(bytes) = decode(key) else { continue };
        let Ok(array) = <[u8; 32]>::try_from(bytes.as_slice()) else { continue };
        let Ok(verifying) = VerifyingKey::from_bytes(&array) else { continue };
        if verifying.verify(body, &signature).is_ok() {
            return Ok(());
        }
    }
    Err("its signature does not match a key this build trusts".into())
}

fn flag(name: &str) -> bool {
    std::env::var(name).map(|v| matches!(v.trim(), "1" | "true" | "yes" | "on")).unwrap_or(false)
}

fn fetch(source: &str) -> Result<Vec<u8>, String> {
    if source.starts_with("https://") || LOOPBACK.iter().any(|p| source.starts_with(p)) {
        let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(8)).build();
        let response = agent.get(source).call().map_err(|e| e.to_string())?;
        let mut body = Vec::new();
        response.into_reader().take(16 * 1024 * 1024).read_to_end(&mut body).map_err(|e| e.to_string())?;
        return Ok(body);
    }
    if source.starts_with("http://") {
        return Err(format!("{source} is not https"));
    }
    let path = source.strip_prefix("file://").unwrap_or(source);
    std::fs::read(path).map_err(|e| format!("{path}: {e}"))
}

fn read_source() -> Result<(Vec<u8>, Option<String>, String), String> {
    let kept = portal_root().join("catalog");
    let from_kept = || -> Result<(Vec<u8>, Option<String>, String), String> {
        let body = std::fs::read(kept.join("apps.json")).map_err(|_| "no copy kept by FormPortal".to_string())?;
        let sig = std::fs::read_to_string(kept.join("apps.json.sig")).ok().map(|s| s.trim().to_string());
        Ok((body, sig, "FormPortal's kept copy".to_string()))
    };
    if flag("FORMMCP_OFFLINE") {
        return from_kept();
    }
    let source = std::env::var("FORMMCP_CATALOG").ok().filter(|s| !s.trim().is_empty()).unwrap_or_else(|| DEFAULT_CATALOG.to_string());
    match fetch(&source) {
        Ok(body) => {
            let sig = fetch(&format!("{source}.sig")).ok().map(|s| String::from_utf8_lossy(&s).trim().to_string()).filter(|s| !s.is_empty());
            Ok((body, sig, source))
        }
        Err(fetch_error) => from_kept().map_err(|_| fetch_error),
    }
}

fn merge(mut family: Family, products: Vec<Product>) -> Family {
    for product in products {
        let version = product.channels.get("stable").or_else(|| product.channels.get("beta")).map(|r| r.version.clone()).unwrap_or_default();
        let released = !product.channels.is_empty();
        match family.apps.iter_mut().find(|a| a.id == product.id) {
            Some(app) => {
                app.name = product.name;
                app.kind = product.kind;
                app.summary = product.summary;
                app.state = product.state;
                app.version = version;
                app.released = released || app.released;
                if let Some(agents) = product.agents {
                    app.mcp = agents.mcp;
                }
            }
            None => {
                let agents = product.agents.unwrap_or(Agents { mcp: false });
                family.apps.push(App {
                    app: product.id.clone(),
                    launcher: product.id.clone(),
                    id: product.id,
                    name: product.name,
                    kind: product.kind,
                    summary: product.summary,
                    repository: product.repository,
                    version,
                    state: product.state,
                    released,
                    mcp: agents.mcp,
                });
            }
        }
    }
    family.apps.sort_by(|a, b| a.id.cmp(&b.id));
    family
}

pub fn load(bundled: Family) -> Family {
    let fallback = |mut family: Family, why: String| {
        family.note = Some(format!("{}; the catalogue was not used: {why}", family.describe_source()));
        family
    };
    let (body, signature, from) = match read_source() {
        Ok(found) => found,
        Err(why) => return fallback(bundled, why),
    };
    let trusted = match &signature {
        Some(sig) => verify(&body, sig, &keys()),
        None if flag("FORMMCP_ALLOW_UNSIGNED") => Ok(()),
        None => Err("it is not signed".to_string()),
    };
    if let Err(why) = trusted {
        return fallback(bundled, format!("{from}: {why}"));
    }
    let catalog: Catalog = match serde_json::from_slice(&body) {
        Ok(c) => c,
        Err(e) => return fallback(bundled, format!("{from}: not a catalogue ({e})")),
    };
    if catalog.format != CATALOG_FORMAT {
        return fallback(bundled, format!("{from}: format {:?}", catalog.format));
    }
    let count = catalog.products.len();
    let signed = if signature.is_some() { "signed" } else { "unsigned, allowed by FORMMCP_ALLOW_UNSIGNED" };
    let mut family = merge(bundled, catalog.products);
    family.note = Some(format!("the family catalogue ({from}, {signed}, {count} products) over the bundled list"));
    family
}
