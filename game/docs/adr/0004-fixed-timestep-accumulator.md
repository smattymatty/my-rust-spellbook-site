# ADR 0004 — Fixed-timestep accumulator, driven by rAF

**Status:** Accepted
**Date:** 2026-06-30

## Context

The sim needs a notion of time that is stable across machines and frame rates,
and that has a single, reusable path for the big time jump that offline progress
represents. Threshold and event logic (a summit reached, a pusher unlocked) must
fire at predictable points, not wherever frames happen to land.

## Decision

JS `requestAnimationFrame` calls `tick(realDt)` each frame. Rust holds an
accumulator: it adds `realDt`, then advances the sim in fixed logical steps
(target STEP ~100ms), draining the accumulator, and renders from the resulting
snapshot. The number of catch-up steps per call is capped to avoid a spiral of
death after a long pause.

Offline progress reuses this exact loop: on load, `dt = now - lastSeen` is fed
in as one large `realDt`, drained in fixed steps under the same cap (see the
offline-progress ADR).

## Alternatives considered

- **Raw frame-delta (`res += rate * dt`)** — simplest, but frame-rate-dependent
  rounding makes it non-deterministic and events fire at irregular intervals.
  Rejected: drift and unpredictability.
- **Rust self-ticks on a fixed timer** — decouples sim from render rate, but
  still needs wall-clock math for offline and adds a second clock to reconcile.
  Rejected: the accumulator with extra plumbing.

## Consequences

- Sim advances in identical discrete steps everywhere; replays and offline
  catch-up are deterministic given the same inputs.
- The catch-up cap is a real game-balance knob and the first line of defence
  against the clock-forward exploit (see offline-progress ADR).
- All rate math is "per STEP," not "per frame" — a discipline formulas follow.
