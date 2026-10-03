//! Deploy dist/ to the Storm Buckets bucket (Garage, S3-compatible).
//!
//! Builds the site, diffs dist/ against the bucket by content (MD5 vs ETag),
//! and mirrors only what actually changed.
//!
//! Run from the project root, where dist/ and .env live:
//!     cargo run -p deploy                 # build, diff, then apply changes
//!     cargo run -p deploy -- --dry-run    # show what would change, touch nothing
//!     cargo run -p deploy -- --yes        # skip the confirmation prompt
//!     cargo run -p deploy -- --no-build   # deploy the existing dist/ as-is

use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{Delete, ObjectIdentifier};
use aws_sdk_s3::{Client, Config};
use walkdir::WalkDir;

#[tokio::main]
async fn main() -> Result<()> {
    let dry_run = std::env::args().any(|a| a == "--dry-run");
    let assume_yes = std::env::args().any(|a| a == "--yes");
    let no_build = std::env::args().any(|a| a == "--no-build");

    // Build the site first so dist/ always reflects current source. The deploy
    // is otherwise a foot-gun: an unbuilt dist/ silently ships stale HTML.
    if !no_build {
        println!("building site...");
        let status = std::process::Command::new("cargo")
            .args(["run", "--quiet", "--bin", "rust-spellbook-static"])
            .status()
            .context("running the site build")?;
        if !status.success() {
            bail!("site build failed; aborting deploy");
        }
        println!();
    }

    let dist = Path::new("dist");
    if !dist.is_dir() {
        bail!(
            "no dist/ directory in {} - build the site first",
            std::env::current_dir()?.display()
        );
    }

    // Every connection detail comes from .env; nothing is hardcoded.
    let env: HashMap<String, String> = load_env(Path::new(".env"))?;
    let get = |key: &str| -> Result<String> {
        env.get(key)
            .filter(|v| !v.is_empty())
            .cloned()
            .ok_or_else(|| anyhow!("missing {key} in .env"))
    };
    let endpoint: String = get("BUCKETS_ENDPOINT")?;
    let bucket: String = get("BUCKETS_KEY_BUCKET")?;

    let creds: Credentials = Credentials::new(
        get("BUCKETS_KEY_ID")?,
        get("BUCKETS_KEY_SECRET")?,
        None,
        None,
        "buckets-env",
    );
    let config = Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new(get("BUCKETS_KEY_REGION")?))
        .endpoint_url(&endpoint)
        .credentials_provider(creds)
        .force_path_style(true) // Garage expects path-style bucket addressing
        .build();
    let client = Client::from_conf(config);

    // Local files, keyed by object key (path relative to dist/, forward slashes).
    let mut local: HashMap<String, PathBuf> = HashMap::new();
    for entry in WalkDir::new(dist) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let rel = entry.path().strip_prefix(dist)?;
            local.insert(to_key(rel), entry.path().to_path_buf());
        }
    }

    // Everything currently in the bucket, keyed by object key -> ETag. For the
    // single-part PUTs this tool makes, Garage's ETag is the hex MD5 of the
    // body, so it doubles as a content fingerprint for change detection.
    let mut remote: HashMap<String, String> = HashMap::new();
    let mut pages = client
        .list_objects_v2()
        .bucket(&bucket)
        .into_paginator()
        .send();
    while let Some(page) = pages.next().await {
        let page = page.context("listing bucket")?;
        for obj in page.contents() {
            if let Some(key) = obj.key() {
                let etag = obj.e_tag().unwrap_or_default().trim_matches('"').to_string();
                remote.insert(key.to_string(), etag);
            }
        }
    }

    // Classify every local file: new, content-changed, or byte-identical.
    let mut to_upload: Vec<String> = Vec::new();
    let mut unchanged: usize = 0;
    for (key, path) in &local {
        let bytes = std::fs::read(path)
            .with_context(|| format!("reading {}", path.display()))?;
        let local_md5 = format!("{:x}", md5::compute(&bytes));
        match remote.get(key) {
            Some(etag) if etag.eq_ignore_ascii_case(&local_md5) => unchanged += 1,
            _ => to_upload.push(key.clone()),
        }
    }
    to_upload.sort();

    // Stale: in the bucket but no longer in dist/.
    let mut stale: Vec<String> = remote
        .keys()
        .filter(|k| !local.contains_key(k.as_str()))
        .cloned()
        .collect();
    stale.sort();

    // Nothing to do - the common case after a no-op rebuild.
    if to_upload.is_empty() && stale.is_empty() {
        println!(
            "no changes - bucket is up to date ({} file(s), {unchanged} unchanged)",
            local.len()
        );
        return Ok(());
    }

    // Show the plan before touching anything.
    println!(
        "{}changes for {bucket} ({endpoint}):\n",
        if dry_run { "DRY RUN - " } else { "" }
    );
    if !to_upload.is_empty() {
        println!("  {} to upload:", to_upload.len());
        for key in &to_upload {
            println!("    + {key}  [{}]", content_type(&local[key]));
        }
    }
    if unchanged > 0 {
        println!("  {unchanged} unchanged (skipped)");
    }
    if !stale.is_empty() {
        println!("\n  {} to delete (not in dist/):", stale.len());
        for key in &stale {
            println!("    - {key}");
        }
    }

    if dry_run {
        println!("\n(dry run - nothing changed)");
        return Ok(());
    }
    if !assume_yes && !confirm("apply these changes?")? {
        println!("aborted - nothing changed");
        return Ok(());
    }
    println!();

    // Uploads first - additive, so an aborted prune still leaves the site current.
    for key in &to_upload {
        let path: &PathBuf = &local[key];
        let ctype: &str = content_type(path);
        println!("  upload: {key}");
        let body: ByteStream = ByteStream::from_path(path)
            .await
            .with_context(|| format!("reading {}", path.display()))?;
        client
            .put_object()
            .bucket(&bucket)
            .key(key)
            .body(body)
            .content_type(ctype)
            .cache_control(cache_control(path))
            .send()
            .await
            .with_context(|| format!("uploading {key}"))?;
    }

    // Mirror: prune objects that no longer exist in dist/.
    if !stale.is_empty() {
        // delete_objects accepts up to 1000 keys per call.
        for chunk in stale.chunks(1000) {
            let objects: Vec<ObjectIdentifier> = chunk
                .iter()
                .map(|k| ObjectIdentifier::builder().key(k).build())
                .collect::<Result<_, _>>()?;
            let delete = Delete::builder().set_objects(Some(objects)).build()?;
            client
                .delete_objects()
                .bucket(&bucket)
                .delete(delete)
                .send()
                .await
                .context("deleting stale objects")?;
        }
        println!("  deleted {} object(s)", stale.len());
    }

    println!("\ndone");
    Ok(())
}

