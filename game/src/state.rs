//! ADR 0002 (rewritten) — three nested scopes, each with its own reset boundary:
//!
//! - **meta** (`GameState`) — survives prestige: defiance, the three skill trees,
//!   unlocks, organizing.
//! - **run** (`Run`) — survives roll-back, dies at prestige: the active universe
//!   and pusher, scorn, in-run upgrade levels.
//! - **climb** (`Climb`) — dies at roll-back: height.
//!
//! What crosses each boundary is a type-level fact: it's which struct the datum
//! lives in.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::content;
use crate::number::Big;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UniverseKind {
    Mountain,
    Code,
    Pallet,
    Market,
}

impl UniverseKind {
    pub fn key(self) -> &'static str {
        match self {
            UniverseKind::Mountain => "mountain",
            UniverseKind::Code => "code",
            UniverseKind::Pallet => "pallet",
            UniverseKind::Market => "market",
        }
    }
}

/// The protagonist identity — distinct from the universe, because after
/// organizing a pusher can push in a universe that isn't their native one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PusherKind {
    Boulder,
    Code,
    Pallet,
    Market,
}

impl PusherKind {
    pub fn key(self) -> &'static str {
        match self {
            PusherKind::Boulder => "boulder",
            PusherKind::Code => "code",
            PusherKind::Pallet => "pallet",
            PusherKind::Market => "market",
        }
    }
}

/// The per-run playstyle (ADR 0009). `None` on the first run (pre-first-prestige);
/// chosen at each prestige thereafter and locked for the run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stance {
    Grind,
    Lurch,
    Patience,
}

/// A phase of the in-game day (ADR 0011). Derived from the clock; also the key
/// for the per-phase defiance tracks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Dawn,
    Morning,
    Noon,
    Afternoon,
    Dusk,
    Midnight,
}

impl Phase {
    pub const ALL: [Phase; 6] = [
        Phase::Dawn,
        Phase::Morning,
        Phase::Noon,
        Phase::Afternoon,
        Phase::Dusk,
        Phase::Midnight,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Phase::Dawn => "dawn",
            Phase::Morning => "morning",
            Phase::Noon => "noon",
            Phase::Afternoon => "afternoon",
            Phase::Dusk => "dusk",
            Phase::Midnight => "midnight",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Phase::Dawn => "Dawn",
            Phase::Morning => "Morning",
            Phase::Noon => "Noon",
            Phase::Afternoon => "Afternoon",
            Phase::Dusk => "Dusk",
            Phase::Midnight => "Midnight",
        }
    }
}

/// Climb scope — reset by roll-back.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Climb {
    pub height: Big,
}

impl Climb {
    pub fn fresh() -> Climb {
        Climb { height: Big::ZERO }
    }
}

/// Run scope — reset by prestige, survives roll-back.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Run {
    pub universe: UniverseKind,
    /// The pusher currently in this universe — its native pusher, until
    /// organizing puts a different one here.
    pub pusher: PusherKind,
    /// In-run currency, earned per roll-back, spent on in-run upgrades.
    pub scorn: Big,
    /// In-run upgrade levels, keyed by the upgrade's `key`. Missing key = level 0.
    /// Keyed (not positional) so the save survives reordering/removing upgrades.
    pub upgrade_levels: BTreeMap<String, u32>,
    /// Roll-backs completed this run — drives the defiance payout at prestige.
    pub rollbacks: Big,
    pub climb: Climb,

    // ── ADR 0009 economy (run scope) ─────────────────────────────────────────
    /// The committed Stance, or `None` on the stance-less first run.
    pub stance: Option<Stance>,
    /// Active-play Momentum stack (manual pushes build it, it decays). Disabled
    /// under Patience.
    pub momentum: f64,
    /// PRNG state for variable push rolls. Persisted so the sequence is
    /// save-stable; advanced only by live manual pushes (offline uses flat
    /// auto-push, so the save/offline guarantees of ADR 0005 are untouched).
    pub rng: u64,
    /// Remaining Heave cooldown in ms (0 = ready).
    pub heave_cd: f64,
    /// Count of summit-milestones claimed this run (count-gated boosts).
    pub milestones: u32,
    /// Stance-branch upgrade levels (keyed by node key). Empty until a Stance is
    /// committed.
    pub branch: BTreeMap<String, u32>,
    /// The chosen sub-build ("race"), unlocked by the Stance keystone. Locked once
    /// set, for the run.
    pub sub_build: Option<String>,
    /// Manual pushes done this run. Drives the onboarding gate: the first
    /// auto-push is granted free at 5 (ADR 0009), and powers the "0/5" UI.
    pub pushes_this_run: u32,
    /// Scorn spent this run — drives Zeal (scorn gain scales with spending).
    #[serde(default)]
    pub scorn_spent: Big,
    /// Cascade proc signal: a counter the UI watches to fire a burst float, and
    /// the value of the last proc. Run scope, not persisted meaningfully.
    #[serde(default)]
    pub cascade_seq: u64,
    #[serde(default)]
    pub cascade_effort: Big,
    /// Seconds accumulated toward the next Cascade burst (deterministic timer).
    #[serde(default)]
    pub cascade_timer: f64,
    /// Milliseconds since the last manual action — Grind holds momentum while this
    /// is under the active window.
    #[serde(default)]
    pub since_action: f64,
    /// Seconds accumulated toward Patience's next battery charge (+1/cycle).
    #[serde(default)]
    pub battery_timer: f64,
    /// Overshoot countdown: seconds left before the boulder auto-rolls back once
    /// you're at/above the summit. `-1` = not counting (below the summit).
    #[serde(default = "neg_one")]
    pub rollback_countdown: f64,
    /// Best single push / heave effort, and best scorn/roll-back, THIS run — shown
    /// on the cards, reset at prestige.
    #[serde(default)]
    pub best_push: Big,
    #[serde(default)]
    pub best_heave: Big,
    #[serde(default)]
    pub best_scorn: Big,
    /// Seconds of continuous manual pushing (resets on a gap) — drives Streak.
    #[serde(default)]
    pub push_streak_secs: f64,
}

