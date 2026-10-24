use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::family::App;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "via", rename_all = "kebab-case")]
pub enum Route {
    Dev { checkout: PathBuf, command: PathBuf },
    Formportal { program: PathBuf, product: String },
}

impl Route {
    pub fn label(&self) -> &'static str {
        match self {
            Route::Dev { .. } => "dev",
            Route::Formportal { .. } => "formportal",
        }
    }

    pub fn command(&self) -> (PathBuf, Vec<String>, Option<PathBuf>) {
        match self {
            Route::Dev { checkout, command } => {
                if cfg!(windows) {
                    (PathBuf::from("cmd"), vec!["/c".into(), command.to_string_lossy().into_owned(), "mcp".into()], Some(checkout.clone()))
                } else {
                    (command.clone(), vec!["mcp".into()], Some(checkout.clone()))
                }
            }
            Route::Formportal { program, product } => (program.clone(), vec!["app-mcp".into(), product.clone()], None),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Reach {
    pub installed: bool,
    pub route: Option<Route>,
}

#[derive(Debug, Clone)]
pub struct Places {
    pub dev_roots: Vec<PathBuf>,
    pub portal_root: PathBuf,
    pub portal_program: Option<PathBuf>,
    installed: HashSet<String>,
}

#[derive(Deserialize)]
struct Installed {
    #[serde(default)]
    installs: Vec<Install>,
}

#[derive(Deserialize)]
struct Install {
    product: String,
    channel: String,
}

pub fn portal_root() -> PathBuf {
    if let Some(root) = std::env::var_os("FORMPORTAL_ROOT") {
        return PathBuf::from(root);
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from).unwrap_or_default();
    if cfg!(target_os = "macos") {
        home.join("Library/Application Support/dev.formai.portal")
    } else if cfg!(target_os = "windows") {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(|| home.join("AppData/Local")).join("FormPortal")
    } else {
        std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|| home.join(".local/share")).join("formportal")
    }
}

fn which(program: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&paths) {
        for name in [program.to_string(), format!("{program}.exe")] {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

pub fn portal_program() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("FORMPORTAL_BIN").map(PathBuf::from) {
        return path.is_file().then_some(path);
    }
    if let Some(found) = which("formportal-cli").or_else(|| which("formportal")) {
        return Some(found);
    }
    let mac = PathBuf::from("/Applications/FormPortal.app/Contents/MacOS/FormPortal");
    (cfg!(target_os = "macos") && mac.is_file()).then_some(mac)
}

fn read_installed(root: &Path) -> HashSet<String> {
    std::fs::read_to_string(root.join("installed.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<Installed>(&t).ok())
        .map(|i| i.installs.into_iter().filter(|x| x.channel == "stable").map(|x| x.product).collect())
        .unwrap_or_default()
}

impl Places {
    pub fn from_env() -> Places {
        let dev_roots = std::env::var_os("FORMMCP_DEV_ROOTS").map(|v| std::env::split_paths(&v).filter(|p| p.is_dir()).collect()).unwrap_or_default();
        let portal_root = portal_root();
        let installed = read_installed(&portal_root);
        Places { dev_roots, portal_root, portal_program: portal_program(), installed }
    }

    pub fn refresh(&mut self) {
        self.installed = read_installed(&self.portal_root);
    }

    fn checkout(&self, app: &App) -> Option<(PathBuf, PathBuf)> {
        let launcher = if app.launcher.is_empty() { app.app.as_str() } else { app.launcher.as_str() };
        let script = if cfg!(windows) { format!("{launcher}.bat") } else { launcher.to_string() };
        for root in &self.dev_roots {
            for folder in [app.app.as_str(), app.id.as_str()] {
                let checkout = root.join(folder);
                let command = checkout.join(&script);
                if command.is_file() {
                    return Some((checkout, command));
                }
            }
        }
        None
    }

    pub fn reach(&self, app: &App) -> Reach {
        let installed = self.installed.contains(&app.id);
        if !app.mcp {
            return Reach { installed, route: None };
        }
        if let Some((checkout, command)) = self.checkout(app) {
            return Reach { installed, route: Some(Route::Dev { checkout, command }) };
        }
        if installed {
            if let Some(program) = &self.portal_program {
                return Reach { installed, route: Some(Route::Formportal { program: program.clone(), product: app.id.clone() }) };
            }
        }
        Reach { installed, route: None }
    }
}
