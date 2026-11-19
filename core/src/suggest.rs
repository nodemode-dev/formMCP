use serde::Serialize;

use crate::family::{App, Family};
use crate::reach::Places;

const PORTAL_ID: &str = "formai";
const PORTAL_PAGE: &str = "https://github.com/nodemode-dev/formportal";

const STOP: &[&str] = &[
    "a", "an", "the", "and", "or", "to", "of", "in", "on", "for", "with", "my", "me", "i", "it", "this", "that", "is", "are", "be", "can", "you", "your", "some", "from", "into", "by", "at", "as", "do", "make", "want", "need", "please", "help", "new", "using", "use", "app", "apps", "file", "files", "form", "out", "own", "machine", "one", "all", "every", "more", "up", "get", "like", "something", "thing", "things", "work", "way",
];

const KIND_WORDS: &[(&str, &[&str])] = &[
    ("image", &["photo", "picture", "image", "edit", "retouch", "layer", "mask", "background", "remove", "cutout", "crop", "resize", "thumbnail", "poster", "blur", "png", "jpeg", "psd"]),
    ("photo", &["photo", "raw", "develop", "library", "grade", "exposure", "lightroom", "camera", "film", "look", "preset", "grain"]),
    ("video", &["video", "cut", "clip", "trim", "footage", "caption", "subtitle", "reel", "timeline", "film", "edit", "mp4"]),
    ("audio", &["audio", "sound", "mix", "podcast", "music", "voice", "record", "wav", "mp3", "noise", "master"]),
    ("motion", &["motion", "animate", "animation", "effects", "vfx", "composite", "compositing", "title", "keyframe", "sting", "ident", "intro", "outro", "bumper", "chroma", "key", "greenscreen", "roto", "rotoscope"]),
    ("encoder", &["convert", "compress", "transcode", "export", "batch", "encode", "format", "mov", "mp4", "mkv", "webm", "avi", "h264", "264", "h265", "265", "hevc", "av1", "prores", "mp3", "wav", "flac", "aac"]),
    ("layout", &["layout", "page", "magazine", "flyer", "poster", "print", "brochure", "book", "publish", "pdf"]),
    ("vector", &["vector", "illustration", "logo", "svg", "path", "icon", "draw", "artboard", "shape"]),
    ("pdf", &["pdf", "sign", "form", "annotate", "fill", "comment", "redact"]),
    ("notes", &["note", "notes", "doc", "docs", "wiki", "database", "markdown", "write", "journal", "kanban"]),
    ("document", &["document", "word", "letter", "report", "text", "essay", "docx", "write"]),
    ("slides", &["slides", "slide", "presentation", "deck", "pitch", "keynote", "powerpoint", "talk"]),
    ("spreadsheet", &["spreadsheet", "excel", "table", "formula", "csv", "budget", "sheet", "xlsx"]),
    ("library", &["asset", "assets", "library", "organise", "organize", "catalogue", "catalog", "tag", "collection", "dam"]),
    ("reference", &["reference", "moodboard", "board", "inspiration", "collect", "clipping"]),
    ("sculpt", &["sculpt", "3d", "model", "mesh", "clay", "character"]),
    ("texture", &["texture", "material", "pbr", "3d", "paint", "surface"]),
    ("web", &["web", "website", "site", "html", "landing", "page", "blog"]),
    ("chat", &["chat", "agent", "claude", "conversation", "assistant", "artifact"]),
    ("portal", &["install", "update", "uninstall", "download", "rollback", "version"]),
    ("launcher", &["launcher", "dock", "switch", "switcher", "launch"]),
];

#[derive(Debug, Clone, Serialize)]
pub struct Suggestion {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub summary: String,
    pub score: u32,
    pub mcp: bool,
    pub installed: bool,
    pub reach: Option<String>,
    pub state: String,
    pub next: String,
}

fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty() && !STOP.contains(w))
        .map(str::to_string)
        .collect()
}

