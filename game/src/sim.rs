//! ADR 0004 — fixed-timestep accumulator, plus the rules inside a step and the
//! effect layer that reads the three skill trees.

use std::cmp::Ordering;

use crate::command::{Command, SkillTree};
use crate::content;
use crate::number::Big;
use crate::state::{GameState, Phase, Run, Stance, UniverseKind};

pub const STEP_MS: f64 = 100.0;
pub const OFFLINE_CAP_MS: f64 = 8.0 * 60.0 * 60.0 * 1000.0; // 8 hours (ADR 0005)
pub const MAX_STEPS: u64 = (OFFLINE_CAP_MS / STEP_MS) as u64;

// ── ADR 0011: the day/night clock ─────────────────────────────────────────────
pub const DAY_SECONDS: f64 = 720.0; // a 12-minute in-game day
pub const PHASE_SECONDS: f64 = DAY_SECONDS / 6.0;

/// 0..1 through the day (0 = start of Dawn).
pub fn day_fraction(state: &GameState) -> f64 {
    (state.clock / DAY_SECONDS).rem_euclid(1.0)
}
/// The current phase of the day.
pub fn current_phase(state: &GameState) -> Phase {
    let idx = (state.clock / PHASE_SECONDS).floor() as i64;
    Phase::ALL[idx.rem_euclid(6) as usize]
}
/// Per-phase push bonus from the active phase's defiance track.
fn phase_push_mult(state: &GameState) -> f64 {
    1.0 + 0.25 * state.skills.phase_rank(current_phase(state).key(), "push") as f64
}
/// Per-phase scorn bonus from the active phase's defiance track.
fn phase_scorn_mult(state: &GameState) -> f64 {
    1.0 + 0.25 * state.skills.phase_rank(current_phase(state).key(), "scorn") as f64
}

fn level(run: &Run, key: &str) -> u32 {
    run.upgrade_levels.get(key).copied().unwrap_or(0)
}

/// Ensure auto-push is at least level 1 (the onboarding grant and Patience's
/// pre-automation both call this — same effect, different trigger).
fn grant_auto_push(run: &mut Run) {
    let e = run.upgrade_levels.entry(content::U_AUTO_PUSH.to_string()).or_insert(0);
    *e = (*e).max(1);
}

// ── Effect layer: push, summit, scorn, auto behaviours ────────────────────────

/// The labelled multipliers that compose base push power: Grip (+10%/lvl
/// compounding), the three skill trees, and the active phase. `push_amount` is
/// their product × the universe base — so this list is the single source for both
/// the value and the "your edge" hover breakdown (they can't drift).
pub fn push_sources(state: &GameState) -> Vec<(&'static str, f64)> {
    let pusher = state.run.pusher.key();
    let universe = state.run.universe.key();
    vec![
        ("Grip", 1.1_f64.powi(level(&state.run, content::U_PUSH_POWER) as i32)),
        ("Might", 1.0 + 0.25 * state.skills.global_rank(content::G_MIGHT) as f64),
        ("Calluses", 1.0 + 0.50 * state.skills.pusher_rank(pusher, content::P_CALLUSES) as f64),
        ("Loose Scree", 1.0 + 0.30 * state.skills.universe_rank(universe, content::UNI_LOOSE_SCREE) as f64),
        ("phase", phase_push_mult(state)),
    ]
}

/// The flat base push power, before any multiplier: universe base + Muscle
/// (+1/level). This is the *value* Muscle grows; `push_sources` then multiply it.
pub fn push_base_flat(state: &GameState) -> Big {
    let muscle = level(&state.run, content::U_MUSCLE) as f64;
    content::universe_def(state.run.universe).base_push.add(Big::from_f64(muscle))
}

/// Height added per manual push (base value), all multipliers folded in. Muscle
/// lifts the flat base; Grip/skills/phase multiply it — so both feed Push/auto/Heave.
pub fn push_amount(state: &GameState) -> Big {
    let mut out = push_base_flat(state);
    for (_, m) in push_sources(state) {
        out = out.mul_f64(m);
    }
    out
}

/// Round effort to a whole number — no partial pushes. Rounds to nearest
/// (1.10→1, 1.51→2), never below 1, and is a no-op once numbers are large
/// (an f64 round would just lose precision there anyway).
pub fn whole_effort(e: Big) -> Big {
    let v = e.to_f64();
    if v < 1e15 {
        Big::from_f64(v.round().max(1.0))
    } else {
        e
    }
}

/// Effective summit height — the place tree can lower it.
/// Deterministic roll-back escalation (the operator's tiered curve): each
/// roll-back makes the next summit taller. Rate steps up — +5% (1–5), +6%
/// (6–12), +7% (13–24), +8% (25–48), then +1% per 24 beyond. Returns the
/// cumulative summit multiplier after `n` roll-backs.
pub fn rollback_scale(n: f64) -> f64 {
    let n = n.max(0.0);
    let mut s = 1.05_f64.powf(n.min(5.0));
    if n > 5.0 {
        s *= 1.06_f64.powf((n - 5.0).min(7.0));
    }
    if n > 12.0 {
        s *= 1.07_f64.powf((n - 12.0).min(12.0));
    }
    if n > 24.0 {
        s *= 1.08_f64.powf((n - 24.0).min(24.0));
    }
    if n > 48.0 {
        let (mut rem, mut rate) = (n - 48.0, 0.09_f64);
        while rem > 0.0 {
            s *= (1.0 + rate).powf(rem.min(24.0));
            rem -= 24.0;
            rate += 0.01;
        }
    }
    s
}

pub fn summit(state: &GameState) -> Big {
    let def = content::universe_def(state.run.universe);
    let ranks = state.skills.universe_rank(state.run.universe.key(), content::UNI_LOWER_SUMMIT);
    let factor = (1.0 - 0.08 * ranks as f64).max(0.1);
    // Each roll-back this run raises the bar (the wall).
    let esc = rollback_scale(state.run.rollbacks.to_f64());
    def.summit.mul_f64(factor).mul_f64(esc)
}

/// Rolling's per-shove multiplier on your push: +50% of your push per level, starting at
/// ×1 (level 1). So level 2 = ×1.5, level 3 = ×2 — gentle, and still scales with
/// push power (unlike a dead flat). Level 0 (locked) reads ×1 but size is 0.
pub fn rolling_mult(state: &GameState) -> f64 {
    let lvl = level(&state.run, content::U_AUTO_PUSH);
    1.0 + 0.5 * (lvl.saturating_sub(1) as f64)
}

/// Size of one auto-shove: your push × Rolling (folds in the stance auto bonus).
/// Auto-push is `size / interval` per second; splitting it lets the UI show the
/// two levers (Rolling = size, Haste = interval) instead of one opaque rate.
pub fn auto_push_size(state: &GameState) -> Big {
    let lvl = level(&state.run, content::U_AUTO_PUSH);
    if lvl == 0 {
        return Big::ZERO;
    }
    let mut shove = push_amount(state)
        .mul_f64(rolling_mult(state))
        .mul_f64(patience_auto_bonus(state));
    // Patience innate: with no manual push to receive it, the battery's momentum
    // bonus (momentum × Conduit value) is added flat to each auto-shove instead —
    // so the battery is useful the moment it fills, before any Conduction rank.
    if state.run.stance == Some(Stance::Patience) {
        shove = shove.add(Big::from_f64(state.run.momentum * momentum_value(state)));
    }
    shove.mul_f64(push_multiplier(state)) // Leverage boosts auto too (final)
}

