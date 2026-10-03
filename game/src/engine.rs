//! The engine ties the core together and is the whole testable API. The
//! wasm-bindgen layer is a thin forwarder over this.

use serde::Serialize;

use crate::command::Command;
use crate::content::{self, SkillNode};
use crate::number::Big;
use crate::save::{self, LoadOutcome};
use crate::sim;
use crate::snapshot::{
    PhaseTrackView, RunView, SkillNodeView, Snapshot, StatView, SubBuildView, TabView,
    UniverseChoice, UpgradeView,
};
use crate::state::{GameState, Phase, Stance};

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum LoadReport {
    Started,
    Resumed { offline_seconds: f64, steps: u64 },
    Refused { found: u32, current: u32 },
    Corrupt { message: String },
}

pub struct Engine {
    state: GameState,
    acc: f64,
    /// ADR 0007: false when the loaded blob was Refused/Corrupt, so we never
    /// clobber a save we couldn't safely read.
    savable: bool,
    last_report: LoadReport,
}

fn progress_ratio(height: &Big, summit: &Big) -> f64 {
    if height.is_zero() || summit.is_zero() {
        return 0.0;
    }
    // sqrt of the linear ratio: quicker than linear early, but far less
    // front-loaded than log — so the climb doesn't rocket up then crawl the last
    // stretch near the summit.
    height.div(*summit).to_f64().clamp(0.0, 1.0).sqrt()
}

fn skill_views<F>(nodes: &[SkillNode], rank_of: F, defiance: &Big) -> Vec<SkillNodeView>
where
    F: Fn(&str) -> u32,
{
    nodes
        .iter()
        .map(|n| {
            let rank = rank_of(n.key);
            let maxed = rank >= n.max_rank;
            let unlocked = n.prereqs.iter().all(|p| rank_of(p) >= 1);
            let cost = content::skill_cost(n, rank);
            SkillNodeView {
                key: n.key.to_string(),
                name: n.name.to_string(),
                desc: n.desc.to_string(),
                rank,
                max_rank: n.max_rank,
                maxed,
                cost: if maxed { "—".to_string() } else { cost.format() },
                affordable: !maxed && unlocked && defiance.gte(&cost),
                unlocked,
                prereqs: n.prereqs.iter().map(|s| s.to_string()).collect(),
            }
        })
        .collect()
}

