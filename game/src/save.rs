//! ADR 0006 — versioned save blob with an ordered, forward-only migration chain.
//! ADR 0007 — the two-way version guard: behind current migrates forward, ahead
//! of current is refused and preserved untouched.
//!
//! The Django-migrations mental model holds, with one correction: there is no
//! migrations *table*. The blob carries a single `schema_version` integer — that
//! is the high-water mark, and the chain replays from it. Migrations are pure
//! data transforms on the raw JSON, run *before* typed deserialization, so they
//! never depend on current struct definitions.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::state::GameState;

/// The current schema version. Bump this and add exactly one migration step when
/// the shape changes.
pub const CURRENT_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
pub struct SaveBlob {
    pub schema_version: u32,
    pub state: GameState,
}

/// What loading a blob produced.
#[derive(Debug)]
pub enum LoadOutcome {
    /// Migrated (if needed) and deserialized successfully.
    Loaded(Box<GameState>),
    /// `schema_version` is ahead of this build — refused and preserved untouched
    /// (ADR 0007). The caller must NOT overwrite the stored blob.
    Refused { found: u32, current: u32 },
    /// Not loadable: invalid JSON, missing/zero version, or a shape that won't
    /// deserialize even after migration.
    Corrupt(String),
}

/// A migration mutates the raw blob object in place, upgrading it by one version.
type Migration = fn(&mut Map<String, Value>);

/// The ordered chain. `MIGRATIONS[v - 1]` upgrades a version-`v` blob to `v + 1`.
/// Empty at v1 — the first release has no prior shape to migrate from. When
/// `CURRENT_VERSION` becomes 2, push the `1 -> 2` step here.
fn migrations() -> &'static [Migration] {
    &[]
}

/// Walk `obj` from `from` up to `to`, applying one migration per version step.
/// Factored out so the chain mechanism is testable independently of the (empty)
/// production chain.
fn apply_migrations(obj: &mut Map<String, Value>, from: u32, to: u32, migs: &[Migration]) {
    for v in from..to {
        let idx = (v - 1) as usize;
        if let Some(m) = migs.get(idx) {
            m(obj);
        }
    }
}

/// Serialize the current state into a versioned blob string. This is exactly
/// what export writes (ADR 0007): there is no separate export format.
pub fn serialize(state: &GameState) -> String {
    let blob = SaveBlob { schema_version: CURRENT_VERSION, state: state.clone() };
    serde_json::to_string(&blob).expect("game state is always serializable")
}

/// Load a blob string (from localStorage or an import). Applies the version
/// guard and migration chain.
pub fn load(raw: &str) -> LoadOutcome {
    load_with(raw, &migrations(), CURRENT_VERSION)
}

