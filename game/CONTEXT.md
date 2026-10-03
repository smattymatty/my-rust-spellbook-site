# One Must Imagine — Context Glossary

The shared language for the idle game. Definitions only — no implementation
details (those live in `game/docs/adr/`). When a term here and the code
disagree, fix the disagreement; don't let it stand.

## The loop, in one breath

You **push** → **height** rises → **summit** → **roll-back** (+**scorn**). Spend
scorn on **in-run upgrades**. Many roll-backs = one **run**. When ready,
**prestige** (+**defiance**) → spend defiance in the three **skill trees** →
choose the next universe. Repeat.

## Terms

**Pusher** — The single protagonist of a universe, and the player. There is
exactly **one** pusher per universe; you never buy or recruit more. The only way
to ever have more than one is the **organizing endgame**. The game never says
"Sisyphus." The pusher is a *character* — the per-pusher skill tree travels with
them.

**Push** — The active act of labor; raises height. Yields a **variable** amount
of effort, not a fixed one — the spread is shaped by the active **Stance**.
Manual at first; an in-run upgrade automates it (**auto-push**).

**Stance** — The per-run playstyle the pusher commits to (free at run start,
locked until prestige). It reshapes the *distribution* of push effort and opens
its own branch of scorn upgrades on top of a shared pool. The buildcrafting
choice — the Realm Grinder / Idle Wizard "pick a playstyle each run" that drives
replayability. Three for v1; more unlock via defiance.
- **Grind** — tight curve, steady reliable effort; high stable **momentum** cap.
  The heads-down active stance.
- **Lurch** — wide curve, big spikes and duds; momentum feeds *volatility*. The
  active gamble.
- **Patience** — manual push and momentum are **disabled entirely**; pure idle.
  Scales with idle time and auto-push (granted at run start). Weak early,
  dominant long-run. The eternal labor that needs no hand on it.

**Momentum** — A stack built by manual pushes, shown as its own bar with a cap
(default 12; Grind higher, Lurch lower, Patience none). In its **default basic
form**: each push adds +1 momentum and grants **flat bonus effort equal to your
current momentum** (the number on the bar *is* the bonus); **Heave** grants
momentum × a multiplier (default 2, raisable with scorn); it decays (~2.5s
half-life), so it's a live rhythm. This default is deliberately simple —
**stances and upgrades reshape it** into something more powerful or trade-off-y
(e.g. Patience, having no manual push, needs its own momentum mechanic
entirely). The reward for active play; auto-push does not build it.

**Effort layers** — The fixed vocabulary for *how* any push/heave/auto number is
built, so a bonus is never mis-described (see ADR 0012 for the order + formula):
- **base (flat)** — a raw additive quantity at the bottom (universe base +
  **Muscle**). Shown as a *value*, never a `×`.
- **multiplier** — scales the base; **compounding** if it stacks on itself
  (**Grip**). Shown as `×N`.
- **final multiplier** — applies *last*, to the whole thing after momentum is
  added (**Leverage**). Shown as `×N final`, separately.
A **flat** is amplified by every multiplier above it — that's its point, and the
reason the distinction is load-bearing: a flat +1 to the base is worth far more
than +1 at the end.

**Height** — Progress up the current climb.

**Summit** — The top of the climb. Reaching it allows a roll-back. The per-
universe tree can lower it.

**Roll-back** — The per-summit reset: height returns to zero and the pusher
begins again, granting **scorn**. Manual at first; an upgrade automates it
(**auto-rollback**). This is the literal myth — the boulder rolling down.

**Scorn** — The **in-run** currency. Earned per roll-back, spent on in-run
upgrades, **reset at prestige**. It does not cross the prestige boundary.

**In-run upgrade** — A scorn-bought improvement, local to the current run and
wiped at prestige. The scorn shop is a **shared pool** (every Stance) plus the
active **Stance's branch**. See ADR 0009 for the full menu.

**Sub-build** — The deeper, mutually-exclusive specialization within a Stance
(the Realm Grinder "race" to the Stance's "alignment"). Earned, not picked up
front: investing scorn into the Stance branch reaches a **keystone** that unlocks
the sub-build fork later in the run.

**Heave** — A strong manual burst-push on a cooldown; the active-engagement verb
that keeps the player the main character. Active Stances only — Patience has no
manual push.

**Second Wind** — A scorn purchase line that dissolves a fatigue softcap near the
summit, turning a production wall into a breakthrough.

**Run** — One playthrough of one universe: many climbs and roll-backs, scorn
compounding into upgrades, ending when you prestige.

**Prestige** — The meta reset. Banks **defiance**, wipes the run (scorn, in-run
upgrades, height), and lets you choose which universe to run next.

**Defiance** — The **permanent** prestige currency. Camus's conscious revolt —
the choosing-to-push-anyway. Spent in the three skill trees.

**Skill trees (three)** — The permanent (defiance-bought) progression, in three
distinct axes:
- **global** — the absurd condition itself; applies to every pusher everywhere.
- **per-pusher** — the *character*; travels with the pusher across universes.
- **per-universe** — the *place*; applies to whoever pushes there.
Nodes are prerequisite-gated and leveled.

**Day cycle / Phase** — A **12-minute in-game day** (no real-time tie) that
advances by elapsed time, including while idle, looping through six 2-minute
**phases**: Dawn, Morning, Noon, Afternoon, Dusk, Midnight. The clock is **purely
ambient** — it turns on its own, you don't control it. It is **neutral until you
invest**: power comes from six new **per-phase tracks** on the defiance side
(alongside global / per-pusher / per-universe), each a line of passive bonuses
that apply only during their phase. A passive supplement, not a build identity.
Camus-rooted: Noon is lucidity, Midnight the confrontation with the absurd.

**Organizing endgame** — The only path to more than one pusher: a pusher from
another universe comes to join you in this one, bringing their per-pusher tree.
The closest thing to a win is *making the system slightly less worse by going
sideways* — not a win, not an end. Architecturally hooked; mechanics are a later
design grill.
