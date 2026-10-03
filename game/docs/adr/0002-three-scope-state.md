# ADR 0002 — Three nested state scopes, each with its own reset boundary

**Status:** Accepted (supersedes the original per-universe-producer framing)
**Date:** 2026-06-30

## Context

The game is an idle game with three progression layers, not one. The original
framing (a flat `Vec<Universe>` of producers under a root) was wrong on two
counts: there are no producers (one pusher per universe, never bought), and
there is more than one reset boundary. The state shape has to make the three
layers — and what survives each reset — a type-level fact.

## Decision

Three nested scopes:

- **meta** (`GameState`) — survives prestige. Holds defiance, the three skill
  trees, unlocked universes, organizing state, lifetime stats.
- **run** (`Run`) — survives roll-back, dies at prestige. Holds the active
  universe and pusher, scorn, and in-run upgrade levels.
- **climb** (`Climb`) — dies at roll-back. Holds height.

The pusher identity (`PusherKind`) is stored separately from the universe
(`UniverseKind`), even though they map 1:1 today, because the organizing endgame
lets a pusher push in a universe that isn't their native one. Without that
separation the per-pusher and per-universe skill trees could not be told apart.

What crosses a given reset is which struct the datum lives in:

| datum | scope | survives roll-back | survives prestige |
|---|---|---|---|
| height | climb | no | no |
| scorn, in-run upgrades | run | yes | no |
| defiance, skill trees, unlocks | meta | yes | yes |

## Consequences

- Roll-back is `climb = Climb::fresh()`. Prestige is `run = Run::fresh(next, &skills)`.
- A fresh run is *seeded* by the permanent trees ("begin each run with…"), so the
  meta layer reaches into a new run at construction.
- The save mirrors this nesting exactly (ADR 0006).
- Moving a datum between scopes later is a visible struct change, not silent drift.
