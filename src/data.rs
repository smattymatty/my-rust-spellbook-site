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
    #[serde(default)]
    pub entities: Vec<Entity>, // knowledge-graph topics; feed Person.knowsAbout + per-post `about`
}

/// A knowledge-graph entity Mathew writes about (a person, a concept, a work).
/// Drives two pieces of invisible schema.org structured data: the author's
/// `knowsAbout` (every entity is a stated area of expertise) and each post's
/// `about` (entities whose `tag` appears in that post's tags). `same_as` should
/// be a stable Wikidata/Wikipedia URL so Google can resolve the entity; omit it
/// for concepts with no clean canonical page.
#[derive(Debug, Deserialize, Clone)]
pub struct Entity {
    pub tag: String,             // matches a post tag; links posts to this entity
    pub name: String,            // display name, e.g. "Simone Weil"
    #[serde(default)]
    pub same_as: Option<String>, // canonical Wikidata/Wikipedia URL; None omits sameAs
}

/// The author identity emitted as schema.org JSON-LD. Drives Google's entity
/// understanding (Knowledge Panel candidacy) and the author/publisher fields on
/// every article. Optional in site.toml; an empty name omits the Person node.
#[derive(Debug, Deserialize, Default)]
pub struct Person {
    #[serde(default)]
    pub name: String,        // "Mathew Storm"
    #[serde(default)]
    pub job_title: String,   // "Cloud Infrastructure Engineer ..." - the VISIBLE title
    #[serde(default)]
    pub occupation: Option<String>, // schema.org hasOccupation, e.g. "Philosopher" -
                                    // an entity claim that never appears in body copy
    #[serde(default)]
    pub same_as: Vec<String>, // canonical profile URLs (LinkedIn, GitHub, company)
}

#[derive(Debug, Deserialize)]
pub struct Newsletter {
    pub name: String,        // "Own Your Stack" - masthead + (Issue 4) feed title
    pub tagline: String,     // masthead tagline
}

/// A project shown on /projects/. The first entry marked `featured` gets the
/// hero treatment - the big card with the rotating image carousel; the rest
/// render as a compact grid below it.
#[derive(Debug, Deserialize)]
pub struct Project {
    pub name: String,
    pub tagline: String,             // one-line pitch under the name
    pub description: String,         // a sentence or two of context
    #[serde(default)]
    pub url: Option<String>,         // primary "visit" link; None = no button
    #[serde(default)]
    pub status: Option<String>,      // badge text, e.g. "LIVE", "BETA" - uppercased in copy already
    #[serde(default)]
    pub featured: bool,              // the hero project (the carousel one)
    #[serde(default)]
    pub tech: Vec<String>,           // stack chips, e.g. ["Garage", "S3 API"]
    #[serde(default)]
    pub links: Vec<ProjectLink>,     // extra links (docs, repo, pricing)
    #[serde(default)]
    pub images: Vec<ProjectImage>,   // explicit carousel slides (custom alt text)
    #[serde(default)]
    pub image_dir: Option<String>,   // site-root folder whose image files are
                                     // folded in as slides automatically (sorted
                                     // by filename, after `images`) - the "just
                                     // drop a screenshot in and rebuild" path
}

#[derive(Debug, Deserialize)]
pub struct ProjectLink {
    pub label: String,
    pub href: String,
}

#[derive(Debug, Deserialize)]
pub struct ProjectImage {
    pub src: String,   // site-root path, e.g. "/media/images/projects/buckets-01.png"
    #[serde(default)]
    pub alt: String,
}

#[derive(Deserialize)]
struct ConnectFile { links: Vec<ConnectLink> }

#[derive(Deserialize)]
struct ForgesFile { entry: Vec<ForgeEntry> }

#[derive(Deserialize)]
struct ProjectsFile { project: Vec<Project> }

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

pub fn load_projects(path: &Path) -> Result<Vec<Project>> {
    let file: ProjectsFile = load_toml(path)?;
    Ok(file.project)
}
