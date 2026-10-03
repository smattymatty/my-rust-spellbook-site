//! ADR 0003 (out-half) — the read-only render view-model. Numbers arrive
//! pre-formatted; a separate type from internal state.

use serde::Serialize;

/// A live derived stat for the Climb tab — shown only once it's above its
/// default (i.e. the player has actually invested in it).
#[derive(Serialize)]
pub struct StatView {
    pub label: String,
    pub value: String,
    /// A source breakdown for the hover tooltip (empty when there's nothing to
    /// break down — e.g. single-source stats).
    #[serde(default)]
    pub detail: String,
}

#[derive(Serialize)]
pub struct TabView {
    pub key: String,
    pub name: String,
    pub unlocked: bool,
    /// Shown in the hover popover on the locked "next" tab.
    pub hint: String,
}

#[derive(Serialize)]
pub struct Snapshot {
    /// Tabs in order (ADR 0010 breadcrumb-of-one disclosure). The UI shows
    /// unlocked tabs plus the first locked one, hiding the rest.
    pub tabs: Vec<TabView>,
    /// The day/night cycle (ADR 0011): 0..1 through the day, and the current phase.
    pub day_fraction: f64,
    pub phase: String,
    pub phase_key: String,
    /// Live upgrade-derived stats for the Climb tab (only those above default).
    pub stats: Vec<StatView>,
    /// The active stance's identity: its always-on innate rule + its shop passive's
    /// current effect (empty when stance-less). Rendered as its own purple set.
    pub stance_stats: Vec<StatView>,
    /// The most recent manual action, for the floating crit number.
    pub last_effort: String,
    pub last_frac: f64,
    pub last_best: bool,
    pub last_kind: String, // "push" / "heave" / ""
    /// All-time best push / heave / scorn-per-roll-back as a hover breakdown
    /// (value · stance · N prestiges ago). Empty until earned. Per-run card values
    /// are on RunView.
    pub best_push_detail: String,
    pub best_heave_detail: String,
    pub best_scorn_detail: String,
    /// The last roll-back's scorn gain + how it ranks vs the per-run best — drives
    /// the scorn crit-float on the counter. `rollbacks_n` rising is the trigger.
    pub last_scorn: String,
    pub last_scorn_frac: f64,
    pub last_scorn_best: bool,
    /// Manual-action count (push/heave only) — the UI fires a float when it rises.
    pub action_count: f64,
    /// Cascade proc signal: a counter the UI watches (rises → fire a burst float)
    /// and the value of the last burst.
    pub cascade_seq: f64,
    pub cascade_effort: String,
    pub defiance: String,
    pub total_pushes: String,
    pub total_prestiges: String,
    pub organizing_unlocked: bool,
    pub run: RunView,
    /// The three permanent skill trees.
    pub global_skills: Vec<SkillNodeView>,
    pub pusher_skills: Vec<SkillNodeView>,
    pub universe_skills: Vec<SkillNodeView>,
    /// The six per-phase defiance tracks (ADR 0011); the active one is flagged.
    pub phase_tracks: Vec<PhaseTrackView>,
    pub can_prestige: bool,
    pub prestige_gain: String,
    /// Roll-backs needed to reach the NEXT defiance point (defiance = ⌊√(rb/10)⌋,
    /// so the next point is at (gain+1)²·10). Drives the prestige "X/Y" hint.
    pub next_defiance_at: f64,
    pub universe_choices: Vec<UniverseChoice>,
}

