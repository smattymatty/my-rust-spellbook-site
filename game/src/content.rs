//! Static definitions: universes, in-run (scorn) upgrades, and the THREE
//! permanent (defiance) skill trees.
//!
//! The three permanent axes are deliberately distinct:
//! - **global** — the absurd condition itself; applies to every pusher, every
//!   universe.
//! - **per-pusher** — the *character*. Travels with the pusher; after organizing,
//!   a pusher carries their tree into another universe.
//! - **per-universe** — the *place*. Applies to whoever is pushing there.
//!
//! Definitions live in code; the save stores only mutable progress.

use crate::number::Big;
use crate::state::{Phase, PusherKind, Stance, UniverseKind};

// ── In-run upgrade keys (bought with scorn, reset every prestige) ─────────────
// Identity is the upgrade's own `key`, not its position — the sim references
// these, `run_upgrades` defines them, and `upgrade_levels` stores them, all by
// the same string. Reordering or removing an upgrade is a pure data edit.
pub const U_MUSCLE: &str = "muscle"; // flat base-push adder (the early-game lever)
pub const U_PUSH_POWER: &str = "grip";
pub const U_AUTO_PUSH: &str = "rolling";
pub const U_SCORN_GAIN: &str = "spite";
pub const U_AUTO_HASTE: &str = "haste";
pub const U_HEAVE_MULT: &str = "force";
// The designed roster (grill 2026): auto · momentum · heave · economy.
pub const U_CASCADE: &str = "cascade";
pub const U_REACH: &str = "reach";
pub const U_CONDUIT: &str = "conduit"; // momentum-value conversion (was Anchor)
pub const U_BRAWN: &str = "brawn";
pub const U_WEIGHT: &str = "weight";
pub const U_ZEAL: &str = "zeal";
pub const U_SURPLUS: &str = "surplus";
// The Manual family (grill 2026) — manual-PUSH only, Grind's dedicated lane.
pub const U_POWER: &str = "power"; // +% manual push effort
pub const U_SPEED: &str = "speed"; // raises the hold-to-push rate cap
pub const U_STREAK: &str = "streak"; // sustained-hold ramping bonus
pub const U_THRIFT: &str = "thrift";
pub const U_LEVERAGE: &str = "leverage"; // premium global multiplier on final push output

// ── Skill keys read by the effect layer ───────────────────────────────────────
// global (the condition)
pub const G_MIGHT: &str = "g_might";
pub const G_GRACE: &str = "g_grace";
pub const G_ENDURANCE: &str = "g_endurance";
pub const G_RESOLVE: &str = "g_resolve";
// per-pusher (the Boulder Pusher — the character)
pub const P_CALLUSES: &str = "p_calluses";
pub const P_CADENCE: &str = "p_cadence";
pub const P_DEFIANT_HANDS: &str = "p_defiant_hands";
// per-universe (the Mountain — the place)
pub const UNI_LOWER_SUMMIT: &str = "uni_lower_summit";
pub const UNI_LOOSE_SCREE: &str = "uni_loose_scree";
pub const UNI_THIN_AIR: &str = "uni_thin_air";

pub struct RunUpgradeDef {
    pub key: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    /// Cost model (operator's): linear `base + inc·level`, with step multipliers
    /// near the cap. `cost = (base + inc·level) × ramp(level)`.
    pub base_cost: Big, // cost of rank 1 (level 0)
    pub cost_inc: Big,  // + this much scorn per rank
    pub max_level: u32,
    /// Cost ×2 once level ≥ `double_at`, ×3 once level ≥ `triple_at`
    /// (`UNLIMITED` = never — for the flat-linear infinite upgrades).
    pub double_at: u32,
    pub triple_at: u32,
}

pub struct SkillNode {
    pub key: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub base_cost: Big, // defiance
    pub cost_growth: f64,
    pub max_rank: u32,
    /// Node keys (same tree) that must be rank >= 1 before this unlocks.
    pub prereqs: Vec<&'static str>,
}