/// Seconds between auto-shoves: Haste shortens it (+30%/level faster), base 1s.
pub fn auto_push_interval(state: &GameState) -> f64 {
    1.0 / (1.0 + 0.3 * level(&state.run, content::U_AUTO_HASTE) as f64)
}

/// Passive height/sec from auto-push (0 if not yet unlocked) = size / interval.
pub fn auto_push_rate(state: &GameState) -> Big {
    let ivl = auto_push_interval(state);
    if ivl <= 0.0 {
        return Big::ZERO;
    }
    auto_push_size(state).mul_f64(1.0 / ivl)
}

/// Heave's momentum multiplier: base ×2, +1/level from Force, plus Weight's
/// rank-capped roll-back scaling. The single source for both the effect and the
/// "heave force" stat — so they can't drift.
pub fn heave_mult(state: &GameState) -> f64 {
    let weight_lvl = level(&state.run, content::U_WEIGHT) as f64;
    let counted = state.run.rollbacks.to_f64().min(10.0 * weight_lvl);
    2.0 + level(&state.run, content::U_HEAVE_MULT) as f64 + 0.01 * weight_lvl * counted
}



/// Heave recharge as 0..1 (1 = ready). Drives the button-as-cooldown-bar UI.
pub fn heave_progress(state: &GameState) -> f64 {
    if state.run.heave_cd <= 0.0 {
        1.0
    } else {
        (1.0 - state.run.heave_cd / effective_heave_cooldown(state)).clamp(0.0, 1.0)
    }
}

/// The labelled multipliers on scorn: the three skill trees (Endurance / Defiant
/// Hands / Thin Air), the active phase, and Zeal. `scorn_base` is `(base + Spite)`
/// × their product — the single source for the value and the "your edge" hover.
pub fn scorn_sources(state: &GameState) -> Vec<(&'static str, f64)> {
    let pusher = state.run.pusher.key();
    let universe = state.run.universe.key();
    vec![
        ("Endurance", 1.0 + 0.30 * state.skills.global_rank(content::G_ENDURANCE) as f64),
        ("Defiant Hands", 1.0 + 0.40 * state.skills.pusher_rank(pusher, content::P_DEFIANT_HANDS) as f64),
        ("Thin Air", 1.0 + 0.50 * state.skills.universe_rank(universe, content::UNI_THIN_AIR) as f64),
        ("phase", phase_scorn_mult(state)),
        ("Zeal", zeal_mult(state)),
    ]
}

/// Scorn granted by one roll-back at the summit (before any overshoot bonus).
/// Spite is a flat per-roll-back bonus on the base; the rest multiply.
pub fn scorn_base(state: &GameState) -> Big {
    let def = content::universe_def(state.run.universe);
    let spite_flat = level(&state.run, content::U_SCORN_GAIN) as f64; // Spite: +1 flat/lvl
    let mut out = def.base_scorn.add(Big::from_f64(spite_flat));
    for (_, m) in scorn_sources(state) {
        out = out.mul_f64(m);
    }
    out
}

/// Manual roll-back rewards letting the boulder climb past the summit.
fn overshoot_factor(height: &Big, summit: &Big) -> f64 {
    if summit.is_zero() || !height.gte(summit) {
        1.0
    } else {
        height.div(*summit).to_f64().clamp(1.0, 1000.0)
    }
}

/// Defiance banked by prestiging now. Scales with roll-backs done this run;
/// below the threshold it is zero and prestige is refused.
pub fn defiance_gain(state: &GameState) -> Big {
    let r = state.run.rollbacks.to_f64();
    let g = (r / 10.0).sqrt().floor();
    if g < 1.0 {
        Big::ZERO
    } else {
        Big::from_f64(g)
    }
}

// ── ADR 0009: variable push, momentum, stances ───────────────────────────────

const MOMENTUM_HALFLIFE_S: f64 = 2.5;
const HEAVE_COOLDOWN_MS: f64 = 3000.0;
/// Grind holds momentum if a manual action happened within this window (ms).
const GRIND_ACTIVE_MS: f64 = 2000.0;
/// The overshoot countdown: seconds at/above the summit before the auto-roll-back
/// fires. A fixed innate constant — no upgrade may change it (only stance keystones).
const ROLLBACK_COUNTDOWN_S: f64 = 5.0;
const HEAVE_MULT: f64 = 8.0;
/// Onboarding gate (ADR 0009): the first auto-push is free after this many
/// manual pushes in a run.
pub const FREE_AUTO_PUSH_AT: u32 = 5;

/// xorshift64 — advance the run PRNG, return a value in [0, 1).
fn next_unit(rng: &mut u64) -> f64 {
    let mut x = if *rng == 0 { 0x9E3779B97F4A7C15 } else { *rng };
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *rng = x;
    ((x >> 11) as f64) / ((1u64 << 53) as f64)
}

fn branch_level(state: &GameState, key: &str) -> u32 {
    state.run.branch.get(key).copied().unwrap_or(0)
}

// The three stance load-bearing passives, as rates — the single source for both
// the effect (below) and the purple stance stats in "your edge" (so they can't drift).
/// Grind's Iron Hold: +30%/rank to Push & Heave power while momentum is at max.
pub fn grind_max_bonus(state: &GameState) -> f64 { 0.30 * branch_level(state, "grind_consistency") as f64 }
/// Lurch's Recklessness: +20%/rank per point of MISSING momentum (on top of innate).
pub fn lurch_missing_rate(state: &GameState) -> f64 { 0.20 * branch_level(state, "lurch_recklessness") as f64 }
/// Lurch's INNATE Heave multiplier per missing point (before Recklessness). This is
/// the identity's real power — an empty Coil slams huge with zero upgrades.
pub const LURCH_INNATE_RATE: f64 = 1.0;
/// Lurch's current power multiplier from the Coil, applied to BOTH Push and Heave:
/// `1 + (innate + Recklessness) × missing`. Single source for the effect and the
/// stance card (no drift).
pub fn lurch_missing_mult(state: &GameState) -> f64 {
    let missing = (momentum_cap(state) - state.run.momentum).max(0.0);
    1.0 + (LURCH_INNATE_RATE + lurch_missing_rate(state)) * missing
}
/// Patience's Conduction: +5%/rank auto-push per point of battery momentum.
pub fn patience_conduction_rate(state: &GameState) -> f64 { 0.05 * branch_level(state, "patience_drift") as f64 }

/// Multiplier on the mean push from the stance's distribution, reshaped by the
/// stance branch and sub-build (ADR 0009). `r` in [0,1).
/// A tight ± band of variance on every manual push. Stance identity now lives in
/// the momentum passives (Grind at-max / Lurch missing), not the roll shape.
fn push_roll(_state: &GameState, r: f64) -> f64 {
    0.85 + r * 0.30 // 0.85 .. 1.15
}

