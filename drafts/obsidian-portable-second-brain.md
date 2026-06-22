---
# DRAFT - lives in /drafts (outside content/, so it is never built).
# When it's ready: polish, then move it to content/journey/ and the generator
# will pick it up. `draft: true` is a second safety net if you move it early.
title: A Portable Second Brain - Obsidian on Any S3
published_at: 2026-06-19
kind: post
draft: true
featured: false
description: "Part two: you don't have to run your own Garage to own your notes. Any S3-compatible storage turns an Obsidian vault into a portable second brain - here's the note-taking strategies compared, and the no-ops way to sync."
quote: "TODO - pick a punchy pull-quote. Candidate: A second brain you can't move isn't yours - it's rented."
tags:
  - obsidian
  - object-storage
  - self-hosting
  - canadian-digital-sovereignty
  - second-brain
---

<!--
DRAFT STATUS - sections marked ✍️ TODO are yours to write in your own words.
Everything else is a first-pass draft you can keep, cut, or rewrite.
-->

## The sequel

[Part one](/journey/why-im-so-excited-about-garage.html) was for the self-host nerds - the people who want to get down and dirty, spin up a VPS, run their own Garage node, terminate TLS with Caddy, and own the metal their notes live on. If that's you, go read it. It's the deep end, and it's worth it.

This one is for everyone else.

You don't have to run your own storage server to own your notes. The same trick - an Obsidian vault synced to S3-compatible storage - works against *any* S3 endpoint. Backblaze, Wasabi, AWS itself, or - the one I'd point you at, because I make it - [Storm Buckets](https://stormdevelopments.ca/buckets). Same portability, same ownership, none of the Docker.

This is the part-one promise without the part-one homework.

## ✍️ TODO: your hook - why a second brain at all

> Write the personal opener here. The first post never talked about *how you actually take notes* - just the plumbing. This is where that goes.
>
> Ideas to pull from:
> - When/why you started using Obsidian.
> - What broke before (Notion lock-in? lost notes? a tool that shut down?).
> - What "second brain" actually means to you day to day.
> - Keep it personal - this is the part only you can write.

## What makes a second brain *portable*

Here's the thing most note apps get wrong: they own your notes. Notion, Evernote, Roam - your thinking lives in their database, in their format, reachable only through their app, exportable only on their terms. The day they raise prices, change direction, or shut down, your second brain goes with them.

Obsidian flips that. Your vault is just a folder of plain Markdown files on your own disk. No database, no proprietary format - text files you could open in Notepad in twenty years. That's already most of the battle.

The last piece is sync. You want your notes on your phone, your laptop, your desktop - the same brain everywhere. Obsidian sells its own sync service, and it's fine. But the *portable* answer is to sync those plain files to storage you control. That's where S3 comes in: it's the universal protocol for "put a file somewhere and get it back," spoken by every cloud and every self-hosted alternative. Sync your vault to an S3 bucket and your second brain is yours - the files, the format, and the place they live.

Plain files. A protocol everyone speaks. Storage you choose. That's portability.

## The strategies, compared

The plumbing is solved. The harder question - the one part one skipped entirely - is *how you organize what's inside the vault*. There's no shortage of opinions. Here are the four that actually matter, what each is good at, and where each falls down.

### PARA - organize by action

Tiago Forte's system: every note goes into one of four buckets - **P**rojects (active, with a deadline), **A**reas (ongoing responsibilities), **R**esources (topics of interest), **A**rchives (done, inactive). The sorting question is "how actionable is this?"

- **Best for:** getting things done. If your notes are in service of *output* - shipping projects, running your life - PARA keeps the active stuff in front of you.
- **The tradeoff:** it's an organizing system, not a thinking system. It tells you where to *put* a note, not how notes *connect*.

### Zettelkasten - organize by connection

Niklas Luhmann's method, the one that launched a thousand blog posts. Write **atomic** notes - one idea each - in your own words, and densely link them. You don't file notes into folders; you wire them to related notes, and structure *emerges* from the links.

- **Best for:** thinking and writing. If you're doing research, developing ideas over years, or writing long-form, the link graph becomes a thinking partner.
- **The tradeoff:** high discipline, slow payoff. An empty Zettelkasten is just homework. It rewards you in year two, not week one.

### Johnny.Decimal - organize by address

A strict numbering scheme: areas (10-19, 20-29...), categories within them, and a unique decimal ID per item (e.g. `12.04`). Everything has one home and one number.

- **Best for:** findability and teams. When you need to *locate* something fast and never wonder "where did I put that," the address system is unbeatable.
- **The tradeoff:** rigid. It fights the messy, associative way ideas actually grow - great for reference, awkward for thinking-in-progress.

### MOCs / Linking Your Thinking - organize by emergence

Nick Milo's Maps of Content (building on Andy Matuschak's evergreen notes). Don't pre-build folders; let notes accumulate, then create **Maps of Content** - hub notes that link out to related notes - *when* a cluster gets big enough to need one. Structure grows on demand.