pub struct UniverseDef {
    pub kind: UniverseKind,
    pub name: &'static str,
    pub action_verb: &'static str,
    /// The protagonist that natively belongs to this universe.
    pub native_pusher: PusherKind,
    pub summit: Big,
    pub base_push: Big,
    pub base_scorn: Big,
    pub run_upgrades: Vec<RunUpgradeDef>,
}

const UNLIMITED: u32 = u32::MAX;

pub fn universe_def(kind: UniverseKind) -> UniverseDef {
    match kind {
        UniverseKind::Mountain => UniverseDef {
            kind,
            name: "The Mountain",
            action_verb: "Push",
            native_pusher: PusherKind::Boulder,
            // Provisional balance — real numbers are a later design grill.
            summit: Big::new(1.0, 3),
            base_push: Big::ONE,
            base_scorn: Big::ONE,
            // Cost model (operator): linear `base + inc·level`, ×2 at `double_at`,
            // ×3 at `triple_at`. Capped upgrades ramp 5→7; Anchor ramps 5→9.
            run_upgrades: vec![
                RunUpgradeDef {
                    key: "grip", name: "Grip",
                    desc: "+10% push power per level, compounding (Push, auto-push, and Heave).",
                    base_cost: Big::ONE, cost_inc: Big::ONE,
                    max_level: UNLIMITED, double_at: UNLIMITED, triple_at: UNLIMITED,
                },
                RunUpgradeDef {
                    key: "muscle", name: "Muscle",
                    desc: "+1 flat push power per level — the early-game kickstart (Push, auto, and Heave).",
                    base_cost: Big::ONE, cost_inc: Big::from_f64(9.0),
                    max_level: 8, double_at: 5, triple_at: 7,
                },
                RunUpgradeDef {
                    key: "leverage", name: "Leverage",
                    desc: "×25% per level to your FINAL output, applied AFTER all other multipliers — Push, Heave, and auto (momentum included).",
                    base_cost: Big::from_f64(30.0), cost_inc: Big::from_f64(30.0),
                    max_level: 8, double_at: 5, triple_at: 7,
                },
                // ── Manual family (grill 2026): manual-PUSH only — Grind's lane ──
                RunUpgradeDef {
                    key: "power", name: "Power",
                    desc: "+25% MANUAL push power per level (Push only — not auto, not Heave).",
                    base_cost: Big::from_f64(4.0), cost_inc: Big::from_f64(4.0),
                    max_level: 8, double_at: 5, triple_at: 7,
                },
                RunUpgradeDef {
                    key: "speed", name: "Speed",
                    desc: "Hold-to-push gets +1/s faster per level (5/s base → up to 13/s), so you push more often.",
                    base_cost: Big::from_f64(4.0), cost_inc: Big::from_f64(4.0),
                    max_level: 8, double_at: 5, triple_at: 7,
                },
                RunUpgradeDef {
                    key: "streak", name: "Streak",
                    desc: "Hold without stopping to build +10% push power per second, up to +10% per level. Resets when you let go.",
                    base_cost: Big::from_f64(4.0), cost_inc: Big::from_f64(4.0),
                    max_level: 8, double_at: 5, triple_at: 7,
                },
                RunUpgradeDef {
                    key: "rolling", name: "Rolling",
                    desc: "Each level adds +50% of your push to every auto-shove (level 1 = 1× your push, level 2 = 1.5×, level 3 = 2×).",
                    base_cost: Big::from_f64(1.0), cost_inc: Big::from_f64(1.0),
                    max_level: UNLIMITED, double_at: 4, triple_at: 8,
                },
                RunUpgradeDef {
                    key: "spite", name: "Spite",
                    desc: "+1 base scorn per roll-back, per level.",
                    base_cost: Big::from_f64(5.0), cost_inc: Big::from_f64(5.0),
                    max_level: UNLIMITED, double_at: UNLIMITED, triple_at: UNLIMITED,
                },
                RunUpgradeDef {
                    key: "haste", name: "Haste",
                    desc: "Auto-push fires 30% more often per level (shorter interval between shoves).",
                    base_cost: Big::from_f64(5.0), cost_inc: Big::from_f64(5.0),
                    max_level: 8, double_at: 5, triple_at: 7,
                },
                RunUpgradeDef {
                    key: "force", name: "Force",
                    desc: "+1 to Heave's momentum multiplier, per level (base ×2).",
                    base_cost: Big::from_f64(5.0), cost_inc: Big::from_f64(5.0),
                    max_level: 8, double_at: 5, triple_at: 7,
                },
                // ── grill roster 2026 (numbers to tune) ──
                RunUpgradeDef {
                    key: "cascade", name: "Cascade",
                    desc: "Auto-push bursts (~3s of climb) every 6÷level seconds — more often per level.",
                    base_cost: Big::from_f64(10.0), cost_inc: Big::from_f64(10.0),
                    max_level: 8, double_at: 5, triple_at: 7,
                },
                RunUpgradeDef {
                    key: "reach", name: "Reach",
                    desc: "+1 maximum momentum, per level.",
                    base_cost: Big::from_f64(3.0), cost_inc: Big::from_f64(3.0),
                    max_level: 8, double_at: UNLIMITED, triple_at: UNLIMITED,
                },
                RunUpgradeDef {
                    key: "conduit", name: "Conduit",
                    desc: "+20% momentum value per level — each point of momentum adds more to every Push and Heave.",
                    base_cost: Big::from_f64(5.0), cost_inc: Big::from_f64(5.0),
                    max_level: 8, double_at: 5, triple_at: 7,
                },
                RunUpgradeDef {
                    key: "brawn", name: "Brawn",
                    desc: "+20% to Heave's base shove, per level.",
                    base_cost: Big::from_f64(3.0), cost_inc: Big::from_f64(3.0),
                    max_level: UNLIMITED, double_at: UNLIMITED, triple_at: UNLIMITED,
                },
                RunUpgradeDef {
                    key: "weight", name: "Weight",
                    desc: "+1% Heave multiplier per roll-back this run, per level (first 10×level roll-backs count).",
                    base_cost: Big::from_f64(5.0), cost_inc: Big::from_f64(5.0),
                    max_level: 8, double_at: 5, triple_at: 7,
                },
                RunUpgradeDef {
                    key: "surplus", name: "Surplus",
                    desc: "+50% per level to overshoot scorn — climbing past the summit during the roll-back countdown banks more; this multiplies that bonus.",
                    base_cost: Big::from_f64(5.0), cost_inc: Big::from_f64(5.0),
                    max_level: 8, double_at: 5, triple_at: 7,
                },
                RunUpgradeDef {
                    key: "thrift", name: "Thrift",
                    desc: "All scorn upgrades ~5% cheaper per level (diminishing — never free).",
                    base_cost: Big::from_f64(5.0), cost_inc: Big::from_f64(5.0),
                    max_level: 8, double_at: 5, triple_at: 7,
                },
                RunUpgradeDef {
                    key: "zeal", name: "Zeal",
                    desc: "Up to +5% scorn per level, scaling with how much scorn you've spent this run.",
                    base_cost: Big::from_f64(10.0), cost_inc: Big::from_f64(10.0),
                    max_level: 8, double_at: UNLIMITED, triple_at: UNLIMITED,
                },
            ],
        },

        // Not built for v1 — the long-term prestige horizon. Stubs keep the
        // match total; unreachable because only the Mountain is unlocked.
        other => UniverseDef {
            kind: other,
            name: "—",
            action_verb: "Push",
            native_pusher: native_pusher_for(other),
            summit: Big::new(1.0, 6),
            base_push: Big::ONE,
            base_scorn: Big::ONE,
            run_upgrades: vec![],
        },
    }
}