/// Patience's innate battery-amp rate — the fuller the battery, the more it
/// amplifies its automation, per point.
pub const PATIENCE_AMP_RATE: f64 = 0.06;

/// Patience's battery amp, applied to BOTH auto-push and auto-heave: a fuller
/// battery hits harder. `1 + (innate 6% + Conduction 5%/rank) × battery`. Returns
/// 1.0 off-Patience. The single source for the effect and the stance card.
pub fn patience_amp(state: &GameState) -> f64 {
    if state.run.stance != Some(Stance::Patience) {
        return 1.0;
    }
    1.0 + (PATIENCE_AMP_RATE + patience_conduction_rate(state)) * state.run.momentum
}

/// Patience's auto-push factor: 2× innate, times the battery amp (which folds in
/// the innate +6%/point and Conduction's +5%/rank/point).
fn patience_auto_bonus(state: &GameState) -> f64 {
    if state.run.stance != Some(Stance::Patience) {
        return 1.0;
    }
    2.0 * patience_amp(state)
}

/// The momentum cap = 12 base + Reach. Grind DOUBLES it (innate); others use it
/// straight — Lurch as its Coil range, Patience as its battery size.
fn momentum_cap(state: &GameState) -> f64 {
    let base = 12.0 + level(&state.run, content::U_REACH) as f64;
    if state.run.stance == Some(Stance::Grind) {
        base * 2.0
    } else {
        base
    }
}

/// A manual push/heave resolves momentum by stance identity, and returns
/// `(momentum_for_flat_bonus, stance_multiplier)`:
/// - **Grind** builds toward the (doubled) cap and pays +30%/rank *at max*.
/// - **Lurch** — only the **Heave** re-coils to full. Its momentum bonus scales
///   with how much was *missing* (empty Coil = huge slam), innately; Recklessness
///   then multiplies that by +20%/rank per missing point. A push is cheap filler:
///   it neither re-coils nor slams, so the Coil is only ever spent by the big hit.
/// - **vanilla** just builds; no multiplier.
/// `build` is the momentum gained (push = 1, heave = 5).
fn resolve_manual_momentum(state: &mut GameState, build: f64, is_heave: bool) -> (f64, f64) {
    let cap = momentum_cap(state);
    match state.run.stance {
        Some(Stance::Lurch) => {
            if is_heave {
                // Heave: the big missing slam + the missing MULTIPLIER, then re-coil to
                // full — only the Heave refills momentum (the "charge").
                let missing = (cap - state.run.momentum).max(0.0);
                let mult = lurch_missing_mult(state); // pre-recoil, uses current missing
                state.run.momentum = cap;
                (missing, mult)
            } else {
                // Push: SPENDS the charged momentum via the flat bonus (momentum × value) —
                // big right after a heave, fading as it drains. No missing multiplier and
                // no re-coil (charge-spend loop). Grind is the manual-push king.
                (state.run.momentum, 1.0)
            }
        }
        Some(Stance::Grind) => {
            state.run.momentum = (state.run.momentum + build).min(cap);
            let at_max = state.run.momentum >= cap - 1e-6;
            (state.run.momentum, if at_max { 1.0 + grind_max_bonus(state) } else { 1.0 })
        }
        _ => {
            state.run.momentum = (state.run.momentum + build).min(cap);
            (state.run.momentum, 1.0)
        }
    }
}

/// The current Momentum cap, for the UI bar.
pub fn momentum_cap_now(state: &GameState) -> f64 {
    momentum_cap(state)
}

/// Anchor: momentum never drops below this (bounded by the cap).
pub fn momentum_value(state: &GameState) -> f64 {
    1.0 + 0.2 * level(&state.run, content::U_CONDUIT) as f64
}

/// The Heave cooldown — a fixed base (no generic upgrade reduces it).
pub fn effective_heave_cooldown(_state: &GameState) -> f64 {
    HEAVE_COOLDOWN_MS
}

/// Cascade: seconds between auto-push bursts (0 = not owned).
pub fn cascade_interval(state: &GameState) -> f64 {
    let c = level(&state.run, content::U_CASCADE);
    if c == 0 { 0.0 } else { 6.0 / c as f64 }
}

/// Surplus: multiplier on the overshoot BONUS (+50%/level on the part above 1).
pub fn surplus_mult(state: &GameState) -> f64 {
    1.0 + 0.5 * level(&state.run, content::U_SURPLUS) as f64
}

/// The scorn multiplier a roll-back would bank right now: `height / summit`
/// (+25% height over the summit → +25% scorn), with Surplus amplifying the
/// overshoot part. 1.0 at/below the summit.
pub fn overshoot_mult(state: &GameState) -> f64 {
    let summit = summit(state);
    let over = overshoot_factor(&state.run.climb.height, &summit);
    1.0 + (over - 1.0) * surplus_mult(state)
}

/// Bank one roll-back: scorn scaled by the overshoot multiplier, then reset the
/// climb and the countdown. Shared by the auto-countdown and manual "bank now".
fn do_rollback(state: &mut GameState) {
    let summit = summit(state);
    if summit.is_zero() || state.run.climb.height.compare(&summit) == Ordering::Less {
        return;
    }
    // Scorn is always a whole number per roll-back (no fractional banking) — rounds
    // the base × overshoot, so the counter and the best-haul can never disagree.
    let gain = whole_effort(scorn_base(state).mul_f64(overshoot_mult(state)));
    record_rollback(state, gain);
    state.run.scorn = state.run.scorn.add(gain);
    state.run.rollbacks = state.run.rollbacks.add(Big::ONE);
    state.run.climb = crate::state::Climb::fresh();
    state.run.rollback_countdown = -1.0;
    state.ever_rolled_back = true;
    claim_milestones(state);
}

/// Brawn: multiplier on Heave's base shove (+20%/level).
pub fn heave_base_mult(state: &GameState) -> f64 {
    1.0 + 0.2 * level(&state.run, content::U_BRAWN) as f64
}

/// Leverage: a global multiplier on the FINAL effort of every push, Heave, and
/// auto-shove (momentum included) — +25%/level, compounding. The premium lever
/// that reaches what Grip can't (the flat momentum add).
pub fn push_multiplier(state: &GameState) -> f64 {
    1.25_f64.powi(level(&state.run, content::U_LEVERAGE) as i32)
}

// ── The Manual family: manual-PUSH-only levers (Grind's lane) ──────────────────
/// Streak: continuous holding grows +10%/sec, capped at +10% per Streak level.
const STREAK_RATE: f64 = 0.10;
/// A gap longer than this (ms) between manual pushes breaks the streak.
const STREAK_GAP_MS: f64 = 500.0;

