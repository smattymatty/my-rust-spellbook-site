# ADR 0010 — Presentation: site-embedded, progressively disclosed, ASCII

**Status:** Accepted
**Date:** 2026-06-30

## Context

The first UI was garbage: a standalone dark page with no site chrome, every
permanent skill tree dumped on screen at once, a hill sloping the wrong way, and
a featureless dot for the pusher. It read as a foreign app bolted next to the
site, not part of it. This ADR fixes the presentation top to bottom.

## Decisions

1. **Embed in the site.** `/boulder/` is a real SSG page that **extends
   `templates/base.html`** — it inherits the site nav, footer, fonts, and CSS
   variables. The game board lives in `{% block content %}`; the board's CSS
   shrinks to just the board and uses the site's type/color tokens. One source of
   chrome, no drift.

2. **Progressive disclosure — the "breadcrumb of one."** Never show what the
   player can't use yet. The player sees their **owned tabs plus exactly ONE
   greyed "next" tab**, hover-revealing a popover of how to unlock it — never the
   tabs beyond that. Unlocks happen one at a time; the game stays small until it
   grows.

3. **Tab order:** **Climb → Scorn → Defiance → Universes.**
   - *Climb* — owned at start: the ASCII scene, Push, momentum, Heave, roll-back.
   - *Scorn* — unlocks at the first roll-back: the in-run shop.
   - *Defiance* — unlocks at the first prestige: the three permanent skill trees.
     They do **not** appear during a normal run.
   - *Universes* — bought with **defiance** (AdVenture-Capitalist-style zones).
   Auto-push (after 5 pushes) and the stance branch appear *inside* their tab, not
   as new tabs.

4. **The board is one ASCII scene**, ascending **left→right** to a summit at
   top-right. The ASCII pusher + boulder sit along it by progress, climbing up;
   on roll-back the boulder tumbles back to bottom-left and starts again — the
   myth, drawn. Replaces the CSS line entirely (which sloped the wrong way).

5. **The pusher is an animated ASCII figure** heaving a boulder, with frames — a
   push/strain cycle, the climb, a slump at the summit. Personality through pose
   and motion. On-brand for a text-forward site; cheap to make expressive and
   funny.

6. **Hover-help popovers** (a deliberate dwell, ~1.5–3s, tunable) explain every
   confusing mechanic in place — Heave, Momentum, stats, locked tabs. The game
   teaches itself; no manual. Same popover mechanism as the locked-tab unlock
   hints.

7. **Aesthetic: a framed monospace "console"** — the ASCII scene and controls sit
   in a bordered terminal-like panel embedded in the editorial page, tabs/stats
   styled with the site's type and accent. Intentional contrast: a restrained
   editorial page with a little terminal you play in. On-brand for a
   build-your-own-tools site.

## Consequences

- The snapshot must expose **disclosure state**: which tabs are unlocked, the
  single next-locked tab and its unlock hint, and per-mechanic help text — so the
  UI is data-driven and the reveal rules live in one place.
- **Onboarding gate (ADR 0009) finally gets built:** the first auto-push is free
  after 5 manual pushes, and the early curve is gentled so the opening isn't a
  hand-click slog to 1000. The current "click to 1000" is a bug, not the design.
- The front-end is largely rewritten: tabbed, gated, ASCII-rendered, console-
  framed, reading from a richer snapshot.
- `frontend-design` craft is applied during the build so the result is
  intentional, not default-y.
