use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NowEntry {
    pub name: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub href: Option<String>,
    #[serde(default)]
    pub started_at: Option<String>,  // YYYY-MM-DD; for future history page
    #[serde(default)]
    pub ended_at: Option<String>,    // YYYY-MM-DD; absent = still active
}

#[derive(Debug, Deserialize)]
pub struct Now {
    #[serde(default)]
    pub building: Vec<NowEntry>,
    #[serde(default)]
    pub writing: Vec<NowEntry>,
    #[serde(default)]
    pub contributing: Vec<NowEntry>,
}

impl Now {
    /// Keep only entries that haven't been marked ended.
    /// This is what the sidebar renders; ended entries are kept in the file for a future history view.
    pub fn active(&self) -> Now {
        Now {
            building: self.building.iter().filter(|e| e.ended_at.is_none()).cloned().collect(),
            writing: self.writing.iter().filter(|e| e.ended_at.is_none()).cloned().collect(),
            contributing: self.contributing.iter().filter(|e| e.ended_at.is_none()).cloned().collect(),
        }
    }
}

// Clone needed for the active() filter above.
impl Clone for NowEntry {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            note: self.note.clone(),
            href: self.href.clone(),
            started_at: self.started_at.clone(),
            ended_at: self.ended_at.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ConnectLink {
    pub key: String,   // shown text AND the link (e.g. "LINKEDIN", "MASTODON")
    pub href: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ForgeEntry {
    pub category: String,    // "WORK" | "OPEN SOURCE" | "CONTRIBUTING"
    pub name: String,        // bold display name (e.g. "Storm Forge")
    pub url: String,         // link target
    pub display: String,     // URL as displayed text below name
}

#[derive(Debug, Deserialize)]
pub struct Site {
    pub base_url: String,    // public origin readers arrive at; no trailing slash
    pub description: String, // site-wide meta/OG description; fallback for posts without one
    pub og_image: String,    // path to the social-share image (e.g. "/og-image.png"); "" = none
    #[serde(default)]
    pub person: Person,      // identity for schema.org JSON-LD (Person / author / publisher)
}

/// The author identity emitted as schema.org JSON-LD. Drives Google's entity
/// understanding (Knowledge Panel candidacy) and the author/publisher fields on
/// every article. Optional in site.toml; an empty name omits the Person node.
#[derive(Debug, Deserialize, Default)]
pub struct Person {
    #[serde(default)]
    pub name: String,        // "Mathew Storm"
    #[serde(default)]
    pub job_title: String,   // "Cloud Infrastructure Engineer ..."
    #[serde(default)]
    pub same_as: Vec<String>, // canonical profile URLs (LinkedIn, GitHub, company)
}

#[derive(Debug, Deserialize)]
pub struct Newsletter {
    pub name: String,        // "Own Your Stack" - masthead + (Issue 4) feed title
    pub tagline: String,     // masthead tagline
}

#[derive(Deserialize)]
struct ConnectFile { links: Vec<ConnectLink> }

#[derive(Deserialize)]
struct ForgesFile { entry: Vec<ForgeEntry> }

fn load_toml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&raw)
        .with_context(|| format!("parsing {}", path.display()))
}

pub fn load_now(path: &Path) -> Result<Now> {
    load_toml(path)
}

pub fn load_connect(path: &Path) -> Result<Vec<ConnectLink>> {
    let file: ConnectFile = load_toml(path)?;
    Ok(file.links)
}

pub fn load_forges(path: &Path) -> Result<Vec<ForgeEntry>> {
    let file: ForgesFile = load_toml(path)?;
    Ok(file.entry)
}

pub fn load_site(path: &Path) -> Result<Site> {
    load_toml(path)
}

pub fn load_newsletter(path: &Path) -> Result<Newsletter> {
    load_toml(path)
}
