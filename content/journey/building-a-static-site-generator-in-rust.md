---
title: Building a Static Site Generator in Rust
published_at: 2026-06-19
kind: post
featured: false
description: "Why I rebuilt my personal site as a static site generator in Rust - the Spellbook lineage from Django, compile-time templates with Askama, and hosting the whole thing on my own Storm Buckets."
quote: I maintain Django Spellbook [...] But Django is a lot of machine for a personal site. [...] So I built the Rust version of the idea.
tags:
  - rust
  - static-site-generator
  - self-hosting
  - open-source
---

The site you're reading runs on a static site generator I wrote in Rust. No framework, no database, no runtime - just a binary that turns a folder of Markdown into a folder of HTML, and a second binary that pushes that HTML to a bucket I host myself. This is the story of why I built it, and how it works.

## The Django Spellbook backstory

For years my whole thing was Django. I maintain [Django Spellbook](https://django-spellbook.org) - a Markdown-to-Django CMS that turns Markdown files into rendered templates with auto-generated views and URLs. It's got 10K+ downloads on PyPI, and I still genuinely love it. The core idea - write in Markdown, extend it with custom blocks, let the tooling generate the boring parts - is one I'd bet on again.

But Django is a lot of machine for a personal site. A database I don't need, a request/response cycle for pages that never change, a server that has to stay up. For a blog and a newsletter, every one of those is a moving part that can break at 2am for no reason. I wanted the opposite: something that builds once, produces dead-simple files, and then just sits there being fast.

So I built the Rust version of the idea. Same Spellbook DNA - Markdown in, custom blocks, generated output - but compiled to a single static binary and emitting plain HTML. Think of it as Django Spellbook with the framework boiled off.

## Why this, why now

The other half of the reason is that I'm building [Storm Developments](https://stormdevelopments.ca). Our flagship is [Storm Buckets](https://stormdevelopments.ca/buckets) - managed, S3-compatible object storage hosted entirely in Canada, built on the open-source Garage engine. If I'm going to ask anyone to trust Storm Buckets with their production data, the least I can do is run my own on it.

So this site is dogfood. The generator spits out a `dist/` folder, and a deploy tool mirrors it straight into a Storm Bucket over the standard S3 API. My personal site is served out of my own product, on Canadian infrastructure, on an open stack I can read end to end. Elbows up. That's the whole digital-sovereignty bet in miniature: own your tools, own your output, own the ground it sits on.

## Why Rust

I could have reached for one of the dozen existing static site generators. I didn't, for three reasons.

First, **compile-time guarantees.** I wanted the build to fail loudly and early when something's wrong - a malformed date, a missing tag, a typo in a template - not to silently ship a broken page. Rust's type system plus a strict build step gives me exactly that.

Second, **one binary, no toolchain.** The whole generator compiles to a single executable with no runtime dependencies. There's no Node version to pin, no virtualenv to activate, nothing to rot. I can come back in two years, `cargo run`, and it just works.

Third, I wanted to learn it properly, and the best way to learn a language is to build a real thing you'll actually use. This is that thing.

## How it works

The pipeline is small and boring on purpose. Boring is a feature.

**Markdown to HTML.** I use [`pulldown-cmark`](https://crates.io/crates/pulldown-cmark) for the core Markdown parsing - it's fast, spec-compliant, and unopinionated. Frontmatter (the YAML block at the top of every post) is pulled out with `gray_matter`.

**Validation that fails loud.** Before anything renders, every post's frontmatter is validated: title present, date in `YYYY-MM-DD`, at least one tag, a known `kind`. A newsletter issue *must* carry an issue number. If anything's off, the build aborts with a list of exactly what's wrong and in which file. I'd rather break the build than ship a page with a broken date.

**Compile-time templates with Askama.** This is the part I love most. [Askama](https://crates.io/crates/askama) checks your templates *at compile time* - the variables, the loops, the conditionals, all type-checked against Rust structs before the binary even exists. If I reference a field that isn't there, it's a compiler error, not a blank space on a live page. It feels like the templating equivalent of a seatbelt, and after years of runtime template errors it's a genuine relief.

**Data in TOML, not in code.** The sidebar, the "currently building" list, the forge links, the site config - all of it lives in plain `.toml` files. Editing what's on the page never means recompiling logic; it means editing a data file. Clean separation between the machine and the content.

**SpellBlocks.** This is the Spellbook lineage showing through. Plain Markdown can't do callouts, cards, or accordions, so I built a small block syntax on top of it:

```
{~ alert type="info" ~}
This is a SpellBlock - Markdown plus a little structure.
{~~}
```

Which renders as:

{~ alert type="info" ~}
This is a SpellBlock - Markdown plus a little structure. Same idea as Django Spellbook's blocks, reimplemented in Rust.
{~~}

The block parser is its own module, and adding a new block type is a contained change. It's the one place I let the system get a little clever, because it's the part that makes the writing feel like *mine*.

**The build, end to end.** Discover every Markdown file, parse it, validate it, drop the drafts, render each one, then write it all out. There's a second pass too: once every post is known, the generator computes "related posts" by shared tags and links them at the foot of each article - so the writing builds its own internal web instead of sitting in isolated silos.

**Deploy is its own crate.** A separate binary builds the site, then diffs `dist/` against the live bucket by content - hashing each file and comparing against the object's ETag - and uploads only what actually changed, pruning anything that no longer exists. It talks to Storm Buckets over the AWS S3 SDK with path-style addressing, because Garage speaks plain S3. One command: build, diff, mirror, done.

## What I got out of it

A site that's a single binary plus a folder of Markdown, that fails loudly when I make a mistake, that I can read top to bottom, and that's served out of infrastructure I own. Every layer - the generator, the templates, the data, the storage - is something I control and could fix myself.

That's the point. Not that Rust is better than Django (it isn't, they're different tools for different jobs), but that a personal site is exactly the kind of thing worth owning all the way down. The generator is mine, the HTML is mine, and the bucket it lives in is [mine](https://stormdevelopments.ca/buckets).

Own your tools. Own your future.