fn native_pusher_for(kind: UniverseKind) -> PusherKind {
    match kind {
        UniverseKind::Mountain => PusherKind::Boulder,
        UniverseKind::Code => PusherKind::Code,
        UniverseKind::Pallet => PusherKind::Pallet,
        UniverseKind::Market => PusherKind::Market,
    }
}

/// Tree 1 — global. The condition itself; applies everywhere.
pub fn global_skills() -> Vec<SkillNode> {
    vec![
        SkillNode {
            key: G_MIGHT,
            name: "Might",
            desc: "+25% push power everywhere, per rank.",
            base_cost: Big::ONE,
            cost_growth: 2.0,
            max_rank: 10,
            prereqs: vec![],
        },
        SkillNode {
            key: G_GRACE,
            name: "Grace",
            desc: "Other hands. Each run begins with +1 auto-push level, per rank.",
            base_cost: Big::from_f64(3.0),
            cost_growth: 3.0,
            max_rank: 5,
            prereqs: vec![G_MIGHT],
        },
        SkillNode {
            key: G_ENDURANCE,
            name: "Endurance",
            desc: "+30% scorn from every roll-back, per rank.",
            base_cost: Big::from_f64(2.0),
            cost_growth: 2.0,
            max_rank: 10,
            prereqs: vec![G_MIGHT],
        },
        SkillNode {
            key: G_RESOLVE,
            name: "Resolve",
            desc: "Each run begins with +1 scorn-gain level, per rank.",
            base_cost: Big::from_f64(5.0),
            cost_growth: 3.0,
            max_rank: 5,
            prereqs: vec![G_ENDURANCE],
        },
    ]
}