/// Power: a multiplier applied to MANUAL pushes only (not auto, not heave).
pub fn manual_push_mult(state: &GameState) -> f64 {
    1.0 + 0.25 * level(&state.run, content::U_POWER) as f64
}
/// Streak's current bonus: +10%/sec held, capped at +10% × Streak level.
pub fn streak_bonus(state: &GameState) -> f64 {
    let cap = level(&state.run, content::U_STREAK) as f64;
    STREAK_RATE * state.run.push_streak_secs.min(cap)
}
/// The max hold-to-push rate (per second): 5 base + 1/level from Speed. Read by
/// the client to cap its input ramp.
pub fn push_rate_max(state: &GameState) -> f64 {
    5.0 + level(&state.run, content::U_SPEED) as f64
}

/// Thrift discounts all scorn-upgrade costs — divide form, never free.
pub fn thrift_mult(state: &GameState) -> f64 {
    1.0 / (1.0 + 0.05 * level(&state.run, content::U_THRIFT) as f64)
}

/// Zeal: scorn gain grows with scorn spent this run (asymptotic, rank-bounded).
pub fn zeal_mult(state: &GameState) -> f64 {
    let z = level(&state.run, content::U_ZEAL) as f64;
    if z == 0.0 {
        return 1.0;
    }
    let spent = state.run.scorn_spent.to_f64();
    let frac = spent / (spent + 1000.0); // → 1 as spending grows (1000 = tuning knob)
    1.0 + 0.05 * z * frac
}

/// Whether the active stance can push by hand at all (Patience cannot).
fn manual_enabled(state: &GameState) -> bool {
    state.run.stance != Some(Stance::Patience)
}

/// Effort must clear this floor before it can register as a personal best (and
/// fire the CRIT). Below it the opening pushes read as faint warm-up numbers —
/// so the first real best is earned on the way up, not handed out on click one.
const BEST_FLOOR: f64 = 10.0;

/// Record a manual action for the floating-number feedback: its value, how close
/// to your personal best (0..1.5), whether it set a new best, and its kind
/// (1 push, 2 heave). Push and heave keep separate bests so each feels big.
fn stance_label(s: Option<Stance>) -> String {
    match s {
        Some(Stance::Grind) => "Grind",
        Some(Stance::Lurch) => "Lurch",
        Some(Stance::Patience) => "Patience",
        None => "no stance",
    }
    .to_string()
}

/// Record a roll-back's scorn gain for the crit-float: per-run best drives the
/// "NEW BEST" spectacle (so every run has records to beat), all-time drives the
/// hover. Mirrors `record_action` for push/heave.
fn record_rollback(state: &mut GameState, gain: Big) {
    let run_old = state.run.best_scorn;
    let is_best =
        gain.to_f64() >= BEST_FLOOR && !run_old.gte(&gain) && gain.format() != run_old.format();
    let frac = if run_old.is_zero() {
        (gain.to_f64() / BEST_FLOOR).min(1.5)
    } else {
        gain.div(run_old).to_f64().min(1.5)
    };
    if !run_old.gte(&gain) {
        state.run.best_scorn = gain;
    }
    if !state.best_scorn.gte(&gain) {
        state.best_scorn = gain;
        state.best_scorn_stance = stance_label(state.run.stance);
        state.best_scorn_prestige = state.total_prestiges;
    }
    state.last_scorn = gain;
    state.last_scorn_frac = frac;
    state.last_scorn_best = is_best;
}

/// Update the per-run and all-time best for a push (kind 1) or heave (kind 2) —
/// no float/CRIT side effects. Used by both manual actions (via `record_action`)
/// and Patience's AUTO push/heave, so its bests populate from automation.
fn note_best(state: &mut GameState, effort: Big, kind: u8) {
    let run_old = if kind == 2 { state.run.best_heave } else { state.run.best_push };
    if !run_old.gte(&effort) {
        if kind == 2 { state.run.best_heave = effort; } else { state.run.best_push = effort; }
    }
    let all_old = if kind == 2 { state.best_heave } else { state.best_push };
    if !all_old.gte(&effort) {
        let stance = stance_label(state.run.stance);
        let at = state.total_prestiges;
        if kind == 2 {
            state.best_heave = effort;
            state.best_heave_stance = stance;
            state.best_heave_prestige = at;
        } else {
            state.best_push = effort;
            state.best_push_stance = stance;
            state.best_push_prestige = at;
        }
    }
}

fn record_action(state: &mut GameState, effort: Big, kind: u8) {
    // The CRIT "NEW BEST" + the float brightness track the PER-RUN best — so every
    // run has achievable records to chase (the all-time best from a past strong run
    // would otherwise be unbeatable and never fire in a fresh run).
    let run_old = if kind == 2 { state.run.best_heave } else { state.run.best_push };
    // Celebrate only when the number the PLAYER SEES ticks up. Near the momentum
    // cap the raw effort jitters by fractions (11.02 → 11.05 → 11.03), all
    // rendering "11"; comparing the FORMATTED value kills that spam — only a
    // genuine "12" re-fires. Must also clear the floor.
    let is_best =
        effort.to_f64() >= BEST_FLOOR && !run_old.gte(&effort) && effort.format() != run_old.format();
    let frac = if run_old.is_zero() {
        (effort.to_f64() / BEST_FLOOR).min(1.5)
    } else {
        effort.div(run_old).to_f64().min(1.5)
    };
    note_best(state, effort, kind);
    state.last_effort = effort;
    state.last_frac = frac;
    state.last_best = is_best;
    state.last_kind = kind;
}

/// Roll-back counts this run that grant a milestone boost.
const MILESTONES: [f64; 3] = [10.0, 50.0, 200.0];

/// Run-scoped production multiplier from claimed summit-milestones.
pub fn milestone_bonus(state: &GameState) -> Big {
    Big::from_f64(1.0 + 0.25 * state.run.milestones as f64)
}

/// Production multiplier across a climb. Two gentle shoulders, flat and fast in
/// between: the opening ~20% eases in from a slow start (so the first pushes
/// have weight instead of instantly blurring past), then full speed through the
/// middle, then the summit softcap drags the last ~20% down to a floor.
fn fatigue_factor(state: &GameState, height: &Big) -> f64 {
    let summit = summit(state);
    if summit.is_zero() {
        return 1.0;
    }
    let frac = height.div(summit).to_f64();

    // Ease-in: the first ~20% ramps from a slow floor up to full speed via a
    // smoothstep, so every climb opens with a little weight (fixes the "first
    // clicks rocket up" feel) before settling into the fast middle.
    let warm = 0.20;
    if frac < warm {
        let warm_floor = 0.55;
        let t = (frac / warm).clamp(0.0, 1.0);
        let eased = t * t * (3.0 - 2.0 * t);
        return warm_floor + eased * (1.0 - warm_floor);
    }

    // Summit softcap: only the last ~20% drags, and only down to 60%.
    let threshold = 0.78;
    if frac <= threshold {
        return 1.0;
    }
    let floor = 0.6;
    let over = ((frac - threshold) / (1.0 - threshold)).clamp(0.0, 1.0);
    (1.0 - over * (1.0 - floor)).max(floor)
}

/// Claim any milestones the run's roll-back count has reached.
fn claim_milestones(state: &mut GameState) {
    let r = state.run.rollbacks.to_f64();
    while (state.run.milestones as usize) < MILESTONES.len()
        && r >= MILESTONES[state.run.milestones as usize]
    {
        state.run.milestones += 1;
    }
}

