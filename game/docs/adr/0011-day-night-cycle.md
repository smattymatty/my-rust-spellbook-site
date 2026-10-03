# ADR 0011 — The day/night cycle

**Status:** Accepted · **Implemented** 2026-06-30 (clock + animated sky + six phase tracks; 47 tests)
**Date:** 2026-06-30
**Grew from** the night-sky vista (moon + stars) and [day-night-seed.md](../day-night-seed.md).

## Context

The scene already had a moon and stars. The operator wanted to grow that into a
real **time-of-day mechanic** rooted in Camus — noon as lucidity, midnight as the
confrontation with the absurd — where time becomes a buildcrafting axis, not
decoration.

## Decisions

1. **A pure in-game clock.** A **12-minute day**, no tie to real wall-clock time,
   starting from a fixed point. It advances by **elapsed time, including while
   idle** (like the rest of the sim). It loops through **six 2-minute phases**:
   **Dawn, Morning, Noon, Afternoon, Dusk, Midnight.**

2. **Purely ambient.** The clock turns on its own; the player does **not** control
   it (no hold/skip/extend). Phase bonuses are a passive supplement, not a build
   identity, so chasing uptime isn't necessary.

3. **Neutral until you invest.** A phase does nothing on its own — it's a timing
   context. A new player sees the sky move (atmosphere) and feels nothing
   mechanically until they buy phase power.

4. **Power lives in six per-phase tracks on the defiance side.** Alongside the
   existing global / per-pusher / per-universe trees (ADR 0009), the defiance
   layer gains a **fourth category: per-phase tracks**, one per phase. Each is a
   line of passive bonuses that apply **only during their phase**. You invest
   across them over time.

5. **Full sky journey (the visual payoff).** A sun **arcs** across the scene —
   rises at Dawn (low-left), peaks at Noon (high), sets at Dusk (low-right) — then
   the **moon and starfield** (the current night vista) own Midnight. The sky
   **gradient shifts colour per phase**: dawn rose, midday pale, dusk amber, night
   the deep blue already in place. The mountain and boulder are constant; the sky
   tells the time.

## Consequences

- **New meta-scope state:** the clock (seconds into the day, 0..720), on
  `GameState`, advanced by elapsed time in `advance()` (online + offline). It
  persists across runs and prestiges — the cycle is a constant backdrop.
- **Phase tracks** are defiance-bought, keyed by phase; the active phase gates
  which of their bonuses are live. Effects read the current phase in the effect
  layer (push/scorn/etc.).
- **Scene rendering** gains sun position (by phase fraction), a per-phase sky
  gradient, and the existing moon/stars as the Midnight state — smooth colour
  transitions across phase boundaries.
- Implementation is a substantial build (clock + phase effects + six tracks of
  content + the animated sky); it is its own pass, not a quick edit.

## Alternatives considered

- Real wall-clock time — rejected: you'd only ever see one phase per session and
  couldn't build around the rotation.
- Baseline buff per phase, or phase-gating actions — rejected: the operator wants
  phases neutral until invested, as a passive supplement rather than a forced
  timing puzzle.
- Player control of the clock — rejected: it would dissolve the "live with the
  cycle" feel and let you park on one phase.
