//! Deploy dist/ to the Storm Cellar bucket (Garage, S3-compatible).
//!
//! Run from the project root, where dist/ and .env live:
//!     cargo run -p deploy                 # mirror dist/ into the bucket
//!     cargo run -p deploy -- --dry-run    # show what would change, touch nothing
//!     cargo run -p deploy -- --yes        # skip the confirmation before deletes

use std::collections::{HashMap, HashSet};
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

    let dist = Path::new("dist");
    if !dist.is_dir() {
        bail!(
            "no dist/ directory in {} — build the site first",
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
    let endpoint: String = get("CELLAR_ENDPOINT")?;
    let bucket: String = get("CELLAR_KEY_BUCKET")?;

    let creds: Credentials = Credentials::new(
        get("CELLAR_KEY_ID")?,
        get("CELLAR_KEY_SECRET")?,
        None,
        None,
        "cellar-env",
    );
    let config = Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new(get("CELLAR_KEY_REGION")?))
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

    // Everything currently in the bucket.
    let mut remote: HashSet<String> = HashSet::new();
    let mut pages = client
        .list_objects_v2()
        .bucket(&bucket)
        .into_paginator()
        .send();
    while let Some(page) = pages.next().await {
        let page = page.context("listing bucket")?;
        for obj in page.contents() {
            if let Some(key) = obj.key() {
                remote.insert(key.to_string());
            }
        }
    }

    let mut stale: Vec<String> = remote
        .iter()
        .filter(|k| !local.contains_key(k.as_str()))
        .cloned()
        .collect();
    stale.sort();

    println!(
        "{}deploying {} file(s) to {bucket} ({endpoint})\n",
        if dry_run { "DRY RUN — " } else { "" },
        local.len(),
    );

    // Uploads first — additive, so an aborted prune still leaves the site current.
    let mut keys: Vec<&String> = local.keys().collect();
    keys.sort();
    for key in keys {
        let path: &PathBuf = &local[key];
        let ctype: &str = content_type(path);
        println!(
            "  {}: {key}  [{ctype}]",
            if dry_run { "would upload" } else { "upload" }
        );
        if !dry_run {
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
    }

    // Mirror: prune objects that no longer exist in dist/.
    if stale.is_empty() {
        println!("\nno stale objects");
    } else {
        println!("\n{} stale object(s) not present in dist/:", stale.len());
        for key in &stale {
            println!("  - {key}");
        }
        if dry_run {
            println!("(dry run — nothing deleted)");
        } else if !assume_yes && !confirm("delete these from the bucket?")? {
            println!("skipped deletions");
        } else {
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
            println!("deleted {} object(s)", stale.len());
        }
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

/// Object key for a path relative to dist/ — forward slashes on every OS.
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

fn confirm(prompt: &str) -> Result<bool> {
    print!("\n{prompt} [y/N] ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(answer.trim().eq_ignore_ascii_case("y"))
}