// ── Tick ──────────────────────────────────────────────────────────────────────

fn step(state: &mut GameState, dt_ms: f64) {
    let dt_s = dt_ms / 1000.0;
    state.run.since_action += dt_ms;
    // Let go for longer than the gap → the Streak resets (kept in sync for the UI).
    if state.run.since_action > STREAK_GAP_MS {
        state.run.push_streak_secs = 0.0;
    }

    // Momentum, by stance identity:
    // - Patience: never decays (fills +1/cycle in `advance()`, clock-driven).
    // - Grind: no decay while active (recent manual action); decays once idle.
    // - Lurch / vanilla: exponential decay toward zero (Lurch's Coil drain).
    match state.run.stance {
        // Patience: the battery never decays, and it fills in `advance()` driven
        // by the day clock (so it ticks exactly when the visible phase countdown
        // hits 0). Nothing to do per-step.
        Some(Stance::Patience) => {}
        Some(Stance::Grind) if state.run.since_action < GRIND_ACTIVE_MS => { /* held */ }
        _ => {
            if state.run.momentum > 0.0 {
                state.run.momentum *= 0.5f64.powf(dt_s / MOMENTUM_HALFLIFE_S);
                if state.run.momentum < 1e-3 {
                    state.run.momentum = 0.0;
                }
            }
        }
    }
    if state.run.heave_cd > 0.0 {
        state.run.heave_cd = (state.run.heave_cd - dt_ms).max(0.0);
    }

    let rate = auto_push_rate(state);
    if !rate.is_zero() {
        let height = state.run.climb.height;
        let fm = fatigue_factor(state, &height);
        let gain = rate.mul(milestone_bonus(state)).mul_f64(dt_s * fm);
        state.run.climb.height = state.run.climb.height.add(gain);
        // Patience has no manual push, so its auto-shove is what "best push" tracks.
        if !manual_enabled(state) {
            note_best(state, auto_push_size(state), 1);
        }
    }

    // Patience is the sole auto-heaver: on the heave cooldown, it slams using the
    // battery momentum (without spending it), powering the heave tree for idle play.
    if state.run.stance == Some(Stance::Patience) && state.run.heave_cd <= 0.0 {
        let fm = fatigue_factor(state, &state.run.climb.height);
        let base = push_amount(state).mul(milestone_bonus(state)).mul_f64(HEAVE_MULT * fm * heave_base_mult(state));
        let bonus = Big::from_f64(state.run.momentum * heave_mult(state) * momentum_value(state));
        // Battery amp lifts the auto-heave too (same rule as auto-push).
        let slam = whole_effort(base.add(bonus).mul_f64(push_multiplier(state) * patience_amp(state)));
        state.run.climb.height = state.run.climb.height.add(slam);
        note_best(state, slam, 2); // the auto-heave slam counts toward best heave
        state.run.cascade_seq += 1; // reuse the burst-float channel for the visual
        state.run.cascade_effort = slam;
        state.run.heave_cd = effective_heave_cooldown(state);
    }

    // Cascade: auto-push periodically bursts with a big extra shove (a float
    // signal too). Deterministic timer — no RNG, so offline catch-up is stable.
    let interval = cascade_interval(state);
    if interval > 0.0 && !rate.is_zero() {
        state.run.cascade_timer += dt_s;
        if state.run.cascade_timer >= interval {
            state.run.cascade_timer -= interval;
            let fm = fatigue_factor(state, &state.run.climb.height);
            let burst = rate.mul(milestone_bonus(state)).mul_f64(3.0 * fm); // ~3s of auto
            state.run.climb.height = state.run.climb.height.add(burst);
            state.run.cascade_seq += 1;
            state.run.cascade_effort = burst;
        }
    }

    // Overshoot countdown (innate auto-roll-back): once at/above the summit a 5s
    // timer runs while you keep climbing (overshoot); when it hits zero the boulder
    // auto-rolls back, banking scorn scaled by how far past the summit you got.
    // Manual "bank now" (Command::Rollback) can end it early.
    let s = summit(state);
    if !s.is_zero() && state.run.climb.height.gte(&s) {
        if state.run.rollback_countdown < 0.0 {
            state.run.rollback_countdown = ROLLBACK_COUNTDOWN_S; // just crested the summit
        } else {
            state.run.rollback_countdown -= dt_s;
            if state.run.rollback_countdown <= 0.0 {
                do_rollback(state);
            }
        }
    } else if state.run.rollback_countdown >= 0.0 {
        state.run.rollback_countdown = -1.0; // fell back below the summit — reset
    }
}

/// The single time entry point — live frames and offline catch-up alike.
pub fn advance(state: &mut GameState, acc: &mut f64, real_dt_ms: f64) -> u64 {
    // The day clock turns by true elapsed time (it just wraps, so no cap needed).
    let phase_pos = state.clock.rem_euclid(PHASE_SECONDS); // seconds into the current phase, BEFORE the turn
    state.clock = (state.clock + real_dt_ms / 1000.0).rem_euclid(DAY_SECONDS);
    let dt = real_dt_ms.clamp(0.0, OFFLINE_CAP_MS);
    // Patience battery: +1 for each phase boundary the day clock crosses — so it
    // ticks in lock-step with the visible phase countdown (fixes the drift), and
    // offline fills the right number of cycles (capped by the momentum cap).
    if state.run.stance == Some(Stance::Patience) {
        let crossings = ((phase_pos + dt / 1000.0) / PHASE_SECONDS).floor();
        if crossings > 0.0 {
            let cap = momentum_cap(state);
            state.run.momentum = (state.run.momentum + crossings).min(cap);
        }
    }
    *acc += dt;
    let mut steps = 0u64;
    while *acc >= STEP_MS && steps < MAX_STEPS {
        step(state, STEP_MS);
        *acc -= STEP_MS;
        steps += 1;
    }
    if !state.seen_prestige && defiance_gain(state).gte(&Big::ONE) {
        state.seen_prestige = true; // first time prestige becomes possible
    }
    steps
}

// ── Commands ──────────────────────────────────────────────────────────────────

fn current_rank(state: &GameState, tree: SkillTree, key: &str) -> u32 {
    match tree {
        SkillTree::Global => state.skills.global_rank(key),
        SkillTree::Pusher => state.skills.pusher_rank(state.run.pusher.key(), key),
        SkillTree::Universe => state.skills.universe_rank(state.run.universe.key(), key),
    }
}