fn stem(word: &str) -> String {
    let mut w = word.to_string();
    let long = |w: &String, n: usize| w.chars().count() > n;
    if long(&w, 3) && w.ends_with('s') && !w.ends_with("ss") {
        w.pop();
    }
    if let Some(suffix) = ["ing", "ion", "ed"].into_iter().find(|s| long(&w, s.len() + 2) && w.ends_with(s)) {
        w.truncate(w.len() - suffix.len());
        let b = w.as_bytes();
        if b.len() >= 2 && b[b.len() - 1] == b[b.len() - 2] && b[b.len() - 1].is_ascii_alphabetic() && !b"aeioulsz".contains(&b[b.len() - 1]) {
            w.pop();
        }
    }
    if long(&w, 4) && w.ends_with('e') {
        w.pop();
    }
    w
}

fn points(word: &str, app: &App) -> u32 {
    if word == app.id || word == app.name.to_lowercase() || word == app.app.to_lowercase() {
        return 5;
    }
    let wanted = stem(word);
    let kind_words = KIND_WORDS.iter().find(|(k, _)| *k == app.kind).map(|(_, w)| *w).unwrap_or_default();
    if stem(&app.kind) == wanted || kind_words.iter().any(|w| stem(w) == wanted) {
        return 2;
    }
    let hay = words(&format!("{} {} {}", app.name, app.id, app.summary));
    u32::from(hay.iter().any(|h| stem(h) == wanted))
}

fn next_step(app: &App, places: &Places, reach: Option<&str>, installed: bool) -> String {
    if let Some(via) = reach {
        return format!("{} can be reached here ({via}): call app_tools with app {}.", app.name, app.id);
    }
    if !app.mcp {
        let open = if installed { format!("{} is installed; the person can open it.", app.name) } else { format!("Install {} with FormPortal.", app.name) };
        return format!("{} has no MCP server yet, so an agent cannot drive it. {open}", app.name);
    }
    if installed {
        return format!("{} is installed, but formMCP cannot start its server here: its install names no launcher yet, or FormPortal's program was not found (FORMPORTAL_BIN).", app.name);
    }
    if !app.released {
        return format!("{} is not released yet ({}).", app.name, app.state);
    }
    let portal_installed = places.portal_program.is_some() || places.portal_root.join("installed.json").is_file();
    if portal_installed {
        format!("Install {} with FormPortal (open FormPortal, or its install tool for {}). formMCP never installs.", app.name, app.id)
    } else {
        format!("Install FormPortal first ({PORTAL_PAGE}), then install {} with it. formMCP never installs.", app.name)
    }
}

pub fn suggest(family: &Family, places: &Places, running: &dyn Fn(&str) -> Option<String>, task: &str, limit: usize) -> Vec<Suggestion> {
    let task_words = words(task);
    let about_installing = task_words.iter().any(|w| matches!(w.as_str(), "install" | "update" | "uninstall" | "rollback"));
    let apps: Vec<&App> = family.apps.iter().filter(|a| a.id != PORTAL_ID || about_installing).collect();
    let table: Vec<Vec<u32>> = apps.iter().map(|a| task_words.iter().map(|w| points(w, a)).collect()).collect();
    let weight: Vec<f64> = (0..task_words.len())
        .map(|i| {
            let matched = table.iter().filter(|row| row[i] > 0).count().max(1);
            1.0 + (apps.len() as f64 / matched as f64).ln()
        })
        .collect();
    let mut ranked: Vec<(u32, usize, &App)> = apps
        .iter()
        .zip(&table)
        .map(|(a, row)| {
            let total: f64 = row.iter().zip(&weight).map(|(p, w)| f64::from(*p) * w).sum();
            ((total * 10.0).round() as u32, row.iter().filter(|p| **p == 2).count(), *a)
        })
        .filter(|(s, _, _)| *s > 0)
        .collect();
    ranked.sort_by(|x, y| y.0.cmp(&x.0).then(y.1.cmp(&x.1)).then(y.2.released.cmp(&x.2.released)).then(x.2.id.cmp(&y.2.id)));
    ranked
        .into_iter()
        .take(limit.clamp(1, 5))
        .map(|(score, _, app)| {
            let reach = places.reach(app);
            let via = running(&app.id).or_else(|| reach.route.as_ref().map(|r| r.label().to_string()));
            Suggestion {
                next: next_step(app, places, via.as_deref(), reach.installed),
                id: app.id.clone(),
                name: app.name.clone(),
                kind: app.kind.clone(),
                summary: app.summary.clone(),
                score,
                mcp: app.mcp,
                installed: reach.installed,
                reach: via,
                state: app.state.clone(),
            }
        })
        .collect()
}
