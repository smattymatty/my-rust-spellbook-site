# ADR 0012 — Effort composition: the base / flat / multiplier / final layering

**Status:** Accepted (sealed model)
**Date:** 2026-07-01

## Context

Every source of push/heave/auto effort stacks in one fixed order. Until now that
order lived only in `sim.rs`, so upgrade *descriptions* and the *"your edge"*
stats kept mislabelling a flat as a multiplier (and vice-versa), and every
rebalance risked drift. The confusion has a real cost: a flat adder at the base
is amplified by every multiplier downstream, so mis-calling one for the other
makes the numbers look "wrong" when they are correct (e.g. Muscle +1 turning a
level-2 Rolling from +3 into +4 — right, but surprising if you thought Muscle was
a multiplier). This ADR seals the vocabulary and the order.

## Decision — the six layers (applied in this order, then rounded)

For a manual push (`sim::apply` Push arm), the canonical form is:

```
effort = round(
    ( base_flat × intrinsic_mult × situational  +  momentum × momentum_value )
      × final_mult
)
```

| # | Layer | Kind | Sources | Shown as |
|---|-------|------|---------|----------|
| 1 | **base (flat)** | additive | universe base + **Muscle** (+1/lvl) | a **value** ("base push 2") |
| 2 | **intrinsic multipliers** | multiply the base | **Grip** (×1.1/lvl) · Might · Calluses · Loose Scree · phase | **×N** ("push power ×1.10") |
| 3 | **situational** | multiply | fatigue · roll spread · milestone | not an upgrade; not shown |
| 4 | **momentum** | additive (after the base is multiplied) | momentum × **Conduit** value | the momentum bar + "momentum value ×N" |
| 5 | **final multiplier** | multiplies the *whole* (base + momentum) | **Leverage** (×1.25/lvl) + the active stance passive | **×N final** ("push multiplier") |
| 6 | **round** | — | `whole_effort` (no partial pushes) | — |

**Heave** (`Heave` arm) is the same skeleton: base is `base_flat × 2 × Brawn ×
situational` (the ×2 is the heave constant), momentum's additive term is
`momentum × heave_mult(Force/Weight) × value`, then ×final, round.

**Auto-push** (`step`) has no stance passive and no RNG roll: `size = base_flat ×
intrinsic × Rolling_lvl × final`, and `gain = size / interval × situational`.
Leverage is folded into `size` — **once**.

## The load-bearing consequence

**A flat (layer 1) is multiplied by everything in layers 2–5.** That is the
*point* of a flat, and the source of most "the math is off!" confusion:

- Muscle +1 (base 1 → 2) doesn't just add 1 to a push — it adds 1 × every
  downstream multiplier, and it lifts anything that scales off your push
  (auto-push, heave). Rolling multiplies *your push*, so a bigger base lifts the
  auto-shove too (push 2 × Rolling ×1.5 = 3).
- Grip (layer 2) multiplies the *base*, so it's weak while the base is ~1 —
  which is why Muscle is the correct early buy and Grip is the trap first-pick.
- Leverage (layer 5) multiplies *after* momentum is added, so it's the only lever
  that scales your momentum bonus too — hence "final, after all other multipliers".

## Display contract (binds `engine.rs` + `main.js`)

- A **flat** is rendered as a value, never `×`. (Regression that prompted this
  ADR: Muscle's flat showed as "base push power ×2.00".)
- An **intrinsic multiplier** is `×N`; a **final multiplier** is `×N final`, on
  its own line — the two never merge.
- Every "your edge" stat and every `effect → effect_next` reads its **real sim
  accessor** (ADR 0003 out-half), so the label's *kind* and the number can't
  drift from the formula above.

**Completeness — every *numeric* upgrade must surface.** A purchased upgrade that
changes a number with no visible "your edge" stat is a regression (it's how
Muscle read as a phantom "×2.00" and Grind's 2× cap showed nothing). Pure
**toggles** (Let Go's auto-rollback) are *modes*, not edges — they're surfaced in
the run UI (the roll-back just happens), not the bonus list, and are deliberately
absent here. Bundled effects share one stat, but the bundle's **hover names each
contributor**. Guarded by the `every_invested_upgrade_surfaces_in_your_edge`
test — add a row here and a case there when you add an upgrade:

| Upgrade(s) | "your edge" stat | Layer / kind |
|---|---|---|
| Muscle | base push | base (flat, a value) |
| Grip (+ skills, phase) | push power | intrinsic × |
| Leverage | push multiplier | final × |
| Power | manual push | manual-only × |
| Speed | push rate | hold-rate cap (client) |
| Streak | push streak | manual-only × (sustained hold) |
| Rolling · Haste | auto-push (hover: push × Rolling × Leverage) | size × / interval |
| *(innate overshoot countdown)* · Surplus | overshoot | reward × (auto-roll is innate; Let Go removed) |
| Cascade | auto-burst | interval |
| Reach (+ Grind ×2) | max momentum | cap |
| Conduit | momentum value | × |
| Force · Weight | heave force (hover: base ×2 · Force · Weight) | heave × |
| Brawn | heave base | heave base × |
| Spite (+ Zeal) | scorn / roll-back | flat + × |
| Zeal | scorn bonus | × |
| Thrift | upgrade costs | cost ÷ |

## Consequences

- `content.rs` descriptions must name the layer ("flat", "compounding
  multiplier", "final multiplier") — a bare "+25%" is ambiguous and banned.
- Adding an upgrade means deciding *which layer* first; the layer dictates both
  the formula slot and the display kind.
- Rebalancing a coefficient can't change an upgrade's layer without updating its
  desc and its "your edge" rendering in lockstep.