- **Best for:** people who hate rigid systems and want organization to follow their actual thinking, not precede it.
- **The tradeoff:** it can drift. Without occasional gardening, "emergent" becomes "messy."

### Where I land

> ✍️ TODO: this is YOUR section - which do you actually use? Be honest.
>
> Most people land on a hybrid (PARA for the action layer + links/MOCs for the
> thinking layer is a common combo). Say what you do, what you tried and dropped,
> and why. This is the opinion readers came for - don't outsource it to me.

## Why S3 is the right sync layer

> ✍️ Light TODO - keep/trim this; make sure it matches your actual setup.

Whatever organizing system you pick, it's still just folders and Markdown files underneath. That's exactly what makes S3 the right place to sync them:

- **It's universal.** Every cloud and every self-hosted store speaks S3. Pick a provider today, switch tomorrow - same protocol, one config change.
- **It's just files.** No proprietary sync format. What's in the bucket is your vault, readable by anything.
- **You choose the jurisdiction.** Your notes are some of the most personal data you own. Putting them on storage *you* picked - in a country you picked - is the whole sovereignty argument in miniature.

## The setup - the easy way

In part one I ran my own Garage node for this: VPS, Docker, Caddy, DNS, the works. Worth it if you want to own the hardware. Overkill if you just want your notes synced.

The shortcut is a managed S3 bucket. No server to run, no TLS to terminate, no node to keep alive - you get an endpoint, a key, and a secret, and you point Obsidian at it.

{~ card ~}
I'll be straight with you: the one I'd point you at is my own. **[Storm Buckets](https://stormdevelopments.ca/buckets)** is managed, S3-compatible object storage hosted entirely in Canada, built on the same Garage engine from part one - without you having to run it. Founding Alpha slots are 100 GB, free during alpha. It's the no-ops version of everything part one set up by hand.
{~~}

The Obsidian side is identical to part one - the [Remotely Save](https://github.com/remotely-save/remotely-save) plugin, "S3 or compatible" - just pointed at a managed endpoint instead of your own:

- **Endpoint:** your provider's S3 URL
- **Region:** whatever your provider specifies (for Storm Buckets / Garage it's `garage`, not `us-east-1` - the classic gotcha)
- **Access Key ID / Secret:** from your provider's dashboard
- **Bucket Name:** your bucket
- **S3 URL Style:** Path-Style

> ✍️ TODO: drop in the exact Storm Buckets values / a screenshot once you've
> run through it yourself, so this is a real walkthrough and not hand-waving.
> (You did this for NorthTube already - same muscle.)

Hit check, hit sync. Your second brain now lives on storage you chose, in a country you chose, in plain files you'll be able to read forever.

## Close

> ✍️ TODO: land the plane in your own words. The thread to pull:
> part one was about owning the *server*; this one is about owning the *thinking*
> that lives on it. A second brain you can pick up and move is a second brain you
> actually own. Tie it back to the sovereignty thesis and sign off.

Own your tools. Own your future. Own your notes.
