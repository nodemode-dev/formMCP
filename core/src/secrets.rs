use keyring::Entry;
use serde::Serialize;

use crate::redact;
use crate::settings::Settings;

pub const SERVICE: &str = "formMCP";
const LONGEST: usize = 16 * 1024;
const RESERVED: [&str; 10] = ["PATH", "HOME", "USERPROFILE", "SHELL", "TMPDIR", "TEMP", "TMP", "NODE_OPTIONS", "PYTHONPATH", "PYTHONSTARTUP"];

#[derive(Debug, Clone, Serialize)]
pub struct SecretRow {
    pub app: String,
    pub name: String,
    pub stored: bool,
    pub error: Option<String>,
}

fn account(app: &str, name: &str) -> String {
    format!("{app}/{name}")
}

fn entry(app: &str, name: &str) -> Result<Entry, String> {
    Entry::new(SERVICE, &account(app, name)).map_err(store_error)
}

fn store_error(e: keyring::Error) -> String {
    match e {
        keyring::Error::PlatformFailure(inner) | keyring::Error::NoStorageAccess(inner) => format!("the system keychain could not be used ({inner}); on Linux a Secret Service such as GNOME Keyring or KWallet must be running and unlocked"),
        other => format!("the system keychain refused: {other}"),
    }
}

pub fn name_problem(name: &str) -> Option<String> {
    let ok_shape = !name.is_empty() && name.len() <= 64 && name.chars().next().is_some_and(|c| c.is_ascii_uppercase() || c == '_') && name.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
    if !ok_shape {
        return Some(format!("{name:?} is not an environment variable name formMCP sets: capitals, digits and underscores, starting with a capital or underscore, at most 64"));
    }
    if RESERVED.contains(&name) || name.starts_with("LD_") || name.starts_with("DYLD_") || name.starts_with("FORMMCP_") {
        return Some(format!("{name} changes how programs run, so formMCP does not set it from the keychain"));
    }
    None
}

pub fn set(app: &str, name: &str, value: &str) -> Result<(), String> {
    if let Some(problem) = name_problem(name) {
        return Err(problem);
    }
    let value = value.trim_end_matches(['\n', '\r']);
    if value.is_empty() || value.len() > LONGEST {
        return Err(format!("a secret is 1 to {LONGEST} bytes"));
    }
    entry(app, name)?.set_password(value).map_err(store_error)?;
    let mut settings = Settings::saved();
    let names = settings.secrets.entry(app.to_string()).or_default();
    if !names.iter().any(|n| n == name) {
        names.push(name.to_string());
        names.sort();
    }
    settings.save()?;
    redact::remember(value);
    Ok(())
}

pub fn remove(app: &str, name: &str) -> Result<(), String> {
    let mut settings = Settings::saved();
    let listed = settings.secrets.get(app).is_some_and(|l| l.iter().any(|n| n == name));
    match entry(app, name)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => {}
        Err(e) if listed => return Err(store_error(e)),
        Err(_) => {}
    }
    if !listed {
        return Err(format!("formMCP keeps no secret {name} for {app}"));
    }
    if let Some(list) = settings.secrets.get_mut(app) {
        list.retain(|n| n != name);
    }
    settings.secrets.retain(|_, l| !l.is_empty());
    settings.save()?;
    Ok(())
}

pub fn env_for(settings: &Settings, app: &str) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    for name in settings.secrets.get(app).cloned().unwrap_or_default() {
        match entry(app, &name)?.get_password() {
            Ok(value) => {
                redact::remember(&value);
                out.push((name, value));
            }
            Err(keyring::Error::NoEntry) => return Err(format!("the secret {name} for {app} is listed in formMCP but missing from the system keychain; set it again in formMCP's window or with formmcp-cli secret set")),
            Err(e) => return Err(store_error(e)),
        }
    }
    Ok(out)
}

pub fn remember_all(settings: &Settings) {
    for (app, names) in &settings.secrets {
        for name in names {
            if let Ok(value) = entry(app, name).and_then(|e| e.get_password().map_err(store_error)) {
                redact::remember(&value);
            }
        }
    }
}

pub fn list(settings: &Settings) -> Vec<SecretRow> {
    let mut rows = Vec::new();
    for (app, names) in &settings.secrets {
        for name in names {
            let (stored, error) = match entry(app, name).map(|e| e.get_password()) {
                Ok(Ok(_)) => (true, None),
                Ok(Err(keyring::Error::NoEntry)) => (false, Some("missing from the keychain".to_string())),
                Ok(Err(e)) => (false, Some(store_error(e))),
                Err(e) => (false, Some(e)),
            };
            rows.push(SecretRow { app: app.clone(), name: name.clone(), stored, error });
        }
    }
    rows
}
