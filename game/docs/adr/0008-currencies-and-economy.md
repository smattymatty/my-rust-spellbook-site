# ADR 0008 — Two currencies, no producers, three permanent skill trees

**Status:** Accepted (architecture) · scorn-sink **RESOLVED** by
[ADR 0009](./0009-scorn-economy-and-stances.md)
**Date:** 2026-06-30

## Context

The economy is the make-or-break of an idle game. The corrected design has two
currencies on two reset layers and no purchasable producers — a deliberate break
from the cookie-clicker default of "buy more generators."

## Decision

**No producers.** One pusher per universe; you never buy more. Throughput grows
through *upgrades*, not through multiplying producers. The only way to more than
one pusher is the organizing endgame.

**Two currencies, two layers:**
- **Scorn** — in-run. Earned per roll-back, spent on **in-run upgrades**, reset
  at prestige.
- **Defiance** — permanent. Earned at prestige, spent in the **three skill
  trees** (global / per-pusher / per-universe), never reset.

**Skill trees** are prerequisite-gated and leveled; the three axes are the
condition, the character, and the place (ADR 0002).

## OPEN — the scorn sink (research → grill, do not freeze yet)

What scorn actually *buys* is unresolved and is the difference between a live run
and a boring one. The current code ships a **provisional** four-upgrade set
(push power, auto-push, auto-rollback, scorn gain) purely so the loop runs and is
testable. It is explicitly a placeholder.

Known design intents to fold in:
- An onboarding gate: the first auto-push is *free* after N manual pushes (a
  `0/5`-style counter), so the player feels the manual grind before it lifts.
- A cheap per-universe permanent node: "start each run with auto-push already."
- The worry that drove this ADR open: **if scorn only buys auto-push, the run is
  boring.** The sink needs variety — distinct, meaningfully-different things to
  spend scorn on across a run.

The resolution path: a research pass on incremental-game economy design (what
makes the in-run spend satisfying, what players praise and criticize), then a
`/grill-with-docs` session on the scorn economy specifically. This ADR is
amended with the result before the scorn sink is treated as set.

The research pass is done — see [`../research/incremental-economy.md`](../research/incremental-economy.md).
Its headline resolves the worry directly: **don't sell auto-push, sell the
auto-push control panel** (unlock/interval/threshold), and anchor the menu on a
mutually-exclusive **Stance** choice plus a **softcap-breaker**, reserving
new-*mechanic* unlocks for the defiance tree. The candidate 12-flavor menu and
the validated "free first auto-push" onboarding are inputs to the pending grill,
not yet decisions.

## Consequences

- The currency split and tree structure are stable to build against now.
- `content.rs`'s `run_upgrades` list is the one place expected to churn; the rest
  of the engine does not care what the upgrades are, only that they cost scorn
  and change effect multipliers.
