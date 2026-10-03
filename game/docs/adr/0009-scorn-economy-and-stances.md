# ADR 0009 — The scorn economy: variable push, Stances, and two-tier builds

**Status:** Accepted · **Implemented** 2026-06-30 (46 tests; wasm + UI wired)
**Date:** 2026-06-30
**Resolves the OPEN scorn-sink in** [ADR 0008](./0008-currencies-and-economy.md).
**Grounded in** [`../research/incremental-economy.md`](../research/incremental-economy.md).

## Context

The in-run spend layer is where idle games live or die; "scorn only buys
auto-push" is a dead run. The research's headline fixes: don't sell auto-push,
sell its *control panel*; anchor on a mutually-exclusive build choice; build the
wall-then-breakthrough rhythm in. The operator's north star is Realm Grinder /
Idle Wizard buildcrafting — "pick a playstyle each run, with different strategies
inside it" — fused with the game's theme (the boulder doesn't move the same every
push; the absurd labor has good days and bad).

## Decision

**Variable push, shaped by Stance.** A push yields a *random* amount around a
mean; the active Stance sets the *shape* of that distribution. Variance is a
strategic choice, not noise.

**Momentum.** Manual pushes build a Momentum stack (+1/push, raisable by the
Drive upgrade; cap is stance-dependent) that **decays when you stop** (~2.5s
half-life). In its default form it is **additive and legible**: each push adds
*flat* bonus effort equal to your current momentum — the number on the bar *is*
the bonus — and **Heave adds momentum × a multiplier** (base 2, raisable by the
Force upgrade). (Superseded the original 1.6× multiplier, which was invisible.)
It is the reward for active play: **auto-push does not build it**, Patience does
not use it. The default is deliberately simple; stances and upgrades reshape it,
and Patience needs its own momentum mechanic entirely. Shown as its own bar.

**Stances (3 for v1), chosen at prestige, locked until the next prestige.** The
first run has *no* Stance (default active play, to teach the loop); the **first
prestige unlocks the Stance system** as its reward. Thereafter you pick a Stance
for each run you start.
- **Grind** — tight curve, reliable; high stable Momentum cap.
- **Lurch** — wide curve, spikes and duds; Momentum feeds volatility.
- **Patience** — **manual push and Momentum disabled entirely**; pure idle,
  auto-push granted at run start, weak early and dominant long-run.
More Stances unlock later via the defiance tree.

**Scorn shop = shared pool + the active Stance's branch.**
- *Shared pool* (every Stance): auto-push control panel (separate interval /
  strength sinks), summit & scorn scalers, QoL (auto-rollback, bulk-buy),
  **Heave** (a strong manual burst on cooldown — active stances only),
  **roll-back amplifier** (more scorn per roll-back, lose more height),
  **Second Wind** (dissolves a fatigue softcap near the summit — the
  wall→breakthrough rhythm), **milestones** (summit-count thresholds this run
  grant flat boosts, count-gated not priced).
- *Stance branch* (only the active one shows): the playstyle's unique upgrades.

**Two-tier build (the Realm Grinder move).** Stance is the *alignment*, chosen
early. Investing scorn into the Stance branch reaches a **keystone**, which
unlocks a later, mutually-exclusive **sub-build** ("race") fork — the deeper
specialization, earned by committing to the Stance. E.g. Lurch: Big Spikes vs
Frequent Spikes; Grind: Wide Floor vs Steady Climb; Patience: Offline vs
Active-idle.

**Reserved for the defiance tree** (permanent layer): new Stances, new
sub-builds, and new-*mechanic* unlocks — so the permanent layer reshapes what
scorn can even buy, keeping the frequent reset fresh.

## Consequences

- New run-scope state: chosen Stance, chosen sub-build, Momentum, Heave cooldown,
  milestone progress, and the per-Stance branch levels. All reset at prestige
  (run scope, ADR 0002).
- The sim gains a PRNG for variable manual pushes. Offline catch-up stays
  deterministic because it runs only auto-push (a flat rate), never random manual
  pushes — so the save/offline guarantees (ADR 0005) are untouched.
- Patience makes "no manual interaction" a *legitimate* build, not a bug — the
  UI must handle a stance with the push button disabled.
- Balance numbers (distribution widths, costs, keystone thresholds, softcap
  curve) are provisional and tuned later; the *structure* here is the decision.

## Alternatives considered

- Deterministic push (no variance) — rejected; drops the strategic variance and
  the Stance-as-distribution fusion.
- Stance switchable mid-run — rejected; mutual exclusivity is what makes a run a
  build (the research's core finding).
- Sub-build chosen at run start alongside Stance — rejected; the Realm Grinder
  two-tier reveal (alignment now, race later, earned) is the praised structure.
- Auto-push builds Momentum — rejected; it collapses active vs idle and removes
  the player from the loop.