/// Load against an explicit chain and target version. The real entry point is
/// `load`; this exists so tests can drive a non-empty chain and a future target.
fn load_with(raw: &str, migs: &[Migration], current: u32) -> LoadOutcome {
    let mut value: Value = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(e) => return LoadOutcome::Corrupt(format!("not valid JSON: {e}")),
    };

    let obj = match value.as_object_mut() {
        Some(o) => o,
        None => return LoadOutcome::Corrupt("save root is not an object".into()),
    };

    let found = match obj.get("schema_version").and_then(Value::as_u64) {
        Some(v) => v as u32,
        None => return LoadOutcome::Corrupt("missing schema_version".into()),
    };
    if found == 0 {
        return LoadOutcome::Corrupt("invalid schema_version 0".into());
    }
    // ADR 0007: a blob from a newer build has no forward path — refuse, preserve.
    if found > current {
        return LoadOutcome::Refused { found, current };
    }

    apply_migrations(obj, found, current, migs);
    obj.insert("schema_version".into(), Value::from(current));

    match serde_json::from_value::<SaveBlob>(value) {
        Ok(blob) => LoadOutcome::Loaded(Box::new(blob.state)),
        Err(e) => LoadOutcome::Corrupt(format!("deserialize failed after migration: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::number::Big;

    fn loaded(outcome: LoadOutcome) -> GameState {
        match outcome {
            LoadOutcome::Loaded(s) => *s,
            other => panic!("expected Loaded, got {other:?}"),
        }
    }

    #[test]
    fn round_trips_current_version() {
        let mut s = GameState::new();
        s.defiance = Big::from_f64(42.0);
        s.run.scorn = Big::from_f64(17.0);
        s.run.climb.height = Big::new(3.0, 5);
        let raw = serialize(&s);
        let back = loaded(load(&raw));
        assert_eq!(back.defiance, s.defiance);
        assert_eq!(back.run.scorn, s.run.scorn);
        assert_eq!(back.run.climb.height, s.run.climb.height);
    }

    #[test]
    fn refuses_future_version_and_does_not_panic() {
        // a blob claiming a newer schema than we know
        let s = GameState::new();
        let mut blob: Value = serde_json::from_str(&serialize(&s)).unwrap();
        blob["schema_version"] = Value::from(999u32);
        let raw = serde_json::to_string(&blob).unwrap();
        match load(&raw) {
            LoadOutcome::Refused { found, current } => {
                assert_eq!(found, 999);
                assert_eq!(current, CURRENT_VERSION);
            }
            other => panic!("expected Refused, got {other:?}"),
        }
    }

    #[test]
    fn rejects_garbage() {
        assert!(matches!(load("not json at all"), LoadOutcome::Corrupt(_)));
        assert!(matches!(load("[1,2,3]"), LoadOutcome::Corrupt(_)));
        assert!(matches!(load("{}"), LoadOutcome::Corrupt(_))); // no version
    }

    #[test]
    fn missing_version_is_corrupt_not_assumed() {
        // we always write schema_version; a blob without one is not trusted
        let raw = r#"{"state":{}}"#;
        assert!(matches!(load(raw), LoadOutcome::Corrupt(_)));
    }

    // Proves the migration ENGINE walks a multi-step chain in order, even though
    // the production chain is empty at v1. Simulates shipping v3 with two steps.
    #[test]
    fn migration_chain_walks_in_order() {
        fn v1_to_v2(obj: &mut Map<String, Value>) {
            // v2 added a "scorn" field that v1 saves lacked
            let st = obj.get_mut("state").unwrap().as_object_mut().unwrap();
            st.entry("added_in_v2").or_insert(Value::from(true));
        }
        fn v2_to_v3(obj: &mut Map<String, Value>) {
            let st = obj.get_mut("state").unwrap().as_object_mut().unwrap();
            st.insert("added_in_v3".into(), Value::from(7));
        }
        let chain: &[Migration] = &[v1_to_v2, v2_to_v3];

        let mut obj: Map<String, Value> = serde_json::from_str(
            r#"{"schema_version":1,"state":{}}"#,
        )
        .unwrap();
        apply_migrations(&mut obj, 1, 3, chain);

        let st = obj["state"].as_object().unwrap();
        assert_eq!(st["added_in_v2"], Value::from(true));
        assert_eq!(st["added_in_v3"], Value::from(7));
    }

    // A v1 blob meeting "v2 code": the new field is supplied by a migration, so
    // it deserializes cleanly. This is the ADR 0006 promise end-to-end.
    #[test]
    fn old_save_loads_under_newer_code() {
        // pretend current is 2 and a migration adds the field a v1 blob lacks
        fn fill_missing(obj: &mut Map<String, Value>) {
            let st = obj.get_mut("state").unwrap().as_object_mut().unwrap();
            // imagine v2 added `organizing`; ensure it exists
            st.entry("organizing")
                .or_insert(serde_json::json!({ "unlocked": false }));
        }
        let chain: &[Migration] = &[fill_missing];

        // a "v1" blob: full valid state but we strip organizing to simulate the
        // older shape, then migrate it forward to v2.
        let s = GameState::new();
        let mut blob: Value = serde_json::from_str(&serialize(&s)).unwrap();
        blob["schema_version"] = Value::from(1u32);
        blob["state"].as_object_mut().unwrap().remove("organizing");
        let raw = serde_json::to_string(&blob).unwrap();

        match load_with(&raw, chain, 2) {
            LoadOutcome::Loaded(st) => assert!(!st.organizing.unlocked),
            other => panic!("expected Loaded, got {other:?}"),
        }
    }
}
