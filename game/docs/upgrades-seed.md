# Generic run-upgrade design — seed

The shared, **non-stance** in-run upgrades (bought with scorn, reset at prestige).
Sorted into four families in the UI: **Auto · Momentum · Heave · Economy** (+ Core).

## The rubric: "based" vs "sussy"

- **Based** = scaling is bounded by **rank** — a number *you* buy and the cost gates.
- **Sussy** = scaling keys off an **unbounded runtime quantity** (roll-back count,
  wall-clock time, unspent scorn). These self-amplify and break balance.

Every generic upgrade must be based. If a design wants runtime-flavored scaling, it
must be **rank-capped** (e.g. "+1% per roll-back, but only the first `10 × rank`
roll-backs count") so investment, not game state, sets the ceiling.

## Scaling-shape rule (safe at any rank, so tokens can stack freely)

Because achievements will push ranks past the nominal cap, every formula must be
structurally safe at *any* rank — no eyeballing "stay under 8":

- gets **bigger** (power, rate, gain, mult, cap) → `base × (1 + level·k)` — unbounded but safe.
- gets **smaller / faster / shorter** (cooldown, interval, decay) → `base / (1 + level·k)` — asymptotic to 0, never reaches it.
- **never** `base × (1 − level·k)` unless hard-clamped with `.max(floor)` (hits 0, then goes negative).

Audit (2026): no live zero-wall bugs. Auto-push is a continuous *rate* (not an
interval), so Haste `rate × (1 + k·level)` is already the safe form; momentum
decay is an exponential half-life (Stamina extends the half-life → safe); the
only true interval is the Heave cooldown (no reducer yet — if added, use the
divide form). Haste's `k` set to **0.1** (+10%/rank, ≈+80% at rank 8).

## Cap model (decided): hard wall + rank tokens

- Capped upgrades **hard-wall** at their base cap (nominally rank 8).
- Later, **achievements grant discrete +1-rank tokens** that lift a specific
  upgrade's ceiling to 9, 10, 12+. The achievement/token system is a **future
  build** — for now the upgrades just hard-cap and expose the hook.
- Infinite upgrades (Grip, Rolling, Spite, Brawn) stay infinite.

## Slot status

- **Core:** Grip (∞), **Leverage** (cap — premium global final multiplier on Push/Heave/auto, momentum included; ×25%/lvl compounding).
- **Auto:** Rolling (∞), Haste (cap), Let Go (1), **Cascade** (cap). ~~Instinct~~ cut (auto-fired heave ruined the active playstyle + broke Patience).
  - **Cascade:** auto-push periodically bursts (deterministic timer, `6÷level`s) — extra height + a floating number. EV-equivalent to a rate boost; the burst + float make it feel distinct without re-modelling auto-push as ticks.
- **Momentum:** Reach (cap — +1 max momentum/rank), **Conduit** (cap — +20%/lvl momentum *value*: each point worth more effort). **Finished at two.** ~~Drive~~, ~~Stamina~~, ~~Anchor~~, ~~Surge~~ all cut — see the battery architecture below.
- **Heave:** Brawn (∞), Force (cap), Weight (cap, roll-backs-counted scales with rank), **Tempo** (cap). ~~Ramp~~ cut.
  - **Tempo:** reduces the Heave cooldown via `HEAVE_COOLDOWN_MS / (1 + level·k)` — asymptotic, never zero, safe at any rank. A new axis (heave *frequency*, not power; power is already covered 3 ways). Rewards active play; useless-but-harmless under Patience.
- **Economy:** Spite (∞), Zeal (cap — was Hoard; now rewards scorn *spent* this run), **Surplus** (cap), **Thrift** (cap).
  - **Surplus:** boosts the overshoot bonus (`overshoot_factor`, rolling back from ABOVE the summit), rank-capped mult. Rewards *how* you roll back — and only on **manual** roll-back, so it pulls against Let Go's auto-rollback (a real build fork, not a conflict).
  - **Thrift:** rank% off all scorn upgrade costs via the safe `/(1 + level·k)` form — asymptotic, never free. The scorn *sink* axis (Spite/Zeal/Surplus are all source).

## Fixes banked

- **Weight:** +1% heave mult per roll-back this run, per level; only the first
  `10 × level` roll-backs count (cap grows with rank, not a flat wall).
- **Reach:** was ∞ max momentum → now capped (token-extensible).
- **Hoard → Zeal:** flipped to scale with scorn *spent* this run (rewards the loop).
- **Anchor → Conduit:** min-momentum floor was dead/detrimental in every stance;
  replaced with the momentum-*value* conversion (universally positive).
- **Drive, Stamina, Surge, Instinct, Ramp:** cut.

## Momentum-as-battery architecture (sealed 2026-07)

Momentum is the **battery** that powers each stance's identity. The generic tree only
**sizes and charges** it: **Reach = capacity**, **Conduit = output per charge**. What the
battery *powers* is stance-specific, so interesting momentum design lives in the
**stance shops by design, not by shortfall.**

Why the generic momentum tree caps at two: a universal node must be non-negative in
three *opposite* stances (Grind holds momentum, Lurch drains it, Patience batteries it).
Only **cap** and **value** clear that bar — **rate** effects die (each stance's rate
mechanic differs) and **amount** effects favour high momentum (dead/bad for Lurch). A
"third generic" is always a stance node in a generic costume — it belongs in a stance shop.

## Stance design (identities sealed; mechanics = next build)

Each stance = one **innate rule** (always on, unscalable — the identity) + one
**load-bearing passive** (the one rank-bounded knob its tree levels). Three relationships
to momentum: Grind holds, Lurch drains, Patience batteries.

| Stance | Innate | Load-bearing passive | Tension |
|---|---|---|---|
| **Grind** | 2× max momentum; no decay *while active* (manual only — auto-push must NOT count) | +30%/rank final multi *while at max momentum* | hold the ceiling |
| **Lurch** | starts at max, drains (**Coil**); manual push/heave refills to max; pays on *missing* | +20%/rank multi per point of *missing* momentum | wait between hits |
| **Patience** | manual disabled; 2× auto-push; **battery** = momentum starts 0, **+1/cycle flat, never decays, hard-capped at max**; **sole auto-heaver** | +X%/rank auto-push per point of battery momentum (the *conversion*) | let the battery fill |

Refinements sealed: Patience battery is a fixed **+1/cycle** (the wait is the cost, not a
knob); its scalable knob is the **conversion** (% auto per momentum). Patience is the
**only** stance that auto-heaves (Instinct, correctly homed) — powered by the battery,
so heave upgrades (Force/Brawn/Weight/Tempo) finally work for every stance. Stance-shop
momentum nodes (rate/directional/identity-amplifying) are wide open — that's where the
depth lives.

### Open (balance/impl, not identity)
- Numbers (30/20/10%, drift rates) untuned.
- Lurch **Coil drain rate** sets the whole rhythm — needs a real number vs sim time.
- Grind "active" definition (what resets the no-decay window; auto-push excluded).
- Patience **cycle length** (~2 min = one phase?) — confirm vs the day/phase system.
- The three **stance shops** are the next real design surface. Current code still runs
  the *old* stance mechanics (caps 60/25/0, push_roll distributions) — the rework above
  is not wired yet.