#[derive(Serialize)]
pub struct RunView {
    pub universe: String,
    pub universe_name: String,
    pub pusher: String,
    pub pusher_name: String,
    pub action_verb: String,
    pub scorn: String,
    pub height: String,
    pub summit: String,
    pub progress: f64,
    pub at_summit: bool,
    pub rollbacks: String,
    /// Roll-backs as a number — drives how long the hill grows / how far it scrolls.
    pub rollbacks_n: f64,
    pub push_amount: String,
    pub auto_push_rate: String,
    /// Auto-push split into its two levers: shove size (Rolling) and the seconds
    /// between shoves (Haste) — so the UI shows "+size every Ts", not one rate.
    pub auto_push_size: String,
    pub auto_push_interval: f64,
    pub can_rollback: bool,
    /// Seconds left before the overshoot auto-roll-back fires (−1 = below summit).
    pub rollback_countdown: f64,
    /// The scorn multiplier a roll-back would bank right now (height/summit, Surplus-boosted).
    pub overshoot_mult: f64,
    /// The committed Stance ("grind"/"lurch"/"patience"), or null on the
    /// stance-less first run.
    pub stance: Option<String>,
    /// Current Momentum stack, and its cap (for the bar).
    pub momentum: f64,
    pub momentum_max: f64,
    /// Whether Heave can be used right now (off cooldown, active stance).
    pub heave_ready: bool,
    /// Heave recharge 0..1 (1 = ready) — the button renders itself as this bar.
    pub heave_cooldown: f64,
    /// True under Patience — the UI hides/disables the manual push button.
    pub manual_disabled: bool,
    /// Shared-pool scorn upgrades.
    pub upgrades: Vec<UpgradeView>,
    /// The active Stance's branch (empty when stance-less).
    pub stance_branch: Vec<UpgradeView>,
    /// The committed sub-build key, if chosen.
    pub sub_build: Option<String>,
    /// Sub-build options, populated only once the keystone is bought and no
    /// sub-build is chosen yet.
    pub sub_build_choices: Vec<SubBuildView>,
    /// Summit-milestones claimed this run.
    pub milestones: u32,
    /// Manual pushes done this run (for the "0/5 → free auto-push" onboarding UI).
    pub pushes_this_run: u32,
    /// Whether auto-push is unlocked yet (level >= 1).
    pub auto_unlocked: bool,
    /// Best push / heave / scorn-per-roll-back THIS run, formatted (empty until
    /// earned) — the card values.
    pub best_push: String,
    pub best_heave: String,
    pub best_scorn: String,
    /// Max hold-to-push rate per second (Speed raises it) — the client caps its
    /// input ramp to this.
    pub push_rate_max: f64,
}

#[derive(Serialize)]
pub struct SubBuildView {
    pub key: String,
    pub name: String,
    pub desc: String,
}

#[derive(Serialize)]
pub struct PhaseTrackView {
    pub phase_key: String,
    pub phase_name: String,
    pub active: bool,
    pub nodes: Vec<SkillNodeView>,
}

#[derive(Serialize)]
pub struct UpgradeView {
    pub key: String,
    pub name: String,
    pub desc: String,
    pub level: u32,
    /// The upgrade's live effect at the current rank, and at the next rank — the
    /// UI shows "now → next". Computed by reading the real sim accessor (and a
    /// +1-rank clone) so it can never drift from the actual effect.
    pub effect: String,
    pub effect_next: String,
    /// Max rank (u32::MAX for "infinite" — the UI treats a huge value as ∞).
    pub max_level: u32,
    pub maxed: bool,
    pub cost: String,
    /// Numeric cost, so the UI can sort by price (the formatted string can't).
    pub cost_n: f64,
    pub affordable: bool,
    /// Gated by an unmet prerequisite (e.g. a keystone before its prereq).
    #[serde(default)]
    pub locked: bool,
}

#[derive(Serialize)]
pub struct SkillNodeView {
    pub key: String,
    pub name: String,
    pub desc: String,
    pub rank: u32,
    pub max_rank: u32,
    pub maxed: bool,
    pub cost: String,
    pub affordable: bool,
    /// All prerequisites met — the node is purchasable (funds aside).
    pub unlocked: bool,
    pub prereqs: Vec<String>,
}

#[derive(Serialize)]
pub struct UniverseChoice {
    pub kind: String,
    pub name: String,
}
