# ADR 0003 — Sim/render boundary is snapshot-out, commands-in

**Status:** Accepted
**Date:** 2026-06-30

## Context

Rust owns all state and rules; JS owns the DOM. Every frame the JS UI must read
state to draw it, and the player's input must reach the sim. How those two
crossings work decides how tightly the UI couples to the sim — and the design
calls for genuinely distinct UIs per universe, so the UI must be free to change
without dragging the sim with it.

## Decision

A unidirectional contract:

- **Out:** `tick(dt) -> Snapshot`. Rust advances the sim, then returns one
  read-only render snapshot (a view-model, not the internal state). JS renders
  purely from the snapshot.
- **In:** `dispatch(cmd: Command)`. All player input is an explicit command
  (`Push`, `BuyPusher`, `Prestige`, …). JS never writes state directly.

State flows one way: Rust → JS is read-only, JS → Rust is write-only and only
through the command vocabulary.

## Alternatives considered

- **Fine-grained getters** (`get_height()`, `get_scorn()`, …) — many crossings
  per frame and the UI couples to the field layout, so every sim refactor
  ripples into JS. Rejected: chatty and brittle.
- **Shared linear memory, JS reads raw** — zero-copy and fastest, but binds the
  UI to Rust's in-memory struct layout. Rejected: unsafe and a premature
  optimization at idle-game tick rates.

## Consequences

- The `Snapshot` view-model is a separate type from internal state — a
  deliberate translation layer. Internal refactors that don't change the
  snapshot don't touch JS at all.
- New player actions are new `Command` variants — a closed, auditable vocabulary
  of everything the player can do.
- Per-universe UIs are pure render functions over the snapshot; swapping a
  universe's look is a JS-only change.
- One serialize per tick. Negligible at idle-game scale; revisit only if a
  profile ever says so.
