//! ADR 0003 — the wasm-bindgen boundary. A thin forwarder over `Engine`: every
//! method here delegates to logic that is tested natively. Compiled only for
//! `wasm32` (see the target-gated dependency in Cargo.toml).
//!
//! Contract: state flows one way. `tick`/`snapshot` return a JSON snapshot
//! (read-only); `dispatch` takes a JSON command (write-only). JS never touches
//! state directly.

use wasm_bindgen::prelude::*;

use crate::engine::Engine;

#[wasm_bindgen]
pub struct Game {
    engine: Engine,
}

#[wasm_bindgen]
impl Game {
    /// Construct from a save string (`""` for a new game). `now_ms` is
    /// `Date.now()`. Applies offline catch-up; the load report is available via
    /// `load_report()`.
    #[wasm_bindgen(constructor)]
    pub fn new(saved: &str, now_ms: f64) -> Game {
        Game { engine: Engine::load_or_new(saved, now_ms) }
    }

    /// Advance one frame and return the fresh render snapshot as JSON.
    pub fn tick(&mut self, dt_ms: f64) -> String {
        self.engine.tick(dt_ms);
        self.engine.snapshot_json()
    }

    /// Apply a player command (JSON), e.g. `{"kind":"push"}`.
    pub fn dispatch(&mut self, cmd_json: &str) {
        self.engine.dispatch_json(cmd_json);
    }

    /// Current render snapshot as JSON, without advancing.
    pub fn snapshot(&self) -> String {
        self.engine.snapshot_json()
    }

    /// Serialize for localStorage. Returns `""` when saving is blocked (a
    /// refused/corrupt prior blob, ADR 0007) — JS must treat empty as "do not
    /// write".
    pub fn save(&mut self, now_ms: f64) -> String {
        self.engine.serialize(now_ms).unwrap_or_default()
    }

    /// Export the save blob for download. Identical to `save` — the exported
    /// file IS the blob (ADR 0007).
    pub fn export(&mut self, now_ms: f64) -> String {
        self.engine.serialize(now_ms).unwrap_or_default()
    }

    /// Import a save file. Returns the load report as JSON. On refuse/corrupt the
    /// current game is left untouched.
    pub fn import(&mut self, raw: &str, now_ms: f64) -> String {
        self.engine.import_json(raw, now_ms)
    }

    /// The report from the last load/import (JSON).
    pub fn load_report(&self) -> String {
        self.engine.last_report_json()
    }

    /// Whether saving is currently allowed.
    pub fn savable(&self) -> bool {
        self.engine.savable()
    }

    /// Abandon a refused/corrupt save and start fresh (the only thing allowed to
    /// discard it).
    pub fn new_game(&mut self) {
        self.engine.force_new_game();
    }
}