/// Tree 2 — per-pusher. The character; travels with the pusher across universes.
pub fn pusher_skills(pusher: PusherKind) -> Vec<SkillNode> {
    match pusher {
        PusherKind::Boulder => vec![
            SkillNode {
                key: P_CALLUSES,
                name: "Calluses",
                desc: "The Boulder Pusher's hands harden. +50% of this pusher's push, per rank.",
                base_cost: Big::ONE,
                cost_growth: 1.8,
                max_rank: 10,
                prereqs: vec![],
            },
            SkillNode {
                key: P_CADENCE,
                name: "Cadence",
                desc: "Begin each run with Surplus rank 1 — a head start on overshoot scorn.",
                base_cost: Big::from_f64(8.0),
                cost_growth: 1.0,
                max_rank: 1,
                prereqs: vec![P_CALLUSES],
            },
            SkillNode {
                key: P_DEFIANT_HANDS,
                name: "Defiant Hands",
                desc: "+40% of this pusher's scorn, per rank.",
                base_cost: Big::from_f64(2.0),
                cost_growth: 2.0,
                max_rank: 8,
                prereqs: vec![P_CALLUSES],
            },
        ],
        _ => vec![],
    }
}

/// Tree 3 — per-universe. The place; applies to whoever pushes here.
pub fn universe_skills(kind: UniverseKind) -> Vec<SkillNode> {
    match kind {
        UniverseKind::Mountain => vec![
            SkillNode {
                key: UNI_LOWER_SUMMIT,
                name: "Lower Summit",
                desc: "The Mountain itself is shorter. -8% summit height, per rank.",
                base_cost: Big::from_f64(4.0),
                cost_growth: 2.5,
                max_rank: 5,
                prereqs: vec![],
            },
            SkillNode {
                key: UNI_LOOSE_SCREE,
                name: "Loose Scree",
                desc: "The slope gives. +30% push on the Mountain, per rank.",
                base_cost: Big::from_f64(2.0),
                cost_growth: 2.0,
                max_rank: 8,
                prereqs: vec![],
            },
            SkillNode {
                key: UNI_THIN_AIR,
                name: "Thin Air",
                desc: "+50% scorn on the Mountain, per rank.",
                base_cost: Big::from_f64(6.0),
                cost_growth: 2.2,
                max_rank: 6,
                prereqs: vec![UNI_LOOSE_SCREE],
            },
        ],
        _ => vec![],
    }
}

