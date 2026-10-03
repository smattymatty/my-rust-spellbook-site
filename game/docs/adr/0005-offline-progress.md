# ADR 0005 — Offline progress: capped catch-up from a persisted lastSeen

**Status:** Accepted
**Date:** 2026-06-30

## Context

It is an idle game — idling is the genre's whole promise — and the theme says
the labor is eternal, which argues the boulder should climb while the tab is
closed. Crediting time away requires knowing how much time passed, which
requires persisting a `lastSeen` timestamp. That opens the classic exploit: a
player sets their system clock forward to manufacture progress. This decision is
hard to reverse (it shapes the save format), surprising without context, and a
genuine trade-off (eternal labor vs. exploitability).

## Decision

Offline progress is **on, and capped**.

- On save, persist `lastSeen = now`.
- On load, compute `dt = now - lastSeen`, clamp it to a catch-up cap
  (initial target ~8h), and feed it through the fixed-step accumulator
  (ADR 0004) — the same loop a live frame uses.

The clock-forward exploit is **accepted but bounded**. The cap limits any single
jump, and the game is single-player with no leaderboard or shared economy, so
the entire blast radius of cheating is "the player spoils their own game."
We spend no complexity on clock-tamper detection.

## Privacy (Law 25 / PIPEDA)

`lastSeen` is a timestamp written to the player's own `localStorage`. It is
never transmitted to a Storm server, never collected, never retained by us.
There is no personal information flow and no new data-subject class — the
hosted-site-visitor concern does not arise because nothing leaves the device.
If a future version ever syncs saves server-side, that is a new collection and
triggers a fresh assessment before it ships.

## Alternatives considered

- **Uncapped offline** — thematically pure ("labor is eternal") but clock-forward
  yields unbounded resources and trivializes prestige. Rejected: mechanically
  broken.
- **No offline progress** — no timestamp, no exploit, maximally simple and
  private, but it contradicts the genre and the theme. Rejected: an idle game
  that doesn't idle.

## Consequences

- The save format carries `lastSeen` (see save-schema ADR).
- The catch-up cap is a tuned balance knob, not a constant to set once.
- "A player sets their clock forward an hour — what do they get, and why?" is a
  settled question: up to the cap's worth of catch-up, and we don't care,
  because it is single-player. (This is the recall/drift-check target later.)