fn neg_one() -> f64 {
    -1.0
}

impl Run {
    /// A fresh run of a universe, seeded by the permanent skill trees (the
    /// "begin each run with…" effects).
    pub fn fresh(universe: UniverseKind, skills: &SkillState) -> Run {
        let def = content::universe_def(universe);
        let pusher = def.native_pusher;
        let mut levels: BTreeMap<String, u32> = BTreeMap::new();

        // Permanent head-starts from the skill trees — seed only what's non-zero;
        // a missing key already reads as level 0.
        let mut seed = |key: &str, rank: u32| {
            if rank > 0 {
                levels.insert(key.to_string(), rank);
            }
        };
        seed(content::U_AUTO_PUSH, skills.global_rank(content::G_GRACE));
        seed(content::U_SCORN_GAIN, skills.global_rank(content::G_RESOLVE));
        seed(content::U_SURPLUS, skills.pusher_rank(pusher.key(), content::P_CADENCE));

        Run {
            universe,
            pusher,
            scorn: Big::ZERO,
            upgrade_levels: levels,
            rollbacks: Big::ZERO,
            climb: Climb::fresh(),
            stance: None,
            momentum: 0.0,
            // Non-zero seed mixed with the universe so different runs roll
            // different sequences. A real per-run mix comes with the behavioural
            // increment; the field exists now so the save shape is stable.
            rng: 0x9E3779B97F4A7C15 ^ (universe as u64).wrapping_mul(0x100000001B3),
            heave_cd: 0.0,
            milestones: 0,
            branch: BTreeMap::new(),
            sub_build: None,
            pushes_this_run: 0,
            scorn_spent: Big::ZERO,
            cascade_seq: 0,
            cascade_effort: Big::ZERO,
            cascade_timer: 0.0,
            since_action: 1e9, // "not active" until the first push
            battery_timer: 0.0,
            rollback_countdown: -1.0, // not counting until the summit is reached
            best_push: Big::ZERO,
            best_heave: Big::ZERO,
            best_scorn: Big::ZERO,
            push_streak_secs: 0.0,
        }
    }
}

/// The three permanent skill trees. Ranks keyed by node key; the per-pusher and
/// per-universe maps are keyed by `PusherKind::key` / `UniverseKind::key`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SkillState {
    pub global: BTreeMap<String, u32>,
    pub per_pusher: BTreeMap<String, BTreeMap<String, u32>>,
    pub per_universe: BTreeMap<String, BTreeMap<String, u32>>,
    /// Per-phase tracks (ADR 0011), keyed by `Phase::key`.
    #[serde(default)]
    pub per_phase: BTreeMap<String, BTreeMap<String, u32>>,
}

impl SkillState {
    pub fn new() -> SkillState {
        SkillState::default()
    }

    pub fn global_rank(&self, key: &str) -> u32 {
        self.global.get(key).copied().unwrap_or(0)
    }
    pub fn pusher_rank(&self, pusher: &str, key: &str) -> u32 {
        self.per_pusher.get(pusher).and_then(|m| m.get(key)).copied().unwrap_or(0)
    }
    pub fn universe_rank(&self, universe: &str, key: &str) -> u32 {
        self.per_universe.get(universe).and_then(|m| m.get(key)).copied().unwrap_or(0)
    }
    pub fn phase_rank(&self, phase: &str, key: &str) -> u32 {
        self.per_phase.get(phase).and_then(|m| m.get(key)).copied().unwrap_or(0)
    }