pub fn apply(state: &mut GameState, cmd: &Command) {
    match cmd {
        Command::Push => {
            if !manual_enabled(state) {
                return; // Patience: no manual push
            }
            // Streak: continuous holding accumulates seconds; a gap longer than
            // STREAK_GAP_MS breaks it (so it rewards sustained rapid pushing).
            let gap = state.run.since_action;
            state.run.push_streak_secs = if gap <= STREAK_GAP_MS {
                (state.run.push_streak_secs + gap / 1000.0).min(20.0)
            } else {
                0.0
            };
            state.run.since_action = 0.0; // Grind: this counts as "active"
            let r = next_unit(&mut state.run.rng);
            let roll = push_roll(state, r);
            let fm = fatigue_factor(state, &state.run.climb.height);
            // Stance resolves momentum + its passive multiplier (Grind at-max / Lurch missing).
            let (mom, stance_mult) = resolve_manual_momentum(state, 1.0, false);
            let base = push_amount(state).mul(milestone_bonus(state)).mul_f64(roll * fm);
            // Conduit makes each momentum point worth more; the stance passive and
            // Leverage multiply the whole final effort; the Manual family (Power ×,
            // Streak +%) multiplies MANUAL pushes only; then round.
            let flat = mom * momentum_value(state);
            let manual = manual_push_mult(state) * (1.0 + streak_bonus(state));
            let effort = whole_effort(
                base.add(Big::from_f64(flat)).mul_f64(stance_mult * push_multiplier(state) * manual),
            );
            state.run.climb.height = state.run.climb.height.add(effort);
            record_action(state, effort, 1);
            state.total_pushes = state.total_pushes.add(Big::ONE);
            state.run.pushes_this_run += 1;
            // Onboarding: free the first auto-push after 5 manual pushes.
            if state.run.pushes_this_run >= FREE_AUTO_PUSH_AT {
                grant_auto_push(&mut state.run);
            }
        }

        Command::Heave => {
            if !manual_enabled(state) || state.run.heave_cd > 0.0 {
                return;
            }
            state.run.since_action = 0.0;
            let fm = fatigue_factor(state, &state.run.climb.height);
            // Stance resolves momentum + its passive multiplier, same as a push (build 5).
            let (mom, stance_mult) = resolve_manual_momentum(state, 5.0, true);
            // Brawn raises the base shove; Force/Weight raise the momentum mult;
            // Conduit + stance passive + Leverage multiply the whole total; round.
            let base = push_amount(state).mul(milestone_bonus(state)).mul_f64(HEAVE_MULT * fm * heave_base_mult(state));
            let bonus = Big::from_f64(mom * heave_mult(state) * momentum_value(state));
            let total = whole_effort(base.add(bonus).mul_f64(stance_mult * push_multiplier(state)));
            state.run.climb.height = state.run.climb.height.add(total);
            record_action(state, total, 2);
            state.total_pushes = state.total_pushes.add(Big::ONE);
            state.run.heave_cd = effective_heave_cooldown(state);
        }

        Command::BuyUpgrade { key } => {
            let def = content::universe_def(state.run.universe);
            let ud = match def.run_upgrades.iter().find(|u| u.key == *key) {
                Some(u) => u,
                None => return,
            };
            let lvl = level(&state.run, key);
            if lvl >= ud.max_level {
                return;
            }
            let cost = content::run_upgrade_cost(ud, lvl).mul_f64(thrift_mult(state)); // Thrift discount
            if state.run.scorn.gte(&cost) {
                state.run.scorn = state.run.scorn.sub(cost);
                state.run.scorn_spent = state.run.scorn_spent.add(cost); // Zeal tracks spending
                state.run.upgrade_levels.insert(key.clone(), lvl + 1);
            }
        }

        Command::BuyBranch { key } => {
            let stance = match state.run.stance {
                Some(s) => s,
                None => return, // no branch without a committed Stance
            };
            let nodes = content::stance_branch(stance);
            let node = match nodes.iter().find(|n| n.key == *key) {
                Some(n) => n,
                None => return,
            };
            let lvl = branch_level(state, key);
            if lvl >= node.max_level {
                return;
            }
            // The keystone needs its prerequisite node leveled first.
            if *key == content::keystone_key(stance)
                && branch_level(state, content::keystone_prereq_key(stance))
                    < content::KEYSTONE_PREREQ_LEVEL
            {
                return;
            }
            let cost = content::run_upgrade_cost(node, lvl);
            if state.run.scorn.gte(&cost) {
                state.run.scorn = state.run.scorn.sub(cost);
                *state.run.branch.entry(key.clone()).or_insert(0) += 1;
            }
        }

        Command::ChooseSubBuild { key } => {
            let stance = match state.run.stance {
                Some(s) => s,
                None => return,
            };
            if state.run.sub_build.is_some() {
                return; // locked once chosen
            }
            if branch_level(state, content::keystone_key(stance)) < 1 {
                return; // keystone not yet bought
            }
            if content::find_sub_build(stance, key) {
                state.run.sub_build = Some(key.clone());
            }
        }

        Command::Rollback => {
            // Manual "bank now" — ends the overshoot countdown early (trade a bigger
            // overshoot for a faster next climb). Same overshoot payout as the auto-roll.
            do_rollback(state);
        }

        Command::BuySkill { tree, key } => {
            let nodes = match tree {
                SkillTree::Global => content::global_skills(),
                SkillTree::Pusher => content::pusher_skills(state.run.pusher),
                SkillTree::Universe => content::universe_skills(state.run.universe),
            };
            let node = match content::find_skill(&nodes, key) {
                Some(n) => n,
                None => return,
            };
            let rank = current_rank(state, *tree, key);
            if rank >= node.max_rank {
                return;
            }
            for pk in &node.prereqs {
                if current_rank(state, *tree, pk) == 0 {
                    return; // prerequisite not yet owned
                }
            }
            let cost = content::skill_cost(node, rank);
            if !state.defiance.gte(&cost) {
                return;
            }
            state.defiance = state.defiance.sub(cost);
            match tree {
                SkillTree::Global => state.skills.bump_global(key),
                SkillTree::Pusher => {
                    let p = state.run.pusher.key();
                    state.skills.bump_pusher(p, key);
                }
                SkillTree::Universe => {
                    let u = state.run.universe.key();
                    state.skills.bump_universe(u, key);
                }
            }
        }

        Command::BuyPhaseSkill { phase, key } => {
            let nodes = content::phase_skills(*phase);
            let node = match content::find_skill(&nodes, key) {
                Some(n) => n,
                None => return,
            };
            let rank = state.skills.phase_rank(phase.key(), key);
            if rank >= node.max_rank {
                return;
            }
            let cost = content::skill_cost(node, rank);
            if !state.defiance.gte(&cost) {
                return;
            }
            state.defiance = state.defiance.sub(cost);
            state.skills.bump_phase(phase.key(), key);
        }

        Command::Prestige { universe, stance } => {
            let gain = defiance_gain(state);
            if gain.is_zero() {
                return; // not enough progress this run to prestige
            }
            state.defiance = state.defiance.add(gain);
            state.total_prestiges = state.total_prestiges.add(Big::ONE);
            let next = if state.unlocked.contains(universe) {
                *universe
            } else {
                state.run.universe
            };
            state.run = Run::fresh(next, &state.skills);
            state.run.stance = *stance; // commit the chosen stance for the new run
            match state.run.stance {
                // Lurch begins fully Coiled — momentum at max, then it drains.
                Some(Stance::Lurch) => {
                    let cap = momentum_cap(state);
                    state.run.momentum = cap;
                }
                // Patience can't push by hand, so it starts already automated.
                Some(Stance::Patience) => grant_auto_push(&mut state.run),
                _ => {}
            }
        }
    }
}

