# mathewstorm.ca

A hand-rolled static site generator in Rust that builds [mathewstorm.ca](https://mathewstorm.ca)
from Markdown + TOML. Markdown becomes HTML via `pulldown-cmark`, templates are
compile-time-checked with `askama`, and the result is a plain `dist/` folder of
static files. A second crate (`deploy/`) mirrors `dist/` to a Storm Buckets
(Garage, S3-compatible) bucket, uploading only what changed.

## Layout

```
content/        Markdown posts, grouped by section (journey/, tools/, newsletter/)
data/           Site config + sidebar data (site.toml, now.toml, forges.toml, ...)
templates/      askama HTML templates (base, article, index, newsletter, feeds)
static/         Copied verbatim into dist/ (CSS, JS, images, robots.txt)
src/            The generator
deploy/         The S3/Garage deploy crate
dist/           Build output (git-ignored, regenerated every build)
```

## Build

```bash
cargo run                 # build the site into dist/
```

The build clears `dist/`, copies `static/`, renders every non-draft post, and
writes the index, `/newsletter/`, the Atom feed, and `sitemap.xml`. It fails
loudly on bad frontmatter (see the rules below).

## Preview locally

```bash
cd dist && python -m http.server 8000     # then open http://localhost:8000
```

## Write a post

Drop a Markdown file in the right `content/<section>/` folder. Frontmatter:

```yaml
---
title: The Internet Wasn't Inevitable      # required
published_at: 2026-05-14                    # required, YYYY-MM-DD
kind: newsletter                           # post | newsletter | contribution | dispatch | tutorial
tags: [infrastructure, open-source]        # required, at least one
description: "One-line summary for SEO + social cards."
quote: A pull-quote that feeds the index rotation.   # optional
featured: true                             # optional - at most ONE post site-wide
cover: /media/images/newsletter/foo.png    # optional - hero image + social card
cover_alt: "Describe the cover for a11y + SEO."
issue: 1                                    # required when kind: newsletter
draft: true                                # optional - excludes from the build
---
```

Rules the build enforces:

- **One featured post, ever.** Two `featured: true` posts fail the build, naming
  the newest to keep and the older one(s) to un-feature.
- **Images** go in `static/media/images/...` and are referenced from the site
  root, e.g. `![alt](/media/images/newsletter/chart.png)`.
- **SpellBlocks** add rich blocks, e.g. `{~ alert type="info" ~} ... {~~}`
  (types: info, warning, success, danger).

## Deploy

Needs a `.env` in the project root with the bucket credentials:

```
BUCKETS_ENDPOINT=...
BUCKETS_KEY_BUCKET=...
BUCKETS_KEY_ID=...
BUCKETS_KEY_SECRET=...
BUCKETS_KEY_REGION=...
```

Then:

```bash
cargo run -p deploy                 # build, diff against the bucket, upload changes
cargo run -p deploy -- --dry-run    # show what would change, touch nothing
cargo run -p deploy -- --yes        # skip the confirmation prompt
cargo run -p deploy -- --no-build   # deploy the existing dist/ as-is
```

The deploy rebuilds first (so `dist/` is never stale), fingerprints every file
by content (MD5 vs the bucket's ETag), uploads only new/changed files, and prunes
objects no longer in `dist/`. It prints the full plan and waits for confirmation
before touching anything.

---

Design notes and the original build spec live in [`_architecture/`](_architecture/).
