# Keystone specialization tier — design seed

**Status:** FUTURE / later-era. **Not built.** This is a scaffold + naming/design
TODO, not an implementation. Do not wire half of it — half-built endgame rots.

## The shape

Each stance's branch stays: a **load-bearing passive** you level (Grind **Iron
Hold**, Lurch **Recklessness**, Patience **Conduction**), which **gates** the
keystone tier at passive **rank 3** (`KEYSTONE_PREREQ_LEVEL`).

Then the tier opens as a **fork of TWO keystones** (mutually exclusive — pick
one). Each keystone unlocks a **3-grouped identity**: three unique,
playstyle-*altering* upgrades that cohere into one direction.

```
passive (rank 3) ──▶ Tier-1 fork ──┬─▶ Keystone A ─▶ [ A1 · A2 · A3 ]  (a 3-group)
                                    └─▶ Keystone B ─▶ [ B1 · B2 · B3 ]  (a 3-group)
```

- **6 new upgrades per stance** (2 keystones × 3). You only ever access **3** —
  your chosen keystone's group.
- Across Grind / Lurch / Patience that's **18 to design** (6 × 3 stances).

## The rule this lives under

This is the **build-defining zone**. Per [`generics-not-build-defining`], generic
(scorn) upgrades may only *scale power within* a playstyle. **Keystone-locked
upgrades are the ONE place allowed to warp mechanics** — change timing windows,
add new resources, invert a stance's rhythm. So these 18 should feel genuinely
transformative, not "+10% more."

## Gating / flow (mostly already scaffolded)

1. Level the passive → rank 3 → the two keystones become purchasable.
2. Buy **one** keystone (flat scorn, max 1) — the choice **locks**.
3. That keystone's **3-upgrade group** opens for purchase.
4. `run.sub_build` (or a successor field) records the chosen keystone; the effect
   layer must **read** it — today it's written but **never read** (the current
   sub-builds are inert placeholders).

## What replaces the current scaffold

The present system is: 1 keystone → 2 inert "sub-build" *choices* (Wide Floor /
Big Spikes / Offline / …) that **do nothing** and describe the *old* variable-push
model. That gets replaced by this 2-keystone × 3-upgrade structure. **The current
sub-build names are placeholders — rename + redesign around the CURRENT
mechanics** (momentum passives, charge-spend Lurch, the battery amp, the Manual
family, the overshoot countdown).

## Placeholder slots (to name + design later)

Blank on purpose. Each group's three should form one coherent identity, and the
two keystones per stance should pull in *different* directions.

| Stance | Keystone A → 3-group | Keystone B → 3-group |
|---|---|---|
| **Grind** | `A1 · A2 · A3` (identity: ?) | `B1 · B2 · B3` (identity: ?) |
| **Lurch** | `A1 · A2 · A3` (identity: ?) | `B1 · B2 · B3` (identity: ?) |
| **Patience** | `A1 · A2 · A3` (identity: ?) | `B1 · B2 · B3` (identity: ?) |

Seed directions (illustrative, NOT decided) to spark the later grill:
- **Lurch** — one keystone deepens the *slam* (bigger coil / steeper missing
  curve / a super-heave), the other rewards *frequency* (faster drain / shorter
  heave cooldown / pushes partly re-coil).
- **Grind** — one leans *ceiling* (higher cap, held longer), the other leans
  *rhythm* (Streak/Speed synergies, momentum that never drops while active).
- **Patience** — one leans *offline* (amp while away, bigger battery), the other
  leans *active-idle* (faster cycles while watched, auto-heave that spends+refills).

## Open questions for the design grill (when the era arrives)
- Whether every stance gets both keystones day one, or they roll out over time.
- Do the 3 in a group unlock all at once, or gate each other?
- Cost curve for the group upgrades (they're build-defining, so premium).
- How `run.sub_build` maps to reading three separate upgrade levels.