/// Minimal .env parser: KEY=VALUE per line, optional quotes, # comments.
fn load_env(path: &Path) -> Result<HashMap<String, String>> {
    let text: String = std::fs::read_to_string(path)
        .with_context(|| format!("no .env at {}", path.display()))?;
    let mut env = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim().trim_matches('"').trim_matches('\'');
            env.insert(key.trim().to_string(), value.to_string());
        }
    }
    Ok(env)
}

/// Object key for a path relative to dist/ - forward slashes on every OS.
fn to_key(rel: &Path) -> String {
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Content types Garage won't reliably infer on its own.
fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("txt") => "text/plain; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        Some("wasm") => "application/wasm",
        _ => "application/octet-stream",
    }
}

/// HTML is rewritten every build, so keep it fresh; other assets can sit a while.
fn cache_control(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "public, max-age=60",
        _ => "public, max-age=3600",
    }
}

/// Default-yes confirmation: a bare Enter (or "y") proceeds; only an explicit
/// "n" backs out. The capital Y in "[Y/n]" signals that Enter means go.
fn confirm(prompt: &str) -> Result<bool> {
    print!("\n{prompt} [Y/n] ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    let answer = answer.trim();
    Ok(answer.is_empty() || answer.eq_ignore_ascii_case("y"))
}