/// Per-phase defiance track (ADR 0011): two nodes per phase, keyed "push" and
/// "scorn", giving bonuses that apply only during that phase. Flavour is in the
/// node names; the copy stays plain.
pub fn phase_skills(phase: Phase) -> Vec<SkillNode> {
    let (push_name, scorn_name) = match phase {
        Phase::Dawn => ("First Light", "Dew"),
        Phase::Morning => ("Warmth", "Routine"),
        Phase::Noon => ("Zenith", "Clarity"),
        Phase::Afternoon => ("High Sun", "Long Light"),
        Phase::Dusk => ("Last Light", "Golden Hour"),
        Phase::Midnight => ("Vigil", "The Abyss"),
    };
    vec![
        SkillNode {
            key: "push",
            name: push_name,
            desc: "+25% push during this phase, per rank.",
            base_cost: Big::from_f64(4.0),
            cost_growth: 2.0,
            max_rank: 8,
            prereqs: vec![],
        },
        SkillNode {
            key: "scorn",
            name: scorn_name,
            desc: "+25% scorn during this phase, per rank.",
            base_cost: Big::from_f64(4.0),
            cost_growth: 2.0,
            max_rank: 8,
            prereqs: vec![],
        },
    ]
}

pub fn find_skill<'a>(nodes: &'a [SkillNode], key: &str) -> Option<&'a SkillNode> {
    nodes.iter().find(|n| n.key == key)
}

pub fn run_upgrade_cost(def: &RunUpgradeDef, level: u32) -> Big {
    let mult = if level >= def.triple_at {
        3.0
    } else if level >= def.double_at {
        2.0
    } else {
        1.0
    };
    def.base_cost
        .add(def.cost_inc.mul_f64(level as f64))
        .mul_f64(mult)
}

pub fn skill_cost(node: &SkillNode, rank: u32) -> Big {
    node.base_cost.mul(Big::from_f64(node.cost_growth.powi(rank as i32)))
}

// ── Stance branches & sub-builds (ADR 0009) ───────────────────────────────────

pub struct SubBuildDef {
    pub key: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
}

/// The scorn-bought branch unique to a Stance: a normal node, then the
/// **keystone** (the last entry) which unlocks the sub-build choice. The keystone
/// requires the normal node at level >= `KEYSTONE_PREREQ_LEVEL`.
pub const KEYSTONE_PREREQ_LEVEL: u32 = 3;

pub fn stance_branch(stance: Stance) -> Vec<RunUpgradeDef> {
    match stance {
        Stance::Grind => vec![
            RunUpgradeDef {
                key: "grind_consistency", name: "Iron Hold",
                desc: "+30% Push & Heave power while at MAXIMUM momentum, per level. (Grind's core: hold the ceiling.)",
                base_cost: Big::from_f64(20.0), cost_inc: Big::from_f64(20.0),
                max_level: 20, double_at: UNLIMITED, triple_at: UNLIMITED,
            },
            RunUpgradeDef {
                key: "grind_keystone", name: "Tempo (keystone)",
                desc: "Find your rhythm — unlocks a Grind specialization.",
                base_cost: Big::from_f64(500.0), cost_inc: Big::ZERO,
                max_level: 1, double_at: UNLIMITED, triple_at: UNLIMITED,
            },
        ],
        Stance::Lurch => vec![
            RunUpgradeDef {
                key: "lurch_recklessness", name: "Recklessness",
                desc: "+20% HEAVE power per point of MISSING momentum, per level. (Lurch's core: wait for the Coil, then slam.)",
                base_cost: Big::from_f64(20.0), cost_inc: Big::from_f64(20.0),
                max_level: 20, double_at: UNLIMITED, triple_at: UNLIMITED,
            },
            RunUpgradeDef {
                key: "lurch_keystone", name: "Abandon (keystone)",
                desc: "Throw yourself fully in — unlocks a Lurch specialization.",
                base_cost: Big::from_f64(500.0), cost_inc: Big::ZERO,
                max_level: 1, double_at: UNLIMITED, triple_at: UNLIMITED,
            },
        ],
        Stance::Patience => vec![
            RunUpgradeDef {
                key: "patience_drift", name: "Conduction",
                desc: "+5% per level per point of momentum to your battery amp — both auto-push AND auto-heave, stacked on the innate. (Patience's core: convert your momentum.)",
                base_cost: Big::from_f64(20.0), cost_inc: Big::from_f64(20.0),
                max_level: 20, double_at: UNLIMITED, triple_at: UNLIMITED,
            },
            RunUpgradeDef {
                key: "patience_keystone", name: "Stillness (keystone)",
                desc: "Stop fighting it — unlocks a Patience specialization.",
                base_cost: Big::from_f64(500.0), cost_inc: Big::ZERO,
                max_level: 1, double_at: UNLIMITED, triple_at: UNLIMITED,
            },
        ],
    }
}