/// A short live-effect string for an upgrade, read from the real sim accessors
/// (so it can't drift from the actual effect). The shop shows this at the current
/// rank and — via a +1-level clone — at the next rank ("now → next").
fn upgrade_bonus(st: &GameState, key: &str) -> String {
    let trim = |v: f64| {
        let s = format!("{v:.2}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    };
    let lvl = st.run.upgrade_levels.get(key).copied().unwrap_or(0);
    match key {
        content::U_MUSCLE => format!("+{} power", lvl),
        content::U_POWER => format!("×{} push", trim(sim::manual_push_mult(st))),
        content::U_SPEED => format!("{}/s", sim::push_rate_max(st) as i64),
        content::U_STREAK => format!("+{}% cap", lvl * 10),
        content::U_PUSH_POWER => {
            // just Grip's own factor (not Muscle/skills/phase)
            let grip = sim::push_sources(st).iter().find(|(n, _)| *n == "Grip").map(|(_, m)| *m).unwrap_or(1.0);
            format!("×{} power", trim(grip))
        }
        content::U_LEVERAGE => format!("×{} final", trim(sim::push_multiplier(st))),
        content::U_CONDUIT => format!("×{} mom", trim(sim::momentum_value(st))),
        content::U_HEAVE_MULT | content::U_WEIGHT => format!("×{} heave", trim(sim::heave_mult(st))),
        content::U_BRAWN => format!("×{} base", trim(sim::heave_base_mult(st))),
        content::U_REACH => format!("{} cap", sim::momentum_cap_now(st) as i64),
        content::U_AUTO_PUSH => format!("×{} shove", trim(sim::rolling_mult(st))),
        content::U_AUTO_HASTE => format!("{}s int", trim(sim::auto_push_interval(st))),
        content::U_CASCADE => {
            let i = sim::cascade_interval(st);
            if i > 0.0 { format!("{}s", trim(i)) } else { "off".to_string() }
        }
        content::U_SCORN_GAIN => format!("+{} scorn", lvl),
        content::U_ZEAL => format!("+{:.0}% scorn", (sim::zeal_mult(st) - 1.0) * 100.0),
        content::U_SURPLUS => format!("+{:.0}% over", (sim::surplus_mult(st) - 1.0) * 100.0),
        content::U_THRIFT => format!("-{:.0}% cost", (1.0 - sim::thrift_mult(st)) * 100.0),
        // stance-branch load-bearing passives
        "grind_consistency" => format!("+{}% at max", 30 * lvl),
        "lurch_recklessness" => format!("+{}%/missing", 20 * lvl),
        "patience_drift" => format!("+{}%/mom", 5 * lvl),
        _ => String::new(),
    }
}

/// The all-time-best hover: "all-time best V · set in STANCE · N prestiges ago".
/// Empty until a best exists.
fn best_detail(val: &Big, stance: &str, at: &Big, now: &Big) -> String {
    if val.is_zero() {
        return String::new();
    }
    let ago = (now.to_f64() - at.to_f64()).max(0.0) as i64;
    let when = if ago <= 0 { "this prestige".to_string() } else { format!("{ago} prestige{} ago", if ago == 1 { "" } else { "s" }) };
    let stance = if stance.is_empty() { "no stance" } else { stance };
    format!("all-time best {} · set in {} · {}", val.format(), stance, when)
}

/// The active stance's identity as purple "your edge" cards: its always-on innate
/// rule (shown immediately), plus its shop passive's live effect (once ranked).
/// Reads the same sim accessors the effect uses, so it can't drift.
fn stance_stats(st: &GameState) -> Vec<StatView> {
    let stance = match st.run.stance { Some(s) => s, None => return Vec::new() };
    let mom = st.run.momentum;
    let cap = sim::momentum_cap_now(st);
    let branch = |k: &str| st.run.branch.get(k).copied().unwrap_or(0);
    let card = |label: &str, value: String, detail: String| StatView { label: label.into(), value, detail };
    match stance {
        Stance::Grind => vec![
            card("Grind — innate", "2× cap · holds".into(),
                format!("Momentum cap is DOUBLED to {} and won't decay while you keep acting — a real Push/Heave every ~2s (auto-push does NOT count).", cap as i64)),
            {
                let rank = branch("grind_consistency");
                let bonus = sim::grind_max_bonus(st);
                let at_max = mom >= cap - 1e-6;
                card("Iron Hold",
                    if rank == 0 { "unranked".into() } else { format!("+{:.0}%{}", bonus * 100.0, if at_max { " · NOW" } else { "" }) },
                    format!("+30%/rank to Push & Heave power while momentum is at MAX. Rank {rank}. {}", if at_max { "At max now — active." } else { "Fill momentum to max to trigger." }))
            },
        ],
        Stance::Lurch => {
            let missing = (cap - mom).max(0.0);
            let mult = sim::lurch_missing_mult(st); // innate + Recklessness at current missing — Push & Heave
            let rank = branch("lurch_recklessness");
            let reck_add = sim::lurch_missing_rate(st) * missing; // Recklessness's share of the mult
            vec![
                card("Lurch — innate", format!("heave ×{mult:.1}"),
                    format!("Momentum drains fast (~2.5s). Only the HEAVE re-coils it to full, and it slams based on how EMPTY the Coil was — scaled by +{:.0}% per missing point (now ×{mult:.1}, biggest at empty). Then spend that momentum: each Push adds it as a flat bonus, strong right after a heave, fading as it drains. Loop: drain → heave → spam push → repeat.", sim::LURCH_INNATE_RATE * 100.0)),
                card("Recklessness",
                    if rank == 0 { "unranked".into() } else { format!("+{:.0}% heave", reck_add * 100.0) },
                    format!("EXTRA per missing point on top of the innate, on your HEAVE: +20%/rank. Rank {rank}. Missing now {missing:.0} → +{:.0}%.", reck_add * 100.0)),
            ]
        }
        Stance::Patience => {
            let amp_pct = (sim::patience_amp(st) - 1.0) * 100.0; // live battery amp (innate + Conduction)
            vec![
                card("Patience — innate", format!("+{amp_pct:.0}% auto"),
                    format!("No manual push. The fuller your momentum battery, the harder it drives everything: +{:.0}% to BOTH auto-push and auto-heave per point (now +{amp_pct:.0}%). It fills +1/cycle (~2 min), never decays, up to your momentum cap ({cap} now, raised by Reach). Each point also adds flat effort to every auto-shove; double auto-push; sole auto-Heaver (the bar by Push shows the slam).", sim::PATIENCE_AMP_RATE * 100.0)),
                {
                    let rank = branch("patience_drift");
                    let pct = sim::patience_conduction_rate(st) * mom * 100.0;
                    card("Conduction",
                        if rank == 0 { "unranked".into() } else { format!("+{pct:.0}% amp") },
                        format!("+5%/rank per point of momentum, stacked into the battery amp (both auto-push AND auto-heave). Rank {rank}. Momentum now {mom:.0} → +{pct:.0}% on top of the innate."))
                },
            ]
        }
    }
}

/// `(effect, effect_next)` for an upgrade at its current level — the next value is
/// read from a clone with the key bumped one rank (or "—" when maxed).
fn upgrade_effects(st: &GameState, key: &str, level: u32, maxed: bool) -> (String, String) {
    let now = upgrade_bonus(st, key);
    let next = if maxed {
        "—".to_string()
    } else {
        let mut clone = st.clone();
        clone.run.upgrade_levels.insert(key.to_string(), level + 1);
        upgrade_bonus(&clone, key)
    };
    (now, next)
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            state: GameState::new(),
            acc: 0.0,
            savable: true,
            last_report: LoadReport::Started,
        }
    }

    pub fn load_or_new(saved: &str, now_ms: f64) -> Engine {
        if saved.trim().is_empty() {
            return Engine::new();
        }
        match save::load(saved) {
            LoadOutcome::Loaded(state) => {
                let mut state = *state;
                let mut acc = 0.0;
                let dt = (now_ms - state.last_seen).max(0.0);
                let steps = sim::advance(&mut state, &mut acc, dt);
                Engine {
                    state,
                    acc,
                    savable: true,
                    last_report: LoadReport::Resumed {
                        offline_seconds: (steps as f64 * sim::STEP_MS) / 1000.0,
                        steps,
                    },
                }
            }
            LoadOutcome::Refused { found, current } => Engine {
                state: GameState::new(),
                acc: 0.0,
                savable: false,
                last_report: LoadReport::Refused { found, current },
            },
            LoadOutcome::Corrupt(message) => Engine {
                state: GameState::new(),
                acc: 0.0,
                savable: false,
                last_report: LoadReport::Corrupt { message },
            },
        }
    }

    pub fn last_report(&self) -> &LoadReport {
        &self.last_report
    }
    pub fn last_report_json(&self) -> String {
        serde_json::to_string(&self.last_report).expect("report serializes")
    }
    pub fn savable(&self) -> bool {
        self.savable
    }

    pub fn tick(&mut self, real_dt_ms: f64) {
        sim::advance(&mut self.state, &mut self.acc, real_dt_ms);
    }

    pub fn dispatch(&mut self, cmd: &Command) {
        sim::apply(&mut self.state, cmd);
    }

    pub fn dispatch_json(&mut self, cmd_json: &str) {
        if let Ok(cmd) = serde_json::from_str::<Command>(cmd_json) {
            self.dispatch(&cmd);
        }
    }

    pub fn serialize(&mut self, now_ms: f64) -> Option<String> {
        if !self.savable {
            return None;
        }
        self.state.last_seen = now_ms;
        Some(save::serialize(&self.state))
    }

    pub fn import_json(&mut self, raw: &str, now_ms: f64) -> String {
        match save::load(raw) {
            LoadOutcome::Loaded(state) => {
                let mut state = *state;
                let mut acc = 0.0;
                let dt = (now_ms - state.last_seen).max(0.0);
                let steps = sim::advance(&mut state, &mut acc, dt);
                self.state = state;
                self.acc = acc;
                self.savable = true;
                self.last_report = LoadReport::Resumed {
                    offline_seconds: (steps as f64 * sim::STEP_MS) / 1000.0,
                    steps,
                };
            }
            LoadOutcome::Refused { found, current } => {
                self.last_report = LoadReport::Refused { found, current };
            }
            LoadOutcome::Corrupt(message) => {
                self.last_report = LoadReport::Corrupt { message };
            }
        }
        self.last_report_json()
    }

    pub fn force_new_game(&mut self) {
        self.state = GameState::new();
        self.acc = 0.0;
        self.savable = true;
        self.last_report = LoadReport::Started;
    }

    pub fn snapshot(&self) -> Snapshot {
        let st = &self.state;
        let run = &st.run;
        let def = content::universe_def(run.universe);
        let summit = sim::summit(st);
        let height = run.climb.height;

        let upgrades = def
            .run_upgrades
            .iter()
            .map(|ud| {
                let level = run.upgrade_levels.get(ud.key).copied().unwrap_or(0);
                let maxed = level >= ud.max_level;
                let cost = content::run_upgrade_cost(ud, level).mul_f64(sim::thrift_mult(st)); // Thrift discount
                let (effect, effect_next) = upgrade_effects(st, ud.key, level, maxed);
                UpgradeView {
                    key: ud.key.to_string(),
                    name: ud.name.to_string(),
                    desc: ud.desc.to_string(),
                    level,
                    effect,
                    effect_next,
                    max_level: ud.max_level,
                    maxed,
                    cost: if maxed { "—".to_string() } else { cost.format() },
                    cost_n: cost.to_f64(),
                    affordable: !maxed && run.scorn.gte(&cost),
                    locked: false,
                }
            })
            .collect();

        // The active Stance's branch (with keystone gating), and the sub-build
        // fork once the keystone is bought.
        let mut stance_branch: Vec<UpgradeView> = Vec::new();
        let mut sub_build_choices: Vec<SubBuildView> = Vec::new();
        if let Some(stance) = run.stance {
            let keystone = content::keystone_key(stance);
            let prereq = content::keystone_prereq_key(stance);
            let prereq_met =
                run.branch.get(prereq).copied().unwrap_or(0) >= content::KEYSTONE_PREREQ_LEVEL;
            for nd in content::stance_branch(stance) {
                let level = run.branch.get(nd.key).copied().unwrap_or(0);
                let maxed = level >= nd.max_level;
                let cost = content::run_upgrade_cost(&nd, level);
                let locked = nd.key == keystone && !prereq_met;
                // stance passives read from run.branch — compute their effect inline
                let passive = |l: u32| match nd.key {
                    "grind_consistency" => format!("+{}% at max", 30 * l),
                    "lurch_recklessness" => format!("+{}%/missing", 20 * l),
                    "patience_drift" => format!("+{}%/mom", 5 * l),
                    _ => String::new(),
                };
                let effect = passive(level);
                let effect_next = if maxed { "—".to_string() } else { passive(level + 1) };
                stance_branch.push(UpgradeView {
                    key: nd.key.to_string(),
                    name: nd.name.to_string(),
                    desc: nd.desc.to_string(),
                    level,
                    effect,
                    effect_next,
                    max_level: nd.max_level,
                    maxed,
                    cost: if maxed { "—".to_string() } else { cost.format() },
                    cost_n: cost.to_f64(),
                    affordable: !maxed && !locked && run.scorn.gte(&cost),
                    locked,
                });
            }
            let keystone_bought = run.branch.get(keystone).copied().unwrap_or(0) >= 1;
            if keystone_bought && run.sub_build.is_none() {
                for sb in content::sub_builds(stance) {
                    sub_build_choices.push(SubBuildView {
                        key: sb.key.to_string(),
                        name: sb.name.to_string(),
                        desc: sb.desc.to_string(),
                    });
                }
            }
        }

        let run_view = RunView {
            universe: run.universe.key().to_string(),
            universe_name: def.name.to_string(),
            pusher: run.pusher.key().to_string(),
            pusher_name: format!("{:?} Pusher", run.pusher).replace("Boulder", "Boulder"),
            action_verb: def.action_verb.to_string(),
            scorn: run.scorn.format(),
            height: height.format(),
            summit: summit.format(),
            progress: progress_ratio(&height, &summit),
            at_summit: height.gte(&summit),
            rollbacks: run.rollbacks.format(),
            rollbacks_n: run.rollbacks.to_f64(),
            push_amount: sim::push_amount(st).format(),
            auto_push_rate: sim::auto_push_rate(st).format(),
            auto_push_size: sim::auto_push_size(st).format(),
            auto_push_interval: sim::auto_push_interval(st),
            rollback_countdown: run.rollback_countdown,
            overshoot_mult: sim::overshoot_mult(st),
            can_rollback: height.gte(&summit),
            stance: run.stance.map(|s| format!("{s:?}").to_lowercase()),
            momentum: run.momentum,
            momentum_max: sim::momentum_cap_now(st),
            heave_ready: run.heave_cd <= 0.0 && run.stance != Some(Stance::Patience),
            heave_cooldown: sim::heave_progress(st),
            manual_disabled: run.stance == Some(Stance::Patience),
            upgrades,
            stance_branch,
            sub_build: run.sub_build.clone(),
            sub_build_choices,
            milestones: run.milestones,
            pushes_this_run: run.pushes_this_run,
            auto_unlocked: run.upgrade_levels.get(content::U_AUTO_PUSH).copied().unwrap_or(0) >= 1,
            best_push: if run.best_push.is_zero() { String::new() } else { run.best_push.format() },
            best_heave: if run.best_heave.is_zero() { String::new() } else { run.best_heave.format() },
            best_scorn: if run.best_scorn.is_zero() { String::new() } else { run.best_scorn.format() },
            push_rate_max: sim::push_rate_max(st),
        };

        let pusher_key = run.pusher.key();
        let universe_key = run.universe.key();
        let global_skills = skill_views(
            &content::global_skills(),
            |k| st.skills.global_rank(k),
            &st.defiance,
        );
        let pusher_skills = skill_views(
            &content::pusher_skills(run.pusher),
            |k| st.skills.pusher_rank(pusher_key, k),
            &st.defiance,
        );
        let universe_skills = skill_views(
            &content::universe_skills(run.universe),
            |k| st.skills.universe_rank(universe_key, k),
            &st.defiance,
        );

        let cur_phase = sim::current_phase(st);
        let phase_tracks: Vec<PhaseTrackView> = Phase::ALL
            .iter()
            .map(|&ph| {
                let nodes = content::phase_skills(ph)
                    .iter()
                    .map(|n| {
                        let rank = st.skills.phase_rank(ph.key(), n.key);
                        let maxed = rank >= n.max_rank;
                        let cost = content::skill_cost(n, rank);
                        SkillNodeView {
                            key: n.key.to_string(),
                            name: n.name.to_string(),
                            desc: n.desc.to_string(),
                            rank,
                            max_rank: n.max_rank,
                            maxed,
                            cost: if maxed { "—".to_string() } else { cost.format() },
                            affordable: !maxed && st.defiance.gte(&cost),
                            unlocked: true,
                            prereqs: vec![],
                        }
                    })
                    .collect();
                PhaseTrackView {
                    phase_key: ph.key().to_string(),
                    phase_name: ph.name().to_string(),
                    active: ph == cur_phase,
                    nodes,
                }
            })
            .collect();

        // Live upgrade-derived stats for the Climb tab — each surfaces only once
        // it's above its default. Read from the sim so a rebalance flows through.
        let lvl = |k: &str| run.upgrade_levels.get(k).copied().unwrap_or(0);
        let mut stats: Vec<StatView> = Vec::new();
        let trim = |v: f64| { let s = format!("{v:.2}"); s.trim_end_matches('0').trim_end_matches('.').to_string() };

        // base push — the FLAT base push power Muscle grows (universe base + Muscle),
        // shown as a value, not a multiplier.
        let base_flat = sim::push_base_flat(st);
        if base_flat.to_f64() > def.base_push.to_f64() + 1e-9 {
            stats.push(StatView {
                label: "base push".into(),
                value: base_flat.format(),
                detail: format!("universe base {} + Muscle +{}", def.base_push.format(), lvl(content::U_MUSCLE)),
            });
        }
        // push power — the multipliers ON that base (Grip · skills · phase), separate
        // from the flat. Hover breaks down the same sources `push_amount` multiplies.
        let pmult: f64 = sim::push_sources(st).iter().map(|(_, s)| *s).product();
        if pmult > 1.0001 {
            let detail = sim::push_sources(st).iter()
                .filter(|(_, s)| *s > 1.0001)
                .map(|(name, s)| format!("{name} ×{s:.2}"))
                .collect::<Vec<_>>().join(" · ");
            stats.push(StatView { label: "push power".into(), value: format!("×{pmult:.2}"), detail });
        }
        let pm = sim::push_multiplier(st);
        if pm > 1.0 {
            stats.push(StatView { label: "push multiplier".into(), value: format!("×{pm:.2}"), detail: "from Leverage — multiplies your FINAL push/heave/auto (momentum included)".into() });
        }
        // Manual family — manual-push-only levers (Grind's lane).
        if lvl(content::U_POWER) > 0 {
            stats.push(StatView { label: "manual push".into(), value: format!("×{:.2}", sim::manual_push_mult(st)),
                detail: format!("Power — +25%/level to MANUAL pushes only (not auto/heave). Level {}.", lvl(content::U_POWER)) });
        }
        if lvl(content::U_SPEED) > 0 {
            stats.push(StatView { label: "push rate".into(), value: format!("{}/s max", sim::push_rate_max(st) as i64),
                detail: format!("Speed — the hold-to-push cap: 5/s base + 1/s per level. Level {}.", lvl(content::U_SPEED)) });
        }
        if lvl(content::U_STREAK) > 0 {
            let now = sim::streak_bonus(st) * 100.0;
            let cap = lvl(content::U_STREAK) * 10;
            stats.push(StatView { label: "push streak".into(), value: format!("+{now:.0}% / +{cap}%"),
                detail: format!("Streak — hold without stopping: +10%/sec, capped at +{cap}% (level {}). Now +{now:.0}%.", lvl(content::U_STREAK)) });
        }
        if run_view.auto_unlocked {
            let patience = run_view.stance.as_deref() == Some("patience");
            let mut detail = format!("push {} × Rolling ×{}", sim::push_amount(st).format(), trim(sim::rolling_mult(st)));
            if patience {
                let battery = st.run.momentum * sim::momentum_value(st);
                detail.push_str(&format!(" × 2 × amp {:.2} (Patience) + momentum {battery:.0} flat", sim::patience_amp(st)));
            }
            if sim::push_multiplier(st) > 1.0 { detail.push_str(&format!(" · then Leverage ×{}", trim(sim::push_multiplier(st)))); }
            stats.push(StatView {
                label: "auto-push".into(),
                value: format!("+{} every {}s", sim::auto_push_size(st).format(), trim(sim::auto_push_interval(st))),
                detail,
            });
        }
        let hm = sim::heave_mult(st);
        if hm > 2.0 {
            let value = if hm.fract() == 0.0 { format!("×{}", hm as i64) } else { format!("×{hm:.2}") };
            let detail = format!("base ×2 · Force +{} · Weight +{:.2}", lvl(content::U_HEAVE_MULT), hm - 2.0 - lvl(content::U_HEAVE_MULT) as f64);
            stats.push(StatView { label: "heave force".into(), value, detail });
        }
        // scorn / roll-back — shown when ANYTHING boosts it: Spite (flat), or a
        // scorn skill (Endurance / Defiant Hands / Thin Air), phase, or Zeal.
        let scorn = sim::scorn_base(st);
        let spite = lvl(content::U_SCORN_GAIN);
        if scorn.to_f64() > def.base_scorn.to_f64() + 1e-9 {
            let mut parts: Vec<String> = Vec::new();
            if spite > 0 { parts.push(format!("Spite +{spite} flat")); }
            for (name, m) in sim::scorn_sources(st) {
                if m > 1.0001 { parts.push(format!("{name} ×{m:.2}")); }
            }
            stats.push(StatView { label: "scorn / roll-back".into(), value: scorn.format(), detail: parts.join(" · ") });
        }
        if lvl(content::U_BRAWN) > 0 {
            stats.push(StatView { label: "heave base".into(), value: format!("×{:.2}", sim::heave_base_mult(st)),
                detail: format!("Brawn — +20%/level to your Heave's base shove. Level {}.", lvl(content::U_BRAWN)) });
        }
        // max momentum — Reach adds, and Grind doubles the whole cap. Show it
        // whenever it's above the default 12 (so Grind's 2× surfaces even at Reach 0).
        let cap = sim::momentum_cap_now(st) as i64;
        if cap != 12 {
            let mut detail = format!("base 12 + Reach +{}", lvl(content::U_REACH));
            if run_view.stance.as_deref() == Some("grind") { detail.push_str(" · ×2 (Grind)"); }
            stats.push(StatView { label: "max momentum".into(), value: format!("{cap}"), detail });
        }
        let mv = sim::momentum_value(st);
        if mv > 1.0 {
            stats.push(StatView { label: "momentum value".into(), value: format!("×{mv:.2}"),
                detail: format!("Conduit — +20%/level: each momentum point adds more effort to Push & Heave. Level {}.", lvl(content::U_CONDUIT)) });
        }
        let ci = sim::cascade_interval(st);
        if ci > 0.0 {
            stats.push(StatView { label: "auto-burst".into(), value: format!("every {}s", trim(ci)),
                detail: format!("Cascade — auto-push bursts ~3s of climb every 6÷level seconds. Level {}.", lvl(content::U_CASCADE)) });
        }
        if lvl(content::U_ZEAL) > 0 {
            stats.push(StatView { label: "scorn bonus".into(), value: format!("+{:.0}%", (sim::zeal_mult(st) - 1.0) * 100.0),
                detail: format!("Zeal — up to +5%/level scorn, scaling with how much scorn you've spent this run. Level {}.", lvl(content::U_ZEAL)) });
        }
        {
            // Always shown — it's the core roll-back reward, not just a Surplus stat.
            let sm = sim::surplus_mult(st);
            stats.push(StatView {
                label: "overshoot".into(),
                value: format!("2× height → ×{:.1}", 1.0 + sm),
                detail: format!("At the summit a 5s countdown runs while you keep climbing — roll back from higher for more scorn (scorn × height ÷ summit). Base: +100% over = +100% scorn. Surplus multiplies the overshoot part (now ×{}).", trim(sm)),
            });
        }
        if lvl(content::U_THRIFT) > 0 {
            stats.push(StatView { label: "upgrade costs".into(), value: format!("-{:.0}%", (1.0 - sim::thrift_mult(st)) * 100.0),
                detail: format!("Thrift — all scorn upgrades ~5%/level cheaper (diminishing, never free). Level {}.", lvl(content::U_THRIFT)) });
        }

        let prestige_gain = sim::defiance_gain(st);
        let universe_choices = st
            .unlocked
            .iter()
            .map(|k| UniverseChoice {
                kind: k.key().to_string(),
                name: content::universe_def(*k).name.to_string(),
            })
            .collect();

        let tabs = vec![
            TabView { key: "climb".into(), name: "Climb".into(), unlocked: true, hint: String::new() },
            TabView {
                key: "scorn".into(),
                name: "Scorn".into(),
                unlocked: st.ever_rolled_back,
                hint: "Reach the summit and roll back — each roll-back earns scorn to spend here.".into(),
            },
            TabView {
                key: "defiance".into(),
                name: "Defiance".into(),
                unlocked: st.seen_prestige || !st.total_prestiges.is_zero(),
                hint: "Prestige to bank defiance and open the permanent skill trees.".into(),
            },
            TabView {
                key: "universes".into(),
                name: "Universes".into(),
                unlocked: false,
                hint: "Spend defiance to open a new universe — a new mask for the same labor.".into(),
            },
        ];

        Snapshot {
            tabs,
            day_fraction: sim::day_fraction(st),
            phase: sim::current_phase(st).name().to_string(),
            phase_key: sim::current_phase(st).key().to_string(),
            stats,
            stance_stats: stance_stats(st),
            last_effort: st.last_effort.format(),
            last_frac: st.last_frac,
            last_best: st.last_best,
            // All-time bests as a hover breakdown (value · stance · N prestiges ago).
            best_push_detail: best_detail(&st.best_push, &st.best_push_stance, &st.best_push_prestige, &st.total_prestiges),
            best_heave_detail: best_detail(&st.best_heave, &st.best_heave_stance, &st.best_heave_prestige, &st.total_prestiges),
            best_scorn_detail: best_detail(&st.best_scorn, &st.best_scorn_stance, &st.best_scorn_prestige, &st.total_prestiges),
            last_scorn: st.last_scorn.format(),
            last_scorn_frac: st.last_scorn_frac,
            last_scorn_best: st.last_scorn_best,
            last_kind: match st.last_kind {
                1 => "push".to_string(),
                2 => "heave".to_string(),
                _ => String::new(),
            },
            action_count: st.total_pushes.to_f64(),
            cascade_seq: run.cascade_seq as f64,
            cascade_effort: run.cascade_effort.format(),
            defiance: st.defiance.format(),
            total_pushes: st.total_pushes.format(),
            total_prestiges: st.total_prestiges.format(),
            organizing_unlocked: st.organizing.unlocked,
            run: run_view,
            global_skills,
            pusher_skills,
            universe_skills,
            phase_tracks,
            can_prestige: !prestige_gain.is_zero(),
            prestige_gain: prestige_gain.format(),
            next_defiance_at: (prestige_gain.to_f64() + 1.0).powi(2) * 10.0,
            universe_choices,
        }
    }

    pub fn snapshot_json(&self) -> String {
        serde_json::to_string(&self.snapshot()).expect("snapshot serializes")
    }
}

