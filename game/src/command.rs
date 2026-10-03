//! ADR 0003 (in-half) — the closed vocabulary of player actions.

use serde::{Deserialize, Serialize};

use crate::state::{Phase, Stance, UniverseKind};

/// Which permanent skill tree a purchase targets.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkillTree {
    /// The condition itself (applies everywhere).
    Global,
    /// The active pusher's tree (the character).
    Pusher,
    /// The active universe's tree (the place).
    Universe,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Command {
    /// One active push on the current climb.
    Push,
    /// Buy a level of a shared in-run upgrade (by key), spending scorn.
    BuyUpgrade { key: String },
    /// Buy a level of a Stance-branch node (by key), spending scorn.
    BuyBranch { key: String },
    /// Commit to a sub-build (by key), once the Stance keystone is bought.
    ChooseSubBuild { key: String },
    /// A strong manual burst-push (ADR 0009). Active stances only.
    Heave,
    /// Manually roll the boulder back, if at the summit. Grants scorn.
    Rollback,
    /// Buy a rank of a permanent skill node, spending defiance.
    BuySkill { tree: SkillTree, key: String },
    /// Buy a rank in a phase track (ADR 0011), spending defiance.
    BuyPhaseSkill { phase: Phase, key: String },
    /// Bank defiance and start a fresh run of the chosen universe, committing to a
    /// Stance (`None` only on the very first prestige if the UI offers no choice).
    Prestige {
        universe: UniverseKind,
        #[serde(default)]
        stance: Option<Stance>,
    },
}
