use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use walkdir::WalkDir;

pub fn write_html(
    content_root: &Path,
    output_root: &Path,
    source: &Path,
    html: &str,
) -> Result<PathBuf> {
    // content/blog/post.md → blog/post.md
    let relative = source
        .strip_prefix(content_root)
        .context("source path not under content root")?;

    // blog/post.md → blog/post.html
    let output_path = output_root.join(relative).with_extension("html");

    // Make sure dist/blog/ exists before writing into it
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }

    fs::write(&output_path, html)
        .with_context(|| format!("writing {}", output_path.display()))?;

    Ok(output_path)
}

pub fn url_for(content_root: &Path, source: &Path) -> Result<String> {
    let relative = source
        .strip_prefix(content_root)
        .context("source path not under content root")?;
    let with_html = relative.with_extension("html");
    Ok(format!("/{}", with_html.display()))
}

pub fn write_index(output_root: &Path, html: &str) -> Result<()> {
    let path = output_root.join("index.html");
    fs::write(&path, html)
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

/// Write `html` to `output_root`/`rel`, creating parent directories as needed.
/// For one-off pages whose path is not derived from a content file.
pub fn write_at(output_root: &Path, rel: &str, html: &str) -> Result<PathBuf> {
    let path = output_root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    fs::write(&path, html)
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

pub fn copy_static(static_root: &Path, output_root: &Path) -> Result<usize> {
    if !static_root.exists() {
        return Ok(0);
    }
    let mut count = 0;
    for entry in WalkDir::new(static_root) {
        let entry = entry.context("walking static directory")?;
        if !entry.file_type().is_file() {
            continue;
        }
        let relative = entry.path()
            .strip_prefix(static_root)
            .context("static path not under static root")?;
        let dest = output_root.join(relative);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        fs::copy(entry.path(), &dest)
            .with_context(|| format!("copying {} to {}", entry.path().display(), dest.display()))?;
        count += 1;
    }
    Ok(count)
}
