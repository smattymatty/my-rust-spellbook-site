# Seed — roll-back escalation & the side-scrolling mountain

The wall that gives the roll-back loop a point (operator's design).

## The escalation (implemented in sim.rs `rollback_scale`)

Each roll-back this run raises the next summit. Deterministic tiered rate:
- roll-backs 1–5: **+5%** each
- 6–12: **+6%**
- 13–24: **+7%**
- 25–48: **+8%**
- beyond 48: **+1% every 24** (49–72 → 9%, 73–96 → 10%, …)

Summit after n roll-backs = base × Π(1 + rate(i)). Cheap early, brutal late; the
wall lands where the escalation outruns your run's scaling (~49-ish, a tuning
target, not a fixed point). **Scorn scales too but sub-linearly (`^0.6`)** so
roll-backs stay net-positive until cost finally beats reward. Prestige + defiance
+ build changes push the wall further next run. Extra walls can live elsewhere.

## The side-scrolling mountain (NOT yet built — banked visual)

As the summit escalates, the hill should **get physically longer**, to the point
where the view **side-scrolls** to follow the boulder up an ever-longer slope.
You'd literally *see* the wall receding — the summit further away each run. The
camera pans to keep the pusher in frame as the mountain extends past the console
width. A great "the labor is getting harder" visual, and it pairs exactly with
the escalation curve above. Build it when the escalation feel is tuned.
