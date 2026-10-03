# ADR 0007 — Portable save (export/import JSON) with a two-way version guard

**Status:** Accepted
**Date:** 2026-06-30

## Context

The save is already a versioned JSON blob (ADR 0006). Making it exportable and
importable is nearly free and aligns with the project's ownership stance: the
player owns their run, can download it, and is not locked into one device or one
deployment. But once a blob can come from *anywhere* — an export from a newer
build, a friend's file, an old tab left open across a deploy — the loader must
handle a blob whose `schema_version` is ahead of the running code, not just
behind it.

## Decision

**Portability.** Export writes the current versioned blob to a downloadable JSON
file; import reads such a file and routes it through the exact same load path as
a `localStorage` blob — including the migration chain (ADR 0006). An exported
file is nothing more than the save blob; there is no separate export format.

**Two-way version guard on load (localStorage or import):**

- `blob.version < CURRENT` → run the forward migration chain to current, then
  deserialize. (ADR 0006.)
- `blob.version == CURRENT` → deserialize directly.
- `blob.version > CURRENT` → **refuse and preserve.** Do not migrate (there is no
  forward path into shapes this code doesn't know) and do not wipe. Surface
  "this save is from a newer version" and leave the blob byte-for-byte intact so
  newer code can recover it later.

The forward-version stance mirrors Django refusing to operate when the database
is ahead of the code's known migrations: an unknown-future state is a stop, not
a guess.

## Security & privacy

Imported JSON is untrusted input, but the blast radius is bounded: the game is
single-player and fully client-side, so a malformed or hand-edited blob can only
break or cheat the importer's own save. Import validates that it parses and
carries a `schema_version`; beyond that, a doctored save just means the player
spoiled their own run. No data leaves the device; export is a local file
download, import is a local file read. No Storm server, no collection.

## Consequences

- One load path serves localStorage and import alike — the migration chain and
  version guard are written once and exercised by both.
- "Refuse & preserve" means a stale cached tab across a deploy never corrupts a
  newer save; it just declines to touch it.
- Export/import is the manual backup story and the no-lock-in story in a single
  feature, at almost no extra cost over ADR 0006.