    pub fn bump_global(&mut self, key: &str) {
        *self.global.entry(key.to_string()).or_insert(0) += 1;
    }
    pub fn bump_pusher(&mut self, pusher: &str, key: &str) {
        *self
            .per_pusher
            .entry(pusher.to_string())
            .or_default()
            .entry(key.to_string())
            .or_insert(0) += 1;
    }
    pub fn bump_universe(&mut self, universe: &str, key: &str) {
        *self
            .per_universe
            .entry(universe.to_string())
            .or_default()
            .entry(key.to_string())
            .or_insert(0) += 1;
    }
    pub fn bump_phase(&mut self, phase: &str, key: &str) {
        *self
            .per_phase
            .entry(phase.to_string())
            .or_default()
            .entry(key.to_string())
            .or_insert(0) += 1;
    }
}

/// Organizing endgame state — architecturally hooked at the root; mechanics are
/// a later design grill.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OrganizingState {
    pub unlocked: bool,
}

/// Meta scope — survives prestige.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    /// The permanent prestige currency.
    pub defiance: Big,
    pub skills: SkillState,
    pub unlocked: Vec<UniverseKind>,
    pub organizing: OrganizingState,
    pub run: Run,
    pub total_pushes: Big,
    pub total_prestiges: Big,
    /// Has the player ever reached a summit and rolled back? Permanently unlocks
    /// the Scorn tab (progressive disclosure, ADR 0010).
    #[serde(default)]
    pub ever_rolled_back: bool,
    /// Has the player ever been able to prestige? Permanently unlocks the
    /// Defiance tab, which houses the prestige action itself.
    #[serde(default)]
    pub seen_prestige: bool,
    /// Seconds into the in-game day (0..720), advanced by elapsed time. Drives the
    /// day/night cycle and phases (ADR 0011). Meta scope — survives everything.
    #[serde(default)]
    pub clock: f64,
    /// ALL-TIME best single push / heave effort — drives the floating crit
    /// numbers, and shown on hover. Tracked separately so pushes and heaves each
    /// feel proportional. (Per-run bests, shown on the card, live on `Run`.)
    #[serde(default)]
    pub best_push: Big,
    #[serde(default)]
    pub best_heave: Big,
    /// Context for each all-time best: the stance it was set in, and the prestige
    /// count at the time (so the hover can say "N prestiges ago").
    #[serde(default)]
    pub best_push_stance: String,
    #[serde(default)]
    pub best_push_prestige: Big,
    #[serde(default)]
    pub best_heave_stance: String,
    #[serde(default)]
    pub best_heave_prestige: Big,
    /// All-time best scorn banked in one roll-back, + its context, + the last
    /// roll-back's gain / frac / new-best flag (drives the scorn crit-float).
    #[serde(default)]
    pub best_scorn: Big,
    #[serde(default)]
    pub best_scorn_stance: String,
    #[serde(default)]
    pub best_scorn_prestige: Big,
    #[serde(default)]
    pub last_scorn: Big,
    #[serde(default)]
    pub last_scorn_frac: f64,
    #[serde(default)]
    pub last_scorn_best: bool,
    /// The most recent manual action, for spawning its floating number.
    #[serde(default)]
    pub last_effort: Big,
    #[serde(default)]
    pub last_frac: f64,
    #[serde(default)]
    pub last_best: bool,
    #[serde(default)]
    pub last_kind: u8, // 0 none · 1 push · 2 heave
    /// Wall-clock ms at last save (ADR 0005).
    pub last_seen: f64,
}

impl GameState {
    pub fn new() -> GameState {
        let skills = SkillState::new();
        let run = Run::fresh(UniverseKind::Mountain, &skills);
        GameState {
            defiance: Big::ZERO,
            skills,
            unlocked: vec![UniverseKind::Mountain],
            organizing: OrganizingState::default(),
            run,
            total_pushes: Big::ZERO,
            total_prestiges: Big::ZERO,
            ever_rolled_back: false,
            seen_prestige: false,
            clock: 0.0, // begin at Dawn — a new player watches the sun rise
            best_push: Big::ZERO,
            best_heave: Big::ZERO,
            best_push_stance: String::new(),
            best_push_prestige: Big::ZERO,
            best_heave_stance: String::new(),
            best_heave_prestige: Big::ZERO,
            best_scorn: Big::ZERO,
            best_scorn_stance: String::new(),
            best_scorn_prestige: Big::ZERO,
            last_scorn: Big::ZERO,
            last_scorn_frac: 0.0,
            last_scorn_best: false,
            last_effort: Big::ZERO,
            last_frac: 0.0,
            last_best: false,
            last_kind: 0,
            last_seen: 0.0,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState::new()
    }
}
