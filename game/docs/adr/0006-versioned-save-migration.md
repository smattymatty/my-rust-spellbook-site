# ADR 0006 — Versioned save blob with an ordered migration chain

**Status:** Accepted
**Date:** 2026-06-30

## Context

The save lives in `localStorage` and must survive both a browser refresh and,
critically, shipping new versions of a long-term game people invest weeks in.
A v1 save will routinely meet v2/v3 code after an update — when a new pusher
type or a whole new universe has been added. Losing or corrupting a run on
update is unacceptable. This is hard to reverse (the loader's contract is set
early), surprising (why carry migration machinery in a toy idle game?), and a
real trade-off (migration code vs. simpler reset-or-defaults).

## Decision

The save is a JSON blob with a top-level `schema_version`. Its shape mirrors the
runtime state (ADR 0002): a root object holding `scorn`, `unlocked`, the
`universes` array, `organizing`, and `lastSeen` (ADR 0005); numbers serialize in
the big-float form (ADR 0001).

On load, the raw blob is run through an ordered chain of migration functions
(`v1→v2→v3 …`) until it reaches the current version, then deserialized into
typed state. Each shipped schema change adds exactly one migration step.

## Alternatives considered

- **Serde defaults, no version field** — `#[serde(default)]` fills missing
  fields with no migration code, but cannot express renames, splits, or value
  reinterpretation, and a v1 blob is indistinguishable from a v3 one. Rejected:
  drift becomes silent corruption with no version to reason about.
- **Reject & reset on mismatch** — trivial, but wipes the player's entire run on
  every update. Rejected: fatal for a game whose whole point is a long climb.

## Consequences

- Every schema change ships with its migration step and a test that loads a
  fixture of the previous version. The migration chain is itself a tested
  artifact.
- `schema_version` is the source of truth for "what shape is this blob,"
  removing the guesswork the defaults approach leaves behind.
- The drift-check later — "a v1 save hits v2 code, what happens?" — has a
  concrete answer: it walks the migration chain to current, deterministically.
- Migrations are pure data transforms on the raw blob, run before typed
  deserialization, so they never depend on current struct definitions.
