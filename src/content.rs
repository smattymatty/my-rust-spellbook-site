use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use serde::Deserialize;
use gray_matter::{Matter, engine::YAML};
use walkdir::WalkDir;

#[derive(Debug, Default, Deserialize)]
pub struct Frontmatter {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub published_at: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub issue: Option<u32>,
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub project_url: Option<String>,
    #[serde(default)]
    pub pr_url: Option<String>,
    #[serde(default)]
    pub quote: Option<String>,   // a pull-quote from this post; feeds the index rotation
    #[serde(default)]
    pub featured: bool,          // elevates the post into the FEATURED slot above RECENT WRITING
    #[serde(default)]
    pub cover: Option<String>,   // site-root path to the post's cover image, e.g.
                                 // "/media/images/newsletter/foo.png". Becomes the
                                 // og:image (social card) AND a hero atop the article.
    #[serde(default)]
    pub cover_alt: Option<String>, // alt text / og:image:alt for the cover. Strongly
                                   // recommended whenever `cover` is set (a11y + SEO).
}

fn default_kind() -> String { "post".to_string() }

pub struct Document {
    pub frontmatter: Frontmatter,
    pub body: String,
}

pub fn discover(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in WalkDir::new(root) {
        let entry = entry.context("walking content directory")?;
        if entry.file_type().is_file()
            && entry.path().extension().map_or(false, |ext| ext == "md")
        {
            paths.push(entry.path().to_path_buf());
        }
    }
    Ok(paths)
}

pub fn parse(path: &Path) -> Result<Document> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;

    let matter = Matter::<YAML>::new();
    let result = matter
        .parse::<Frontmatter>(&raw)
        .with_context(|| format!("parsing frontmatter in {}", path.display()))?;

    Ok(Document {
        frontmatter: result.data.with_context(|| format!("No frontmatter for {}", path.display()))?,
        body: result.content,
    })
}
