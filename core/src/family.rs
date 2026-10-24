use serde::{Deserialize, Serialize};

const BUNDLED: &str = include_str!("../data/family.json");
pub const FAMILY_FORMAT: &str = "formmcp-family";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct App {
    pub id: String,
    pub app: String,
    pub name: String,
    pub kind: String,
    pub summary: String,
    pub repository: String,
    #[serde(default)]
    pub launcher: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub version: String,
    pub state: String,
    pub released: bool,
    pub mcp: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub repository: String,
    pub version: String,
    pub commit: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Family {
    pub format: String,
    pub format_version: u32,
    pub source: Source,
    pub apps: Vec<App>,
    #[serde(skip)]
    pub note: Option<String>,
}

impl Family {
    pub fn bundled() -> Result<Family, String> {
        let family: Family = serde_json::from_str(BUNDLED).map_err(|e| format!("the bundled family list is not valid ({e}); regenerate it with setup/family.py"))?;
        if family.format != FAMILY_FORMAT {
            return Err(format!("the bundled family list has format {:?}, not {FAMILY_FORMAT:?}", family.format));
        }
        Ok(family)
    }

    pub fn get(&self, name: &str) -> Option<&App> {
        let wanted = name.trim().to_lowercase();
        self.apps.iter().find(|a| a.id == wanted || a.app.to_lowercase() == wanted || a.name.to_lowercase() == wanted)
    }

    pub fn with_mcp(&self) -> impl Iterator<Item = &App> {
        self.apps.iter().filter(|a| a.mcp)
    }

    pub fn describe_source(&self) -> String {
        if let Some(note) = &self.note {
            return note.clone();
        }
        let commit: String = self.source.commit.chars().take(7).collect();
        if self.source.version.is_empty() {
            format!("bundled list from {} at {commit}", self.source.repository)
        } else {
            format!("bundled list from {} {} ({commit})", self.source.repository, self.source.version)
        }
    }
}
