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
    ("photo", &["photo", "raw", "develop", "library", "grade", "exposure", "lightroom", "camera"]),
    ("video", &["video", "cut", "clip", "trim", "footage", "caption", "subtitle", "reel", "timeline", "film", "edit", "mp4"]),
    ("audio", &["audio", "sound", "mix", "podcast", "music", "voice", "record", "wav", "mp3", "noise", "master"]),
    ("motion", &["motion", "animate", "animation", "effects", "vfx", "composite", "compositing", "title", "keyframe"]),
    ("encoder", &["convert", "compress", "transcode", "export", "batch", "encode", "format"]),
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

fn close(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let stem = |w: &str| w.chars().take(5).collect::<String>();
    a.len() >= 4 && b.len() >= 4 && stem(a) == stem(b)
}

fn score(task: &[String], app: &App) -> u32 {
    let mut hay = words(&format!("{} {} {} {}", app.name, app.id, app.kind, app.summary));
    if let Some((_, extra)) = KIND_WORDS.iter().find(|(k, _)| *k == app.kind) {
        hay.extend(extra.iter().map(|w| w.to_string()));
    }
    let mut total = 0;
    for word in task {
        if word == &app.id || word == &app.name.to_lowercase() || word == &app.app.to_lowercase() {
            total += 5;
        } else if word == &app.kind {
            total += 2;
        } else if hay.iter().any(|h| close(word, h)) {
            total += 1;
        }
    }
    total
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
    let mut ranked: Vec<(u32, &App)> = family
        .apps
        .iter()
        .filter(|a| a.id != PORTAL_ID || about_installing)
        .map(|a| (score(&task_words, a), a))
        .filter(|(s, _)| *s > 0)
        .collect();
    ranked.sort_by(|x, y| y.0.cmp(&x.0).then(y.1.released.cmp(&x.1.released)).then(x.1.id.cmp(&y.1.id)));
    ranked
        .into_iter()
        .take(limit.clamp(1, 5))
        .map(|(score, app)| {
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