impl Default for Engine {
    fn default() -> Self {
        Engine::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ADR 0012 completeness contract: every invested upgrade surfaces a "your
    // edge" stat. If you add an upgrade, add its expected label here (and its
    // row to ADR 0012's table) — a purchased upgrade with no stat is a bug.
    #[test]
    fn every_invested_upgrade_surfaces_in_your_edge() {
        let mut e = Engine::new();
        e.state.run.rollbacks = crate::number::Big::from_f64(20.0); // so Weight has counted roll-backs
        for ud in content::universe_def(sim::STARTING_UNIVERSE).run_upgrades {
            for _ in 0..3 {
                e.state.run.scorn = crate::number::Big::from_f64(1e12);
                e.dispatch(&Command::BuyUpgrade { key: ud.key.to_string() });
            }
        }
        let labels: Vec<String> = e.snapshot().stats.iter().map(|s| s.label.clone()).collect();
        for expected in [
            "base push", "push power", "push multiplier", "manual push", "push rate", "push streak", "auto-push",
            "auto-burst", "heave force", "heave base", "max momentum",
            "momentum value", "scorn / roll-back", "scorn bonus", "overshoot", "upgrade costs",
        ] {
            assert!(labels.iter().any(|l| l == expected), "missing your-edge stat: {expected}\ngot: {labels:?}");
        }
    }

    #[test]
    fn empty_save_starts_fresh() {
        let e = Engine::load_or_new("", 1000.0);
        assert!(matches!(e.last_report(), LoadReport::Started));
        assert!(e.savable());
    }

    #[test]
    fn load_applies_offline_catch_up() {
        let mut s = GameState::new();
        s.last_seen = 0.0;
        let raw = save::serialize(&s);
        let e = Engine::load_or_new(&raw, 1000.0); // 1 second later
        match e.last_report() {
            LoadReport::Resumed { steps, .. } => assert_eq!(*steps, 10),
            other => panic!("expected Resumed, got {other:?}"),
        }
    }

    #[test]
    fn refused_save_blocks_overwrite() {
        let s = GameState::new();
        let mut blob: serde_json::Value =
            serde_json::from_str(&save::serialize(&s)).unwrap();
        blob["schema_version"] = serde_json::Value::from(999u32);
        let raw = serde_json::to_string(&blob).unwrap();

        let mut e = Engine::load_or_new(&raw, 1000.0);
        assert!(matches!(e.last_report(), LoadReport::Refused { .. }));
        assert!(!e.savable());
        assert!(e.serialize(2000.0).is_none());

        e.force_new_game();
        assert!(e.serialize(2000.0).is_some());
    }

    #[test]
    fn import_refused_keeps_current_game() {
        let mut e = Engine::new();
        e.dispatch(&Command::Push);
        let before = e.snapshot_json();

        let s = GameState::new();
        let mut blob: serde_json::Value =
            serde_json::from_str(&save::serialize(&s)).unwrap();
        blob["schema_version"] = serde_json::Value::from(999u32);
        let raw = serde_json::to_string(&blob).unwrap();
        let report = e.import_json(&raw, 5000.0);

        assert!(report.contains("refused"));
        assert_eq!(e.snapshot_json(), before);
    }

    #[test]
    fn dispatch_json_round_trips_a_command() {
        let mut e = Engine::new();
        e.dispatch_json(r#"{"kind":"push"}"#);
        assert_eq!(e.snapshot().total_pushes, "1");
    }

    #[test]
    fn dispatch_json_buys_a_skill() {
        let mut e = Engine::new();
        e.state.defiance = Big::from_f64(10.0);
        e.dispatch_json(r#"{"kind":"buy_skill","tree":"global","key":"g_might"}"#);
        assert_eq!(e.state.skills.global_rank(content::G_MIGHT), 1);
    }

    #[test]
    fn malformed_command_is_ignored() {
        let mut e = Engine::new();
        e.dispatch_json(r#"{"kind":"explode_the_sun"}"#);
        assert_eq!(e.snapshot().total_pushes, "0");
    }

    #[test]
    fn snapshot_has_expected_shape() {
        let e = Engine::new();
        let v: serde_json::Value = serde_json::from_str(&e.snapshot_json()).unwrap();
        assert_eq!(v["run"]["universe_name"], "The Mountain");
        assert_eq!(v["defiance"], "0");
        assert!(v["global_skills"].is_array());
        assert!(v["pusher_skills"].is_array());
        assert!(v["universe_skills"].is_array());
        assert_eq!(v["universe_choices"][0]["name"], "The Mountain");
    }

    #[test]
    fn export_then_import_is_identity() {
        let mut e = Engine::new();
        e.dispatch(&Command::Push);
        e.dispatch(&Command::Push);
        let exported = e.serialize(1234.0).unwrap();

        let mut e2 = Engine::new();
        e2.import_json(&exported, 1234.0);
        assert_eq!(e2.snapshot().total_pushes, "2");
    }
}