/// Key of the keystone node (the last branch entry).
pub fn keystone_key(stance: Stance) -> &'static str {
    match stance {
        Stance::Grind => "grind_keystone",
        Stance::Lurch => "lurch_keystone",
        Stance::Patience => "patience_keystone",
    }
}

/// Key of the normal node the keystone depends on.
pub fn keystone_prereq_key(stance: Stance) -> &'static str {
    match stance {
        Stance::Grind => "grind_consistency",
        Stance::Lurch => "lurch_recklessness",
        Stance::Patience => "patience_drift",
    }
}

/// The two mutually-exclusive sub-builds a Stance's keystone unlocks.
pub fn sub_builds(stance: Stance) -> Vec<SubBuildDef> {
    match stance {
        Stance::Grind => vec![
            SubBuildDef { key: "wide_floor", name: "Wide Floor", desc: "Raise the minimum push higher still — rock-solid reliability." },
            SubBuildDef { key: "steady_climb", name: "Steady Climb", desc: "Raise the average push — a higher, slightly looser band." },
        ],
        Stance::Lurch => vec![
            SubBuildDef { key: "big_spikes", name: "Big Spikes", desc: "Spikes are rarer but enormous." },
            SubBuildDef { key: "frequent_spikes", name: "Frequent Spikes", desc: "Spikes are smaller but come far more often." },
        ],
        Stance::Patience => vec![
            SubBuildDef { key: "offline", name: "Offline", desc: "Lean into time away — steady, relentless auto-push." },
            SubBuildDef { key: "active_idle", name: "Active Idle", desc: "A stronger auto-push for the watcher who stays." },
        ],
    }
}

pub fn find_sub_build(stance: Stance, key: &str) -> bool {
    sub_builds(stance).iter().any(|s| s.key == key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::UniverseKind;

    /// The sim references upgrades by these key constants; the data table defines
    /// them by literal. This fails loudly the moment the two drift (e.g. a key is
    /// renamed in the table but not the constant), so a stale reference can never
    /// silently read level 0.
    #[test]
    fn referenced_upgrade_keys_exist() {
        let keys = [
            U_PUSH_POWER, U_AUTO_PUSH,
            U_MUSCLE, U_POWER, U_SPEED, U_STREAK, U_SCORN_GAIN, U_AUTO_HASTE, U_HEAVE_MULT,
            U_CASCADE, U_REACH, U_CONDUIT, U_LEVERAGE,
            U_BRAWN, U_WEIGHT, U_ZEAL, U_SURPLUS, U_THRIFT,
        ];
        let def = universe_def(UniverseKind::Mountain);
        for k in keys {
            assert!(
                def.run_upgrades.iter().any(|u| u.key == k),
                "upgrade key constant `{k}` has no matching entry in run_upgrades"
            );
        }
    }
}