pub const STARTING_UNIVERSE: UniverseKind = UniverseKind::Mountain;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::PusherKind;

    fn give_scorn(s: &mut GameState, n: f64) {
        s.run.scorn = Big::from_f64(n);
    }

    #[test]
    fn push_yields_a_whole_number() {
        let mut s = GameState::new();
        apply(&mut s, &Command::Push);
        // No partial pushes — effort always rounds to a whole number, at least 1.
        let h = s.run.climb.height.to_f64();
        assert_eq!(h, h.round(), "push should land on a whole number: {h}");
        assert!(h >= 1.0, "push should be at least 1: {h}");
        assert_eq!(s.run.momentum, 1.0);
        assert_eq!(s.total_pushes, Big::ONE);
    }

    #[test]
    fn pushes_actually_vary() {
        let mut s = GameState::new();
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..20 {
            s.run.climb.height = Big::ZERO;
            s.run.momentum = 0.0;
            apply(&mut s, &Command::Push);
            seen.insert(format!("{:.6}", s.run.climb.height.to_f64()));
        }
        assert!(seen.len() > 1, "variable push produced identical values");
    }

    #[test]
    fn free_auto_push_after_five_manual_pushes() {
        let mut s = GameState::new();
        assert_eq!(level(&s.run, content::U_AUTO_PUSH), 0);
        for _ in 0..4 {
            apply(&mut s, &Command::Push);
        }
        assert_eq!(level(&s.run, content::U_AUTO_PUSH), 0); // not yet
        apply(&mut s, &Command::Push); // the 5th
        assert_eq!(level(&s.run, content::U_AUTO_PUSH), 1); // granted free
        assert!(!auto_push_rate(&s).is_zero());
    }

    #[test]
    fn patience_disables_manual_push() {
        let mut s = GameState::new();
        s.run.stance = Some(Stance::Patience);
        apply(&mut s, &Command::Push);
        assert!(s.run.climb.height.is_zero());
        assert_eq!(s.total_pushes, Big::ZERO);
    }

    #[test]
    fn momentum_builds_on_push_and_decays() {
        let mut s = GameState::new();
        s.run.stance = Some(Stance::Grind);
        for _ in 0..5 {
            apply(&mut s, &Command::Push);
        }
        let built = s.run.momentum;
        assert!(built >= 5.0);
        // a long idle stretch decays it
        let mut acc = 0.0;
        advance(&mut s, &mut acc, 10_000.0);
        assert!(s.run.momentum < built);
    }

    #[test]
    fn heave_is_strong_then_on_cooldown() {
        let mut s = GameState::new();
        s.run.stance = Some(Stance::Grind);
        let fm0 = fatigue_factor(&s, &Big::ZERO); // opening ease-in scales the shove
        apply(&mut s, &Command::Heave);
        assert!(s.run.climb.height.gte(&push_amount(&s).mul_f64(7.0 * fm0)));
        assert!(s.run.heave_cd > 0.0);
        let before = s.run.climb.height;
        apply(&mut s, &Command::Heave); // on cooldown -> no-op
        assert_eq!(s.run.climb.height, before);
    }

    #[test]
    fn buying_in_run_upgrade_spends_scorn_and_levels() {
        let mut s = GameState::new();
        give_scorn(&mut s, 1000.0);
        let before = s.run.scorn;
        apply(&mut s, &Command::BuyUpgrade { key: content::U_PUSH_POWER.to_string() });
        assert_eq!(level(&s.run, content::U_PUSH_POWER), 1);
        assert!(s.run.scorn < before);
        // Grip is +10%/level, compounding — one level → ×1.1
        assert_eq!(push_amount(&s), Big::from_f64(1.1));
    }

    #[test]
    fn manual_rollback_needs_summit_then_grants_scorn() {
        let mut s = GameState::new();
        apply(&mut s, &Command::Rollback); // below summit, no-op
        assert!(s.run.scorn.is_zero());
        assert!(s.run.rollbacks.is_zero());

        s.run.climb.height = summit(&s); // exactly at summit
        apply(&mut s, &Command::Rollback);
        assert!(s.run.scorn.gte(&Big::ONE));
        assert_eq!(s.run.rollbacks, Big::ONE);
        assert!(s.run.climb.height.is_zero()); // climb reset
    }

    #[test]
    fn auto_push_accrues_height() {
        let mut s = GameState::new();
        s.run.upgrade_levels.insert(content::U_AUTO_PUSH.to_string(), 1); // throughput = push_amount/sec
        let rate = auto_push_rate(&s);
        assert!(!rate.is_zero());
        let fm0 = fatigue_factor(&s, &Big::ZERO); // opening second is eased in
        let mut acc = 0.0;
        advance(&mut s, &mut acc, 1000.0); // 1s
        // ~ push_amount * 1s, eased down by the slow start of the climb
        assert!(s.run.climb.height.gte(&Big::from_f64(0.9 * fm0)));
    }

    #[test]
    fn overshoot_countdown_auto_rolls_back_offline() {
        let mut s = GameState::new();
        s.run.upgrade_levels.insert(content::U_AUTO_PUSH.to_string(), 5);
        let mut acc = 0.0;
        // Innate overshoot countdown — no Let Go needed. A full offline window
        // should still bank roll-backs (climb → 5s overshoot → auto-roll → repeat).
        let steps = advance(&mut s, &mut acc, OFFLINE_CAP_MS * 2.0);
        assert_eq!(steps, MAX_STEPS);
        assert!(s.run.rollbacks.gte(&Big::ONE));
        assert!(s.run.scorn.gte(&Big::ONE));
    }

    #[test]
    fn lurch_heave_pays_on_missing_innately() {
        // With NO Recklessness rank, an empty-Coil heave must still slam far harder
        // than a full-Coil one — the missing bonus is innate, not upgrade-gated.
        let heave_at = |momentum: f64| {
            let mut s = GameState::new();
            s.run.stance = Some(Stance::Lurch);
            s.run.momentum = momentum;
            apply(&mut s, &Command::Heave);
            s.run.climb.height.to_f64()
        };
        let empty = heave_at(0.0); // fully drained → max missing
        let mut full_state = GameState::new();
        full_state.run.stance = Some(Stance::Lurch);
        let full = heave_at(momentum_cap_now(&full_state)); // full Coil → no missing
        assert!(empty > full * 2.0, "empty-Coil heave should slam harder: {empty} vs {full}");
    }

    #[test]
    fn overshoot_pays_more_than_a_bare_summit_rollback() {
        // Rolling back from 2× the summit banks ~2× the scorn (base Surplus = 1:1).
        let at_summit = {
            let mut s = GameState::new();
            let sm = summit(&s);
            s.run.climb.height = sm;
            apply(&mut s, &Command::Rollback);
            s.run.scorn.to_f64()
        };
        let overshot = {
            let mut s = GameState::new();
            let sm = summit(&s);
            s.run.climb.height = sm.mul_f64(2.0);
            apply(&mut s, &Command::Rollback);
            s.run.scorn.to_f64()
        };
        assert!(overshot > at_summit * 1.8, "2× summit should bank ~2× scorn: {overshot} vs {at_summit}");
    }

    #[test]
    fn skill_prereqs_gate_purchase() {
        let mut s = GameState::new();
        s.defiance = Big::from_f64(100.0);
        // Grace requires Might first
        apply(&mut s, &Command::BuySkill {
            tree: SkillTree::Global,
            key: content::G_GRACE.to_string(),
        });
        assert_eq!(s.skills.global_rank(content::G_GRACE), 0); // blocked

        apply(&mut s, &Command::BuySkill {
            tree: SkillTree::Global,
            key: content::G_MIGHT.to_string(),
        });
        assert_eq!(s.skills.global_rank(content::G_MIGHT), 1);
        apply(&mut s, &Command::BuySkill {
            tree: SkillTree::Global,
            key: content::G_GRACE.to_string(),
        });
        assert_eq!(s.skills.global_rank(content::G_GRACE), 1); // now allowed
    }

    #[test]
    fn global_grace_seeds_auto_push_each_run() {
        let mut s = GameState::new();
        s.skills.global.insert(content::G_GRACE.to_string(), 3);
        // a fresh run reads the permanent head-start
        s.run = Run::fresh(UniverseKind::Mountain, &s.skills);
        assert_eq!(level(&s.run, content::U_AUTO_PUSH), 3);
    }

    #[test]
    fn pusher_tree_is_keyed_by_character_not_universe() {
        let mut s = GameState::new();
        s.defiance = Big::from_f64(100.0);
        apply(&mut s, &Command::BuySkill {
            tree: SkillTree::Pusher,
            key: content::P_CALLUSES.to_string(),
        });
        // stored under the pusher's key, not the universe's
        assert_eq!(s.skills.pusher_rank(PusherKind::Boulder.key(), content::P_CALLUSES), 1);
        assert!(push_amount(&s) > Big::ONE); // +50%
    }

    #[test]
    fn summit_escalates_with_rollbacks() {
        let mut s = GameState::new();
        let base = summit(&s); // 0 roll-backs → base
        s.run.rollbacks = Big::from_f64(10.0);
        let ten = summit(&s);
        assert!(ten > base); // taller after roll-backs
        s.run.rollbacks = Big::from_f64(50.0);
        assert!(summit(&s) > ten); // and it keeps climbing — the wall
        // scorn is NOT scaled by the escalation (operator's call)
        assert_eq!(rollback_scale(0.0), 1.0);
        assert!(rollback_scale(5.0) > 1.27 && rollback_scale(5.0) < 1.28);
    }

    #[test]
    fn universe_tree_lowers_summit() {
        let mut s = GameState::new();
        let base = summit(&s);
        s.skills.bump_universe(UniverseKind::Mountain.key(), content::UNI_LOWER_SUMMIT);
        assert!(summit(&s) < base);
    }

    #[test]
    fn prestige_banks_defiance_and_keeps_meta() {
        let mut s = GameState::new();
        s.skills.global.insert(content::G_MIGHT.to_string(), 2);
        s.run.rollbacks = Big::from_f64(40.0); // sqrt(40/10)=2 defiance
        apply(
            &mut s,
            &Command::Prestige {
                universe: UniverseKind::Mountain,
                stance: Some(Stance::Lurch),
            },
        );
        assert_eq!(s.defiance, Big::from_f64(2.0));
        assert_eq!(s.total_prestiges, Big::ONE);
        // meta survives
        assert_eq!(s.skills.global_rank(content::G_MIGHT), 2);
        // run reset, with the chosen stance committed
        assert!(s.run.rollbacks.is_zero());
        assert!(s.run.scorn.is_zero());
        assert_eq!(s.run.stance, Some(Stance::Lurch));
    }

    #[test]
    fn prestige_refused_below_threshold() {
        let mut s = GameState::new();
        s.run.rollbacks = Big::from_f64(3.0); // sqrt(0.3)=0 -> refused
        apply(
            &mut s,
            &Command::Prestige {
                universe: UniverseKind::Mountain,
                stance: None,
            },
        );
        assert!(s.defiance.is_zero());
        assert_eq!(s.run.rollbacks, Big::from_f64(3.0)); // untouched
    }

    #[test]
    fn haste_speeds_auto_push() {
        let mut s = GameState::new();
        s.run.upgrade_levels.insert(content::U_AUTO_PUSH.to_string(), 1);
        let base = auto_push_rate(&s);
        s.run.upgrade_levels.insert(content::U_AUTO_HASTE.to_string(), 2);
        assert!(auto_push_rate(&s) > base);
    }

    #[test]
    fn milestones_claim_on_rollback() {
        let mut s = GameState::new();
        s.run.rollbacks = Big::from_f64(9.0);
        s.run.climb.height = summit(&s);
        apply(&mut s, &Command::Rollback); // -> 10 roll-backs -> milestone 1
        assert_eq!(s.run.milestones, 1);
        assert!(milestone_bonus(&s) > Big::ONE);
    }

    #[test]
    fn keystone_gates_sub_build_choice() {
        let mut s = GameState::new();
        s.run.stance = Some(Stance::Lurch);
        s.run.scorn = Big::new(1.0, 6); // plenty

        // can't choose a sub-build before the keystone
        apply(&mut s, &Command::ChooseSubBuild { key: "big_spikes".into() });
        assert!(s.run.sub_build.is_none());

        // can't buy the keystone before its prereq reaches level 3
        apply(&mut s, &Command::BuyBranch { key: "lurch_keystone".into() });
        assert_eq!(branch_level(&s, "lurch_keystone"), 0);

        for _ in 0..3 {
            apply(&mut s, &Command::BuyBranch { key: "lurch_recklessness".into() });
        }
        assert_eq!(branch_level(&s, "lurch_recklessness"), 3);

        // now the keystone is buyable, and it unlocks the sub-build fork
        apply(&mut s, &Command::BuyBranch { key: "lurch_keystone".into() });
        assert_eq!(branch_level(&s, "lurch_keystone"), 1);
        apply(&mut s, &Command::ChooseSubBuild { key: "big_spikes".into() });
        assert_eq!(s.run.sub_build.as_deref(), Some("big_spikes"));

        // the choice is locked for the run
        apply(&mut s, &Command::ChooseSubBuild { key: "frequent_spikes".into() });
        assert_eq!(s.run.sub_build.as_deref(), Some("big_spikes"));
    }

    #[test]
    fn branch_and_sub_build_reset_at_prestige() {
        let mut s = GameState::new();
        s.run.stance = Some(Stance::Grind);
        s.run.branch.insert("grind_consistency".into(), 5);
        s.run.sub_build = Some("steady_climb".into());
        s.run.rollbacks = Big::from_f64(40.0);
        apply(
            &mut s,
            &Command::Prestige {
                universe: UniverseKind::Mountain,
                stance: Some(Stance::Lurch),
            },
        );
        assert!(s.run.branch.is_empty());
        assert!(s.run.sub_build.is_none());
    }
}
